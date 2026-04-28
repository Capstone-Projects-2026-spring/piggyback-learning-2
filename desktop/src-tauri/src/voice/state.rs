use ort::session::{builder::GraphOptimizationLevel, Session};
use std::path::Path;
use std::sync::{Mutex, OnceLock};
use whisper_rs::{WhisperContext, WhisperContextParameters};

static WHISPER_CTX: OnceLock<WhisperContext> = OnceLock::new();
static SILERO_SESSION: OnceLock<Mutex<Session>> = OnceLock::new();

pub fn init_whisper(model_path: &Path) {
    WHISPER_CTX.get_or_init(|| {
        let path = model_path
            .to_str()
            .expect("[state] whisper model path must be valid UTF-8");
        eprintln!("[state] loading whisper from {path}");
        WhisperContext::new_with_params(path, WhisperContextParameters::default())
            .unwrap_or_else(|e| panic!("[state] failed to load whisper model: {e}"))
    });
}

pub fn get_whisper() -> &'static WhisperContext {
    WHISPER_CTX
        .get()
        .expect("[state] whisper not initialised - call init_whisper() before starting capture")
}

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
