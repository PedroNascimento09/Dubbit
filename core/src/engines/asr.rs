use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;

use crate::error::{engine_failed, engine_missing, filesystem_failed, no_speech, Result};
use crate::paths;
use crate::segments::Segment;

#[derive(Debug, Clone)]
pub struct TranscribeOptions {
    pub model: PathBuf,
    pub language: Option<String>,
    pub translate: bool,
    pub threads: usize,
}

#[derive(Debug, Clone)]
pub struct TranscribeOutput {
    pub segments: Vec<Segment>,
    pub detected_language: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WhisperSpan {
    from: u64,
    to: u64,
}

#[derive(Debug, Deserialize)]
struct WhisperSegment {
    offsets: WhisperSpan,
    text: String,
}

#[derive(Debug, Deserialize)]
struct WhisperOutput {
    #[serde(default)]
    result: Option<WhisperResult>,
    #[serde(default)]
    transcription: Vec<WhisperSegment>,
}

#[derive(Debug, Deserialize)]
struct WhisperResult {
    #[serde(default)]
    language: Option<String>,
}

pub fn resolve() -> Result<PathBuf> {
    paths::find_binary("whisper-cli", "DUBBIT_ASR").ok_or_else(|| {
        engine_missing(
            "whisper-cli",
            "no speech recognition executable was found on this machine.",
        )
    })
}

pub fn transcribe(input: &Path, options: &TranscribeOptions) -> Result<TranscribeOutput> {
    let binary = resolve()?;
    if !options.model.is_file() {
        return Err(engine_failed(
            "whisper-cli",
            "the selected model file is missing",
        ));
    }
    let workdir = input
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(paths::temp_dir);
    let stem = format!("asr-{}", std::process::id());
    let mut command = Command::new(&binary);
    command
        .arg("-m")
        .arg(&options.model)
        .arg("-f")
        .arg(input)
        .arg("-oj")
        .arg("-of")
        .arg(workdir.join(&stem))
        .arg("-pp")
        .arg("-np");
    if let Some(language) = options.language.clone() {
        command.arg("-l").arg(language);
    } else {
        command.arg("-l").arg("auto");
    }
    if options.translate {
        command.arg("-tr");
    }
    if options.threads > 0 {
        command.arg("-t").arg(options.threads.to_string());
    }
    let output = command.output().map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            engine_missing("whisper-cli", binary.display().to_string())
        } else {
            engine_failed("whisper-cli", error.to_string())
        }
    })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let compact = stderr.lines().last().unwrap_or("no output").to_string();
        return Err(engine_failed("whisper-cli", &compact));
    }
    let json_path = workdir.join(format!("{stem}.json"));
    let text = fs::read_to_string(&json_path)
        .map_err(|error| filesystem_failed(&json_path, &error.to_string()))?;
    let _ = fs::remove_file(&json_path);
    parse(&text)
}

pub fn parse(text: &str) -> Result<TranscribeOutput> {
    let parsed: WhisperOutput = serde_json::from_str(text)
        .map_err(|error| engine_failed("whisper-cli", error.to_string()))?;
    let detected_language = parsed.result.and_then(|result| result.language);
    let segments = parsed
        .transcription
        .into_iter()
        .filter(|item| !item.text.trim().is_empty() && item.offsets.to > item.offsets.from)
        .enumerate()
        .map(|(index, item)| {
            Segment::new(
                index,
                item.offsets.from as f64 / 1000.0,
                item.offsets.to as f64 / 1000.0,
                item.text.trim(),
            )
        })
        .collect::<Vec<Segment>>();
    if segments.is_empty() {
        return Err(no_speech());
    }
    Ok(TranscribeOutput {
        segments,
        detected_language,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_whisper_json() {
        let text = r#"{"result":{"language":"pt"},"transcription":[{"offsets":{"from":0,"to":1500},"text":" Ola. "},{"offsets":{"from":1500,"to":1500},"text":"  "}]}"#;
        let output = parse(text).unwrap();
        assert_eq!(output.detected_language.as_deref(), Some("pt"));
        assert_eq!(output.segments.len(), 1);
        assert_eq!(output.segments[0].text, "Ola.");
        assert_eq!(output.segments[0].end, 1.5);
    }

    #[test]
    fn rejects_empty_results() {
        let error = parse(r#"{"transcription":[]}"#).unwrap_err();
        assert_eq!(error.code, "DUB-2101");
    }
}
