use std::path::Path;
use std::sync::OnceLock;
use whisper_rs::{WhisperContext, WhisperContextParameters};

static WHISPER_CTX: OnceLock<WhisperContext> = OnceLock::new();

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
        .expect("[state] whisper not initialised — call init_whisper() before starting capture")
}
