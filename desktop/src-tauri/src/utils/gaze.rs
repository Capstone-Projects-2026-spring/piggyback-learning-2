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

// When Some, the gaze loop captures one frame and sends it down this channel.
// The oneshot is consumed on fulfillment so each request gets exactly one frame.
static SNAPSHOT_TX: OnceLock<TokioMutex<Option<oneshot::Sender<Vec<u8>>>>> = OnceLock::new();

pub fn init_snapshot_channel() {
    SNAPSHOT_TX.get_or_init(|| TokioMutex::new(None));
}

/// Request a single JPEG frame from the gaze camera.
/// Returns None if the gaze loop isn't running or the response times out.
pub async fn request_snapshot() -> Option<Vec<u8>> {
    let (tx, rx) = oneshot::channel();
    *SNAPSHOT_TX.get()?.lock().await = Some(tx);
    tokio::time::timeout(Duration::from_secs(2), rx)
        .await
        .ok()?
        .ok()
}

static GAZE_SESSION: OnceLock<Mutex<Session>> = OnceLock::new();
static GAZE_RUNNING: AtomicBool = AtomicBool::new(false);
static GAZE_PAUSED: AtomicBool = AtomicBool::new(false);

pub fn init_gaze(model_path: &Path) {
    GAZE_SESSION.get_or_init(|| {
        eprintln!("[gaze] loading ultraface ONNX");
        let session = Session::builder()
            .unwrap()
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .unwrap()
            .commit_from_file(model_path)
            .expect("[gaze] failed to load model");
        eprintln!("[gaze] ready");
        Mutex::new(session)
    });
}

// Internal control

fn start_gaze_loop() {
    if GAZE_RUNNING
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        eprintln!("[gaze] already running");
        return;
    }

    thread::spawn(|| {
        eprintln!("[gaze] loop started");
        gaze_loop();
        GAZE_RUNNING.store(false, Ordering::SeqCst);
        eprintln!("[gaze] loop stopped");
    });
}

// Internal commands

#[tauri::command]
pub fn gaze_start() {
    start_gaze_loop();
}

#[tauri::command]
pub fn gaze_pause() {
    GAZE_PAUSED.store(true, Ordering::SeqCst);
    eprintln!("[gaze] paused");
}

#[tauri::command]
pub fn gaze_resume() {
    GAZE_PAUSED.store(false, Ordering::SeqCst);
    eprintln!("[gaze] resumed");
}

#[tauri::command]
pub fn gaze_stop() {
    GAZE_RUNNING.store(false, Ordering::SeqCst);
    eprintln!("[gaze] stop requested");
}

// Loop constants

const INPUT_W: u32 = 320;
const INPUT_H: u32 = 240;
const CONFIDENCE_THRESHOLD: f32 = 0.7;

/// Consecutive away frames before firing the away event (~2s at 4fps).
const AWAY_FRAMES_THRESHOLD: u32 = 8;
/// Consecutive present frames before firing the return event.
const RETURN_FRAMES_THRESHOLD: u32 = 5;

fn gaze_loop() {
    let cameras = match query(ApiBackend::Auto) {
        Ok(c) if !c.is_empty() => c,
        Ok(_) => {
            eprintln!("[gaze] no cameras found");
            return;
        }
        Err(e) => {
            eprintln!("[gaze] camera query failed: {e}");
            return;
        }
    };

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
        "[gaze] stream open — {}x{}",
        camera.resolution().width(),
        camera.resolution().height(),
    );

    let mut is_away = false;
    let mut away_counter = 0u32;
    let mut return_counter = 0u32;

    while GAZE_RUNNING.load(Ordering::SeqCst) {
        // ~4fps — enough for gaze detection, low CPU usage
        thread::sleep(Duration::from_millis(250));

        if GAZE_PAUSED.load(Ordering::SeqCst) {
            away_counter = 0;
            return_counter = 0;
            if is_away {
                is_away = false;
                emit_gaze("present");
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

        // Fulfill a pending snapshot request before running inference.
        // try_lock is non-blocking so the gaze loop never stalls here.
        try_fulfill_snapshot(&frame, &rgb);

        let face_detected = detect_face(rgb.into_raw(), frame.resolution());

        if !face_detected {
            return_counter = 0;
            away_counter = away_counter.saturating_add(1);
            if !is_away && away_counter >= AWAY_FRAMES_THRESHOLD {
                is_away = true;
                eprintln!("[gaze] away");
                emit_gaze("away");
            }
        } else {
            away_counter = 0;
            return_counter = return_counter.saturating_add(1);
            if is_away && return_counter >= RETURN_FRAMES_THRESHOLD {
                is_away = false;
                eprintln!("[gaze] present");
                emit_gaze("present");
            }
        }
    }

    let _ = camera.stop_stream();
    eprintln!("[gaze] stream closed");
}

fn emit_gaze(status: &str) {
    crate::utils::app_handle::emit("orb://gaze-status", serde_json::json!({ "status": status }));
}

/// If a snapshot was requested, encode the current frame as JPEG and fulfill it.
fn try_fulfill_snapshot(frame: &nokhwa::Buffer, rgb: &RgbImage) {
    let Some(slot) = SNAPSHOT_TX.get() else {
        return;
    };
    let Ok(mut guard) = slot.try_lock() else {
        return;
    };
    if guard.is_none() {
        return;
    }

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
        if let Some(tx) = guard.take() {
            let _ = tx.send(buf.into_inner());
        }
    }
}

// Inference

/// Returns true if a face is detected above the confidence threshold.
/// Defaults to true on any error — assume present rather than falsely pausing.
fn detect_face(raw_rgb: Vec<u8>, resolution: Resolution) -> bool {
    match run_face_detection(raw_rgb, resolution) {
        Ok(detected) => detected,
        Err(e) => {
            eprintln!("[gaze] detection error: {e} — assuming present");
            true
        }
    }
}

fn run_face_detection(raw_rgb: Vec<u8>, resolution: Resolution) -> Result<bool, String> {
    let mut session = GAZE_SESSION
        .get()
        .ok_or("session not loaded")?
        .lock()
        .map_err(|_| "session mutex poisoned")?;

    let img = RgbImage::from_raw(resolution.width(), resolution.height(), raw_rgb)
        .ok_or("failed to build RgbImage from raw bytes")?;

    let resized = image::imageops::resize(&img, INPUT_W, INPUT_H, FilterType::Triangle);

    // Normalise to [-1, 1] and lay out as NCHW [1, 3, H, W]
    let h = INPUT_H as usize;
    let w = INPUT_W as usize;
    let mut input = vec![0f32; 3 * h * w];

    for (i, pixel) in resized.pixels().enumerate() {
        let y = i / w;
        let x = i % w;
        input[0 * h * w + y * w + x] = (pixel[0] as f32 - 127.0) / 128.0;
        input[1 * h * w + y * w + x] = (pixel[1] as f32 - 127.0) / 128.0;
        input[2 * h * w + y * w + x] = (pixel[2] as f32 - 127.0) / 128.0;
    }

    let shape = [1usize, 3, h, w];
    let array = ndarray::Array4::from_shape_vec(shape, input)
        .map_err(|e| format!("tensor shape error: {e}"))?;
    let tensor = Tensor::from_array(array).map_err(|e| format!("tensor error: {e}"))?;

    // Ultraface output: scores [1, N, 2] — index 1 of last dim is face confidence
    let outputs = session
        .run(ort::inputs!["input" => tensor])
        .map_err(|e| format!("inference error: {e}"))?;

    let scores = outputs[0]
        .try_extract_tensor::<f32>()
        .map_err(|e| format!("score extract error: {e}"))?;

    let max_confidence = scores
        .1
        .iter()
        .skip(1)
        .step_by(2)
        .cloned()
        .fold(0.0f32, f32::max);

    eprintln!("[gaze] face confidence: {max_confidence:.3}");
    Ok(max_confidence >= CONFIDENCE_THRESHOLD)
}
