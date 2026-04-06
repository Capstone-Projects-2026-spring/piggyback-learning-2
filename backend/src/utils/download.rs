use serde_json::Value;
use std::{fs, path::Path, process::Command};

pub fn download_video(
    video_id: &str,
) -> Result<Option<(String, String, String, i32, String)>, String> {
    let url = format!("https://www.youtube.com/watch?v={video_id}");
    let dir_path = format!("downloads/{}", video_id);

    if !Path::new(&dir_path).exists() {
        fs::create_dir_all(&dir_path).map_err(|e| e.to_string())?;
    }

    let video_path = format!("{}/{}.mp4", dir_path, video_id);
    if Path::new(&video_path).exists() {
        return Ok(None);
    }

    let output = Command::new("yt-dlp")
        // .arg("--cookies")
        // .arg(std::env::var("YT_DLP_COOKIES").expect("YT_DLP_COOKIES must be set."))
        .arg("-f")
        .arg("mp4")
        .arg("--merge-output-format")
        .arg("mp4")
        .arg("-o")
        .arg(&video_path)
        .arg("--print-json")
        .arg(&url)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(format!(
            "yt-dlp failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let json: Value = serde_json::from_slice(&output.stdout).map_err(|e| {
        format!(
            "Failed to parse yt-dlp JSON: {e}\nstderr: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    })?;

    let video_id = json["id"].as_str().unwrap_or("").to_string();
    let title = json["title"].as_str().unwrap_or("").to_string();
    let thumbnail = json["thumbnail"].as_str().unwrap_or("").to_string();
    let duration = json["duration"].as_i64().unwrap_or(0) as i32;

    Ok(Some((video_id, title, thumbnail, duration, video_path)))
}
