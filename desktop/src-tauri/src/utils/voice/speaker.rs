use ndarray::Array2;
use ort::session::{builder::GraphOptimizationLevel, Session};
use ort::value::Tensor;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

static SPEAKER_SESSION: OnceLock<Mutex<Session>> = OnceLock::new();

pub fn init_speaker(model_path: &Path) {
    SPEAKER_SESSION.get_or_init(|| {
        eprintln!("[speaker] building session builder...");
        let builder = Session::builder().unwrap();
        eprintln!("[speaker] setting optimization level...");
        let mut builder = builder
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .unwrap();
        eprintln!("[speaker] committing model from file (this may take a moment)...");
        let session = builder
            .commit_from_file(model_path)
            .expect("[speaker] failed to load wespeaker ONNX");
        eprintln!("[speaker] session ready");
        Mutex::new(session)
    });
}

pub fn extract_embedding(samples: &[f32]) -> Option<Vec<f32>> {
    let session = SPEAKER_SESSION.get()?;
    let mut session = session.lock().ok()?;

    let input = Array2::from_shape_vec((1, samples.len()), samples.to_vec()).ok()?;
    let tensor = Tensor::from_array(input).ok()?;
    let outputs = session.run(ort::inputs!["feats" => tensor]).ok()?;

    let embedding: Vec<f32> = outputs[0].try_extract_tensor::<f32>().ok()?.1.to_vec();

    eprintln!("[speaker] embedding dim={}", embedding.len());
    Some(embedding)
}

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

pub const SPEAKER_THRESHOLD: f32 = 0.75;
