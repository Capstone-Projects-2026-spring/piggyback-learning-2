use serde_json::Value;
use std::{fs, path::PathBuf};
use tokio::process::Command;

/// All paths relevant to a single downloaded video.
pub struct DownloadedVideo {
    pub id: String,
    pub title: String,
    pub thumbnail: String,
    pub duration: i32,
    pub video_path: String,
    /// Empty if no subtitles were available - non-fatal.
    pub transcript_path: String,
}

/// Download a YouTube video by ID into the local data directory.
/// Returns `Ok(None)` if the video is already on disk.
pub async fn download_video(video_id: &str) -> Result<Option<DownloadedVideo>, String> {
    let data_dir = video_data_dir(video_id)?;
    let video_path = data_dir.join(format!("{video_id}.mp4"));

    if video_path.exists() {
        eprintln!("[download] already exists: {}", video_path.display());
        return Ok(None);
    }

    let url = format!("https://www.youtube.com/watch?v={video_id}");
    eprintln!("[download] starting for {video_id}");

    // Video
    let output = Command::new("yt-dlp")
        .args([
            "-f",
            "mp4",
            "--merge-output-format",
            "mp4",
            "-o",
            video_path.to_string_lossy().as_ref(),
            "--print-json",
            &url,
        ])
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

    eprintln!("[download] video done - '{title}' ({duration}s)");

    // Transcript (non-fatal)
    let transcript_path = download_transcript(video_id, &data_dir, &url).await;

    Ok(Some(DownloadedVideo {
        id,
        title,
        thumbnail,
        duration,
        video_path: video_path.to_string_lossy().to_string(),
        transcript_path,
    }))
}

/// Returns the local data directory for a given video, creating it if needed.
pub fn video_data_dir(video_id: &str) -> Result<PathBuf, String> {
    let dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("piggyback")
        .join("downloads")
        .join(video_id);

    fs::create_dir_all(&dir).map_err(|e| format!("[download] create_dir failed: {e}"))?;
    Ok(dir)
}

/// Download English subtitles for a video. Returns the VTT path on success,
/// empty string if unavailable - never propagates an error since transcripts
/// are optional for the app to function.
async fn download_transcript(video_id: &str, data_dir: &PathBuf, url: &str) -> String {
    let vtt_path = data_dir.join(format!("{video_id}.en.vtt"));

    let result = Command::new("yt-dlp")
        .args([
            "--skip-download",
            "--write-auto-sub",
            "--write-sub",
            "--sub-lang",
            "en",
            "--sub-format",
            "vtt",
            "-o",
            data_dir.join("%(id)s").to_string_lossy().as_ref(),
            url,
        ])
        .output()
        .await;

    match result {
        Ok(out) if out.status.success() && vtt_path.exists() => {
            eprintln!("[download] transcript saved - {}", vtt_path.display());
            vtt_path.to_string_lossy().to_string()
        }
        _ => {
            eprintln!("[download] transcript unavailable for {video_id}");
            String::new()
        }
    }
}
