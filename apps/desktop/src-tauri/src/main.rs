#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod queue;
mod view;

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
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
            commands::data_dir
        ])
        .run(tauri::generate_context!())
        .expect("the application could not start");
}
