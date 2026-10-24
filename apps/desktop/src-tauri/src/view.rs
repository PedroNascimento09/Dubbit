use serde::{Deserialize, Serialize};

use dubbit_core::diagnostics::ToolStatus;
use dubbit_core::hardware::{Hardware, Profile};
use dubbit_core::models::ModelSummary;
use dubbit_core::segments::Segment;

#[derive(Debug, Clone, Serialize)]
pub struct LanguageView {
    pub code: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Environment {
    pub version: String,
    pub platform: String,
    pub hardware: Hardware,
    pub profile: Profile,
    pub tools: Vec<ToolStatus>,
    pub models: Vec<ModelSummary>,
    pub languages: Vec<LanguageView>,
    pub data_dir: String,
    pub target_language: String,
    pub translator_ready: bool,
    pub translator_label: Option<String>,
    pub ready: bool,
    pub blockers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorView {
    pub code: String,
    pub title: String,
    pub detail: Option<String>,
    pub hint: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Queued,
    Running,
    Done,
    Failed,
    Stopped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: u64,
    pub input: String,
    pub name: String,
    pub output: String,
    pub model: String,
    pub source_language: Option<String>,
    pub target_language: Option<String>,
    pub translator: String,
    pub status: JobStatus,
    pub stage: Option<String>,
    pub detail: Option<String>,
    pub fraction: Option<f64>,
    pub queued_at: u64,
    pub started_at: Option<u64>,
    pub finished_at: Option<u64>,
    pub elapsed_seconds: f64,
    pub error: Option<ErrorView>,
    pub files: Vec<String>,
    pub segments: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewJob {
    pub input: String,
    pub model: String,
    pub source_language: Option<String>,
    pub target_language: Option<String>,
    pub translator: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectView {
    pub dir: String,
    pub name: String,
    pub source_path: String,
    pub source_language: Option<String>,
    pub target_language: Option<String>,
    pub translate: bool,
    pub duration: f64,
    pub formats: Vec<String>,
    pub segments: Vec<Segment>,
    pub files: Vec<String>,
    pub audio_path: Option<String>,
}
