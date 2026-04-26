use crate::db::init::get_db;
use crate::handlers::{
    frames::extract_frames,
    tags::{get_or_create_tag, infer_tags_from_text},
};
use crate::utils::{
    app_handle::emit, download::download_video, mpv, voice::session::SharedSession,
};

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
        eprintln!("[videos] search - empty query after filtering");
        return;
    }

    eprintln!("[videos] search - {query}");
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
                        if parts.len() < 4 { return None; }
                        let duration: i64 = parts[2].parse::<f64>().unwrap_or(0.0) as i64;
                        let live_status = parts[3].trim();
                        if live_status != "NA" || duration == 0 || duration > 300 { return None; }
                        Some(serde_json::json!({
                            "video_id":  parts[0],
                            "title":     parts[1],
                            "thumbnail": format!("https://i.ytimg.com/vi/{}/hqdefault.jpg", parts[0]),
                            "duration":  duration,
                        }))
                    })
                    .take(10)
                    .collect();

                eprintln!("[videos] search - {} results", results.len());
                emit(
                    "orb://search-results",
                    serde_json::json!({ "query": query, "results": results }),
                );
            }
            Ok(out) => eprintln!(
                "[videos] search failed: {}",
                String::from_utf8_lossy(&out.stderr)
            ),
            Err(e) => eprintln!("[videos] search spawn failed: {e}"),
        }
    });
}

#[tauri::command]
pub async fn download_video_command(video_id: String) -> Result<(), String> {
    eprintln!("[videos] download - video_id={video_id}");

    tokio::spawn(async move {
        if let Some(session) = SESSION.get() {
            if let Ok(mut s) = session.lock() {
                s.current_video = Some(video_id.clone());
            }
        }

        match download_video(&video_id).await {
            Ok(None) => {
                eprintln!("[videos] already downloaded - {video_id}");
                emit(
                    "orb://video-status",
                    serde_json::json!({
                        "video_id": video_id, "status": "already_exists"
                    }),
                );
            }
            Ok(Some(video)) => {
                if let Err(e) = upsert_video(
                    &video.id,
                    &video.title,
                    &video.thumbnail,
                    video.duration,
                    &video.video_path,
                )
                .await
                {
                    eprintln!("[videos] upsert failed: {e}");
                    emit(
                        "orb://video-status",
                        serde_json::json!({
                            "video_id": video.id, "status": "error", "msg": e
                        }),
                    );
                    return;
                }

                emit(
                    "orb://video-status",
                    serde_json::json!({
                        "video_id":        video.id,
                        "status":          "done",
                        "title":           video.title,
                        "thumbnail":       video.thumbnail,
                        "duration":        video.duration,
                        "video_path":      video.video_path,
                        "transcript_path": video.transcript_path,
                    }),
                );

                if !video.transcript_path.is_empty() {
                    let id_clone = video.id.clone();
                    let transcript_clone = video.transcript_path.clone();
                    tokio::spawn(async move {
                        emit(
                            "orb://processing-status",
                            serde_json::json!({
                                "video_id": id_clone, "stage": "tagging"
                            }),
                        );
                        if let Err(e) = generate_and_assign_tags(&id_clone, &transcript_clone).await
                        {
                            eprintln!("[videos] tag generation failed: {e}");
                        }
                    });
                }

                let id_clone = video.id.clone();
                tokio::spawn(async move {
                    if let Err(e) = extract_frames(&id_clone).await {
                        eprintln!("[videos] frame extraction failed: {e}");
                    }
                });
            }
            Err(e) => {
                eprintln!("[videos] download failed: {e}");
                emit(
                    "orb://video-status",
                    serde_json::json!({
                        "video_id": video_id, "status": "error", "msg": e
                    }),
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
    eprintln!("[videos] launch - {path}");
    tokio::task::spawn_blocking(move || {
        mpv::launch_mpv(&path)?;
        if !mpv::wait_for_socket() {
            return Err("[mpv] socket never appeared".to_string());
        }
        mpv::play()?;
        mpv::start_position_poller(
            segments
                .iter()
                .map(|s| (s.start_seconds, s.end_seconds, s.id))
                .collect(),
        );
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn mpv_play() -> Result<(), String> {
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

// Private helpers

async fn upsert_video(
    id: &str,
    title: &str,
    thumbnail: &str,
    duration: i32,
    path: &str,
) -> Result<(), String> {
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
    .execute(get_db())
    .await
    .map_err(|e| format!("[videos] upsert failed: {e}"))?;
    eprintln!("[videos] upsert ok - id={id}");
    Ok(())
}

async fn generate_and_assign_tags(video_id: &str, vtt_path: &str) -> Result<(), String> {
    let raw =
        std::fs::read_to_string(vtt_path).map_err(|e| format!("[videos] read vtt failed: {e}"))?;

    let text = vtt_to_plain_text(&raw);
    if text.is_empty() {
        eprintln!("[videos] transcript empty after stripping - skipping tags");
        return Ok(());
    }

    let sample: String = text.chars().take(2000).collect();
    let tags = infer_tags_from_text(&sample)?;
    eprintln!("[videos] tags for {video_id}: {tags:?}");

    let pool = get_db();
    for tag_name in &tags {
        let tag_id = get_or_create_tag(tag_name).await?;
        sqlx::query("INSERT OR IGNORE INTO video_tags (video_id, tag_id) VALUES (?, ?)")
            .bind(video_id)
            .bind(tag_id)
            .execute(pool)
            .await
            .map_err(|e| format!("[videos] video_tags insert failed: {e}"))?;
        eprintln!("[videos] tagged '{tag_name}' (id={tag_id}) - {video_id}");
    }

    Ok(())
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
