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

/// Minimum average cosine similarity to accept a speaker match.
const MATCH_THRESHOLD: f32 = 0.82;

/// Minimum gap between best and second-best score.
/// Prevents false positives when two users sound similar.
const MIN_MARGIN: f32 = 0.08;

/// Window size and hop for embedding aggregation (3s windows, 1s hop).
const WINDOW_SAMPLES: usize = 16000 * 3;
const HOP_SAMPLES: usize = 16000;

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

/// Extract a single embedding from a window of raw 16kHz mono samples.
fn extract_embedding_window(samples: &[f32]) -> Option<Vec<f32>> {
    let mut session = SPEAKER_SESSION.get()?.lock().ok()?;

    let fbank = compute_fbank(samples)?;
    let (n_frames, n_mels) = fbank.dim();
    let input = fbank.into_shape_with_order((1, n_frames, n_mels)).ok()?;
    let tensor = Tensor::from_array(input).ok()?;

    let outputs = session.run(ort::inputs!["feats" => tensor]).ok()?;
    let raw = outputs[0].try_extract_tensor::<f32>().ok()?;
    Some(raw.1.iter().copied().collect())
}

/// Extract a stable speaker embedding by averaging across overlapping windows.
/// Falls back to a single full-audio embedding if the audio is too short for windowing.
pub fn extract_embedding(samples: &[f32]) -> Option<Vec<f32>> {
    eprintln!(
        "[speaker] extracting embedding from {} samples",
        samples.len()
    );

    let embeddings: Vec<Vec<f32>> = if samples.len() >= WINDOW_SAMPLES {
        samples
            .windows(WINDOW_SAMPLES)
            .step_by(HOP_SAMPLES)
            .filter_map(|w| extract_embedding_window(w))
            .collect()
    } else {
        // Audio shorter than one window — use it directly
        extract_embedding_window(samples)
            .map(|e| vec![e])
            .unwrap_or_default()
    };

    if embeddings.is_empty() {
        eprintln!("[speaker] no embeddings extracted");
        return None;
    }

    // Mean pool across all windows
    let dim = embeddings[0].len();
    let mut mean = vec![0.0f32; dim];
    for emb in &embeddings {
        for (m, e) in mean.iter_mut().zip(emb.iter()) {
            *m += e;
        }
    }
    let n = embeddings.len() as f32;
    mean.iter_mut().for_each(|m| *m /= n);

    eprintln!(
        "[speaker] embedding dim={dim} from {} windows",
        embeddings.len()
    );
    Some(mean)
}

/// Compare the given embedding against all enrolled users and update the
/// session if a match above MATCH_THRESHOLD is found with sufficient margin.
pub async fn identify_speaker(embedding: &[f32], session: &SharedSession) {
    let pool = get_db();
    let key = get_voice_key();

    // Fetch all users
    let users = match sqlx::query_as::<_, (i64, String, String)>("SELECT id, name, role FROM users")
        .fetch_all(pool)
        .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[speaker] db query failed: {e}");
            return;
        }
    };

    // Fetch all embeddings grouped by user_id
    let embedding_rows = match sqlx::query_as::<_, (i64, Vec<u8>)>(
        "SELECT user_id, embedding FROM voice_embeddings",
    )
    .fetch_all(pool)
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[speaker] embedding query failed: {e}");
            return;
        }
    };

    // Score each user by averaging cosine similarity across all their embeddings
    let mut scored: Vec<(i64, String, String, f32)> = Vec::new();

    for (user_id, name, role) in &users {
        let user_embeddings: Vec<Vec<f32>> = embedding_rows
            .iter()
            .filter(|(uid, _)| uid == user_id)
            .filter_map(|(_, blob)| {
                let raw = crypto::decrypt(key, blob)
                    .map_err(|e| eprintln!("[speaker] decrypt failed user_id={user_id}: {e}"))
                    .ok()?;
                Some(
                    raw.chunks_exact(4)
                        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                        .collect(),
                )
            })
            .collect();

        if user_embeddings.is_empty() {
            continue;
        }

        // Average cosine similarity across all stored embeddings
        let scores: Vec<f32> = user_embeddings
            .iter()
            .map(|stored| cosine_similarity(embedding, stored))
            .collect();
        let avg_score = scores.iter().sum::<f32>() / scores.len() as f32;

        eprintln!(
            "[speaker] user_id={user_id} name={name} avg_score={avg_score:.3} from {} embeddings",
            user_embeddings.len()
        );
        scored.push((*user_id, name.clone(), role.clone(), avg_score));
    }

    if scored.is_empty() {
        eprintln!("[speaker] no enrolled users with embeddings");
        emit_unidentified();
        return;
    }

    // Sort descending by score
    scored.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap());

    let (best_id, best_name, best_role, best_score) = &scored[0];

    // Require minimum threshold
    if *best_score < MATCH_THRESHOLD {
        eprintln!("[speaker] no match above threshold ({best_score:.3})");
        emit_unidentified();
        return;
    }

    // Require minimum margin over second-best to avoid ambiguous matches
    if scored.len() > 1 {
        let second_score = scored[1].3;
        let margin = best_score - second_score;
        if margin < MIN_MARGIN {
            eprintln!(
                "[speaker] ambiguous match: best={best_score:.3} second={second_score:.3} margin={margin:.3}"
            );
            emit_unidentified();
            return;
        }
    }

    eprintln!("[speaker] matched user_id={best_id} score={best_score:.3}");
    session
        .lock()
        .unwrap()
        .set_user(*best_id as i32, best_name.clone(), best_role.clone());

    emit(
        "orb://speaker-identified",
        serde_json::json!({
            "user_id": best_id,
            "name":    best_name,
            "role":    best_role,
            "score":   best_score,
        }),
    );
}

fn emit_unidentified() {
    emit(
        "orb://speaker-identified",
        serde_json::json!({
            "user_id": null,
            "name":    null,
            "role":    null,
        }),
    );
}
