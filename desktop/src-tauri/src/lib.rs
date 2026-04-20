mod commands;
mod utils;

use tauri::Manager;
use utils::voice::state::init_model;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let model_path = app
                .path()
                .resource_dir()
                .expect("[Peppa] resource dir must exist")
                .join("models/vosk-lgraph-model");

            init_model(model_path);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![commands::voice::process_audio,])
        .run(tauri::generate_context!())
        .expect("[Peppa] fatal error during startup")
}
