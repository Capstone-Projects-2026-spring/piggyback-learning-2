use crate::db::init::get_db;
use crate::utils::text::cosine_similarity;
use crate::utils::voice::intent_classifier::embed_strings;

/// Returns the id of an existing tag with this name, or inserts and returns the new one.
pub async fn get_or_create_tag(name: &str) -> Result<i64, String> {
    let pool = get_db();
    let name = name.trim().to_lowercase();

    let existing = sqlx::query_as::<_, (i64,)>("SELECT id FROM tags WHERE name = ?")
        .bind(&name)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("[tags] select failed: {e}"))?;

    if let Some((id,)) = existing {
        return Ok(id);
    }

    let row = sqlx::query("INSERT INTO tags (name) VALUES (?) RETURNING id")
        .bind(&name)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("[tags] insert failed: {e}"))?;

    let id: i64 = sqlx::Row::try_get(&row, "id").map_err(|e| format!("[tags] id fetch: {e}"))?;

    eprintln!("[tags] created tag id={id} name={name}");
    Ok(id)
}

/// Infer up to 5 topic tags from a plain-text transcript using embedding similarity.
/// Matches against a fixed candidate list
pub fn infer_tags_from_text(text: &str) -> Result<Vec<String>, String> {
    let tag_embeddings = embed_strings(TOPIC_CANDIDATES)?;
    let text_embeddings = embed_strings(&[text])?;
    let text_emb = &text_embeddings[0];

    const THRESHOLD: f32 = 0.25;

    let mut scored: Vec<(String, f32)> = TOPIC_CANDIDATES
        .iter()
        .zip(tag_embeddings.iter())
        .map(|(name, emb)| (name.to_string(), cosine_similarity(text_emb, emb)))
        .filter(|(_, s)| *s >= THRESHOLD)
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    scored.truncate(5);

    Ok(scored.into_iter().map(|(name, _)| name).collect())
}

const TOPIC_CANDIDATES: &[&str] = &[
    "mathematics",
    "science",
    "history",
    "geography",
    "biology",
    "physics",
    "chemistry",
    "astronomy",
    "space",
    "animals",
    "dinosaurs",
    "nature",
    "environment",
    "technology",
    "computers",
    "programming",
    "art",
    "music",
    "literature",
    "language",
    "reading",
    "writing",
    "cooking",
    "sports",
    "football",
    "basketball",
    "health",
    "medicine",
    "psychology",
    "philosophy",
    "economics",
    "engineering",
    "robots",
    "ocean",
    "weather",
    "climate",
    "plants",
    "human body",
    "ancient civilizations",
    "world war",
    "inventions",
    "mythology",
    "coding",
    "architecture",
    "superhero",
];
