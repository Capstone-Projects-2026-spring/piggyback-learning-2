use std::path::PathBuf;
use std::sync::OnceLock;
use vosk::Model;

static VOSK_MODEL: OnceLock<Model> = OnceLock::new();

/// Call once at app startup with the resolved bundled lgraph model path.
/// Panics on failure — Peppa cannot function without a working ASR model.
pub fn init_model(model_path: PathBuf) {
    VOSK_MODEL.get_or_init(|| {
        let path_str = model_path
            .to_str()
            .expect("Vosk lgraph model path must be valid UTF-8");

        println!("[Peppa] Loading Vosk lgraph model from: {path_str}");

        Model::new(path_str).unwrap_or_else(|| {
            panic!(
                "[Peppa] Failed to load Vosk lgraph model from '{path_str}'.\n\
                 Did you run scripts/fetch-assets.sh (or .ps1)?"
            )
        })
    });
}

pub fn get_model() -> &'static Model {
    VOSK_MODEL
        .get()
        .expect("[Peppa] Vosk model not initialised — call init_model() at startup")
}
