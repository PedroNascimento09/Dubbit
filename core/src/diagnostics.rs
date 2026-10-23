use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::json;

use crate::config::Settings;
use crate::engines::asr;
use crate::error::Result;
use crate::hardware::{self, Hardware, Profile};
use crate::media;
use crate::models;
use crate::paths;

#[derive(Debug, Clone, Serialize)]
pub struct ToolStatus {
    pub name: String,
    pub found: bool,
    pub location: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub tool: String,
    pub version: String,
    pub os: String,
    pub arch: String,
    pub hardware: Hardware,
    pub profile: Profile,
    pub tools: Vec<ToolStatus>,
    pub models: Vec<models::ModelSummary>,
    pub installed_bytes: u64,
    pub data_dir: String,
    pub settings: Settings,
}

fn locate(name: &str, env_var: &str, version_command: Option<&[&str]>) -> ToolStatus {
    let path = paths::find_binary(name, env_var);
    let version = match (&path, version_command) {
        (Some(path), Some(arguments)) => std::process::Command::new(path)
            .args(arguments)
            .output()
            .ok()
            .filter(|output| output.status.success())
            .and_then(|output| {
                String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .next()
                    .map(str::trim)
                    .map(str::to_string)
            }),
        _ => None,
    };
    ToolStatus {
        name: name.to_string(),
        found: path.is_some(),
        location: path.map(|value| value.display().to_string()),
        version,
    }
}

fn asr_status() -> ToolStatus {
    match asr::resolve() {
        Ok(path) => ToolStatus {
            name: "whisper-cli".to_string(),
            found: true,
            location: Some(path.display().to_string()),
            version: None,
        },
        Err(_) => ToolStatus {
            name: "whisper-cli".to_string(),
            found: false,
            location: None,
            version: None,
        },
    }
}

pub fn build(settings: &Settings) -> Result<Report> {
    let hardware = hardware::detect();
    let profile = hardware::resolve_profile(&hardware, &settings.profile);
    let models = models::catalog()?
        .into_iter()
        .map(|spec| spec.summary())
        .collect::<Vec<_>>();
    let installed_bytes = models::installed_bytes().unwrap_or(0);
    Ok(Report {
        tool: "dubbit".to_string(),
        version: crate::VERSION.to_string(),
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        hardware,
        profile,
        tools: vec![
            locate("ffmpeg", "DUBBIT_FFMPEG", Some(&["-version"])),
            locate("ffprobe", "DUBBIT_FFPROBE", Some(&["-version"])),
            asr_status(),
        ],
        models,
        installed_bytes,
        data_dir: paths::data_dir().display().to_string(),
        settings: settings.clone(),
    })
}

impl Report {
    pub fn to_json(&self) -> serde_json::Value {
        json!(self)
    }

    pub fn backend_version(&self) -> Option<String> {
        media::version().ok()
    }

    pub fn write(&self, path: &Path) -> Result<PathBuf> {
        crate::files::write_json(path, self)?;
        Ok(path.to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_reports_missing_tools_as_missing() {
        let status = locate("dubbit-not-a-real-binary", "DUBBIT_NOT_A_REAL_BINARY", None);
        assert!(!status.found);
        assert!(status.location.is_none());
    }

    #[test]
    fn report_json_has_stable_keys() {
        let report = build(&Settings::default()).unwrap();
        let value = report.to_json();
        for key in ["tool", "version", "hardware", "tools", "models", "data_dir"] {
            assert!(value.get(key).is_some());
        }
    }
}
