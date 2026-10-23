use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::error::{
    checksum_mismatch, download_failed, filesystem_failed, model_missing, model_unknown, Result,
};
use crate::files::{human_size, read_json, sha256_file, write_json};
use crate::paths;

const MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../models/whisper.json"
));
const CHUNK: usize = 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSpec {
    pub id: String,
    pub engine: String,
    pub file: String,
    pub url: String,
    pub size: u64,
    pub sha256: String,
    pub license: String,
    pub ram_mb: u64,
    pub quality: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelSummary {
    pub id: String,
    pub file: String,
    pub size: String,
    pub size_bytes: u64,
    pub ram_mb: u64,
    pub license: String,
    pub quality: String,
    pub installed: bool,
}

#[derive(Debug, Deserialize)]
struct Manifest {
    models: Vec<ModelSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Installed {
    sha256: String,
    installed_at: u64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct State {
    #[serde(default)]
    installed: BTreeMap<String, Installed>,
}

impl ModelSpec {
    pub fn summary(&self) -> ModelSummary {
        ModelSummary {
            id: self.id.clone(),
            file: self.file.clone(),
            size: human_size(self.size),
            size_bytes: self.size,
            ram_mb: self.ram_mb,
            license: self.license.clone(),
            quality: self.quality.clone(),
            installed: is_installed(self),
        }
    }
}

pub fn catalog() -> Result<Vec<ModelSpec>> {
    let manifest: Manifest = serde_json::from_str(MANIFEST)
        .map_err(|error| download_failed("models/whisper.json", &error.to_string()))?;
    Ok(manifest.models)
}

pub fn get(id: &str) -> Result<ModelSpec> {
    catalog()?
        .into_iter()
        .find(|spec| spec.id == id)
        .ok_or_else(|| model_unknown(id))
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs())
        .unwrap_or(0)
}

fn read_state() -> State {
    let path = paths::state_file();
    if !path.is_file() {
        return State::default();
    }
    read_json::<State>(&path).unwrap_or_default()
}

fn write_state(state: &State) -> Result<()> {
    write_json(&paths::state_file(), state)
}

pub fn model_dir(spec: &ModelSpec) -> PathBuf {
    paths::models_dir().join(&spec.id)
}

pub fn model_path(spec: &ModelSpec) -> PathBuf {
    model_dir(spec).join(&spec.file)
}

pub fn is_installed(spec: &ModelSpec) -> bool {
    let state = read_state();
    let Some(entry) = state.installed.get(&spec.id) else {
        return false;
    };
    if entry.sha256 != spec.sha256 {
        return false;
    }
    fs::metadata(model_path(spec))
        .map(|metadata| metadata.len() == spec.size)
        .unwrap_or(false)
}

fn verify_part(part: &PathBuf, spec: &ModelSpec) -> Result<()> {
    let size = fs::metadata(part)
        .map_err(|error| filesystem_failed(part, &error.to_string()))?
        .len();
    if size != spec.size {
        return Err(download_failed(
            &spec.url,
            &format!("expected {} bytes, received {size}", spec.size),
        ));
    }
    if sha256_file(part)? != spec.sha256 {
        let _ = fs::remove_file(part);
        return Err(checksum_mismatch(&spec.file));
    }
    Ok(())
}

fn download(spec: &ModelSpec, progress: &mut dyn FnMut(u64, u64)) -> Result<()> {
    let target = model_path(spec);
    let part = target.with_extension("bin.part");
    fs::create_dir_all(model_dir(spec))
        .map_err(|error| filesystem_failed(&target, &error.to_string()))?;
    let existing = fs::metadata(&part).map(|value| value.len()).unwrap_or(0);
    let existing = if existing > spec.size {
        fs::remove_file(&part).map_err(|error| filesystem_failed(&part, &error.to_string()))?;
        0
    } else {
        existing
    };
    if existing == spec.size {
        verify_part(&part, spec)?;
        fs::rename(&part, &target)
            .map_err(|error| filesystem_failed(&target, &error.to_string()))?;
        progress(spec.size, spec.size);
        return Ok(());
    }
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(120))
        .build();
    let mut request = agent.get(&spec.url);
    if existing > 0 {
        request = request.set("Range", &format!("bytes={existing}-"));
    }
    let response = match request.call() {
        Ok(response) => response,
        Err(ureq::Error::Status(416, _)) if existing == spec.size => {
            verify_part(&part, spec)?;
            fs::rename(&part, &target)
                .map_err(|error| filesystem_failed(&target, &error.to_string()))?;
            progress(spec.size, spec.size);
            return Ok(());
        }
        Err(ureq::Error::Status(code, _)) => {
            return Err(download_failed(&spec.url, &format!("HTTP {code}")));
        }
        Err(error) => return Err(download_failed(&spec.url, &error.to_string())),
    };
    let resume = existing > 0 && response.status() == 206;
    let mut received = if resume { existing } else { 0 };
    let mut file = if resume {
        OpenOptions::new()
            .append(true)
            .open(&part)
            .map_err(|error| filesystem_failed(&part, &error.to_string()))?
    } else {
        File::create(&part).map_err(|error| filesystem_failed(&part, &error.to_string()))?
    };
    let mut reader = response.into_reader();
    let mut buffer = vec![0_u8; CHUNK];
    loop {
        let read = reader
            .read(&mut buffer)
            .map_err(|error| download_failed(&spec.url, &error.to_string()))?;
        if read == 0 {
            break;
        }
        file.write_all(&buffer[..read])
            .map_err(|error| filesystem_failed(&part, &error.to_string()))?;
        received += read as u64;
        progress(received.min(spec.size), spec.size);
    }
    file.flush()
        .map_err(|error| filesystem_failed(&part, &error.to_string()))?;
    drop(file);
    verify_part(&part, spec)?;
    if target.exists() {
        fs::remove_file(&target).map_err(|error| filesystem_failed(&target, &error.to_string()))?;
    }
    fs::rename(&part, &target).map_err(|error| filesystem_failed(&target, &error.to_string()))?;
    Ok(())
}

pub fn install(spec: &ModelSpec, progress: &mut dyn FnMut(u64, u64)) -> Result<PathBuf> {
    let target = model_path(spec);
    if !is_installed(spec) {
        download(spec, progress)?;
    } else {
        progress(spec.size, spec.size);
    }
    let mut state = read_state();
    state.installed.insert(
        spec.id.clone(),
        Installed {
            sha256: spec.sha256.clone(),
            installed_at: now(),
        },
    );
    write_state(&state)?;
    Ok(target)
}

pub fn verify(spec: &ModelSpec) -> Vec<String> {
    let path = model_path(spec);
    if !path.is_file() {
        return vec![format!("{} is not installed", spec.id)];
    }
    let metadata = match fs::metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) => return vec![format!("{} cannot be read: {error}", spec.file)],
    };
    if metadata.len() != spec.size {
        return vec![format!("{} has the wrong size", spec.file)];
    }
    match sha256_file(&path) {
        Ok(digest) if digest == spec.sha256 => Vec::new(),
        Ok(_) => vec![format!("{} has the wrong checksum", spec.file)],
        Err(error) => vec![error.to_string()],
    }
}

pub fn remove(spec: &ModelSpec) -> Result<()> {
    let directory = model_dir(spec);
    if directory.is_dir() {
        fs::remove_dir_all(&directory)
            .map_err(|error| filesystem_failed(&directory, &error.to_string()))?;
    }
    let mut state = read_state();
    state.installed.remove(&spec.id);
    write_state(&state)
}

pub fn resolve_model(id: &str) -> Result<PathBuf> {
    let candidate = PathBuf::from(id);
    if candidate.is_file() {
        return Ok(candidate);
    }
    let spec = get(id)?;
    if !is_installed(&spec) {
        return Err(model_missing(id));
    }
    Ok(model_path(&spec))
}

pub fn installed_specs() -> Result<Vec<ModelSpec>> {
    Ok(catalog()?
        .into_iter()
        .filter(is_installed)
        .collect::<Vec<ModelSpec>>())
}

pub fn installed_bytes() -> Result<u64> {
    Ok(installed_specs()?.iter().map(|spec| spec.size).sum())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_is_pinned_and_hashed() {
        let specs = catalog().unwrap();
        assert!(specs.len() >= 4);
        for spec in specs {
            assert_eq!(spec.license, "MIT");
            assert_eq!(spec.sha256.len(), 64);
            assert!(spec.url.contains("/resolve/"));
            assert!(spec.size > 0);
        }
    }

    #[test]
    fn unknown_models_are_rejected() {
        let error = get("missing").unwrap_err();
        assert_eq!(error.code, "DUB-3103");
    }

    #[test]
    fn installer_name_is_stable() {
        let spec = get("whisper-tiny-q5_1").unwrap();
        assert!(model_path(&spec).ends_with("ggml-tiny-q5_1.bin"));
    }
}
