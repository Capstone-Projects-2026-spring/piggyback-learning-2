use image::{imageops::FilterType, GrayImage};
use ort::session::{builder::GraphOptimizationLevel, Session};
use ort::value::Tensor;
use std::{
    path::Path,
    sync::{Mutex, OnceLock},
};

static MOOD_SESSION: OnceLock<Mutex<Session>> = OnceLock::new();

const LABELS: &[&str] = &[
    "neutral",
    "happiness",
    "surprise",
    "sadness",
    "anger",
    "disgust",
    "fear",
    "contempt",
];

/// FER+ model expects 64×64 greyscale input.
const INPUT_SIZE: u32 = 64;

pub fn init_mood(model_path: &Path) {
    MOOD_SESSION.get_or_init(|| {
        eprintln!("[mood] loading emotion-ferplus ONNX");
        let session = Session::builder()
            .unwrap()
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .unwrap()
            .commit_from_file(model_path)
            .expect("[mood] failed to load model - check models/mood/ path");
        eprintln!("[mood] ready");
        Mutex::new(session)
    });
}

/// Run FER+ inference on a raw JPEG/PNG frame and return the dominant emotion label.
/// Always returns a valid label - falls back to "neutral" on any failure since
/// mood is non-critical metadata that should never block the answer pipeline.
pub fn detect_mood_from_frame(frame_bytes: &[u8]) -> String {
    match run_inference(frame_bytes) {
        Ok(label) => label,
        Err(e) => {
            eprintln!("[mood] {e} - defaulting to neutral");
            "neutral".into()
        }
    }
}

/// Internal inference pipeline. Returns Err with a description on any failure
/// so detect_mood_from_frame can log once and return the fallback.
fn run_inference(frame_bytes: &[u8]) -> Result<String, String> {
    let mutex = MOOD_SESSION
        .get()
        .ok_or("not initialised - was init_mood() called?")?;

    let mut session = mutex
        .lock()
        .map_err(|_| "session mutex poisoned".to_string())?;

    // Decode -> greyscale -> resize to 64×64 (FER+ input spec)
    let img = image::load_from_memory(frame_bytes).map_err(|e| format!("decode failed: {e}"))?;

    let gray: GrayImage = image::imageops::resize(
        &img.to_luma8(),
        INPUT_SIZE,
        INPUT_SIZE,
        FilterType::Triangle,
    );

    let pixel_values: Vec<f32> = gray.pixels().map(|p| p[0] as f32 / 255.0).collect();

    let shape = [1usize, 1, INPUT_SIZE as usize, INPUT_SIZE as usize];
    let array = ndarray::Array4::from_shape_vec(shape, pixel_values)
        .map_err(|e| format!("shape error: {e}"))?;

    let tensor = Tensor::from_array(array).map_err(|e| format!("tensor error: {e}"))?;

    let outputs = session
        .run(ort::inputs!["Input3" => tensor])
        .map_err(|e| format!("inference error: {e}"))?;

    let logits = outputs[0]
        .try_extract_tensor::<f32>()
        .map_err(|e| format!("extract error: {e}"))?;

    let (best_idx, best_prob) = softmax_argmax(logits.1.iter().copied());

    let label = LABELS
        .get(best_idx)
        .copied()
        .unwrap_or("neutral")
        .to_string();
    eprintln!("[mood] {label} ({:.1}%)", best_prob * 100.0);
    Ok(label)
}

/// Numerically stable softmax then argmax over an iterator of logits.
/// Returns (index of max probability, max probability).
fn softmax_argmax(logits: impl Iterator<Item = f32>) -> (usize, f32) {
    let values: Vec<f32> = logits.collect();
    let max = values.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let exps: Vec<f32> = values.iter().map(|x| (x - max).exp()).collect();
    let sum: f32 = exps.iter().sum();

    exps.iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .map(|(i, p)| (i, p / sum))
        .unwrap_or((0, 0.0))
}
