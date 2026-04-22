use serde_json::Value;
use std::fs;
use tokio::process::Command;

/// Returns Ok(None) if already downloaded.
/// Returns Ok(Some((id, title, thumbnail, duration, video_path, transcript_path)))
/// transcript_path is empty string if no subtitles were available.
pub async fn download_video(
    video_id: &str,
) -> Result<Option<(String, String, String, i32, String, String)>, String> {
    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("piggyback")
        .join("downloads")
        .join(video_id);

    fs::create_dir_all(&data_dir).map_err(|e| format!("[download] create_dir failed: {e}"))?;

    let video_path = data_dir.join(format!("{video_id}.mp4"));

    if video_path.exists() {
        eprintln!("[download] already exists: {}", video_path.display());
        return Ok(None);
    }

    let url = format!("https://www.youtube.com/watch?v={video_id}");

    eprintln!("[download] starting yt-dlp for {video_id}");

    // ── Video ────────────────────────────────────────────────────────────────
    let output = Command::new("yt-dlp")
        .arg("-f")
        .arg("mp4")
        .arg("--merge-output-format")
        .arg("mp4")
        .arg("-o")
        .arg(video_path.to_string_lossy().as_ref())
        .arg("--print-json")
        .arg(&url)
        .output()
        .await
        .map_err(|e| format!("[download] yt-dlp spawn failed: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "[download] yt-dlp failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let json: Value = serde_json::from_slice(&output.stdout).map_err(|e| {
        format!(
            "[download] JSON parse failed: {e}\nstderr: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    })?;

    let id = json["id"].as_str().unwrap_or("").to_string();
    let title = json["title"].as_str().unwrap_or("").to_string();
    let thumbnail = json["thumbnail"].as_str().unwrap_or("").to_string();
    let duration = json["duration"].as_i64().unwrap_or(0) as i32;

    eprintln!("[download] video done — '{title}' ({duration}s)");

    // ── Transcript (non-fatal) ───────────────────────────────────────────────
    let vtt_path = data_dir.join(format!("{video_id}.en.vtt"));

    let transcript_result = Command::new("yt-dlp")
        .arg("--skip-download")
        .arg("--write-auto-sub")
        .arg("--write-sub")
        .arg("--sub-lang")
        .arg("en")
        .arg("--sub-format")
        .arg("vtt")
        .arg("-o")
        .arg(data_dir.join("%(id)s").to_string_lossy().as_ref())
        .arg(&url)
        .output()
        .await;

    let transcript_path = match transcript_result {
        Ok(out) if out.status.success() && vtt_path.exists() => {
            eprintln!("[download] transcript saved → {}", vtt_path.display());
            vtt_path.to_string_lossy().to_string()
        }
        _ => {
            eprintln!("[download] transcript unavailable for {video_id}");
            String::new()
        }
    };

    Ok(Some((
        id,
        title,
        thumbnail,
        duration,
        video_path.to_string_lossy().to_string(),
        transcript_path,
    )))
}
