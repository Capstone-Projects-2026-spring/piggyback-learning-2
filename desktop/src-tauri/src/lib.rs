mod db;
mod handlers;
mod utils;

use tauri::Manager;
use utils::voice::{
    capture, intent_classifier,
    onboarding::{self, OnboardingFlow},
    session, speaker,
    state::init_whisper,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            handlers::videos::download_video_command,
            handlers::questions::save_questions,
        ])
        .setup(|app| {
            let res = app
                .path()
                .resource_dir()
                .expect("[Peppa] resource dir must exist");

            // Store global app handle first
            utils::app_handle::init_app_handle(app.handle().clone());

            init_whisper(res.join("models/ggml-base.en.bin"));

            let spk_path = res.join("models/wespeaker.onnx");
            if spk_path.exists() {
                speaker::init_speaker(&spk_path);
            } else {
                eprintln!("[Peppa] wespeaker.onnx not found, speaker ID disabled");
            }

            intent_classifier::init_classifier();

            tauri::async_runtime::block_on(async {
                match db::init::init_db().await {
                    Ok(info) => eprintln!("[app] db ready at {}", info.db_path.display()),
                    Err(e) => eprintln!("[app] db init failed: {e}"),
                }
            });

            let session = session::new_session();
            let onboarding = onboarding::new_onboarding();

            let needs_onboarding =
                tauri::async_runtime::block_on(async { !db::init::has_parent_account().await });

            if needs_onboarding {
                let app_handle = app.handle().clone();
                let onboarding_clone = onboarding.clone();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                    eprintln!("[app] emitting onboarding start");
                    onboarding::start(&app_handle, &onboarding_clone, OnboardingFlow::Parent);
                });
            } else {
                eprintln!("[app] parent account exists — skipping onboarding");
            }

            let handle = capture::start(app.handle().clone(), session, onboarding)
                .unwrap_or_else(|e| panic!("[Peppa] audio capture failed: {e}"));

            Box::leak(Box::new(handle));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("[Peppa] fatal error during startup")
}
