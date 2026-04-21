use crate::db::init::{get_db, get_voice_key};
use crate::utils::crypto;
use tauri::{AppHandle, Emitter};

/// Sentences chosen for maximum phonetic diversity —
/// covers vowels, fricatives, plosives, nasals, and natural prosody
/// so the embedding model gets a rich voice signature.
pub const ENROLLMENT_PROMPTS: &[&str] = &[
    "The quick brown fox jumps over the lazy dog.",
    "She sells seashells by the seashore every summer.",
    "How much wood would a woodchuck chuck if a woodchuck could chuck wood?",
    "Peter Piper picked a peck of pickled peppers.",
    "Around the rugged rocks the ragged rascal ran.",
];

#[derive(Clone, serde::Serialize)]
pub struct EnrollmentEvent {
    pub stage: String, // "greet" | "prompt" | "done" | "error"
    pub message: String,
    pub prompt_index: usize,
    pub total_prompts: usize,
    pub prompts: Vec<String>,
}

pub fn emit_enrollment(app: &AppHandle, event: EnrollmentEvent) {
    if let Err(e) = app.emit("peppa://enrollment", event) {
        eprintln!("[enrollment] emit error: {e}");
    }
}

pub async fn create_parent(name: String, embedding: Vec<f32>) -> Result<i32, String> {
    let pool = get_db();
    let key = get_voice_key();

    // Serialize embedding to raw bytes
    let raw: Vec<u8> = embedding.iter().flat_map(|f| f.to_le_bytes()).collect();

    // Encrypt before storing
    let encrypted = crypto::encrypt(key, &raw)?;

    let row = sqlx::query(
        "INSERT INTO users (name, role, voice_embedding) VALUES (?, 'parent', ?) RETURNING id",
    )
    .bind(&name)
    .bind(encrypted)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("[enrollment] insert failed: {e}"))?;

    let id: i64 =
        sqlx::Row::try_get(&row, "id").map_err(|e| format!("[enrollment] id fetch failed: {e}"))?;

    eprintln!("[enrollment] parent created — id={id} name={name}");
    Ok(id as i32)
}
