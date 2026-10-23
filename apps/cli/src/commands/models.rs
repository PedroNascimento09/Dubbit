use dubbit_core::error::DubbitError;
use dubbit_core::files::human_size;
use dubbit_core::models::{self, ModelSpec};

use crate::args::ModelsCommand;

fn print_table(specs: &[ModelSpec], json: bool) -> Result<(), DubbitError> {
    if json {
        let rows = specs
            .iter()
            .map(|spec| spec.summary())
            .collect::<Vec<models::ModelSummary>>();
        println!(
            "{}",
            serde_json::to_string_pretty(&rows).unwrap_or_default()
        );
        return Ok(());
    }
    for spec in specs {
        let installed = if models::is_installed(spec) {
            "installed"
        } else {
            "not installed"
        };
        println!(
            "{:<32} {:>9} {:>7} MB  {:<12} {:<9} {}",
            spec.id,
            human_size(spec.size),
            spec.ram_mb,
            spec.quality,
            spec.license,
            installed
        );
    }
    Ok(())
}

pub fn run(command: &ModelsCommand, json: bool) -> Result<(), DubbitError> {
    match command {
        ModelsCommand::List => print_table(&models::catalog()?, json),
        ModelsCommand::Install { id } => {
            let spec = models::get(id)?;
            let mut last = 0_u64;
            let mut progress = |received: u64, total: u64| {
                if !json && total > 0 && (received == total || received - last > 8 * 1024 * 1024) {
                    last = received;
                    eprintln!("  {} of {}", human_size(received), human_size(total));
                }
            };
            let path = models::install(&spec, &mut progress)?;
            if json {
                println!(
                    "{}",
                    serde_json::json!({ "model": spec.id, "path": path, "size": spec.size })
                );
            } else {
                println!("{} verified at {}", spec.id, path.display());
            }
            Ok(())
        }
        ModelsCommand::Remove { id } => {
            let spec = models::get(id)?;
            models::remove(&spec)?;
            if !json {
                println!("{} removed", spec.id);
            }
            Ok(())
        }
        ModelsCommand::Verify { id } => {
            let specs = match id {
                Some(id) => vec![models::get(id)?],
                None => models::catalog()?,
            };
            let mut failures = 0;
            for spec in specs {
                let problems = models::verify(&spec);
                if problems.is_empty() {
                    if !json {
                        println!("{} ok", spec.id);
                    }
                } else {
                    failures += 1;
                    for problem in problems {
                        eprintln!("{}: {problem}", spec.id);
                    }
                }
            }
            if failures > 0 {
                return Err(
                    DubbitError::new("DUB-3102", "Some models failed verification")
                        .detail(format!("{failures} model folders need attention."))
                        .hint("Run 'dubbit models install <id>' to download a fresh copy."),
                );
            }
            Ok(())
        }
        ModelsCommand::Path { id } => {
            let spec = models::get(id)?;
            let path = models::model_path(&spec);
            println!("{}", path.display());
            Ok(())
        }
    }
}
