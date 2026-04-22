use tauri::Emitter;

use crate::db::init::get_db;
use crate::utils::download::download_video;

#[tauri::command]
pub async fn download_video_command(video_id: String, app: tauri::AppHandle) -> Result<(), String> {
    eprintln!("[handler:videos] download_video_command — video_id={video_id}");

    tokio::spawn(async move {
        match download_video(&video_id).await {
            Ok(None) => {
                eprintln!("[handler:videos] already downloaded — {video_id}");
                let _ = app.emit(
                    "peppa://video-status",
                    serde_json::json!({
                        "video_id": video_id,
                        "status": "already_exists"
                    }),
                );
            }
            Ok(Some((id, title, thumbnail, duration, video_path, transcript_path))) => {
                if let Err(e) = upsert_video(&id, &title, &thumbnail, duration, &video_path).await {
                    eprintln!("[handler:videos] upsert failed: {e}");
                    let _ = app.emit(
                        "peppa://video-status",
                        serde_json::json!({
                            "video_id": id,
                            "status": "error",
                            "msg": e
                        }),
                    );
                    return;
                }

                // Auto-generate and assign tags from transcript in background
                if !transcript_path.is_empty() {
                    let id_clone = id.clone();
                    let transcript_clone = transcript_path.clone();
                    tokio::spawn(async move {
                        if let Err(e) = generate_and_assign_tags(&id_clone, &transcript_clone).await
                        {
                            eprintln!("[handler:videos] tag generation failed: {e}");
                        }
                    });
                }

                let _ = app.emit(
                    "peppa://video-status",
                    serde_json::json!({
                        "video_id": id,
                        "status": "done",
                        "title": title,
                        "thumbnail": thumbnail,
                        "duration": duration,
                        "video_path": video_path,
                        "transcript_path": transcript_path,
                    }),
                );
            }
            Err(e) => {
                eprintln!("[handler:videos] download failed: {e}");
                let _ = app.emit(
                    "peppa://video-status",
                    serde_json::json!({
                        "video_id": video_id,
                        "status": "error",
                        "msg": e
                    }),
                );
            }
        }
    });

    Ok(())
}

pub async fn search(args: &[String], app: &tauri::AppHandle) {
    use tokio::process::Command;

    let query = args
        .iter()
        .map(|s| s.as_str())
        .filter(|s| {
            !matches!(
                *s,
                "search" | "for" | "find" | "look" | "up" | "me" | "videos" | "about" | "show"
            )
        })
        .collect::<Vec<_>>()
        .join(" ");

    if query.is_empty() {
        eprintln!("[handler:videos] search — empty query after filtering");
        return;
    }

    eprintln!("[handler:videos] search — query={query}");

    let app_clone = app.clone();
    tokio::spawn(async move {
        let output = Command::new("yt-dlp")
            .arg("--dump-json")
            .arg("--no-warnings")
            .arg("--no-playlist")
            .arg(format!("ytsearch10:{query}"))
            .output()
            .await;

        match output {
            Ok(out) if out.status.success() => {
                // yt-dlp --dump-json returns one JSON object per line
                let results: Vec<serde_json::Value> = String::from_utf8_lossy(&out.stdout)
                    .lines()
                    .filter(|l| !l.is_empty())
                    .filter_map(|line| serde_json::from_str(line).ok())
                    .map(|json: serde_json::Value| {
                        let video_id = json["id"].as_str().unwrap_or("").to_string();
                        let thumbnail = format!("https://i.ytimg.com/vi/{video_id}/hqdefault.jpg");
                        serde_json::json!({
                            "video_id":  video_id,
                            "title":     json["title"].as_str().unwrap_or(""),
                            "thumbnail": thumbnail,
                            "duration":  json["duration"].as_i64().unwrap_or(0),
                            "uploader":  json["uploader"].as_str().unwrap_or(""),
                        })
                    })
                    .collect();

                eprintln!("[handler:videos] search → {} results", results.len());

                // Emit as a plain value, not serialized string
                let _ = app_clone.emit(
                    "peppa://search-results",
                    serde_json::json!({
                        "query":   query,
                        "results": results,
                    }),
                );
            }
            Ok(out) => {
                eprintln!(
                    "[handler:videos] search failed: {}",
                    String::from_utf8_lossy(&out.stderr)
                );
            }
            Err(e) => eprintln!("[handler:videos] search spawn failed: {e}"),
        }
    });
}

// ── Internals ────────────────────────────────────────────────────────────────

async fn upsert_video(
    id: &str,
    title: &str,
    thumbnail: &str,
    duration: i32,
    path: &str,
) -> Result<(), String> {
    let pool = get_db();
    sqlx::query(
        "INSERT INTO videos (id, title, thumbnail_url, duration_seconds, local_video_path)
         VALUES (?, ?, ?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET
             title            = excluded.title,
             thumbnail_url    = excluded.thumbnail_url,
             duration_seconds = excluded.duration_seconds,
             local_video_path = excluded.local_video_path",
    )
    .bind(id)
    .bind(title)
    .bind(thumbnail)
    .bind(duration)
    .bind(path)
    .execute(pool)
    .await
    .map_err(|e| format!("[videos] upsert failed: {e}"))?;

    eprintln!("[handler:videos] upsert ok — id={id}");
    Ok(())
}

/// Reads the VTT, extracts plain text, uses fastembed to find the closest
/// tags from a fixed topic list, then inserts them into video_tags.
async fn generate_and_assign_tags(video_id: &str, vtt_path: &str) -> Result<(), String> {
    let raw =
        std::fs::read_to_string(vtt_path).map_err(|e| format!("[tags] read vtt failed: {e}"))?;

    let text = vtt_to_plain_text(&raw);
    if text.is_empty() {
        eprintln!("[tags] transcript empty after stripping — skipping");
        return Ok(());
    }

    // Truncate to ~2000 chars so embedding stays fast
    let sample: String = text.chars().take(2000).collect();

    let tags = infer_tags_from_text(&sample)?;
    eprintln!("[tags] inferred for video {video_id}: {tags:?}");

    if tags.is_empty() {
        return Ok(());
    }

    let pool = get_db();

    for tag_name in &tags {
        // get_or_create tag
        let existing = sqlx::query_as::<_, (i64,)>("SELECT id FROM tags WHERE name = ?")
            .bind(tag_name)
            .fetch_optional(pool)
            .await
            .map_err(|e| format!("[tags] select failed: {e}"))?;

        let tag_id = if let Some((id,)) = existing {
            id
        } else {
            let row = sqlx::query("INSERT INTO tags (name) VALUES (?) RETURNING id")
                .bind(tag_name)
                .fetch_one(pool)
                .await
                .map_err(|e| format!("[tags] insert failed: {e}"))?;
            sqlx::Row::try_get(&row, "id").map_err(|e| format!("[tags] id fetch: {e}"))?
        };

        sqlx::query("INSERT OR IGNORE INTO video_tags (video_id, tag_id) VALUES (?, ?)")
            .bind(video_id)
            .bind(tag_id)
            .execute(pool)
            .await
            .map_err(|e| format!("[tags] video_tags insert failed: {e}"))?;

        eprintln!("[tags] assigned '{tag_name}' (id={tag_id}) to video {video_id}");
    }

    Ok(())
}

/// Candidate topics we match against. Extend this list freely.
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
];

/// Uses fastembed cosine similarity to pick the best matching topics
/// from TOPIC_CANDIDATES given the transcript sample.
fn infer_tags_from_text(text: &str) -> Result<Vec<String>, String> {
    use crate::utils::voice::intent_classifier;

    // Grab the shared embedder from intent_classifier's module
    // by calling a thin helper we'll add there.
    let tag_embeddings = intent_classifier::embed_strings(TOPIC_CANDIDATES)?;
    let text_embeddings = intent_classifier::embed_strings(&[text])?;
    let text_emb = &text_embeddings[0];

    const TAG_THRESHOLD: f32 = 0.25;

    let _matched: Vec<String> = TOPIC_CANDIDATES
        .iter()
        .zip(tag_embeddings.iter())
        .filter_map(|(name, emb)| {
            let score = cosine_similarity(text_emb, emb);
            if score >= TAG_THRESHOLD {
                Some((name.to_string(), score))
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .into_iter()
        .filter_map(|(name, score)| {
            eprintln!("[tags] {name:30} score={score:.3}");
            Some(name)
        })
        .collect();

    // Cap at 5 best — sort by score desc first
    let mut scored: Vec<(String, f32)> = TOPIC_CANDIDATES
        .iter()
        .zip(tag_embeddings.iter())
        .map(|(name, emb)| (name.to_string(), cosine_similarity(text_emb, emb)))
        .filter(|(_, s)| *s >= TAG_THRESHOLD)
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    scored.truncate(5);

    Ok(scored.into_iter().map(|(name, _)| name).collect())
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

fn vtt_to_plain_text(vtt: &str) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut last = String::new();

    for line in vtt.lines() {
        let line = line.trim();
        if line.is_empty()
            || line.starts_with("WEBVTT")
            || line.starts_with("NOTE")
            || line.contains("-->")
            || line
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
        {
            continue;
        }
        let clean = strip_vtt_tags(line);
        if clean.is_empty() || clean == last {
            continue;
        }
        last = clean.clone();
        lines.push(clean);
    }

    lines.join(" ")
}

fn strip_vtt_tags(s: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.trim().to_string()
}
