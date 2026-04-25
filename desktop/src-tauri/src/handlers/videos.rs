use crate::db::init::get_db;
use crate::handlers::frames::extract_frames;
use crate::utils::app_handle::emit;
use crate::utils::download::download_video;
use crate::utils::mpv;
use crate::utils::voice::session::SharedSession;
use std::sync::OnceLock;

static SESSION: OnceLock<SharedSession> = OnceLock::new();

pub fn init_session(session: SharedSession) {
    SESSION.set(session).ok();
}

pub fn bring_tauri_to_front() {
    if let Some(window) =
        tauri::Manager::get_webview_window(crate::utils::app_handle::get_app_handle(), "main")
    {
        let _ = window.set_always_on_top(true);
        let _ = window.set_focus();
    }
}

fn release_tauri_front() {
    if let Some(window) =
        tauri::Manager::get_webview_window(crate::utils::app_handle::get_app_handle(), "main")
    {
        let _ = window.set_always_on_top(false);
    }
}

pub async fn search(args: &[String]) {
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

    emit(
        "orb://search-status",
        serde_json::json!({ "status": "searching", "query": query }),
    );

    tokio::spawn(async move {
        let output = Command::new("yt-dlp")
            .arg("--flat-playlist")
            .arg("--no-cache-dir")
            .arg("--extractor-args")
            .arg("youtube:skip=dash,hls,translated_subs")
            .arg("--print")
            .arg("%(id)s\t%(title)s\t%(duration)s\t%(live_status)s")
            .arg("--no-warnings")
            .arg(format!("ytsearch50:{query}"))
            .output()
            .await;

        match output {
            Ok(out) if out.status.success() => {
                let results: Vec<serde_json::Value> = String::from_utf8_lossy(&out.stdout)
                    .lines()
                    .filter(|l| !l.is_empty())
                    .filter_map(|line| {
                        let parts: Vec<&str> = line.splitn(4, '\t').collect();
                        if parts.len() < 4 {
                            return None;
                        }
                        let video_id = parts[0];
                        let title = parts[1];
                        let duration_str = parts[2];
                        let live_status = parts[3].trim();
                        let duration: i64 = duration_str.parse::<f64>().unwrap_or(0.0) as i64;
                        if live_status != "NA" || duration == 0 || duration > 300 {
                            return None;
                        }
                        let thumbnail = format!("https://i.ytimg.com/vi/{video_id}/hqdefault.jpg");
                        Some(serde_json::json!({
                            "video_id": video_id,
                            "title":    title,
                            "thumbnail": thumbnail,
                            "duration": duration,
                            "uploader": "",
                        }))
                    })
                    .take(10)
                    .collect();

                eprintln!("[handler:videos] search → {} results", results.len());
                emit(
                    "orb://search-results",
                    serde_json::json!({ "query": query, "results": results }),
                );
            }
            Ok(out) => eprintln!(
                "[handler:videos] search failed: {}",
                String::from_utf8_lossy(&out.stderr)
            ),
            Err(e) => eprintln!("[handler:videos] search spawn failed: {e}"),
        }
    });
}

#[tauri::command]
pub async fn download_video_command(video_id: String) -> Result<(), String> {
    eprintln!("[handler:videos] download_video_command — video_id={video_id}");

    tokio::spawn(async move {
        // Set current_video immediately before the match
        if let Some(session) = SESSION.get() {
            if let Ok(mut s) = session.lock() {
                s.current_video = Some(video_id.clone());
                eprintln!("[handler:videos] session.current_video = {video_id}");
            }
        }

        match download_video(&video_id).await {
            Ok(None) => {
                eprintln!("[handler:videos] already downloaded — {video_id}");
                emit(
                    "orb://video-status",
                    serde_json::json!({ "video_id": video_id, "status": "already_exists" }),
                );
            }
            Ok(Some((id, title, thumbnail, duration, video_path, transcript_path))) => {
                if let Err(e) = upsert_video(&id, &title, &thumbnail, duration, &video_path).await {
                    eprintln!("[handler:videos] upsert failed: {e}");
                    emit(
                        "orb://video-status",
                        serde_json::json!({ "video_id": id, "status": "error", "msg": e }),
                    );
                    return;
                }

                emit(
                    "orb://video-status",
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

                if !transcript_path.is_empty() {
                    let id_clone = id.clone();
                    let transcript_clone = transcript_path.clone();
                    tokio::spawn(async move {
                        emit(
                            "orb://processing-status",
                            serde_json::json!({ "video_id": id_clone, "stage": "tagging" }),
                        );
                        if let Err(e) = generate_and_assign_tags(&id_clone, &transcript_clone).await
                        {
                            eprintln!("[handler:videos] tag generation failed: {e}");
                        }
                    });
                }

                let id_clone = id.clone();
                tokio::spawn(async move {
                    if let Err(e) = extract_frames(&id_clone).await {
                        eprintln!("[handler:videos] frame extraction failed: {e}");
                    }
                });
            }
            Err(e) => {
                eprintln!("[handler:videos] download failed: {e}");
                emit(
                    "orb://video-status",
                    serde_json::json!({ "video_id": video_id, "status": "error", "msg": e }),
                );
            }
        }
    });

    Ok(())
}

#[derive(serde::Deserialize)]
pub struct SegmentInfo {
    pub start_seconds: f64,
    pub end_seconds: f64,
    pub id: i64,
}

#[tauri::command]
pub async fn launch_video(path: String, segments: Vec<SegmentInfo>) -> Result<(), String> {
    eprintln!("[handler:videos] launch_video — {path}");

    tokio::task::spawn_blocking(move || {
        mpv::launch_mpv(&path)?;

        if !mpv::wait_for_socket() {
            return Err("[mpv] socket never appeared".to_string());
        }

        mpv::play()?;

        let seg_tuples: Vec<(f64, f64, i64)> = segments
            .iter()
            .map(|s| (s.start_seconds, s.end_seconds, s.id))
            .collect();
        mpv::start_position_poller(seg_tuples);

        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn mpv_play() -> Result<(), String> {
    eprintln!("[mpv_play] restoring + releasing front");
    mpv::restore()?;
    release_tauri_front();
    mpv::play()
}

#[tauri::command]
pub async fn mpv_pause() -> Result<(), String> {
    mpv::pause()
}

#[tauri::command]
pub async fn mpv_seek(seconds: f64) -> Result<(), String> {
    mpv::restore()?;
    release_tauri_front();
    mpv::seek(seconds)
}

#[tauri::command]
pub async fn mpv_minimize() -> Result<(), String> {
    mpv::minimize()?;
    bring_tauri_to_front();
    Ok(())
}

#[tauri::command]
pub async fn mpv_quit() {
    mpv::quit();
    release_tauri_front();
}

// ── Private helpers ───────────────────────────────────────────────────────────

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

async fn generate_and_assign_tags(video_id: &str, vtt_path: &str) -> Result<(), String> {
    let raw =
        std::fs::read_to_string(vtt_path).map_err(|e| format!("[tags] read vtt failed: {e}"))?;

    let text = vtt_to_plain_text(&raw);
    if text.is_empty() {
        eprintln!("[tags] transcript empty after stripping — skipping");
        return Ok(());
    }

    let sample: String = text.chars().take(2000).collect();
    let tags = infer_tags_from_text(&sample)?;
    eprintln!("[tags] inferred for video {video_id}: {tags:?}");

    if tags.is_empty() {
        return Ok(());
    }

    let pool = get_db();

    for tag_name in &tags {
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

fn infer_tags_from_text(text: &str) -> Result<Vec<String>, String> {
    let tag_embeddings = crate::utils::voice::intent_classifier::embed_strings(TOPIC_CANDIDATES)?;
    let text_embeddings = crate::utils::voice::intent_classifier::embed_strings(&[text])?;
    let text_emb = &text_embeddings[0];

    const TAG_THRESHOLD: f32 = 0.25;

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
