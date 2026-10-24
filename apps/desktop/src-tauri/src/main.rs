#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod queue;
mod view;

use std::path::PathBuf;

use tauri::Manager;

fn bundled_tools(app: &tauri::App) -> Option<PathBuf> {
    let root = app.path().resource_dir().ok()?;
    [root.join("bin"), root.join("resources").join("bin")]
        .into_iter()
        .find(|candidate| {
            candidate.join("ffmpeg.exe").is_file() || candidate.join("ffmpeg").is_file()
        })
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if std::env::var_os("DUBBIT_TOOLS_DIR").is_none() {
                if let Some(tools) = bundled_tools(app) {
                    std::env::set_var("DUBBIT_TOOLS_DIR", tools);
                }
            }
            let queue = queue::Queue::start(app.handle().clone());
            app.manage(queue);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::environment,
            commands::inspect,
            commands::enqueue,
            commands::jobs,
            commands::job_action,
            commands::open_project,
            commands::save_review,
            commands::install_model,
            commands::reveal,
            commands::data_dir
        ])
        .run(tauri::generate_context!())
        .expect("the application could not start");
}
