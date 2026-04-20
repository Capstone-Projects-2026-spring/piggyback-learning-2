mod utils;

use tauri::Manager;
use utils::voice::{capture, speaker, state::init_whisper};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let res = app
                .path()
                .resource_dir()
                .expect("[Peppa] resource dir must exist");

            // Whisper STT model
            init_whisper(res.join("models/ggml-base.en.bin"));

            // Speaker embedding model — optional, silently skips if missing
            let spk_path = res.join("models/wespeaker.onnx");
            if spk_path.exists() {
                speaker::init_speaker(&spk_path);
            } else {
                eprintln!("[Peppa] wespeaker.onnx not found, speaker ID disabled");
            }

            let handle = capture::start(app.handle().clone())
                .unwrap_or_else(|e| panic!("[Peppa] audio capture failed: {e}"));
            Box::leak(Box::new(handle));

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("[Peppa] fatal error during startup")
}
