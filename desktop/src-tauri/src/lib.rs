mod db;
mod handlers;
mod utils;

use tauri::Manager;
use utils::voice::{capture, onboarding, session, speaker, state::init_whisper};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let res = app
                .path()
                .resource_dir()
                .expect("[Peppa] resource dir must exist");

            init_whisper(res.join("models/ggml-base.en.bin"));

            let spk_path = res.join("models/wespeaker.onnx");
            if spk_path.exists() {
                speaker::init_speaker(&spk_path);
            } else {
                eprintln!("[Peppa] wespeaker.onnx not found, speaker ID disabled");
            }

            let is_first_run = tauri::async_runtime::block_on(async {
                match db::init::init_db().await {
                    Ok(info) => {
                        eprintln!("[app] db ready at {}", info.db_path.display());
                        info.is_first_run
                    }
                    Err(e) => {
                        eprintln!("[app] db init failed: {e}");
                        false
                    }
                }
            });

            let session = session::new_session();
            let onboarding = onboarding::new_onboarding();

            if is_first_run {
                onboarding::start(app.handle(), &onboarding);
            }

            let handle = capture::start(app.handle().clone(), session, onboarding)
                .unwrap_or_else(|e| panic!("[Peppa] audio capture failed: {e}"));
            Box::leak(Box::new(handle));

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("[Peppa] fatal error during startup")
}
