use once_cell::sync::Lazy;
use std::env;
use vosk::Model;

pub static VOSK_MODEL: Lazy<Model> = Lazy::new(|| {
    Model::new(env::var("VOSK_DIR").expect("VOSK_DIR must be set."))
        .expect("Failed to load Vosk model")
});
