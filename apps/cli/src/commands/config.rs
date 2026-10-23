use dubbit_core::config::Settings;
use dubbit_core::error::DubbitError;
use dubbit_core::paths;

use crate::args::ConfigCommand;

pub fn run(command: &ConfigCommand, json: bool) -> Result<(), DubbitError> {
    match command {
        ConfigCommand::Path => {
            println!("{}", paths::config_file().display());
            Ok(())
        }
        ConfigCommand::Show => {
            let settings = Settings::load();
            if json {
                let value = serde_json::to_value(&settings).map_err(|error| {
                    DubbitError::new("DUB-0001", "Settings could not be read")
                        .detail(error.to_string())
                        .hint("Run 'dubbit config path' to find the file.")
                })?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&value).unwrap_or_default()
                );
                return Ok(());
            }
            println!("profile            {}", settings.profile);
            println!(
                "asr-model          {}",
                settings.asr_model.as_deref().unwrap_or("from profile")
            );
            println!("translator         {}", settings.translator);
            println!("ollama-host        {}", settings.ollama_host);
            println!("ollama-model       {}", settings.ollama_model);
            println!(
                "translator-command {}",
                settings.translator_command.as_deref().unwrap_or("not set")
            );
            println!(
                "glossary           {}",
                settings
                    .glossary
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|| "not set".to_string())
            );
            println!("threads            {}", settings.threads);
            Ok(())
        }
        ConfigCommand::Set { key, value } => {
            let mut settings = Settings::load();
            settings.set(key, value)?;
            settings.save()?;
            if !json {
                println!("{key} updated");
            }
            Ok(())
        }
    }
}
