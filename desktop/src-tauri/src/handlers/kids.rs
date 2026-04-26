use crate::db::init::get_db;
use crate::handlers::tags::get_or_create_tag;
use crate::utils::app_handle::emit;
use crate::utils::text::normalize;
use crate::voice::{
    onboarding::{self, OnboardingFlow, SharedOnboarding},
    session::SharedSession,
};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct AssignedVideo {
    pub id: String,
    pub title: Option<String>,
    pub thumbnail_url: Option<String>,
    pub duration_seconds: Option<i32>,
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

pub async fn get_video_assignments(_args: &[String], session: &SharedSession) {
    let kid_id = {
        let s = session.lock().unwrap();
        match (s.role.as_deref(), s.user_id) {
            (Some("kid"), Some(id)) => id as i64,
            _ => {
                eprintln!("[kids] get_video_assignments - kid session required");
                return;
            }
        }
    };

    let pool = get_db();
    let rows = sqlx::query(
        "SELECT v.id, v.title, v.thumbnail_url, v.duration_seconds
         FROM video_assignments va
         JOIN videos v ON v.id = va.video_id
         WHERE va.kid_id = ?",
    )
    .bind(kid_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let videos: Vec<AssignedVideo> = rows
        .into_iter()
        .map(|r| AssignedVideo {
            id: sqlx::Row::get(&r, "id"),
            title: sqlx::Row::try_get(&r, "title").ok(),
            thumbnail_url: sqlx::Row::try_get(&r, "thumbnail_url").ok(),
            duration_seconds: sqlx::Row::try_get(&r, "duration_seconds").ok(),
        })
        .collect();

    eprintln!(
        "[kids] get_video_assignments - kid_id={kid_id} {} videos",
        videos.len()
    );
    emit("orb://my-videos", serde_json::json!({ "videos": videos }));
}

pub async fn assign_video(args: &[String], session: &SharedSession) {
    let video_id = {
        let s = session.lock().unwrap();
        // TODO: re-enable parent-only guard
        // if s.role.as_deref() != Some("parent") { return; }
        match s.current_video.clone() {
            Some(v) => v,
            None => {
                eprintln!("[kids] assign_video - no current_video in session");
                emit(
                    "orb://assign-error",
                    serde_json::json!({ "message": "No video is currently active." }),
                );
                return;
            }
        }
    };

    let kid_id = match resolve_kid_id(args, session).await {
        Some(id) => id,
        None => {
            eprintln!("[kids] assign_video - could not resolve kid");
            emit(
                "orb://assign-error",
                serde_json::json!({ "message": "Which kid did you mean?" }),
            );
            return;
        }
    };

    let pool = get_db();

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
        Ok(r) => emit(
            "orb://video-assigned",
            serde_json::json!({
                "video_id":        video_id,
                "kid_id":          kid_id,
                "kid_name":        kid_name,
                "already_assigned": r.rows_affected() == 0,
            }),
        ),
        Err(e) => eprintln!("[kids] assign_video failed: {e}"),
    }
}

pub async fn get_recommendations(args: &[String], session: &SharedSession) {
    // TODO: re-enable parent-only guard
    // { let s = session.lock().unwrap(); if s.role.as_deref() != Some("parent") { return; } }

    let kid_id = match resolve_kid_id(args, session).await {
        Some(id) => id,
        None => {
            eprintln!("[kids] get_recommendations - could not resolve kid");
            emit(
                "orb://recommendations-error",
                serde_json::json!({ "message": "Which kid did you mean?" }),
            );
            return;
        }
    };

    let pool = get_db();

    let kid_name: String = sqlx::query_as::<_, (String,)>("SELECT name FROM users WHERE id = ?")
        .bind(kid_id)
        .fetch_optional(pool)
        .await
        .unwrap_or_default()
        .map(|(n,)| n)
        .unwrap_or_else(|| "Unknown".to_string());

    let tags: Vec<String> = sqlx::query_as::<_, (String,)>(
        "SELECT t.name FROM kid_tags kt
         JOIN tags t ON t.id = kt.tag_id
         WHERE kt.kid_id = ? ORDER BY t.name",
    )
    .bind(kid_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|(n,)| n)
    .collect();

    let rows = sqlx::query(
        "SELECT v.id, v.title, v.thumbnail_url, v.duration_seconds,
                COUNT(vt.tag_id) AS score
         FROM video_tags vt
         JOIN videos v ON v.id = vt.video_id
         WHERE vt.tag_id IN (SELECT tag_id FROM kid_tags WHERE kid_id = ?)
           AND v.id NOT IN (SELECT video_id FROM video_assignments WHERE kid_id = ?)
         GROUP BY v.id, v.title, v.thumbnail_url, v.duration_seconds
         ORDER BY score DESC
         LIMIT 20",
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
        "[kids] recommendations for '{kid_name}' (id={kid_id}) - {} results, {} tags",
        recommendations.len(),
        tags.len()
    );

    emit(
        "orb://recommendations",
        &RecommendationsPayload {
            kid_name,
            tags: tags.clone(),
            recommendations,
        },
    );

    // Top up via YouTube search using the kid's tags
    if !tags.is_empty() {
        crate::handlers::videos::search(&tags).await;
    }
}

pub async fn start_kid_enrollment(
    _args: &[String],
    session: &SharedSession,
    onboarding: &SharedOnboarding,
) {
    match session.lock().unwrap().role.as_deref() {
        Some("parent") => {
            eprintln!("[kids] starting kid enrollment");
            onboarding::start(onboarding, OnboardingFlow::Kid);
        }
        Some(r) => eprintln!("[kids] add_kid - role={r} is not parent"),
        None => eprintln!("[kids] add_kid - no user identified"),
    }
}

// Internals

async fn assign_tag_to_kid(kid_id: i64, tag_id: i64) -> Result<(), String> {
    sqlx::query("INSERT OR IGNORE INTO kid_tags (kid_id, tag_id) VALUES (?, ?)")
        .bind(kid_id)
        .bind(tag_id)
        .execute(get_db())
        .await
        .map_err(|e| format!("[kids] kid_tags insert failed: {e}"))?;
    Ok(())
}

/// Resolve which kid the transcript refers to.
/// Priority: name match in transcript -> speaker is a kid -> only one kid exists.
async fn resolve_kid_id(args: &[String], session: &SharedSession) -> Option<i64> {
    let pool = get_db();

    let kids = sqlx::query_as::<_, (i64, String)>(
        "SELECT id, name FROM users WHERE role = 'kid' ORDER BY name",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    if kids.is_empty() {
        eprintln!("[kids] resolve_kid_id - no kids in DB");
        return None;
    }

    let transcript = normalize(&args.join(" "));
    for (id, name) in &kids {
        if transcript.contains(&normalize(name)) {
            eprintln!("[kids] resolve_kid_id - name match '{name}' & id={id}");
            return Some(*id);
        }
    }

    {
        let s = session.lock().unwrap();
        if s.role.as_deref() == Some("kid") {
            if let Some(uid) = s.user_id {
                eprintln!("[kids] resolve_kid_id - speaker is kid, id={uid}");
                return Some(uid as i64);
            }
        }
    }

    if kids.len() == 1 {
        let (id, name) = &kids[0];
        eprintln!("[kids] resolve_kid_id - only one kid '{name}', id={id}");
        return Some(*id);
    }

    eprintln!(
        "[kids] resolve_kid_id - ambiguous ({} kids, no name match)",
        kids.len()
    );
    None
}

/// Extract topic words from natural-language interest phrases.
/// "my kid likes dinosaurs and space" -> ["dinosaurs", "space"]
fn extract_topics(transcript: &str) -> Vec<String> {
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

    let lower = transcript.to_lowercase();
    let remainder = TRIGGERS
        .iter()
        .find_map(|t| lower.find(t).map(|pos| &lower[pos + t.len()..]));

    let Some(raw) = remainder else { return vec![] };

    raw.replace(" and ", ",")
        .replace(" or ", ",")
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && s.split_whitespace().count() <= 3)
        .collect()
}

pub async fn add_tags(args: &[String], session: &SharedSession) {
    let transcript = args.join(" ");
    let topics = extract_topics(&transcript);

    if topics.is_empty() {
        eprintln!("[kids] add_tags - no topics in: {transcript:?}");
        return;
    }

    let kid_id = match resolve_kid_id(args, session).await {
        Some(id) => id,
        None => return,
    };

    eprintln!("[kids] add_tags - kid_id={kid_id} topics={topics:?}");

    for topic in topics {
        let kid_id_copy = kid_id;
        tokio::spawn(async move {
            match get_or_create_tag(&topic).await {
                Ok(tag_id) => {
                    if let Err(e) = assign_tag_to_kid(kid_id_copy, tag_id).await {
                        eprintln!("[kids] assign tag failed: {e}");
                    }
                }
                Err(e) => eprintln!("[kids] get_or_create_tag failed: {e}"),
            }
        });
    }
}
