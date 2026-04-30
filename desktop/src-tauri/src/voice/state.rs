use ort::session::{builder::GraphOptimizationLevel, Session};
use std::path::Path;
use std::sync::{Mutex, OnceLock};

static SILERO_SESSION: OnceLock<Mutex<Session>> = OnceLock::new();

pub fn init_silero(model_path: &Path) {
    SILERO_SESSION.get_or_init(|| {
        eprintln!("[state] loading silero VAD from {}", model_path.display());
        Session::builder()
            .unwrap()
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .unwrap()
            .commit_from_file(model_path)
            .unwrap_or_else(|e| panic!("[state] failed to load silero model: {e}"))
            .into()
    });
}

pub fn get_silero() -> &'static Mutex<Session> {
    SILERO_SESSION
        .get()
        .expect("[state] silero not initialised - call init_silero() before starting capture")
}
