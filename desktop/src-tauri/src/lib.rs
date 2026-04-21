mod db;
mod handlers;
mod utils;

use tauri::Manager;
use utils::voice::{capture, session, speaker, state::init_whisper};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let res = app
                .path()
                .resource_dir()
                .expect("[Peppa] resource dir must exist");

            // Whisper STT model
            init_whisper(res.join("models/ggml-base.en.bin"));

            // Speaker embedding model — optional, silently skips if missing
            let spk_path = res.join("models/wespeaker.onnx");
            if spk_path.exists() {
                speaker::init_speaker(&spk_path);
            } else {
                eprintln!("[Peppa] wespeaker.onnx not found, speaker ID disabled");
            }

            // DB
            tauri::async_runtime::block_on(async {
                match db::init::init_db().await {
                    Ok(info) => {
                        eprintln!("[app] db ready at {}", info.db_path.display());
                        if info.is_first_run {
                            eprintln!("[app] first run — show onboarding");
                            // TODO: emit event to frontend to trigger onboarding flow
                        }
                    }
                    Err(e) => eprintln!("[app] db init failed: {e}"),
                }
            });

            // Shared session — created once, passed into capture
            let session = session::new_session();

            let handle = capture::start(app.handle().clone(), session)
                .unwrap_or_else(|e| panic!("[Peppa] audio capture failed: {e}"));
            Box::leak(Box::new(handle));

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("[Peppa] fatal error during startup")
}
