use serde_json::Value;
use std::{fs, path::Path, process::Command};

pub fn download_video(url: &str) -> Option<(String, String, String, i32, String)> {
    // Get metadata as JSON
    let output = Command::new("yt-dlp").arg("-j").arg(url).output();
    if output.is_err() {
        println!("{:#?}", output);
        return None;
    }
    let output = output.unwrap();

    let json = serde_json::from_slice(&output.stdout);
    if json.is_err() {
        println!("{:#?}", json);
        return None;
    }
    let json: Value = json.unwrap();

    let video_id = json["id"].as_str().unwrap().to_string();
    let title = json["title"].as_str().unwrap_or("").to_string();
    let thumbnail = json["thumbnail"].as_str().unwrap_or("").to_string();
    let duration = json["duration"].as_i64().unwrap_or(0) as i32;

    let dir_path = format!("downloads/{}", video_id);
    if !Path::new(&dir_path).exists() {
        let created = fs::create_dir_all(&dir_path);
        if created.is_err() {
            println!("Error creating downloads directory");
            return None;
        }
    }

    if Path::new(&dir_path).exists() {
        println!("Video already downloaded, skipping...");

        return None;
    }

    // Download video
    let output_template = format!("downloads/{}/%(title)s.%(ext)s", video_id);

    let res = Command::new("yt-dlp")
        .arg(url)
        .arg("-f")
        .arg("mp4")
        .arg("--merge-output-format")
        .arg("mp4")
        .arg("-o")
        .arg(&output_template)
        .status();
    if res.is_err() {
        println!("Error during download: {:#?}", res);
        return None;
    }

    // Build local path
    let local_path = format!("downloads/{}/", video_id);

    Some((video_id, title, thumbnail, duration, local_path))
}
