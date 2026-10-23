use dubbit_core::config::Settings;
use dubbit_core::diagnostics;
use dubbit_core::error::DubbitError;
use dubbit_core::files::human_size;

use crate::args::DoctorArgs;

pub fn run(args: &DoctorArgs, json: bool) -> Result<(), DubbitError> {
    let settings = Settings::load();
    let report = diagnostics::build(&settings)?;
    if let Some(path) = &args.report {
        report.write(path)?;
    }
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report.to_json()).unwrap_or_default()
        );
        return Ok(());
    }

    println!("Dubbit {} on {} {}", report.version, report.os, report.arch);
    println!("{}", report.hardware.summary());
    println!(
        "Profile: {} ({}) with model {}",
        report.profile.name, report.profile.reason, report.profile.asr_model
    );
    println!();
    for tool in &report.tools {
        match (&tool.location, &tool.version) {
            (Some(location), Some(version)) => {
                println!("{:<12} {version}\n             {location}", tool.name)
            }
            (Some(location), None) => println!("{:<12} {location}", tool.name),
            _ => println!("{:<12} not found", tool.name),
        }
    }
    println!();
    println!("Data folder: {}", report.data_dir);
    println!("Models on disk: {}", human_size(report.installed_bytes));
    for spec in &report.models {
        let state = if spec.installed {
            "installed"
        } else {
            "available"
        };
        println!("  {:<32} {:>9} {state}", spec.id, spec.size);
    }
    if let Some(path) = &args.report {
        println!();
        println!("Report written to {}", path.display());
    }
    Ok(())
}
