use image::{imageops::FilterType, GrayImage};
use ort::session::{builder::GraphOptimizationLevel, Session};
use ort::value::Tensor;
use std::{path::Path, sync::{Mutex, OnceLock}};

static MOOD_SESSION: OnceLock<Mutex<Session>> = OnceLock::new();

const LABELS: &[&str] = &[
    "neutral", "happiness", "surprise", "sadness",
    "anger", "disgust", "fear", "contempt",
];
const INPUT_SIZE: u32 = 64;

pub fn init_mood(model_path: &Path) {
    MOOD_SESSION.get_or_init(|| {
        eprintln!("[mood] loading emotion-ferplus ONNX...");
        let session = Session::builder()
            .unwrap()
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .unwrap()
            .commit_from_file(model_path)
            .expect("[mood] failed to load emotion-ferplus ONNX");
        eprintln!("[mood] session ready");
        Mutex::new(session)
    });
}

/// Decode a JPEG/PNG frame and return the dominant emotion label.
/// Returns "neutral" on any failure — never hard-errors.
pub fn detect_mood_from_frame(frame_bytes: &[u8]) -> String {
    let Some(mutex) = MOOD_SESSION.get() else {
        eprintln!("[mood] session not initialised");
        return "neutral".into();
    };
    let mut session = match mutex.lock() {
        Ok(s) => s,
        Err(_) => return "neutral".into(),
    };

    let img = match image::load_from_memory(frame_bytes) {
        Ok(i) => i,
        Err(e) => { eprintln!("[mood] decode error: {e}"); return "neutral".into(); }
    };

    let gray: GrayImage = image::imageops::resize(
        &img.to_luma8(), INPUT_SIZE, INPUT_SIZE, FilterType::Triangle,
    );

    let mut input = vec![0f32; (INPUT_SIZE * INPUT_SIZE) as usize];
    for (i, p) in gray.pixels().enumerate() {
        input[i] = p[0] as f32 / 255.0;
    }

    let shape = [1usize, 1, INPUT_SIZE as usize, INPUT_SIZE as usize];
    let array = match ndarray::Array4::from_shape_vec(shape, input) {
        Ok(a) => a,
        Err(e) => { eprintln!("[mood] shape error: {e}"); return "neutral".into(); }
    };
    let tensor = match Tensor::from_array(array) {
        Ok(t) => t,
        Err(e) => { eprintln!("[mood] tensor error: {e}"); return "neutral".into(); }
    };

    let outputs = match session.run(ort::inputs!["Input3" => tensor]) {
        Ok(o) => o,
        Err(e) => { eprintln!("[mood] inference error: {e}"); return "neutral".into(); }
    };

    let logits = match outputs[0].try_extract_tensor::<f32>() {
        Ok(t) => t,
        Err(e) => { eprintln!("[mood] extract error: {e}"); return "neutral".into(); }
    };

    // Softmax over logits
    let s = logits.1;
    let max = s.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let exps: Vec<f32> = s.iter().map(|x| (x - max).exp()).collect();
    let sum: f32 = exps.iter().sum();
    let probs: Vec<f32> = exps.iter().map(|x| x / sum).collect();

    let (best_idx, best_prob) = probs.iter().enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .map(|(i, p)| (i, *p))
        .unwrap_or((0, 0.0));

    let label = LABELS.get(best_idx).unwrap_or(&"neutral").to_string();
    eprintln!("[mood] detected={label} confidence={best_prob:.3}");
    label
}
