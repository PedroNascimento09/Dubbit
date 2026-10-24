use std::path::PathBuf;

use dubbit_core::config::Settings;
use dubbit_core::error::DubbitError;
use dubbit_core::hardware::{self, resolve_profile};
use dubbit_core::pipeline::{self, Event, Request};

use crate::args::DubArgs;
use crate::output::{done, step};

fn default_output(input: &std::path::Path) -> PathBuf {
    let name = input
        .file_stem()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| "output".to_string());
    input.with_file_name(format!("{name}.dubbit"))
}

fn model_id(args: &DubArgs, settings: &Settings, profile_model: &str) -> String {
    args.model
        .clone()
        .or_else(|| settings.asr_model.clone())
        .unwrap_or_else(|| profile_model.to_string())
}

pub fn run(args: &DubArgs, json: bool, verbose: bool) -> Result<(), DubbitError> {
    let mut settings = Settings::load();
    if let Some(profile) = &args.profile {
        settings.profile = profile.clone();
    }
    settings.validate()?;

    let hardware = hardware::detect();
    let profile = resolve_profile(&hardware, &settings.profile);
    let threads = match (args.threads, settings.threads) {
        (Some(count), _) if count > 0 => count,
        (_, count) if count > 0 => count,
        _ => profile.threads,
    };

    if !args.input.is_file() {
        return Err(dubbit_core::error::media_unreadable(
            &args.input,
            "the file does not exist",
        ));
    }

    let model = model_id(args, &settings, &profile.asr_model);

    let formats = if args.formats.is_empty() {
        vec!["srt".to_string(), "vtt".to_string()]
    } else {
        args.formats.clone()
    };
    let translator = args
        .translator
        .map(Into::into)
        .unwrap_or(settings.translator);

    let request = Request {
        input: args.input.clone(),
        output: args
            .output
            .clone()
            .unwrap_or_else(|| default_output(&args.input)),
        source_language: args.source.clone(),
        target_language: args.target.clone(),
        model,
        translator,
        ollama_host: settings.ollama_host.clone(),
        ollama_model: settings.ollama_model.clone(),
        translator_command: settings.translator_command.clone(),
        glossary: args.glossary.clone().or_else(|| settings.glossary.clone()),
        threads,
        formats,
        translate: !args.no_translate,
        resume: args.resume,
        cancel: None,
    };

    let total = if args.target.is_some() && !args.no_translate {
        4
    } else {
        3
    };
    let mut current = 0;
    let mut progress = |event: Event| match event {
        Event::Stage(stage) => {
            current += 1;
            let label = match stage {
                dubbit_core::project::STAGE_INGEST => "Extracting audio",
                dubbit_core::project::STAGE_TRANSCRIBE => "Transcribing speech",
                dubbit_core::project::STAGE_TRANSLATE => "Translating subtitles",
                dubbit_core::project::STAGE_SUBTITLES => "Writing subtitles",
                other => other,
            };
            step(current, total, label, json);
        }
        Event::Detail(detail) => {
            if verbose {
                eprintln!("  {detail}");
            }
        }
        Event::Download { received, total } => {
            if verbose {
                eprintln!("  {received} of {total} bytes");
            }
        }
    };

    let outcome = pipeline::run(&request, &mut progress)?;

    if json {
        let payload = serde_json::json!({
            "project": outcome.project_dir,
            "files": outcome.files,
            "segments": outcome.segments,
            "source_language": outcome.source_language,
            "target_language": outcome.target_language,
            "elapsed_seconds": outcome.elapsed_seconds,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).unwrap_or_default()
        );
        return Ok(());
    }

    done(
        &format!(
            "{} segments in {:.1}s",
            outcome.segments, outcome.elapsed_seconds
        ),
        json,
    );
    for file in &outcome.files {
        done(
            &format!("  {}", outcome.project_dir.join(file).display()),
            json,
        );
    }
    Ok(())
}
