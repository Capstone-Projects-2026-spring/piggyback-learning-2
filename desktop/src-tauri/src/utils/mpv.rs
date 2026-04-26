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

// IPC socket/pipe path constant so we don't need to lock state just to read it
#[cfg(unix)]
const IPC_PATH: &str = "/tmp/orb-mpv.sock";
#[cfg(windows)]
const IPC_PATH: &str = r"\\.\pipe\orb-mpv";

static MPV_STATE: OnceLock<Mutex<MpvState>> = OnceLock::new();
static MPV_RUNNING: AtomicBool = AtomicBool::new(false);

struct MpvState {
    process: Option<std::process::Child>,
}

impl MpvState {
    fn new() -> Self {
        Self { process: None }
    }
}

fn get_state() -> &'static Mutex<MpvState> {
    MPV_STATE.get_or_init(|| Mutex::new(MpvState::new()))
}

// IPC

fn send_ipc(command: serde_json::Value) -> Result<(), String> {
    let mut msg = command.to_string();
    msg.push('\n');

    #[cfg(unix)]
    {
        let mut stream = UnixStream::connect(IPC_PATH)
            .map_err(|e| format!("[mpv] socket connect failed: {e}"))?;
        stream
            .write_all(msg.as_bytes())
            .map_err(|e| format!("[mpv] socket write failed: {e}"))?;
    }

    #[cfg(windows)]
    {
        use std::fs::OpenOptions;
        let mut pipe = OpenOptions::new()
            .write(true)
            .open(IPC_PATH)
            .map_err(|e| format!("[mpv] pipe open failed: {e}"))?;
        pipe.write_all(msg.as_bytes())
            .map_err(|e| format!("[mpv] pipe write failed: {e}"))?;
    }

    Ok(())
}

fn get_playback_position() -> Option<f64> {
    #[cfg(unix)]
    {
        let mut stream = UnixStream::connect(IPC_PATH).ok()?;
        let cmd = serde_json::json!({
            "command":    ["get_property", "time-pos"],
            "request_id": 1
        });
        let mut msg = cmd.to_string();
        msg.push('\n');
        stream.write_all(msg.as_bytes()).ok()?;
        stream
            .set_read_timeout(Some(Duration::from_millis(200)))
            .ok()?;

        for line in BufReader::new(&stream).lines().flatten() {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) {
                if v["request_id"] == 1 {
                    return v["data"].as_f64();
                }
            }
        }
    }

    None
}

// Public API

pub fn launch_mpv(video_path: &str) -> Result<(), String> {
    let mut state = get_state().lock().map_err(|e| e.to_string())?;

    if let Some(mut child) = state.process.take() {
        let _ = child.kill();
    }

    // Remove stale socket before launching
    #[cfg(unix)]
    let _ = std::fs::remove_file(IPC_PATH);

    let child = std::process::Command::new("mpv")
        .arg(video_path)
        .arg(format!("--input-ipc-server={IPC_PATH}"))
        .arg("--fullscreen")
        .arg("--no-terminal")
        .arg("--keep-open=yes")
        .arg("--pause")
        .spawn()
        .map_err(|e| format!("[mpv] spawn failed: {e}"))?;

    state.process = Some(child);
    MPV_RUNNING.store(true, Ordering::SeqCst);
    eprintln!("[mpv] launched — {video_path}");
    Ok(())
}

/// Poll until the IPC socket/pipe is ready, up to 5 seconds.
pub fn wait_for_socket() -> bool {
    for _ in 0..50 {
        std::thread::sleep(Duration::from_millis(100));

        #[cfg(unix)]
        if std::path::Path::new(IPC_PATH).exists() {
            std::thread::sleep(Duration::from_millis(200));
            eprintln!("[mpv] socket ready");
            return true;
        }

        #[cfg(windows)]
        if std::fs::metadata(IPC_PATH).is_ok() {
            return true;
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
    let _ = std::fs::remove_file(IPC_PATH);

    eprintln!("[mpv] quit");
}

pub fn _is_running() -> bool {
    MPV_RUNNING.load(Ordering::SeqCst)
}

// Position poller

/// Poll mpv playback position every 250ms and emit `orb://mpv-tick`.
/// Fires `orb://segment-end` when playback reaches the end of a segment,
/// then pauses and brings the Tauri window forward for the question UI.
/// `segments` is a list of (start_seconds, end_seconds, segment_id).
pub fn start_position_poller(segments: Vec<(f64, f64, i64)>) {
    std::thread::spawn(move || {
        let mut last_triggered: Option<i64> = None;

        while MPV_RUNNING.load(Ordering::SeqCst) {
            std::thread::sleep(Duration::from_millis(250));

            let Some(pos) = get_playback_position() else {
                continue;
            };

            crate::utils::app_handle::emit(
                "orb://mpv-tick",
                serde_json::json!({ "position": pos }),
            );

            for (start, end, seg_id) in &segments {
                // Only trigger if playback is within this segment and has reached the end
                if pos >= *start && pos >= *end && last_triggered != Some(*seg_id) {
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

                // Reset trigger if user seeks back before segment end
                if pos < *end - 0.5 && last_triggered == Some(*seg_id) {
                    last_triggered = None;
                }
            }
        }

        eprintln!("[mpv] poller stopped");
    });
}
