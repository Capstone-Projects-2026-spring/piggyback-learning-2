use serde_json::Value;
use std::fs;
use tokio::process::Command;

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

    // Download raw first to a temp file, then transcode
    let raw_path = data_dir.join(format!("{video_id}.raw.mp4"));

    let output = Command::new("yt-dlp")
        .arg("-f")
        // Best mp4 at 720p or below — good quality, light enough for software decode
        .arg("bestvideo[height<=720][ext=mp4]+bestaudio[ext=m4a]/best[height<=720][ext=mp4]/best[height<=720]")
        .arg("--merge-output-format")
        .arg("mp4")
        .arg("-o")
        .arg(raw_path.to_string_lossy().as_ref())
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

    eprintln!("[download] yt-dlp done — '{title}' ({duration}s), transcoding...");

    // Transcode to H.264 Baseline profile which WebKit handles well on all platforms
    let transcode = Command::new("ffmpeg")
        .arg("-i")
        .arg(raw_path.to_string_lossy().as_ref())
        .arg("-c:v")
        .arg("libx264")
        .arg("-profile:v")
        .arg("baseline")
        .arg("-level")
        .arg("3.0")
        .arg("-vf")
        .arg("scale=-2:480") // 480p — plenty for a kids app, very light to decode
        .arg("-c:a")
        .arg("aac")
        .arg("-b:a")
        .arg("128k")
        .arg("-movflags")
        .arg("+faststart") // moov atom at front for faster start
        .arg("-y")
        .arg(video_path.to_string_lossy().as_ref())
        .output()
        .await
        .map_err(|e| format!("[download] ffmpeg spawn failed: {e}"))?;

    // Clean up raw file regardless of transcode result
    let _ = fs::remove_file(&raw_path);

    if !transcode.status.success() {
        return Err(format!(
            "[download] ffmpeg transcode failed: {}",
            String::from_utf8_lossy(&transcode.stderr)
        ));
    }

    eprintln!("[download] transcode done → {}", video_path.display());

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
