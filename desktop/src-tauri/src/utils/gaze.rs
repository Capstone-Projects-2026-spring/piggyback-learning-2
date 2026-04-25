use image::{imageops::FilterType, RgbImage};
use nokhwa::{
    pixel_format::RgbFormat,
    query,
    utils::{
        ApiBackend, CameraFormat, CameraIndex, FrameFormat, RequestedFormat, RequestedFormatType,
        Resolution,
    },
    Camera,
};
use ort::session::{builder::GraphOptimizationLevel, Session};
use ort::value::Tensor;
use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex, OnceLock,
    },
    thread,
    time::Duration,
};
use tokio::sync::{oneshot, Mutex as TokioMutex};

// When Some, the gaze loop will capture one frame and send it down this channel
static SNAPSHOT_TX: OnceLock<TokioMutex<Option<oneshot::Sender<Vec<u8>>>>> = OnceLock::new();

pub fn init_snapshot_channel() {
    SNAPSHOT_TX.get_or_init(|| TokioMutex::new(None));
}

/// Request a single JPEG frame from the gaze camera.
/// Returns None if the gaze loop isn't running or times out.
pub async fn request_snapshot() -> Option<Vec<u8>> {
    let (tx, rx) = oneshot::channel();
    {
        let slot = SNAPSHOT_TX.get()?;
        *slot.lock().await = Some(tx);
    }
    tokio::time::timeout(std::time::Duration::from_secs(2), rx)
        .await
        .ok()?
        .ok()
}

// ── Session ───────────────────────────────────────────────────────────────────

static GAZE_SESSION: OnceLock<Mutex<Session>> = OnceLock::new();
static GAZE_RUNNING: AtomicBool = AtomicBool::new(false);
static GAZE_PAUSED: AtomicBool = AtomicBool::new(false);

pub fn init_gaze(model_path: &Path) {
    GAZE_SESSION.get_or_init(|| {
        eprintln!("[gaze] loading ultraface ONNX...");
        let session = Session::builder()
            .unwrap()
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .unwrap()
            .commit_from_file(model_path)
            .expect("[gaze] failed to load ultraface ONNX");
        eprintln!("[gaze] session ready");
        Mutex::new(session)
    });
}

// ── Public control ────────────────────────────────────────────────────────────

/// Start the gaze tracking loop in a background thread.
/// Safe to call multiple times — only starts once.
pub fn start_gaze_loop() {
    if GAZE_RUNNING
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        eprintln!("[gaze] loop already running");
        return;
    }

    thread::spawn(|| {
        eprintln!("[gaze] background loop started");
        gaze_loop();
        GAZE_RUNNING.store(false, Ordering::SeqCst);
        eprintln!("[gaze] background loop stopped");
    });
}

pub fn pause_gaze() {
    GAZE_PAUSED.store(true, Ordering::SeqCst);
}

pub fn resume_gaze() {
    GAZE_PAUSED.store(false, Ordering::SeqCst);
}

pub fn stop_gaze() {
    GAZE_RUNNING.store(false, Ordering::SeqCst);
}

// ── Tauri commands ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn gaze_start() {
    start_gaze_loop();
    eprintln!("[gaze] started");
}

#[tauri::command]
pub fn gaze_pause() {
    pause_gaze();
    eprintln!("[gaze] paused");
}

#[tauri::command]
pub fn gaze_resume() {
    resume_gaze();
    eprintln!("[gaze] resumed");
}

#[tauri::command]
pub fn gaze_stop() {
    stop_gaze();
    eprintln!("[gaze] stopped");
}

// ── Loop ──────────────────────────────────────────────────────────────────────

const INPUT_W: u32 = 320;
const INPUT_H: u32 = 240;
const CONFIDENCE_THRESHOLD: f32 = 0.7;

// How many consecutive away/present frames before we fire the event
// to avoid flapping on single-frame noise
const AWAY_FRAMES_THRESHOLD: u32 = 8; // ~2s at 4fps
const RETURN_FRAMES_THRESHOLD: u32 = 5;

fn gaze_loop() {
    // Find first available camera
    let cameras = match query(ApiBackend::Auto) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[gaze] camera query failed: {e}");
            return;
        }
    };

    if cameras.is_empty() {
        eprintln!("[gaze] no cameras found");
        return;
    }

    eprintln!("[gaze] found {} camera(s), using first", cameras.len());

    let format = RequestedFormat::new::<RgbFormat>(RequestedFormatType::Closest(
        CameraFormat::new(Resolution::new(640, 480), FrameFormat::MJPEG, 30),
    ));
    let mut camera = match Camera::new(CameraIndex::Index(0), format) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[gaze] failed to open camera: {e}");
            return;
        }
    };

    if let Err(e) = camera.open_stream() {
        eprintln!("[gaze] failed to open stream: {e}");
        return;
    }

    eprintln!(
        "[gaze] stream open — format={:?} resolution={}x{}",
        camera.frame_format(),
        camera.resolution().width(),
        camera.resolution().height(),
    );

    let mut is_away = false;
    let mut away_counter: u32 = 0;
    let mut return_counter: u32 = 0;

    while GAZE_RUNNING.load(Ordering::SeqCst) {
        // Run at ~4fps — enough for gaze detection, low CPU usage
        thread::sleep(Duration::from_millis(250));

        if GAZE_PAUSED.load(Ordering::SeqCst) {
            // While paused, reset counters and treat as present
            away_counter = 0;
            return_counter = 0;
            if is_away {
                is_away = false;
                crate::utils::app_handle::emit(
                    "orb://gaze-status",
                    serde_json::json!({ "status": "present" }),
                );
            }
            continue;
        }

        let frame = match camera.frame() {
            Ok(f) => f,
            Err(e) => {
                eprintln!("[gaze] frame error: {e}");
                continue;
            }
        };

        let rgb = match frame.decode_image::<RgbFormat>() {
            Ok(img) => img,
            Err(e) => {
                eprintln!("[gaze] decode error: {e}");
                continue;
            }
        };

        // ── Snapshot request fulfillment ──────────────────────────────────
        if let Some(slot) = SNAPSHOT_TX.get() {
            // Non-blocking try — don't hold up the gaze loop
            if let Ok(mut guard) = slot.try_lock() {
                if guard.is_some() {
                    let dyn_img = image::DynamicImage::ImageRgb8(
                        image::RgbImage::from_raw(
                            frame.resolution().width(),
                            frame.resolution().height(),
                            rgb.as_raw().to_vec(),
                        )
                        .unwrap(),
                    );
                    let mut buf = std::io::Cursor::new(Vec::new());
                    if dyn_img.write_to(&mut buf, image::ImageFormat::Jpeg).is_ok() {
                        // take() so we only fulfill once
                        if let Some(tx) = guard.take() {
                            let _ = tx.send(buf.into_inner());
                        }
                    }
                }
            }
        }

        let face_detected = detect_face(rgb.into_raw(), frame.resolution());

        if !face_detected {
            return_counter = 0;
            away_counter = away_counter.saturating_add(1);

            if !is_away && away_counter >= AWAY_FRAMES_THRESHOLD {
                is_away = true;
                eprintln!("[gaze] look away detected");
                crate::utils::app_handle::emit(
                    "orb://gaze-status",
                    serde_json::json!({ "status": "away" }),
                );
            }
        } else {
            away_counter = 0;
            return_counter = return_counter.saturating_add(1);

            if is_away && return_counter >= RETURN_FRAMES_THRESHOLD {
                is_away = false;
                eprintln!("[gaze] return detected");
                crate::utils::app_handle::emit(
                    "orb://gaze-status",
                    serde_json::json!({ "status": "present" }),
                );
            }
        }
    }

    let _ = camera.stop_stream();
    eprintln!("[gaze] camera stream closed");
}

// ── Inference ─────────────────────────────────────────────────────────────────

fn detect_face(raw_rgb: Vec<u8>, resolution: Resolution) -> bool {
    let Some(mutex) = GAZE_SESSION.get() else {
        return true; // session not loaded, assume present
    };
    let mut session = match mutex.lock() {
        Ok(s) => s,
        Err(_) => return true,
    };

    // Build RgbImage from raw bytes
    let img = match RgbImage::from_raw(resolution.width(), resolution.height(), raw_rgb) {
        Some(i) => i,
        None => return true,
    };

    // Resize to ultraface input size (320x240)
    let resized = image::imageops::resize(&img, INPUT_W, INPUT_H, FilterType::Triangle);

    // Normalise to [-1, 1] and build [1, 3, H, W] NCHW tensor
    let mut input = vec![0f32; (3 * INPUT_H * INPUT_W) as usize];
    for (i, pixel) in resized.pixels().enumerate() {
        let r = (pixel[0] as f32 - 127.0) / 128.0;
        let g = (pixel[1] as f32 - 127.0) / 128.0;
        let b = (pixel[2] as f32 - 127.0) / 128.0;
        let h = INPUT_H as usize;
        let w = INPUT_W as usize;
        let y = i / w;
        let x = i % w;
        input[0 * h * w + y * w + x] = r;
        input[1 * h * w + y * w + x] = g;
        input[2 * h * w + y * w + x] = b;
    }

    let shape = [1usize, 3, INPUT_H as usize, INPUT_W as usize];
    let tensor = match ndarray::Array4::from_shape_vec(shape, input) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("[gaze] tensor shape error: {e}");
            return true;
        }
    };

    let tensor = match Tensor::from_array(tensor) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("[gaze] tensor error: {e}");
            return true;
        }
    };

    // Ultraface outputs: scores [1, N, 2], boxes [1, N, 4]
    // scores[:, :, 1] is the face confidence
    let outputs = match session.run(ort::inputs!["input" => tensor]) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("[gaze] inference error: {e}");
            return true;
        }
    };

    let scores = match outputs[0].try_extract_tensor::<f32>() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[gaze] score extract error: {e}");
            return true;
        }
    };

    // scores shape is [1, N, 2] — face confidence is index 1 of last dim
    let scores_slice = scores.1;
    let max_confidence = scores_slice
        .iter()
        .skip(1) // start at index 1 (face class)
        .step_by(2) // every other value is face confidence
        .cloned()
        .fold(0.0f32, f32::max);

    eprintln!("[gaze] max face confidence: {max_confidence:.3}");
    max_confidence >= CONFIDENCE_THRESHOLD
}
