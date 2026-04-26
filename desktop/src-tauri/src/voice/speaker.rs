use crate::db::init::get_db;
use crate::utils::app_handle::emit;
use crate::utils::crypto::{self, get_voice_key};
use crate::utils::text::cosine_similarity;
use crate::voice::mel::compute_fbank;
use crate::voice::session::SharedSession;
use ort::session::{builder::GraphOptimizationLevel, Session};
use ort::value::Tensor;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

static SPEAKER_SESSION: OnceLock<Mutex<Session>> = OnceLock::new();

/// Minimum cosine similarity to accept a speaker match.
const MATCH_THRESHOLD: f32 = 0.75;

pub fn init_speaker(model_path: &Path) {
    SPEAKER_SESSION.get_or_init(|| {
        eprintln!("[speaker] loading wespeaker ONNX");
        let session = Session::builder()
            .unwrap()
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .unwrap()
            .commit_from_file(model_path)
            .expect("[speaker] failed to load model");
        eprintln!("[speaker] ready");
        Mutex::new(session)
    });
}

/// Extract a speaker embedding from raw 16kHz mono samples.
/// Returns None if the model is not loaded or the audio is too short.
pub fn extract_embedding(samples: &[f32]) -> Option<Vec<f32>> {
    let mut session = SPEAKER_SESSION.get()?.lock().ok()?;

    eprintln!("[speaker] computing fbank for {} samples", samples.len());
    let fbank = compute_fbank(samples)?;
    let (n_frames, n_mels) = fbank.dim();

    // WeSpeaker expects [batch=1, frames, mels=80]
    let input = fbank.into_shape_with_order((1, n_frames, n_mels)).ok()?;
    let tensor = Tensor::from_array(input).ok()?;

    let outputs = session.run(ort::inputs!["feats" => tensor]).ok()?;
    let raw = outputs[0].try_extract_tensor::<f32>().ok()?;
    let embedding = raw.1.iter().copied().collect::<Vec<f32>>();

    eprintln!("[speaker] embedding dim={}", embedding.len());
    Some(embedding)
}

/// Compare the given embedding against all enrolled users and update the
/// session if a match above MATCH_THRESHOLD is found.
pub async fn identify_speaker(embedding: &[f32], session: &SharedSession) {
    let pool = get_db();
    let key = get_voice_key();

    let rows = match sqlx::query_as::<_, (i64, String, String, Option<Vec<u8>>)>(
        "SELECT id, name, role, voice_embedding FROM users WHERE voice_embedding IS NOT NULL",
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

    let mut best_id = None;
    let mut best_name = String::new();
    let mut best_role = String::new();
    let mut best_score = 0.0_f32;

    for (id, name, role, blob) in rows {
        let Some(encrypted) = blob else { continue };

        let raw = match crypto::decrypt(key, &encrypted) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[speaker] decrypt failed user_id={id}: {e}");
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
        }
    }

    if best_score >= MATCH_THRESHOLD {
        if let Some(id) = best_id {
            eprintln!("[speaker] matched user_id={id} score={best_score:.3}");
            session
                .lock()
                .unwrap()
                .set_user(id as i32, best_name.clone(), best_role.clone());

            emit(
                "orb://speaker-identified",
                serde_json::json!({
                    "user_id": id,
                    "name":    best_name,
                    "role":    best_role,
                    "score":   best_score,
                }),
            );
        }
    } else {
        eprintln!("[speaker] no match above threshold ({best_score:.3})");
        emit(
            "orb://speaker-identified",
            serde_json::json!({
                "user_id": null,
                "name":    null,
                "role":    null,
            }),
        );
    }
}
