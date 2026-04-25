use std::{
    io::{BufRead, BufReader, Write},
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex, OnceLock,
    },
    time::Duration,
};

#[cfg(unix)]
use std::os::unix::net::UnixStream;

static MPV_STATE: OnceLock<Mutex<MpvState>> = OnceLock::new();
static MPV_RUNNING: AtomicBool = AtomicBool::new(false);

struct MpvState {
    process: Option<std::process::Child>,
    #[cfg(unix)]
    socket_path: String,
    #[cfg(windows)]
    pipe_name: String,
}

impl MpvState {
    fn new() -> Self {
        Self {
            process: None,
            #[cfg(unix)]
            socket_path: "/tmp/orb-mpv.sock".to_string(),
            #[cfg(windows)]
            pipe_name: r"\\.\pipe\orb-mpv".to_string(),
        }
    }
}

fn get_state() -> &'static Mutex<MpvState> {
    MPV_STATE.get_or_init(|| Mutex::new(MpvState::new()))
}

// ── IPC ───────────────────────────────────────────────────────────────────────

fn send_ipc(command: serde_json::Value) -> Result<(), String> {
    let state = get_state().lock().map_err(|e| e.to_string())?;

    #[cfg(unix)]
    {
        let mut stream = UnixStream::connect(&state.socket_path)
            .map_err(|e| format!("[mpv] socket connect failed: {e}"))?;
        let mut msg = command.to_string();
        msg.push('\n');
        stream
            .write_all(msg.as_bytes())
            .map_err(|e| format!("[mpv] socket write failed: {e}"))?;
    }

    #[cfg(windows)]
    {
        use std::fs::OpenOptions;
        use std::os::windows::fs::OpenOptionsExt;
        let mut pipe = OpenOptions::new()
            .write(true)
            .open(&state.pipe_name)
            .map_err(|e| format!("[mpv] pipe open failed: {e}"))?;
        let mut msg = command.to_string();
        msg.push('\n');
        pipe.write_all(msg.as_bytes())
            .map_err(|e| format!("[mpv] pipe write failed: {e}"))?;
    }

    Ok(())
}

fn get_playback_position() -> Option<f64> {
    let state = get_state().lock().ok()?;

    #[cfg(unix)]
    {
        let mut stream = UnixStream::connect(&state.socket_path).ok()?;
        let cmd = serde_json::json!({
            "command": ["get_property", "time-pos"],
            "request_id": 1
        });
        let mut msg = cmd.to_string();
        msg.push('\n');
        stream.write_all(msg.as_bytes()).ok()?;
        stream
            .set_read_timeout(Some(Duration::from_millis(200)))
            .ok()?;
        let reader = BufReader::new(&stream);
        for line in reader.lines().flatten() {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) {
                if v["request_id"] == 1 {
                    return v["data"].as_f64();
                }
            }
        }
    }

    None
}

// ── Public API ────────────────────────────────────────────────────────────────

pub fn launch_mpv(video_path: &str) -> Result<(), String> {
    let mut state = get_state().lock().map_err(|e| e.to_string())?;

    // Kill existing process if any
    if let Some(mut child) = state.process.take() {
        let _ = child.kill();
    }

    #[cfg(unix)]
    let socket_arg = format!("--input-ipc-server={}", state.socket_path);
    #[cfg(windows)]
    let socket_arg = format!("--input-ipc-server={}", state.pipe_name);

    // Remove stale socket
    #[cfg(unix)]
    let _ = std::fs::remove_file(&state.socket_path);

    let child = std::process::Command::new("mpv")
        .arg(video_path)
        .arg(&socket_arg)
        .arg("--fullscreen")
        .arg("--no-terminal")
        .arg("--keep-open=yes")
        .arg("--pause")
        .spawn()
        .map_err(|e| format!("[mpv] spawn failed: {e}"))?;

    state.process = Some(child);
    MPV_RUNNING.store(true, Ordering::SeqCst);
    eprintln!("[mpv] launched for {video_path}");

    Ok(())
}

pub fn wait_for_socket() -> bool {
    let state = get_state().lock().unwrap();

    #[cfg(unix)]
    let path = state.socket_path.clone();
    drop(state);

    // Poll for up to 5 seconds
    for _ in 0..50 {
        std::thread::sleep(Duration::from_millis(100));
        #[cfg(unix)]
        if std::path::Path::new(&path).exists() {
            // Give mpv a moment after socket appears
            std::thread::sleep(Duration::from_millis(200));
            eprintln!("[mpv] socket ready");
            return true;
        }
        #[cfg(windows)]
        {
            let state = get_state().lock().unwrap();
            if std::fs::metadata(&state.pipe_name).is_ok() {
                return true;
            }
        }
    }
    eprintln!("[mpv] socket timeout");
    false
}

pub fn play() -> Result<(), String> {
    send_ipc(serde_json::json!({ "command": ["set_property", "pause", false] }))
}

pub fn pause() -> Result<(), String> {
    send_ipc(serde_json::json!({ "command": ["set_property", "pause", true] }))
}

pub fn seek(seconds: f64) -> Result<(), String> {
    send_ipc(serde_json::json!({ "command": ["seek", seconds, "absolute"] }))
}

pub fn minimize() -> Result<(), String> {
    send_ipc(serde_json::json!({ "command": ["set_property", "window-minimized", true] }))
}

pub fn restore() -> Result<(), String> {
    send_ipc(serde_json::json!({ "command": ["set_property", "window-minimized", false] }))
}

pub fn quit() {
    let _ = send_ipc(serde_json::json!({ "command": ["quit"] }));
    MPV_RUNNING.store(false, Ordering::SeqCst);
    if let Ok(mut state) = get_state().lock() {
        if let Some(mut child) = state.process.take() {
            let _ = child.kill();
        }
    }
    #[cfg(unix)]
    {
        if let Ok(state) = get_state().lock() {
            let _ = std::fs::remove_file(&state.socket_path);
        }
    }
    eprintln!("[mpv] quit");
}

pub fn is_running() -> bool {
    MPV_RUNNING.load(Ordering::SeqCst)
}

// ── Polling loop ──────────────────────────────────────────────────────────────

/// Starts a background thread that polls mpv playback position
/// and emits orb://mpv-tick every 250ms so the frontend can
/// drive segment-end logic without needing a YouTube player object
pub fn start_position_poller(segments: Vec<(f64, f64, i64)>) {
    // segments: Vec<(start_seconds, end_seconds, segment_id)>
    std::thread::spawn(move || {
        let mut last_triggered: Option<i64> = None;

        while MPV_RUNNING.load(Ordering::SeqCst) {
            std::thread::sleep(Duration::from_millis(250));

            let Some(pos) = get_playback_position() else {
                continue;
            };

            // Emit current position for piggy countdown
            crate::utils::app_handle::emit(
                "orb://mpv-tick",
                serde_json::json!({ "position": pos }),
            );

            // Check segment boundaries
            for (start, end, seg_id) in &segments {
                if pos >= *end && last_triggered != Some(*seg_id) {
                    last_triggered = Some(*seg_id);
                    eprintln!("[mpv] segment end — id={seg_id} pos={pos:.2}");
                    let _ = pause();
                    let _ = minimize();
                    crate::handlers::videos::bring_tauri_to_front();
                    crate::utils::app_handle::emit(
                        "orb://segment-end",
                        serde_json::json!({ "segment_id": seg_id, "position": pos }),
                    );
                    break;
                }
                // Reset trigger if mpv seeked back before segment end
                if pos < *end - 0.5 && last_triggered == Some(*seg_id) {
                    last_triggered = None;
                }
            }
        }

        eprintln!("[mpv] position poller stopped");
    });
}
