use crate::db::init::get_db;
use crate::handlers::tags::get_or_create_tag;
use crate::utils::app_handle::emit;
use crate::utils::voice::{
    onboarding::{self, OnboardingFlow, SharedOnboarding},
    session::SharedSession,
};

use serde::Serialize;
use tauri::AppHandle;

pub async fn get_tags(args: &[String], session: &SharedSession) {
    let kid_id = match resolve_kid_id(args, session).await {
        Some(id) => id,
        None => return,
    };

    let pool = get_db();
    match sqlx::query_as::<_, (i64, String)>(
        "SELECT t.id, t.name
         FROM tags t
         JOIN kid_tags kt ON kt.tag_id = t.id
         WHERE kt.kid_id = ?
         ORDER BY t.name",
    )
    .bind(kid_id)
    .fetch_all(pool)
    .await
    {
        Ok(rows) => {
            eprintln!(
                "[handler:kids] tags for kid_id={kid_id} → {:?}",
                rows.iter().map(|(_, n)| n.as_str()).collect::<Vec<_>>()
            );
        }
        Err(e) => eprintln!("[handler:kids] get_tags failed: {e}"),
    }
}

/// Called by the dispatcher for "add_tag" intent.
/// args = the raw transcript words — we parse tags and the optional kid name out of it.
pub async fn add_tags(args: &[String], session: &SharedSession) {
    // Reconstruct the raw transcript from args
    let transcript = args.join(" ");

    // Extract the topic(s) — everything after "likes", "loves", "enjoys", "into" etc.
    let topics = extract_topics(&transcript);
    if topics.is_empty() {
        eprintln!("[handler:kids] add_tags — could not extract any topics from: {transcript:?}");
        return;
    }

    let kid_id = match resolve_kid_id(args, session).await {
        Some(id) => id,
        None => return,
    };

    eprintln!("[handler:kids] add_tags — kid_id={kid_id} topics={topics:?}");

    // Create tags and assign them all — fire and forget per tag
    for topic in topics {
        let kid_id_copy = kid_id;
        tokio::spawn(async move {
            match get_or_create_tag(&topic).await {
                Ok(tag_id) => {
                    if let Err(e) = assign_tag_to_kid(kid_id_copy, tag_id).await {
                        eprintln!("[handler:kids] assign tag failed: {e}");
                    } else {
                        eprintln!("[handler:kids] assigned tag '{topic}' (id={tag_id}) to kid {kid_id_copy}");
                    }
                }
                Err(e) => eprintln!("[handler:kids] get_or_create_tag failed: {e}"),
            }
        });
    }
}

pub async fn get_video_assignments(args: &[String], _session: &SharedSession) {
    println!("[handler:kids] get_video_assignments — args={args:?}");
}

pub async fn assign_video(args: &[String], session: &SharedSession) {
    // Must be a parent
    let video_id = {
        let s = session.lock().unwrap();
        // Temporary disabled for dev purposes
        // if s.role.as_deref() != Some("parent") {
        //     eprintln!("[handler:kids] assign_video — parents only");
        //     return;
        // }
        match s.current_video.clone() {
            Some(v) => v,
            None => {
                eprintln!("[handler:kids] assign_video — no current_video in session");
                emit(
                    "peppa://assign-error",
                    serde_json::json!({ "message": "No video is currently active." }),
                );
                return;
            }
        }
    };

    let kid_id = match resolve_kid_id(args, session).await {
        Some(id) => id,
        None => {
            eprintln!("[handler:kids] assign_video — could not resolve kid from transcript");
            emit(
                "peppa://assign-error",
                serde_json::json!({ "message": "Which kid did you mean?" }),
            );
            return;
        }
    };

    let pool = get_db();

    // Fetch kid name for the confirmation message
    let kid_name: String = sqlx::query_as::<_, (String,)>("SELECT name FROM users WHERE id = ?")
        .bind(kid_id)
        .fetch_optional(pool)
        .await
        .unwrap_or_default()
        .map(|(n,)| n)
        .unwrap_or_else(|| "Unknown".to_string());

    match sqlx::query("INSERT OR IGNORE INTO video_assignments (kid_id, video_id) VALUES (?, ?)")
        .bind(kid_id)
        .bind(&video_id)
        .execute(pool)
        .await
    {
        Ok(result) if result.rows_affected() == 0 => {
            eprintln!("[handler:kids] assign_video — already assigned");
            emit(
                "peppa://video-assigned",
                serde_json::json!({
                    "video_id": video_id,
                    "kid_id": kid_id,
                    "kid_name": kid_name,
                    "already_assigned": true,
                }),
            );
        }
        Ok(_) => {
            eprintln!("[handler:kids] assign_video — assigned {video_id} to kid_id={kid_id}");
            emit(
                "peppa://video-assigned",
                serde_json::json!({
                    "video_id": video_id,
                    "kid_id": kid_id,
                    "kid_name": kid_name,
                    "already_assigned": false,
                }),
            );
        }
        Err(e) => eprintln!("[handler:kids] assign_video failed: {e}"),
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct RecommendedVideo {
    pub id: String,
    pub title: Option<String>,
    pub thumbnail_url: Option<String>,
    pub duration_seconds: Option<i32>,
    pub score: i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct RecommendationsPayload {
    pub kid_name: String,
    pub tags: Vec<String>,
    pub recommendations: Vec<RecommendedVideo>,
}

pub async fn get_recommendations(args: &[String], session: &SharedSession) {
    // Temporarily disabled for dev purposes
    // // Parent-only guard
    // {
    //     let s = session.lock().unwrap();
    //     if s.role.as_deref() != Some("parent") {
    //         eprintln!(
    //             "[handler:kids] get_recommendations — only parents can request recommendations"
    //         );
    //         return;
    //     }
    // }

    // Resolve kid by name mentioned in the transcript ("show recommendations for Emma")
    let kid_id = match resolve_kid_id(args, session).await {
        Some(id) => id,
        None => {
            eprintln!("[handler:kids] get_recommendations — could not resolve kid from transcript");
            emit(
                "peppa://recommendations-error",
                serde_json::json!({ "message": "Which kid did you mean?" }),
            );
            return;
        }
    };

    let pool = get_db();

    // Fetch the kid's name for the payload
    let kid_name: String = sqlx::query_as::<_, (String,)>("SELECT name FROM users WHERE id = ?")
        .bind(kid_id)
        .fetch_optional(pool)
        .await
        .unwrap_or_default()
        .map(|(n,)| n)
        .unwrap_or_else(|| "Unknown".to_string());

    // Fetch kid's tags
    let tags: Vec<String> = sqlx::query_as::<_, (String,)>(
        "SELECT t.name FROM kid_tags kt JOIN tags t ON t.id = kt.tag_id WHERE kt.kid_id = ? ORDER BY t.name",
    )
    .bind(kid_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|(n,)| n)
    .collect();

    // Fetch top local videos by tag-match score, excluding already-assigned ones
    let rows = sqlx::query(
        r#"
        SELECT
            v.id,
            v.title,
            v.thumbnail_url,
            v.duration_seconds,
            COUNT(vt.tag_id) AS score
        FROM video_tags vt
        JOIN videos v ON v.id = vt.video_id
        WHERE vt.tag_id IN (
            SELECT tag_id FROM kid_tags WHERE kid_id = ?
        )
        AND v.id NOT IN (
            SELECT video_id FROM video_assignments WHERE kid_id = ?
        )
        GROUP BY v.id, v.title, v.thumbnail_url, v.duration_seconds
        ORDER BY score DESC
        LIMIT 20
        "#,
    )
    .bind(kid_id)
    .bind(kid_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let recommendations: Vec<RecommendedVideo> = rows
        .into_iter()
        .map(|r| RecommendedVideo {
            id: sqlx::Row::get(&r, "id"),
            title: sqlx::Row::try_get(&r, "title").ok(),
            thumbnail_url: sqlx::Row::try_get(&r, "thumbnail_url").ok(),
            duration_seconds: sqlx::Row::try_get(&r, "duration_seconds").ok(),
            score: sqlx::Row::get(&r, "score"),
        })
        .collect();

    eprintln!(
        "[handler:kids] recommendations for '{kid_name}' (id={kid_id}) — {} local results, {} tags",
        recommendations.len(),
        tags.len()
    );

    emit(
        "peppa://recommendations",
        &RecommendationsPayload {
            kid_name,
            tags: tags.clone(),
            recommendations,
        },
    );

    // Top up to 10 via YouTube using the kid's tags as the search query
    if !tags.is_empty() {
        let search_args: Vec<String> = tags;
        crate::handlers::videos::search(&search_args).await;
    }
}

pub async fn start_kid_enrollment(
    app: &AppHandle,
    _args: &[String],
    session: &SharedSession,
    onboarding: &SharedOnboarding,
) {
    let role = session.lock().unwrap().role.clone();
    match role.as_deref() {
        Some("parent") => {
            eprintln!("[handler:kids] starting kid enrollment");
            onboarding::start(app, onboarding, OnboardingFlow::Kid);
        }
        Some(r) => eprintln!("[handler:kids] add_kid — role={r} is not a parent, ignoring"),
        None => eprintln!("[handler:kids] add_kid — no user identified, ignoring"),
    }
}

// ── Internals ────────────────────────────────────────────────────────────────

async fn assign_tag_to_kid(kid_id: i64, tag_id: i64) -> Result<(), String> {
    let pool = get_db();
    // INSERT OR IGNORE gives us the same "do nothing on conflict" behaviour
    sqlx::query("INSERT OR IGNORE INTO kid_tags (kid_id, tag_id) VALUES (?, ?)")
        .bind(kid_id)
        .bind(tag_id)
        .execute(pool)
        .await
        .map_err(|e| format!("[kid_tags] insert failed: {e}"))?;
    Ok(())
}

/// Figures out which kid the transcript refers to.
/// Priority:
///   1. Name mentioned in transcript matches a kid in DB
///   2. Only one kid exists → use them
///   3. Multiple kids, no name → log and return None (Peppa should prompt)
async fn resolve_kid_id(args: &[String], session: &SharedSession) -> Option<i64> {
    let pool = get_db();

    // Fetch all kids from DB
    let kids = sqlx::query_as::<_, (i64, String)>(
        "SELECT id, name FROM users WHERE role = 'kid' ORDER BY name",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    if kids.is_empty() {
        eprintln!("[handler:kids] resolve_kid_id — no kids in DB");
        return None;
    }

    // Try to match a kid name from the transcript
    let transcript = args.join(" ").to_lowercase();
    for (id, name) in &kids {
        if transcript.contains(&name.to_lowercase()) {
            eprintln!("[handler:kids] resolve_kid_id — matched name '{name}' → id={id}");
            return Some(*id);
        }
    }

    // Check if the speaker IS a kid (they said "I like...")
    // In that case use their own user_id
    {
        let s = session.lock().unwrap();
        if s.role.as_deref() == Some("kid") {
            if let Some(uid) = s.user_id {
                eprintln!("[handler:kids] resolve_kid_id — speaker is kid, using own id={uid}");
                return Some(uid as i64);
            }
        }
    }

    // Exactly one kid → unambiguous
    if kids.len() == 1 {
        let (id, name) = &kids[0];
        eprintln!("[handler:kids] resolve_kid_id — only one kid '{name}', using id={id}");
        return Some(*id);
    }

    // Ambiguous — multiple kids, no name match
    eprintln!(
        "[handler:kids] resolve_kid_id — ambiguous: {} kids, no name in transcript '{transcript}'",
        kids.len()
    );
    None
}

/// Pulls topic words out of natural language interest phrases.
/// "my kid likes dinosaurs and space" → ["dinosaurs", "space"]
/// "Emma loves reading and math"      → ["reading", "math"]
/// "I'm really into cooking"          → ["cooking"]
fn extract_topics(transcript: &str) -> Vec<String> {
    let lower = transcript.to_lowercase();

    const TRIGGERS: &[&str] = &[
        "is really into ",
        "is interested in ",
        "really likes ",
        "really loves ",
        "really enjoys ",
        "interested in ",
        "really like ",
        "really love ",
        "really enjoy ",
        "likes ",
        "loves ",
        "enjoys ",
        "like ",
        "love ",
        "enjoy ",
        "into ",
    ];

    let remainder = TRIGGERS
        .iter()
        .find_map(|trigger| lower.find(trigger).map(|pos| &lower[pos + trigger.len()..]));

    let Some(raw) = remainder else {
        return vec![];
    };

    raw.replace(" and ", ",")
        .replace(" or ", ",")
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && s.split_whitespace().count() <= 3)
        .collect()
}
