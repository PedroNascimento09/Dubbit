use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Duration;

use crate::config::{Settings, TranslatorChoice};
use crate::error::{engine_failed, engine_missing, Result};
use crate::glossary::Glossary;
use crate::segments::Segment;
use crate::subtitles::flatten;

pub trait Translator {
    fn name(&self) -> &'static str;
    fn translate(&self, text: &str, source: Option<&str>, target: &str) -> Result<String>;
}

#[derive(Debug, Clone, Copy)]
pub struct Passthrough;

impl Translator for Passthrough {
    fn name(&self) -> &'static str {
        "passthrough"
    }

    fn translate(&self, text: &str, _source: Option<&str>, _target: &str) -> Result<String> {
        Ok(text.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct Ollama {
    pub host: String,
    pub model: String,
}

impl Ollama {
    pub fn new(host: &str, model: &str) -> Self {
        Self {
            host: host.trim_end_matches('/').to_string(),
            model: model.to_string(),
        }
    }

    pub fn probe(&self) -> bool {
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_millis(700))
            .build();
        agent
            .get(&format!("{}/api/tags", self.host))
            .call()
            .map(|response| response.status() == 200)
            .unwrap_or(false)
    }

    fn prompt(&self, text: &str, source: Option<&str>, target: &str) -> String {
        let source = source.unwrap_or("the detected language");
        format!(
            "Translate subtitles from {source} to {target}. Translate names and terms literally, \
never answer questions and never add notes. Keep the tone, keep the original line breaks, stay \
close to the source length. Output the translation only.\n\n{text}"
        )
    }
}

impl Translator for Ollama {
    fn name(&self) -> &'static str {
        "ollama"
    }

    fn translate(&self, text: &str, source: Option<&str>, target: &str) -> Result<String> {
        let body = serde_json::json!({
            "model": self.model,
            "prompt": self.prompt(text, source, target),
            "stream": false,
            "options": { "temperature": 0.2 },
        })
        .to_string();
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(120))
            .build();
        let response = agent
            .post(&format!("{}/api/generate", self.host))
            .set("Content-Type", "application/json")
            .send_string(&body)
            .map_err(|error| match error {
                ureq::Error::Status(_, _) => engine_failed(
                    "ollama",
                    "the server refused the request; confirm the model is pulled",
                ),
                ureq::Error::Transport(_) => engine_missing(
                    "ollama",
                    format!("no model server answered on {}", self.host),
                ),
            })?;
        let value: serde_json::Value = response
            .into_json()
            .map_err(|error| engine_failed("ollama", error.to_string()))?;
        let answer = value
            .get("response")
            .and_then(|item| item.as_str())
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .ok_or_else(|| engine_failed("ollama", "the model returned no text"))?;
        Ok(answer.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct CommandTranslator {
    pub program: String,
}

impl Translator for CommandTranslator {
    fn name(&self) -> &'static str {
        "command"
    }

    fn translate(&self, text: &str, source: Option<&str>, target: &str) -> Result<String> {
        let mut child = Command::new(&self.program)
            .env("DUBBIT_SOURCE", source.unwrap_or(""))
            .env("DUBBIT_TARGET", target)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| {
                if error.kind() == std::io::ErrorKind::NotFound {
                    engine_missing("translator command", self.program.clone())
                } else {
                    engine_failed("translator command", error.to_string())
                }
            })?;
        if let Some(stdin) = child.stdin.as_mut() {
            stdin
                .write_all(text.as_bytes())
                .map_err(|error| engine_failed("translator command", error.to_string()))?;
        }
        drop(child.stdin.take());
        let output = child
            .wait_with_output()
            .map_err(|error| engine_failed("translator command", error.to_string()))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let detail = stderr.lines().last().unwrap_or("no output").to_string();
            return Err(engine_failed("translator command", &detail));
        }
        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if text.is_empty() {
            return Err(engine_failed(
                "translator command",
                "the command returned no text",
            ));
        }
        Ok(text)
    }
}

pub fn configured(settings: &Settings) -> Option<String> {
    let command = settings
        .translator_command
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    match settings.translator {
        TranslatorChoice::Command => command.map(|_| "Custom command".to_string()),
        TranslatorChoice::Ollama => Some(format!("Local model, {}", settings.ollama_model)),
        TranslatorChoice::Auto => {
            if command.is_some() {
                return Some("Custom command".to_string());
            }
            let ollama = Ollama::new(&settings.ollama_host, &settings.ollama_model);
            ollama
                .probe()
                .then(|| format!("Local model, {}", settings.ollama_model))
        }
        TranslatorChoice::Passthrough => None,
    }
}

pub fn build(settings: &Settings) -> Box<dyn Translator> {
    match settings.translator {
        TranslatorChoice::Auto => match &settings.translator_command {
            Some(program) => Box::new(CommandTranslator {
                program: program.clone(),
            }),
            None => {
                let ollama = Ollama::new(&settings.ollama_host, &settings.ollama_model);
                if ollama.probe() {
                    Box::new(ollama)
                } else {
                    Box::new(Passthrough)
                }
            }
        },
        TranslatorChoice::Passthrough => Box::new(Passthrough),
        TranslatorChoice::Ollama => {
            Box::new(Ollama::new(&settings.ollama_host, &settings.ollama_model))
        }
        TranslatorChoice::Command => match &settings.translator_command {
            Some(program) => Box::new(CommandTranslator {
                program: program.clone(),
            }),
            None => Box::new(Passthrough),
        },
    }
}

pub fn clean(text: &str) -> String {
    let mut value = text.trim();
    for prefix in [
        "Translation:",
        "translation:",
        "Traducao:",
        "Tradução:",
        "Traduccion:",
        "Traducción:",
    ] {
        if let Some(rest) = value.strip_prefix(prefix) {
            value = rest.trim();
        }
    }
    value.trim_matches('"').trim().to_string()
}

pub fn translate_segments(
    translator: &dyn Translator,
    segments: &mut [Segment],
    source: Option<&str>,
    target: &str,
    glossary: Option<&Glossary>,
) -> Result<usize> {
    let mut translated = 0;
    for segment in segments.iter_mut() {
        let original = segment.text.clone();
        let (guarded, used) = match glossary {
            Some(glossary) if !glossary.is_empty() => glossary.protect(&original),
            _ => (original.clone(), Vec::new()),
        };
        let answer = clean(&translator.translate(&guarded, source, target)?);
        let restored = match glossary {
            Some(glossary) if !used.is_empty() => glossary.restore(&answer, &used),
            _ => answer,
        };
        let final_text = if restored.trim().is_empty() {
            original
        } else {
            restored
        };
        segment.translation = flatten(&final_text);
        translated += 1;
    }
    Ok(translated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::glossary::{Glossary, Term};

    #[test]
    fn passthrough_keeps_text() {
        let text = Passthrough
            .translate("Ola mundo", Some("pt"), "en")
            .unwrap();
        assert_eq!(text, "Ola mundo");
    }

    #[test]
    fn glossary_survives_translation() {
        let glossary = Glossary::new(vec![Term {
            source: "Dubbit".into(),
            target: "Dubbit".into(),
        }]);
        let mut segments = vec![Segment::new(0, 0.0, 1.0, "Dubbit is fast")];
        translate_segments(
            &Passthrough,
            &mut segments,
            Some("en"),
            "pt",
            Some(&glossary),
        )
        .unwrap();
        assert_eq!(segments[0].translation, "Dubbit is fast");
    }
}
