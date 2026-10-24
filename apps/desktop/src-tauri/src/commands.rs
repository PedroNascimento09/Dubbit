use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use dubbit_core::config::Settings;
use dubbit_core::diagnostics;
use dubbit_core::engines::translate;
use dubbit_core::error::{engine_missing, DubbitError, Result};
use dubbit_core::files::human_size;
use dubbit_core::languages;
use dubbit_core::media;
use dubbit_core::models;
use dubbit_core::paths;
use dubbit_core::pipeline;
use dubbit_core::project::Workspace;
use dubbit_core::segments::Segment;

use crate::queue::Queue;
use crate::view::{Environment, ErrorView, Job, LanguageView, NewJob, ProjectView};

#[derive(Debug, Serialize)]
pub struct CommandError {
    #[serde(flatten)]
    view: ErrorView,
}

impl From<DubbitError> for CommandError {
    fn from(error: DubbitError) -> Self {
        Self {
            view: ErrorView {
                code: error.code.to_string(),
                title: error.title,
                detail: error.detail,
                hint: error.hint,
            },
        }
    }
}

fn invalid(message: &str, hint: &str) -> DubbitError {
    DubbitError::new("DUB-0001", message).hint(hint)
}

fn project_view(dir: &Path) -> Result<ProjectView> {
    let workspace = Workspace::open(dir);
    let project = workspace.load()?;
    let audio = workspace.audio_file();
    let duration = project
        .media
        .as_ref()
        .map(|info| info.duration)
        .unwrap_or(0.0);
    Ok(ProjectView {
        dir: workspace.dir.display().to_string(),
        name: dir
            .file_name()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_default(),
        source_path: project.source.path.display().to_string(),
        source_language: project.options.source_language.clone(),
        target_language: project.options.target_language.clone(),
        translate: project.options.translate,
        duration,
        formats: project.options.formats.clone(),
        segments: project.segments.clone(),
        files: pipeline::subtitle_files(&workspace)?,
        audio_path: audio.is_file().then(|| audio.display().to_string()),
    })
}

fn allow_audio(app: &AppHandle, dir: &Path) -> Result<()> {
    app.asset_protocol_scope()
        .allow_directory(dir, true)
        .map_err(|error| {
            invalid(
                &error.to_string(),
                "Restart Dubbit and open the project again.",
            )
        })
}

#[tauri::command]
pub fn environment() -> std::result::Result<Environment, CommandError> {
    let settings = Settings::load();
    let report = diagnostics::build(&settings)?;
    let blockers: Vec<String> = report
        .tools
        .iter()
        .filter(|tool| !tool.found)
        .map(|tool| tool.name.clone())
        .collect();
    let translator = translate::configured(&settings);
    let languages = languages::LANGUAGES
        .iter()
        .map(|(code, name)| LanguageView {
            code: (*code).to_string(),
            name: (*name).to_string(),
        })
        .collect();
    Ok(Environment {
        version: report.version,
        platform: report.hardware.summary(),
        hardware: report.hardware,
        profile: report.profile,
        tools: report.tools,
        models: report.models,
        languages,
        data_dir: report.data_dir,
        target_language: "pt".to_string(),
        translator_ready: translator.is_some(),
        translator_label: translator,
        ready: blockers.is_empty(),
        blockers,
    })
}

#[tauri::command]
pub fn inspect(path: String) -> std::result::Result<media::MediaInfo, CommandError> {
    Ok(media::probe(&PathBuf::from(path))?)
}

#[tauri::command]
pub fn enqueue(
    requests: Vec<NewJob>,
    queue: State<'_, Arc<Queue>>,
) -> std::result::Result<Vec<Job>, CommandError> {
    let translator = translate::configured(&Settings::load());
    let mut accepted = Vec::new();
    for request in requests {
        if request.target_language.is_some() && translator.is_none() {
            return Err(engine_missing(
                "Translation",
                "no local translator is configured on this computer",
            )
            .into());
        }
        let language = |code: Option<String>| -> Result<Option<String>> {
            code.as_deref().map(languages::normalize).transpose()
        };
        let source = language(request.source_language)?;
        let target = language(request.target_language)?;
        if let (Some(source), Some(target)) = (&source, &target) {
            if languages::matches(source, target) {
                return Err(invalid(
                    "The source and target languages are the same",
                    "Choose a different target language, or turn translation off.",
                )
                .into());
            }
        }
        models::get(&request.model)?;
        accepted.push(NewJob {
            input: request.input,
            model: request.model,
            source_language: source,
            target_language: target,
            translator: request.translator,
        });
    }
    Ok(queue.enqueue(accepted))
}

#[tauri::command]
pub fn jobs(queue: State<'_, Arc<Queue>>) -> Vec<Job> {
    queue.snapshot()
}

#[tauri::command]
pub fn job_action(id: u64, action: String, queue: State<'_, Arc<Queue>>) -> Vec<Job> {
    queue.action(id, &action)
}

#[tauri::command]
pub fn open_project(dir: String, app: AppHandle) -> std::result::Result<ProjectView, CommandError> {
    let path = PathBuf::from(&dir);
    allow_audio(&app, &path)?;
    Ok(project_view(&path)?)
}

#[tauri::command]
pub fn save_review(
    dir: String,
    segments: Vec<Segment>,
    app: AppHandle,
) -> std::result::Result<ProjectView, CommandError> {
    let path = PathBuf::from(&dir);
    let workspace = Workspace::open(&path);
    let mut project = workspace.load()?;
    pipeline::save_review(&workspace, &mut project, segments)?;
    allow_audio(&app, &path)?;
    Ok(project_view(&path)?)
}

#[tauri::command]
pub fn install_model(id: String, app: AppHandle) -> std::result::Result<String, CommandError> {
    let spec = models::get(&id)?;
    let label = spec.file.clone();
    let mut last = u64::MAX;
    models::install(&spec, &mut |received, total| {
        let step = received
            .saturating_mul(100)
            .checked_div(total)
            .unwrap_or(100);
        if step != last {
            last = step;
            let _ = app.emit(
                "model",
                serde_json::json!({
                    "id": id,
                    "fraction": step as f64 / 100.0,
                    "detail": format!("{} of {}", human_size(received), human_size(total)),
                }),
            );
        }
    })?;
    Ok(label)
}

#[tauri::command]
pub fn data_dir() -> String {
    paths::data_dir().display().to_string()
}
