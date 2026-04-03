use serde_json::Value;
use std::{fs, path::Path, process::Command};

pub fn download_video(
    video_id: &str,
) -> Result<Option<(String, String, String, i32, String)>, String> {
    let url = format!("https://www.youtube.com/watch?v={video_id}");

    let dir_path = format!("downloads/{}", video_id);
    if !Path::new(&dir_path).exists() {
        let created = fs::create_dir_all(&dir_path);
        if created.is_err() {
            return Err(created.err().unwrap().to_string());
        }
    }

    let video_path = format!("{}/{}.mp4", dir_path, video_id);
    if Path::new(&video_path).exists() {
        return Ok(None);
    }

    let output = Command::new("yt-dlp").arg("-j").arg(&url).output();
    if output.is_err() {
        return Err(output.err().unwrap().to_string());
    }
    let output = output.unwrap();

    let json = serde_json::from_slice(&output.stdout);
    if json.is_err() {
        return Err(json.err().unwrap().to_string());
    }
    let json: Value = json.unwrap();

    let video_id = json["id"].as_str().unwrap().to_string();
    let title = json["title"].as_str().unwrap_or("").to_string();
    let thumbnail = json["thumbnail"].as_str().unwrap_or("").to_string();
    let duration = json["duration"].as_i64().unwrap_or(0) as i32;

    let res = Command::new("yt-dlp")
        .arg(url)
        .arg("-f")
        .arg("mp4")
        .arg("--merge-output-format")
        .arg("mp4")
        .arg("-o")
        .arg(&video_path)
        .status();
    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }

    Ok(Some((video_id, title, thumbnail, duration, video_path)))
}
