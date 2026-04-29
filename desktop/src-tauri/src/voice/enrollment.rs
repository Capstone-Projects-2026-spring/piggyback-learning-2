use crate::db::init::get_db;
use crate::utils::{app_handle, crypto};
use tauri::Emitter;

pub const ENROLLMENT_PROMPTS: &[&str] = &[
    "The quick brown fox jumps over the lazy dog.",
    "She sells seashells by the seashore every summer.",
    "How much wood would a woodchuck chuck if a woodchuck could chuck wood?",
    "Peter Piper picked a peck of pickled peppers.",
    "Around the rugged rocks the ragged rascal ran.",
];

#[derive(Clone, serde::Serialize)]
pub struct EnrollmentEvent {
    pub stage: String,
    pub message: String,
    pub prompt_index: usize,
    pub total_prompts: usize,
    pub prompts: Vec<String>,
    pub flow: String,
}

pub fn emit_enrollment(event: EnrollmentEvent) {
    let app = app_handle::get_app_handle();
    if let Err(e) = app.emit("orb://enrollment", event) {
        eprintln!("[enrollment] emit error: {e}");
    }
}

/// Create a user and store all enrollment embeddings individually.
/// Each embedding is encrypted separately with a fresh nonce.
pub async fn create_user(
    name: String,
    embeddings: Vec<Vec<f32>>,
    role: &str,
) -> Result<i32, String> {
    let key = crypto::get_voice_key();

    let row = sqlx::query("INSERT INTO users (name, role) VALUES (?, ?) RETURNING id")
        .bind(&name)
        .bind(role)
        .fetch_one(get_db())
        .await
        .map_err(|e| format!("[enrollment] insert user failed: {e}"))?;

    let id: i64 =
        sqlx::Row::try_get(&row, "id").map_err(|e| format!("[enrollment] id fetch: {e}"))?;

    for (i, embedding) in embeddings.iter().enumerate() {
        let raw: Vec<u8> = embedding.iter().flat_map(|f| f.to_le_bytes()).collect();
        let encrypted = crypto::encrypt(key, &raw)?;

        sqlx::query("INSERT INTO voice_embeddings (user_id, embedding) VALUES (?, ?)")
            .bind(id)
            .bind(encrypted)
            .execute(get_db())
            .await
            .map_err(|e| format!("[enrollment] insert embedding {i} failed: {e}"))?;
    }

    eprintln!(
        "[enrollment] created {role} id={id} name={name} embeddings={}",
        embeddings.len()
    );
    Ok(id as i32)
}
