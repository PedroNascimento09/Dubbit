use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

use serde::de::DeserializeOwned;
use serde::Serialize;
use sha2::{Digest, Sha256};
use tempfile::NamedTempFile;

use crate::error::{filesystem_failed, invalid_json, Result};

pub fn human_size(bytes: u64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    for unit in units {
        if value < 1024.0 || unit == "TB" {
            if unit == "B" {
                return format!("{value:.0} {unit}");
            }
            return format!("{value:.1} {unit}");
        }
        value /= 1024.0;
    }
    format!("{value:.1} TB")
}

pub fn slugify(input: &str, fallback: &str) -> String {
    let mut slug = String::new();
    let mut pending_dash = false;
    for character in input.chars().flat_map(char::to_lowercase) {
        if character.is_alphanumeric() {
            if pending_dash && !slug.is_empty() {
                slug.push('-');
            }
            pending_dash = false;
            slug.push(character);
        } else {
            pending_dash = true;
        }
        if slug.chars().count() >= 48 {
            break;
        }
    }
    let trimmed = slug.trim_matches('-').to_string();
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed
    }
}

pub fn sha256_file(path: &Path) -> Result<String> {
    let mut file = File::open(path).map_err(|error| filesystem_failed(path, &error.to_string()))?;
    let mut digest = Sha256::new();
    let mut buffer = vec![0_u8; 1024 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| filesystem_failed(path, &error.to_string()))?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

pub fn sha256_text(text: &str) -> String {
    let digest = Sha256::digest(text.as_bytes());
    format!("{digest:x}")
}

pub fn ensure_parent(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| filesystem_failed(parent, &error.to_string()))?;
    }
    Ok(())
}

pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    ensure_parent(path)?;
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let mut temporary = NamedTempFile::new_in(parent)
        .map_err(|error| filesystem_failed(path, &error.to_string()))?;
    temporary
        .write_all(bytes)
        .and_then(|_| temporary.as_file().sync_all())
        .map_err(|error| filesystem_failed(path, &error.to_string()))?;
    temporary
        .persist(path)
        .map_err(|error| filesystem_failed(path, &error.error.to_string()))?;
    Ok(())
}

pub fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| filesystem_failed(path, &error.to_string()))?;
    bytes.push(b'\n');
    atomic_write(path, &bytes)
}

pub fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let text =
        fs::read_to_string(path).map_err(|error| filesystem_failed(path, &error.to_string()))?;
    serde_json::from_str(&text).map_err(|error| invalid_json(path, &error.to_string()))
}

pub fn read_text(path: &Path) -> Result<String> {
    fs::read_to_string(path).map_err(|error| filesystem_failed(path, &error.to_string()))
}
