use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::{invalid_option, DubbitError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TranslatorChoice {
    #[default]
    Auto,
    Passthrough,
    Ollama,
    Command,
}

impl fmt::Display for TranslatorChoice {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Auto => "auto",
            Self::Passthrough => "passthrough",
            Self::Ollama => "ollama",
            Self::Command => "command",
        };
        formatter.write_str(value)
    }
}

impl FromStr for TranslatorChoice {
    type Err = DubbitError;

    fn from_str(value: &str) -> Result<Self> {
        match value.trim().to_lowercase().as_str() {
            "auto" => Ok(Self::Auto),
            "passthrough" => Ok(Self::Passthrough),
            "ollama" => Ok(Self::Ollama),
            "command" => Ok(Self::Command),
            other => Err(invalid_option(
                "translator",
                other,
                "auto, passthrough, ollama or command",
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub profile: String,
    pub asr_model: Option<String>,
    pub translator: TranslatorChoice,
    pub ollama_host: String,
    pub ollama_model: String,
    pub translator_command: Option<String>,
    pub glossary: Option<PathBuf>,
    pub threads: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            profile: "auto".to_string(),
            asr_model: None,
            translator: TranslatorChoice::Auto,
            ollama_host: "http://127.0.0.1:11434".to_string(),
            ollama_model: "llama3.2".to_string(),
            translator_command: None,
            glossary: None,
            threads: 0,
        }
    }
}

impl Settings {
    pub fn load() -> Settings {
        let path = crate::paths::config_file();
        if !path.is_file() {
            return Settings::default();
        }
        crate::files::read_json(&path).unwrap_or_default()
    }

    pub fn save(&self) -> Result<()> {
        crate::files::write_json(&crate::paths::config_file(), self)
    }

    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "profile" => self.profile = value.to_string(),
            "asr-model" => self.asr_model = Some(value.to_string()),
            "translator" => self.translator = value.parse()?,
            "ollama-host" => self.ollama_host = value.to_string(),
            "ollama-model" => self.ollama_model = value.to_string(),
            "translator-command" => self.translator_command = Some(value.to_string()),
            "glossary" => self.glossary = Some(PathBuf::from(value)),
            "threads" => {
                self.threads = value.parse().map_err(|_| {
                    invalid_option("threads", value, "a whole number of worker threads")
                })?;
            }
            other => {
                return Err(invalid_option(
                    "setting",
                    other,
                    "profile, asr-model, translator, ollama-host, ollama-model, translator-command, glossary or threads",
                ));
            }
        }
        self.validate()
    }

    pub fn validate(&self) -> Result<()> {
        let profile = self.profile.trim().to_lowercase();
        if !matches!(profile.as_str(), "auto" | "light" | "balanced" | "max") {
            return Err(invalid_option(
                "profile",
                &self.profile,
                "auto, light, balanced or max",
            ));
        }
        if self.ollama_host.trim().is_empty() {
            return Err(invalid_option(
                "ollama_host",
                &self.ollama_host,
                "a host such as http://127.0.0.1:11434",
            ));
        }
        if self.translator == TranslatorChoice::Command
            && self
                .translator_command
                .as_deref()
                .map(str::trim)
                .unwrap_or_default()
                .is_empty()
        {
            return Err(invalid_option(
                "translator_command",
                &self.translator_command.clone().unwrap_or_default(),
                "a program that reads text on stdin and writes it on stdout",
            ));
        }
        Ok(())
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(|error| {
            DubbitError::new("DUB-0001", "Settings could not be written")
                .detail(error.to_string())
                .hint("Run 'dubbit doctor --report report.json'.")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_validate() {
        assert!(Settings::default().validate().is_ok());
    }

    #[test]
    fn rejects_unknown_profile() {
        let settings = Settings {
            profile: "turbo".to_string(),
            ..Settings::default()
        };
        let error = settings.validate().unwrap_err();
        assert_eq!(error.code, "DUB-0001");
    }

    #[test]
    fn command_translator_requires_a_program() {
        let broken = Settings {
            translator: TranslatorChoice::Command,
            translator_command: Some("  ".to_string()),
            ..Settings::default()
        };
        assert!(broken.validate().is_err());
        let working = Settings {
            translator: TranslatorChoice::Command,
            translator_command: Some("translate.sh".to_string()),
            ..Settings::default()
        };
        assert!(working.validate().is_ok());
    }

    #[test]
    fn json_round_trip() {
        let settings = Settings {
            profile: "light".to_string(),
            asr_model: Some("whisper-base-q5_1".to_string()),
            threads: 4,
            ..Settings::default()
        };
        let json = settings.to_json().unwrap();
        let restored: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.profile, "light");
        assert_eq!(restored.asr_model.as_deref(), Some("whisper-base-q5_1"));
        assert_eq!(restored.threads, 4);
    }

    #[test]
    fn translator_choice_parsing() {
        assert_eq!(
            "ollama".parse::<TranslatorChoice>().unwrap(),
            TranslatorChoice::Ollama
        );
        assert_eq!(TranslatorChoice::Command.to_string(), "command".to_string());
        assert!("garbage".parse::<TranslatorChoice>().is_err());
    }
}
