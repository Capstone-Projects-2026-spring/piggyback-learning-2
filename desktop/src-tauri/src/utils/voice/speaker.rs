use crate::db::init::{get_db, get_voice_key};
use crate::utils::crypto;
use crate::utils::voice::session::SharedSession;

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

const MATCH_THRESHOLD: f32 = 0.75;

pub async fn identify_speaker(embedding: &[f32], session: &SharedSession) {
    let pool = get_db();
    let key = get_voice_key();

    let rows = match sqlx::query_as::<_, (i64, String, String, Option<i64>, Option<Vec<u8>>)>(
        "SELECT id, name, role, parent_id, voice_embedding FROM users WHERE voice_embedding IS NOT NULL",
    )
    .fetch_all(pool)
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[speaker] db query failed: {e}");
            return;
        }
    };

    let mut best_id: Option<i64> = None;
    let mut best_name = String::new();
    let mut best_role = String::new();
    let mut best_parent: Option<i64> = None;
    let mut best_score = 0.0_f32;

    for (id, name, role, parent_id, blob) in rows {
        let Some(encrypted) = blob else { continue };

        // Decrypt the stored embedding
        let raw = match crypto::decrypt(key, &encrypted) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[speaker] decrypt failed for user_id={id}: {e}");
                continue;
            }
        };

        let stored: Vec<f32> = raw
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect();

        let score = cosine_similarity(embedding, &stored);
        eprintln!("[speaker] user_id={id} score={score:.3}");

        if score > best_score {
            best_score = score;
            best_id = Some(id);
            best_name = name;
            best_role = role;
            best_parent = parent_id;
        }
    }

    if best_score >= MATCH_THRESHOLD {
        if let Some(id) = best_id {
            session.lock().unwrap().set_user(
                id as i32,
                best_name,
                best_role,
                best_parent.map(|p| p as i32),
            );
        }
    } else {
        eprintln!("[speaker] no match above threshold ({best_score:.3})");
    }
}

pub async fn enroll_speaker(user_id: i32, embedding: &[f32]) -> Result<(), String> {
    let pool = get_db();
    let key = get_voice_key();

    let raw: Vec<u8> = embedding.iter().flat_map(|f| f.to_le_bytes()).collect();

    let encrypted = crypto::encrypt(key, &raw)?;

    sqlx::query("UPDATE users SET voice_embedding = ? WHERE id = ?")
        .bind(encrypted)
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(|e| format!("[speaker] enroll failed: {e}"))?;

    eprintln!("[speaker] enrolled encrypted embedding for user_id={user_id}");
    Ok(())
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        dot / (na * nb)
    }
}
