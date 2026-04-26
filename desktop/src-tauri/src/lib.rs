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

fn load_models(res: &std::path::Path) {
    init_whisper(&res.join("models/ggml-base.en.bin"));

    let models: &[(&str, fn(&std::path::Path))] = &[
        ("models/wespeaker.onnx", |p| speaker::init_speaker(p)),
        ("models/ultraface.onnx", |p| utils::gaze::init_gaze(p)),
        ("models/emotion-ferplus-8.onnx", |p| {
            utils::mood::init_mood(p)
        }),
    ];

    for (relative, init_fn) in models {
        let path = res.join(relative);
        if path.exists() {
            init_fn(&path);
            eprintln!("[app] loaded {relative}");
        } else {
            eprintln!("[app] {relative} not found — feature disabled");
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            handlers::answers::set_answer_context,
            handlers::answers::clear_answer_context,
            handlers::answers::get_answers,
            handlers::videos::download_video_command,
            handlers::questions::save_questions,
            handlers::questions::get_segments,
            handlers::videos::launch_video,
            handlers::videos::mpv_play,
            handlers::videos::mpv_pause,
            handlers::videos::mpv_seek,
            handlers::videos::mpv_minimize,
            handlers::videos::mpv_quit,
            utils::gaze::gaze_start,
            utils::gaze::gaze_stop,
            utils::gaze::gaze_pause,
            utils::gaze::gaze_resume,
        ])
        .setup(|app| {
            let res = app
                .path()
                .resource_dir()
                .expect("[app] resource dir must exist");

            // init once
            utils::app_handle::init_app_handle(app.handle().clone());
            load_models(&res);
            utils::gaze::init_snapshot_channel();
            intent_classifier::init_classifier();
            utils::openai::init_openai();

            tauri::async_runtime::block_on(async {
                match db::init::init_db().await {
                    Ok(db_path) => eprintln!("[app] db ready at {}", db_path.display()),
                    Err(e) => panic!("[app] db init failed: {e}"),
                }
            });

            let session = session::new_session();
            let onboarding = onboarding::new_onboarding();

            handlers::videos::init_session(session.clone());
            app.manage(session.clone());

            let needs_onboarding =
                tauri::async_runtime::block_on(async { !db::init::has_parent_account().await });

            if needs_onboarding {
                let onboarding_clone = onboarding.clone();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                    eprintln!("[app] starting parent onboarding");
                    onboarding::start(&onboarding_clone, OnboardingFlow::Parent);
                });
            } else {
                eprintln!("[app] parent account exists — skipping onboarding");
            }

            let handle = capture::start(session, onboarding)
                .unwrap_or_else(|e| panic!("[app] audio capture failed: {e}"));
            Box::leak(Box::new(handle));

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("[app] fatal error during startup")
}
