use std::path::PathBuf;
use std::sync::OnceLock;
use whisper_rs::{WhisperContext, WhisperContextParameters};

static WHISPER_CTX: OnceLock<WhisperContext> = OnceLock::new();

pub fn init_whisper(model_path: PathBuf) {
    WHISPER_CTX.get_or_init(|| {
        let path = model_path
            .to_str()
            .expect("[Peppa] whisper model path must be valid UTF-8");

        eprintln!("[state] loading whisper model from: {path}");

        WhisperContext::new_with_params(path, WhisperContextParameters::default())
            .unwrap_or_else(|e| panic!("[Peppa] failed to load whisper model: {e}"))
    });
}

pub fn get_whisper() -> &'static WhisperContext {
    WHISPER_CTX
        .get()
        .expect("[Peppa] whisper model not initialised — call init_whisper() at startup")
}
