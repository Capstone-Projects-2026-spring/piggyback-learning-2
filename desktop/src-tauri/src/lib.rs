mod db;
mod handlers;
mod utils;
mod voice;

use std::path::Path;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tauri::Manager;
use voice::{
    capture, intent_classifier, moonshine,
    onboarding::{self, OnboardingFlow, SharedOnboarding},
    session, speaker,
    state::init_silero,
    tts,
};

#[derive(Clone)]
struct Handshake {
    backend_ready: Arc<AtomicBool>,
    needs_onboarding: Arc<AtomicBool>,
}

impl Handshake {
    fn new(needs_onboarding: bool) -> Self {
        Self {
            backend_ready: Arc::new(AtomicBool::new(false)),
            needs_onboarding: Arc::new(AtomicBool::new(needs_onboarding)),
        }
    }
}

#[tauri::command]
fn is_backend_ready(handshake: tauri::State<Handshake>) -> bool {
    handshake.backend_ready.load(Ordering::SeqCst)
}

#[tauri::command]
fn frontend_ready(
    handshake: tauri::State<Handshake>,
    onboarding: tauri::State<SharedOnboarding>,
) -> bool {
    eprintln!("[app] frontend_ready received");
    if handshake.needs_onboarding.load(Ordering::SeqCst) {
        let onboarding_clone = onboarding.inner().clone();
        tauri::async_runtime::spawn(async move {
            eprintln!("[app] starting parent onboarding");
            onboarding::start(&onboarding_clone, OnboardingFlow::Parent);
        });
    }
    handshake.needs_onboarding.load(Ordering::SeqCst)
}

fn load_models(res: &Path) {
    let moonshine_dir = res.join("models/moonshine-base");
    if moonshine_dir.exists() {
        moonshine::init_moonshine(&moonshine_dir);
        eprintln!("[app] loaded moonshine-base");
    } else {
        eprintln!("[app] moonshine-base not found - STT disabled");
    }

    let models: &[(&str, fn(&Path))] = &[
        ("models/silero_vad.onnx", |p| init_silero(p)),
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
            eprintln!("[app] {relative} not found - feature disabled");
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            is_backend_ready,
            frontend_ready,
            handlers::answers::set_answer_context,
            handlers::answers::clear_answer_context,
            handlers::videos::download_video_command,
            handlers::questions::save_questions,
            handlers::questions::get_segments,
            handlers::videos::launch_video,
            handlers::videos::mpv_play,
            handlers::videos::mpv_pause,
            handlers::videos::mpv_seek,
            handlers::videos::mpv_minimize,
            handlers::videos::mpv_quit,
            tts::speak,
            tts::stop_speaking,
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

            app.manage(tts::init(&res));

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
            app.manage(onboarding.clone());

            let needs_onboarding =
                tauri::async_runtime::block_on(async { !db::init::has_parent_account().await });

            app.manage(Handshake::new(needs_onboarding));

            let handle = capture::start(session, onboarding)
                .unwrap_or_else(|e| panic!("[app] audio capture failed: {e}"));
            Box::leak(Box::new(handle));

            // Signal frontend that backend is ready.
            utils::app_handle::emit("orb://ready", ());
            // Also set the flag for polling fallback.
            app.state::<Handshake>()
                .backend_ready
                .store(true, Ordering::SeqCst);
            eprintln!("[app] backend ready");

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("[app] fatal error during startup")
}
