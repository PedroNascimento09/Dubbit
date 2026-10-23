use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{media_empty, media_unreadable, missing_tool, process_failed, Result};
use crate::paths;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaInfo {
    pub path: PathBuf,
    pub duration: f64,
    pub size_bytes: u64,
    pub has_video: bool,
    pub has_audio: bool,
    pub video_codec: Option<String>,
    pub width: Option<u64>,
    pub height: Option<u64>,
    pub fps: Option<f64>,
    pub audio_codec: Option<String>,
    pub sample_rate: Option<u64>,
    pub channels: Option<u64>,
}

pub fn ffmpeg_binary() -> Result<PathBuf> {
    paths::find_binary("ffmpeg", "DUBBIT_FFMPEG").ok_or_else(|| missing_tool("FFmpeg"))
}

pub fn ffprobe_binary() -> Result<PathBuf> {
    paths::find_binary("ffprobe", "DUBBIT_FFPROBE").ok_or_else(|| missing_tool("FFprobe"))
}

fn tail(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    let trimmed = text.trim();
    if trimmed.len() > 1200 {
        trimmed[trimmed.len() - 1200..].to_string()
    } else {
        trimmed.to_string()
    }
}

fn run(command: &mut Command, label: &str) -> Result<Output> {
    let output = command
        .output()
        .map_err(|error| process_failed(label, &error.to_string()))?;
    if !output.status.success() {
        return Err(process_failed(label, &tail(&output.stderr)));
    }
    Ok(output)
}

pub fn version() -> Result<String> {
    let mut command = Command::new(ffmpeg_binary()?);
    command.arg("-version");
    let output = run(&mut command, "FFmpeg")?;
    let text = String::from_utf8_lossy(&output.stdout);
    Ok(text.lines().next().unwrap_or("ffmpeg").trim().to_string())
}

fn number(value: &Value, key: &str) -> Option<f64> {
    value.get(key).and_then(|item| match item {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.parse().ok(),
        _ => None,
    })
}

fn integer(value: &Value, key: &str) -> Option<u64> {
    value.get(key).and_then(|item| match item {
        Value::Number(number) => number.as_u64(),
        Value::String(text) => text.parse().ok(),
        _ => None,
    })
}

fn text(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_string)
}

fn parse_fps(raw: Option<&str>) -> Option<f64> {
    let value = raw?;
    let (numerator, denominator) = value.split_once('/')?;
    let numerator: f64 = numerator.parse().ok()?;
    let denominator: f64 = denominator.parse().ok()?;
    if denominator == 0.0 {
        return None;
    }
    let fps = numerator / denominator;
    if fps > 0.0 {
        Some((fps * 1000.0).round() / 1000.0)
    } else {
        None
    }
}

fn stream_duration(stream: &Value) -> f64 {
    if let Some(duration) = number(stream, "duration") {
        return duration;
    }
    let tags = stream.get("tags");
    let raw = tags
        .and_then(|value| value.get("DURATION").or_else(|| value.get("duration")))
        .and_then(Value::as_str);
    let Some(raw) = raw else {
        return 0.0;
    };
    let parts: Vec<&str> = raw.split(':').collect();
    if parts.len() != 3 {
        return 0.0;
    }
    let hours: f64 = parts[0].parse().unwrap_or(0.0);
    let minutes: f64 = parts[1].parse().unwrap_or(0.0);
    let seconds: f64 = parts[2].parse().unwrap_or(0.0);
    hours * 3600.0 + minutes * 60.0 + seconds
}

pub fn probe(path: &Path) -> Result<MediaInfo> {
    let source = path
        .canonicalize()
        .map_err(|error| media_unreadable(path, &error.to_string()))?;
    if !source.is_file() {
        return Err(media_unreadable(&source, "the file does not exist"));
    }
    let size_bytes = fs::metadata(&source)
        .map_err(|error| media_unreadable(&source, &error.to_string()))?
        .len();
    if size_bytes == 0 {
        return Err(media_unreadable(&source, "the file is empty"));
    }
    let mut command = Command::new(ffprobe_binary()?);
    command.args([
        "-v",
        "error",
        "-print_format",
        "json",
        "-show_format",
        "-show_streams",
    ]);
    command.arg(&source);
    let output = run(&mut command, "FFprobe")?;
    let payload: Value = serde_json::from_slice(&output.stdout)
        .map_err(|error| media_unreadable(&source, &error.to_string()))?;
    let streams = payload
        .get("streams")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let videos: Vec<&Value> = streams
        .iter()
        .filter(|stream| text(stream, "codec_type").as_deref() == Some("video"))
        .filter(|stream| {
            stream
                .get("disposition")
                .and_then(|value| value.get("attached_pic"))
                .and_then(Value::as_u64)
                .unwrap_or(0)
                == 0
        })
        .collect();
    let audios: Vec<&Value> = streams
        .iter()
        .filter(|stream| text(stream, "codec_type").as_deref() == Some("audio"))
        .collect();
    if videos.is_empty() && audios.is_empty() {
        return Err(media_empty(&source));
    }
    let format = payload.get("format").cloned().unwrap_or(Value::Null);
    let mut duration = number(&format, "duration").unwrap_or(0.0);
    if duration <= 0.0 {
        duration = streams.iter().map(stream_duration).fold(0.0_f64, f64::max);
    }
    if duration <= 0.0 {
        return Err(media_unreadable(&source, "the duration is unknown"));
    }
    let video = videos.first().copied().unwrap_or(&Value::Null);
    let audio = audios.first().copied().unwrap_or(&Value::Null);
    Ok(MediaInfo {
        path: source,
        duration: (duration * 1000.0).round() / 1000.0,
        size_bytes,
        has_video: !videos.is_empty(),
        has_audio: !audios.is_empty(),
        video_codec: text(video, "codec_name"),
        width: integer(video, "width"),
        height: integer(video, "height"),
        fps: parse_fps(text(video, "r_frame_rate").as_deref()),
        audio_codec: text(audio, "codec_name"),
        sample_rate: integer(audio, "sample_rate"),
        channels: integer(audio, "channels"),
    })
}

pub fn extract_audio(source: &Path, destination: &Path, duration: Option<f64>) -> Result<()> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|error| process_failed("FFmpeg", &error.to_string()))?;
    }
    let mut command = Command::new(ffmpeg_binary()?);
    command.args(["-nostdin", "-y", "-hide_banner", "-loglevel", "error", "-i"]);
    command.arg(source);
    if let Some(seconds) = duration {
        command.args(["-t", &format!("{seconds:.3}")]);
    }
    command.args(["-vn", "-ac", "1", "-ar", "16000", "-c:a", "pcm_s16le"]);
    command.arg(destination);
    run(&mut command, "FFmpeg")?;
    if !destination.is_file() {
        return Err(process_failed("FFmpeg", "the audio file was not created"));
    }
    Ok(())
}
