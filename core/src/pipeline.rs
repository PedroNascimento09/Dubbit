use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::Serialize;

use crate::config::TranslatorChoice;
use crate::engines::{asr, translate, Translator};
use crate::error::{filesystem_failed, output_exists, Result};
use crate::glossary;
use crate::languages;
use crate::media;
use crate::models;
use crate::project::{
    fingerprint, Options, Project, SourceRef, Workspace, STAGE_INGEST, STAGE_SUBTITLES,
    STAGE_TRANSCRIBE, STAGE_TRANSLATE,
};
use crate::subtitles;

#[derive(Debug, Clone)]
pub struct Request {
    pub input: PathBuf,
    pub output: PathBuf,
    pub source_language: Option<String>,
    pub target_language: Option<String>,
    pub model: String,
    pub translator: TranslatorChoice,
    pub ollama_host: String,
    pub ollama_model: String,
    pub translator_command: Option<String>,
    pub glossary: Option<PathBuf>,
    pub threads: usize,
    pub formats: Vec<String>,
    pub translate: bool,
    pub resume: bool,
}

#[derive(Debug, Clone)]
pub enum Event {
    Stage(&'static str),
    Detail(String),
    Download { received: u64, total: u64 },
}

#[derive(Debug, Clone, Serialize)]
pub struct Outcome {
    pub project_dir: PathBuf,
    pub files: Vec<String>,
    pub segments: usize,
    pub source_language: String,
    pub target_language: Option<String>,
    pub elapsed_seconds: f64,
}

impl Request {
    pub fn formats(&self) -> Vec<String> {
        let mut formats: Vec<String> = self
            .formats
            .iter()
            .map(|format| format.trim().to_lowercase())
            .filter(|format| matches!(format.as_str(), "srt" | "vtt"))
            .collect();
        formats.sort();
        formats.dedup();
        if formats.is_empty() {
            formats.push("srt".to_string());
        }
        formats
    }
}

fn relative(workspace: &Workspace, path: &Path) -> String {
    path.strip_prefix(&workspace.dir)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn translate_language(request: &Request) -> Result<Option<String>> {
    let target = match request.target_language.as_deref() {
        Some(value) => languages::normalize(value)?,
        None => return Ok(None),
    };
    Ok(Some(target))
}

pub fn run(request: &Request, progress: &mut dyn FnMut(Event)) -> Result<Outcome> {
    let started = Instant::now();
    let workspace = Workspace::open(&request.output);
    let resuming = request.resume && workspace.exists();
    if workspace.exists() && !request.resume {
        return Err(output_exists(&request.output));
    }

    let source = match fs::metadata(&request.input) {
        Ok(metadata) if metadata.is_file() => request
            .input
            .canonicalize()
            .map_err(|error| filesystem_failed(&request.input, &error.to_string()))?,
        _ => {
            return Err(crate::error::media_unreadable(
                &request.input,
                "the file does not exist",
            ))
        }
    };

    let target_language = translate_language(request)?;
    let formats = request.formats();
    let mut project = if resuming {
        let loaded = workspace.load()?;
        let (digest, size_bytes) = fingerprint(&source)?;
        if loaded.source.sha256 != digest || loaded.source.size_bytes != size_bytes {
            return Err(crate::error::project_invalid(
                &workspace.project_file(),
                "the source media changed since the project was created",
            ));
        }
        loaded
    } else {
        if workspace.exists() {
            workspace.remove()?;
        }
        let info = media::probe(&source)?;
        let (digest, size_bytes) = fingerprint(&source)?;
        let options = Options {
            source_language: request
                .source_language
                .as_deref()
                .map(languages::normalize)
                .transpose()?,
            target_language: target_language.clone(),
            asr_model: request.model.clone(),
            translator: request.translator.to_string(),
            translate: request.translate && target_language.is_some(),
            formats,
        };
        let reference = SourceRef {
            path: source.clone(),
            sha256: digest,
            size_bytes,
            duration: info.duration,
        };
        let project = Project::new(reference, info, options);
        let mut project = project;
        workspace.save(&mut project)?;
        project
    };

    if !project.done(STAGE_INGEST) || !workspace.audio_file().is_file() {
        progress(Event::Stage(STAGE_INGEST));
        let duration = project.media.as_ref().map(|info| info.duration);
        media::extract_audio(&source, &workspace.audio_file(), duration)?;
        project.mark(STAGE_INGEST);
        workspace.save(&mut project)?;
    }

    let model_path = models::resolve_model(&project.options.asr_model)?;
    if !project.done(STAGE_TRANSCRIBE) || project.segments.is_empty() {
        progress(Event::Stage(STAGE_TRANSCRIBE));
        let options = asr::TranscribeOptions {
            model: model_path,
            language: project.options.source_language.clone(),
            translate: false,
            threads: request.threads,
        };
        let output = asr::transcribe(&workspace.audio_file(), &options)?;
        let detected = output.detected_language.clone();
        project.segments = output.segments;
        if project.options.source_language.is_none() {
            project.options.source_language = detected.clone();
        }
        workspace.write_segments("source", &project.segments)?;
        project.mark(STAGE_TRANSCRIBE);
        workspace.save(&mut project)?;
    }

    let source_language = project
        .options
        .source_language
        .clone()
        .unwrap_or_else(|| "und".to_string());

    let target = project.options.target_language.clone();
    let wants_translation = project.options.translate && target.is_some();
    if wants_translation && !project.done(STAGE_TRANSLATE) {
        progress(Event::Stage(STAGE_TRANSLATE));
        let glossary = match &request.glossary {
            Some(path) => Some(glossary::load(path)?),
            None => None,
        };
        let translator = translator_for(request);
        let source_code = project.options.source_language.clone();
        let target_code = target.clone().unwrap_or_default();
        let mut segments = project.segments.clone();
        translate::translate_segments(
            translator.as_ref(),
            &mut segments,
            source_code.as_deref(),
            &target_code,
            glossary.as_ref(),
        )?;
        project.segments = segments;
        workspace.write_segments("translated", &project.segments)?;
        project.mark(STAGE_TRANSLATE);
        workspace.save(&mut project)?;
    }

    if !project.done(STAGE_SUBTITLES) {
        progress(Event::Stage(STAGE_SUBTITLES));
        workspace.ensure_dir(&workspace.subtitles_dir())?;
        for format in &project.options.formats {
            for translated in [false, true] {
                if translated && !wants_translation {
                    continue;
                }
                let language = if translated {
                    target.clone().unwrap_or_else(|| "und".to_string())
                } else {
                    source_language.clone()
                };
                let path = workspace.subtitle_path(&language, format);
                match format.as_str() {
                    "vtt" => subtitles::write_vtt(&path, &project.segments, translated)?,
                    _ => subtitles::write_srt(&path, &project.segments, translated)?,
                }
            }
        }
        project.mark(STAGE_SUBTITLES);
        workspace.save(&mut project)?;
    }

    let files = subtitle_files(&workspace)?;
    Ok(Outcome {
        project_dir: workspace.dir.clone(),
        files,
        segments: project.segments.len(),
        source_language,
        target_language: target,
        elapsed_seconds: started.elapsed().as_secs_f64(),
    })
}

fn translator_for(request: &Request) -> Box<dyn Translator> {
    let settings = crate::config::Settings {
        translator: request.translator,
        ollama_host: request.ollama_host.clone(),
        ollama_model: request.ollama_model.clone(),
        translator_command: request.translator_command.clone(),
        threads: request.threads,
        ..crate::config::Settings::default()
    };
    translate::build(&settings)
}

pub fn subtitle_files(workspace: &Workspace) -> Result<Vec<String>> {
    let directory = workspace.subtitles_dir();
    if !directory.is_dir() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    for entry in fs::read_dir(&directory)
        .map_err(|error| filesystem_failed(&directory, &error.to_string()))?
    {
        let entry = entry.map_err(|error| filesystem_failed(&directory, &error.to_string()))?;
        let path = entry.path();
        if path.is_file() {
            files.push(relative(workspace, &path));
        }
    }
    files.sort();
    Ok(files)
}

pub fn stage_summary(project: &Project) -> Vec<(&'static str, bool)> {
    [
        STAGE_INGEST,
        STAGE_TRANSCRIBE,
        STAGE_TRANSLATE,
        STAGE_SUBTITLES,
    ]
    .iter()
    .map(|stage| (*stage, project.done(stage)))
    .collect()
}
