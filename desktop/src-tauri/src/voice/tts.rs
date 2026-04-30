use crate::voice::session::{SessionMode, SharedSession};
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::Mutex;

pub struct TtsState(pub Mutex<Option<String>>);

pub fn init(res: &std::path::Path) -> TtsState {
    let model_path = res.join("models/en_GB-alba-medium.onnx");
    if !model_path.exists() {
        eprintln!("[tts] Alba model not found at {}", model_path.display());
        return TtsState(Mutex::new(None));
    }
    if !which_piper() {
        eprintln!("[tts] piper-tts binary not found in PATH");
        return TtsState(Mutex::new(None));
    }
    eprintln!("[tts] using Piper Alba - {}", model_path.display());
    TtsState(Mutex::new(Some(model_path.to_string_lossy().to_string())))
}

fn which_piper() -> bool {
    Command::new("which")
        .arg("piper-tts")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn kill_tts() {
    Command::new("pkill")
        .args(["-9", "piper-tts"])
        .status()
        .ok();
    Command::new("pkill").args(["-9", "aplay"]).status().ok();
}

#[tauri::command]
pub fn speak(text: String, state: tauri::State<TtsState>, session: tauri::State<SharedSession>) {
    let guard = state.0.lock().unwrap();
    let Some(model) = guard.clone() else {
        eprintln!("[tts] speak called but TTS unavailable");
        return;
    };
    drop(guard);

    let session = session.inner().clone();

    std::thread::spawn(move || {
        // Kill any currently running TTS before starting a new one.
        // This prevents two aplay processes running simultaneously.
        kill_tts();

        session.lock().unwrap().mode = SessionMode::Tts;
        eprintln!("[tts] entering TTS mode");

        let mut child = match Command::new("piper-tts")
            .args(["--model", &model, "--output_raw"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[tts] piper-tts spawn failed: {e}");
                session.lock().unwrap().mode = SessionMode::Command;
                return;
            }
        };

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(text.as_bytes()).ok();
        }

        if let Some(stdout) = child.stdout.take() {
            Command::new("aplay")
                .args(["-r", "22050", "-f", "S16_LE", "-t", "raw", "-"])
                .stdin(stdout)
                .stderr(Stdio::null())
                .status()
                .ok();
        }

        child.wait().ok();

        std::thread::sleep(std::time::Duration::from_millis(300));
        session.lock().unwrap().mode = SessionMode::Command;
        eprintln!("[tts] exiting TTS mode");
    });
}

#[tauri::command]
pub fn stop_speaking(_state: tauri::State<TtsState>, session: tauri::State<SharedSession>) {
    kill_tts();
    session.lock().unwrap().mode = SessionMode::Command;
}
