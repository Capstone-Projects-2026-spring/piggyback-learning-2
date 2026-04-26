use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

pub static TTS_ACTIVE: AtomicBool = AtomicBool::new(false);

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

    eprintln!("[tts] using Piper Alba — {}", model_path.display());
    TtsState(Mutex::new(Some(model_path.to_string_lossy().to_string())))
}

fn which_piper() -> bool {
    Command::new("which")
        .arg("piper-tts")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[tauri::command]
pub fn speak(text: String, state: tauri::State<TtsState>) {
    let guard = state.0.lock().unwrap();
    let Some(model) = guard.clone() else {
        eprintln!("[tts] speak called but TTS unavailable");
        return;
    };
    drop(guard);

    std::thread::spawn(move || {
        TTS_ACTIVE.store(true, Ordering::SeqCst);

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
                TTS_ACTIVE.store(false, Ordering::SeqCst);
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

        // Small buffer after speech ends so the mic doesn't catch reverb
        std::thread::sleep(std::time::Duration::from_millis(300));
        TTS_ACTIVE.store(false, Ordering::SeqCst);
    });
}

#[tauri::command]
pub fn stop_speaking(_state: tauri::State<TtsState>) {
    Command::new("pkill").arg("piper-tts").status().ok();
    Command::new("pkill").arg("aplay").status().ok();
    TTS_ACTIVE.store(false, Ordering::SeqCst);
}
