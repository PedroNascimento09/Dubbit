use std::fmt;
use std::path::Path;

use serde_json::json;

pub type Result<T> = std::result::Result<T, DubbitError>;

#[derive(Debug, Clone)]
pub struct DubbitError {
    pub code: &'static str,
    pub title: String,
    pub detail: Option<String>,
    pub hint: Option<String>,
}

impl DubbitError {
    pub fn new(code: &'static str, title: impl Into<String>) -> Self {
        Self {
            code,
            title: title.into(),
            detail: None,
            hint: None,
        }
    }

    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn json(&self) -> serde_json::Value {
        json!({
            "code": self.code,
            "title": self.title,
            "detail": self.detail,
            "hint": self.hint,
        })
    }
}

impl fmt::Display for DubbitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.title)?;
        if let Some(detail) = &self.detail {
            write!(formatter, "\n{detail}")?;
        }
        if let Some(hint) = &self.hint {
            write!(formatter, "\nNext: {hint}")?;
        }
        Ok(())
    }
}

impl std::error::Error for DubbitError {}

pub fn invalid_option(name: &str, value: &str, expected: &str) -> DubbitError {
    DubbitError::new("DUB-0001", format!("Invalid value for {name}"))
        .detail(format!("{value:?} is not valid. Expected {expected}."))
        .hint("Run 'dubbit --help' to see the accepted values.")
}

pub fn missing_tool(tool: &str) -> DubbitError {
    DubbitError::new("DUB-1001", format!("{tool} was not found"))
        .detail(format!(
            "Dubbit needs the {tool} executable to read and write media."
        ))
        .hint("Install FFmpeg, or point DUBBIT_TOOLS_DIR at a folder that contains it.")
}

pub fn media_unreadable(path: &Path, reason: &str) -> DubbitError {
    DubbitError::new("DUB-1002", "The media file could not be read")
        .detail(format!("{}: {reason}", path.display()))
        .hint("Check that the file plays in a normal player, then try again.")
}

pub fn media_empty(path: &Path) -> DubbitError {
    DubbitError::new("DUB-1003", "The media file has no audio or video")
        .detail(format!("{} contains no usable stream.", path.display()))
        .hint("Choose a file that has at least one audio track.")
}

pub fn process_failed(tool: &str, reason: &str) -> DubbitError {
    DubbitError::new("DUB-1004", format!("{tool} could not process the media"))
        .detail(reason)
        .hint("Check that the file is complete, then run 'dubbit doctor --report report.json'.")
}

pub fn unsupported_language(value: &str) -> DubbitError {
    DubbitError::new("DUB-2001", "Unsupported language")
        .detail(format!("{value:?} is not a language Dubbit knows."))
        .hint("Use a language code such as en, pt or es.")
}

pub fn engine_missing(engine: &str, reason: impl Into<String>) -> DubbitError {
    DubbitError::new("DUB-1101", format!("{engine} is not available"))
        .detail(reason)
        .hint("Install the component, or point its DUBBIT_* variable at the executable.")
}

pub fn engine_failed(engine: &str, reason: impl Into<String>) -> DubbitError {
    DubbitError::new("DUB-1102", format!("{engine} could not finish"))
        .detail(reason)
        .hint("Check the input, then run 'dubbit doctor --report report.json'.")
}

pub fn no_speech() -> DubbitError {
    DubbitError::new("DUB-2101", "No speech detected")
        .detail("The audio holds no speech the recogniser can use.")
        .hint("Check the audio track, or choose the language explicitly.")
}

pub fn cancelled() -> DubbitError {
    DubbitError::new("DUB-0002", "Cancelled")
        .detail("The run was interrupted before the output was written.")
        .hint("Run the same command again; finished stages resume from the checkpoint.")
}

pub fn project_incompatible(schema: u32) -> DubbitError {
    DubbitError::new("DUB-3001", "Project was created by a newer Dubbit")
        .detail(format!(
            "Project schema {schema} is not supported by this version."
        ))
        .hint("Update Dubbit, or start a new project.")
}

pub fn project_invalid(path: &Path, reason: &str) -> DubbitError {
    DubbitError::new("DUB-3002", "Project file is not valid")
        .detail(format!("{}: {reason}", path.display()))
        .hint("Keep the original media and start a new project.")
}

pub fn output_exists(path: &Path) -> DubbitError {
    DubbitError::new("DUB-3003", "Project already exists")
        .detail(format!(
            "{} already contains a Dubbit project.",
            path.display()
        ))
        .hint("Pass --resume to continue it, or choose another output folder.")
}

pub fn filesystem_failed(path: &Path, reason: &str) -> DubbitError {
    DubbitError::new("DUB-3004", "The file could not be written")
        .detail(format!("{}: {reason}", path.display()))
        .hint("Check that the folder exists and that the disk is not full.")
}

pub fn invalid_json(path: &Path, reason: &str) -> DubbitError {
    DubbitError::new("DUB-3005", "The JSON file could not be read")
        .detail(format!("{}: {reason}", path.display()))
        .hint("Restore the file from a backup, or start a new project.")
}

pub fn download_failed(source: &str, reason: &str) -> DubbitError {
    DubbitError::new("DUB-3101", "Download failed")
        .detail(format!("{source}: {reason}"))
        .hint("Check the connection and run the command again; downloads resume.")
}

pub fn checksum_mismatch(name: &str) -> DubbitError {
    DubbitError::new("DUB-3102", "Downloaded file is damaged")
        .detail(format!(
            "The checksum of {name} does not match the manifest."
        ))
        .hint("Run the command again to download a fresh copy.")
}

pub fn model_missing(model_id: &str) -> DubbitError {
    DubbitError::new("DUB-3104", "Model is not installed")
        .detail(format!("{model_id} has not been downloaded yet."))
        .hint(format!("Run: dubbit models install {model_id}"))
}

pub fn model_unknown(model_id: &str) -> DubbitError {
    DubbitError::new("DUB-3103", "Unknown model")
        .detail(format!("{model_id:?} is not listed in the model manifest."))
        .hint("Run 'dubbit models list' to see the available ids.")
}
