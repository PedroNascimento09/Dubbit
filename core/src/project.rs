use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::error::{filesystem_failed, project_incompatible, project_invalid, Result};
use crate::files::{read_json, write_json};
use crate::media::MediaInfo;
use crate::segments::Segment;

pub const SCHEMA: u32 = 1;
pub const PROJECT_FILE: &str = "project.json";
pub const AUDIO_FILE: &str = "audio.wav";

pub const STAGE_INGEST: &str = "ingest";
pub const STAGE_TRANSCRIBE: &str = "transcribe";
pub const STAGE_TRANSLATE: &str = "translate";
pub const STAGE_SUBTITLES: &str = "subtitles";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceRef {
    pub path: PathBuf,
    pub sha256: String,
    pub size_bytes: u64,
    pub duration: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Options {
    pub source_language: Option<String>,
    pub target_language: Option<String>,
    pub asr_model: String,
    pub translator: String,
    pub translate: bool,
    pub formats: Vec<String>,
}

impl Options {
    pub fn formats_label(&self) -> String {
        self.formats.join(",")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StageStatus {
    Running,
    Done,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageState {
    pub status: StageStatus,
    pub at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub schema: u32,
    pub created_at: u64,
    pub updated_at: u64,
    pub source: SourceRef,
    pub options: Options,
    pub media: Option<MediaInfo>,
    #[serde(default)]
    pub segments: Vec<Segment>,
    #[serde(default)]
    pub stages: BTreeMap<String, StageState>,
}

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs())
        .unwrap_or(0)
}

impl Project {
    pub fn new(source: SourceRef, media: MediaInfo, options: Options) -> Self {
        let stamp = now();
        Self {
            schema: SCHEMA,
            created_at: stamp,
            updated_at: stamp,
            source,
            options,
            media: Some(media),
            segments: Vec::new(),
            stages: BTreeMap::new(),
        }
    }

    pub fn done(&self, stage: &str) -> bool {
        self.stages
            .get(stage)
            .map(|state| state.status == StageStatus::Done)
            .unwrap_or(false)
    }

    pub fn mark(&mut self, stage: &str) {
        self.stages.insert(
            stage.to_string(),
            StageState {
                status: StageStatus::Done,
                at: now(),
            },
        );
        self.updated_at = now();
    }

    pub fn clear(&mut self, stage: &str) {
        self.stages.remove(stage);
        self.updated_at = now();
    }

    pub fn source_language(&self) -> Option<&str> {
        self.options.source_language.as_deref()
    }

    pub fn target_language(&self) -> Option<&str> {
        self.options.target_language.as_deref()
    }
}

#[derive(Debug, Clone)]
pub struct Workspace {
    pub dir: PathBuf,
}

impl Workspace {
    pub fn open(dir: &Path) -> Self {
        Self {
            dir: dir.to_path_buf(),
        }
    }

    pub fn project_file(&self) -> PathBuf {
        self.dir.join(PROJECT_FILE)
    }

    pub fn audio_file(&self) -> PathBuf {
        self.dir.join(AUDIO_FILE)
    }

    pub fn subtitles_dir(&self) -> PathBuf {
        self.dir.join("subtitles")
    }

    pub fn report_file(&self) -> PathBuf {
        self.dir.join("report.json")
    }

    pub fn exists(&self) -> bool {
        self.project_file().is_file()
    }

    pub fn load(&self) -> Result<Project> {
        let path = self.project_file();
        let project: Project = read_json(&path)?;
        if project.schema > SCHEMA {
            return Err(project_incompatible(project.schema));
        }
        Ok(project)
    }

    pub fn save(&self, project: &mut Project) -> Result<()> {
        project.updated_at = now();
        fs::create_dir_all(&self.dir)
            .map_err(|error| filesystem_failed(&self.dir, &error.to_string()))?;
        write_json(&self.project_file(), project)
    }

    pub fn remove(&self) -> Result<()> {
        if self.dir.is_dir() {
            fs::remove_dir_all(&self.dir)
                .map_err(|error| filesystem_failed(&self.dir, &error.to_string()))?;
        }
        Ok(())
    }

    pub fn subtitle_path(&self, language: &str, extension: &str) -> PathBuf {
        self.subtitles_dir().join(format!("{language}.{extension}"))
    }

    pub fn ensure_dir(&self, path: &Path) -> Result<()> {
        fs::create_dir_all(path).map_err(|error| filesystem_failed(path, &error.to_string()))
    }

    pub fn write_segments(&self, name: &str, value: &[Segment]) -> Result<()> {
        let path = self.dir.join(format!("segments-{name}.json"));
        write_json(&path, &value.to_vec())
    }

    pub fn read_segments(&self, name: &str) -> Result<Vec<Segment>> {
        let path = self.dir.join(format!("segments-{name}.json"));
        if !path.is_file() {
            return Err(project_invalid(&path, "the segment checkpoint is missing"));
        }
        read_json(&path)
    }
}

pub fn fingerprint(path: &Path) -> Result<(String, u64)> {
    let metadata =
        fs::metadata(path).map_err(|error| filesystem_failed(path, &error.to_string()))?;
    let digest = crate::files::sha256_file(path)?;
    Ok((digest, metadata.len()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::MediaInfo;

    fn media() -> MediaInfo {
        MediaInfo {
            path: PathBuf::from("input.mp4"),
            duration: 12.5,
            size_bytes: 4096,
            has_video: true,
            has_audio: true,
            video_codec: Some("h264".into()),
            width: Some(1920),
            height: Some(1080),
            fps: Some(25.0),
            audio_codec: Some("aac".into()),
            sample_rate: Some(48000),
            channels: Some(2),
        }
    }

    fn options() -> Options {
        Options {
            source_language: Some("en".into()),
            target_language: Some("pt".into()),
            asr_model: "whisper-tiny-q5_1".into(),
            translator: "auto".into(),
            translate: true,
            formats: vec!["srt".into()],
        }
    }

    fn project() -> Project {
        Project::new(
            SourceRef {
                path: PathBuf::from("input.mp4"),
                sha256: "abc".into(),
                size_bytes: 4096,
                duration: 12.5,
            },
            media(),
            options(),
        )
    }

    #[test]
    fn round_trips_through_disk() {
        let directory = tempfile::tempdir().unwrap();
        let workspace = Workspace::open(directory.path());
        let mut project = project();
        project.segments = vec![Segment::new(0, 0.0, 1.0, "hello")];
        project.mark(STAGE_INGEST);
        workspace.save(&mut project).unwrap();
        let loaded = workspace.load().unwrap();
        assert_eq!(loaded.segments.len(), 1);
        assert!(loaded.done(STAGE_INGEST));
        assert!(!loaded.done(STAGE_TRANSCRIBE));
    }

    #[test]
    fn newer_schema_is_rejected() {
        let directory = tempfile::tempdir().unwrap();
        let workspace = Workspace::open(directory.path());
        let mut project = project();
        project.schema = SCHEMA + 1;
        write_json(&workspace.project_file(), &project).unwrap();
        let error = workspace.load().unwrap_err();
        assert_eq!(error.code, "DUB-3001");
    }

    #[test]
    fn segment_checkpoints_round_trip() {
        let directory = tempfile::tempdir().unwrap();
        let workspace = Workspace::open(directory.path());
        let segments = vec![Segment::new(0, 0.0, 1.0, "hello")];
        workspace.write_segments("source", &segments).unwrap();
        assert_eq!(workspace.read_segments("source").unwrap().len(), 1);
        assert_eq!(
            workspace.read_segments("missing").unwrap_err().code,
            "DUB-3002"
        );
    }
}
