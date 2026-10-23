use std::path::Path;

use dubbit_core::error::DubbitError;
use dubbit_core::pipeline;
use dubbit_core::project::Workspace;

pub fn run(directory: &Path, json: bool) -> Result<(), DubbitError> {
    let workspace = Workspace::open(directory);
    if !workspace.exists() {
        return Err(DubbitError::new("DUB-3002", "No project in that folder")
            .detail(format!("{} has no project file.", directory.display()))
            .hint("Pass the folder printed after a finished run."));
    }
    let project = workspace.load()?;
    let stages = pipeline::stage_summary(&project);
    let files = pipeline::subtitle_files(&workspace)?;

    if json {
        let payload = serde_json::json!({
            "source": project.source,
            "options": project.options,
            "segments": project.segments.len(),
            "stages": stages
                .iter()
                .map(|(name, done)| serde_json::json!({ "stage": name, "done": done }))
                .collect::<Vec<serde_json::Value>>(),
            "files": files,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).unwrap_or_default()
        );
        return Ok(());
    }

    println!("Source      {}", project.source.path.display());
    println!("Segments    {}", project.segments.len());
    println!(
        "Languages   {} to {}",
        project.options.source_language.as_deref().unwrap_or("auto"),
        project.options.target_language.as_deref().unwrap_or("none")
    );
    println!("Model       {}", project.options.asr_model);
    println!("Translator  {}", project.options.translator);
    println!("Formats     {}", project.options.formats_label());
    println!("Stages");
    for (name, done) in stages {
        println!("  {:<11} {}", name, if done { "done" } else { "pending" });
    }
    if !files.is_empty() {
        println!("Files");
        for file in files {
            println!("  {}", directory.join(file).display());
        }
    }
    Ok(())
}
