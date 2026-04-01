use once_cell::sync::Lazy;
use vosk::Model;

pub static VOSK_MODEL: Lazy<Model> = Lazy::new(|| {
    Model::new("vosk/vosk-model-small-en-us-0.15").expect("Failed to load Vosk model")
});
