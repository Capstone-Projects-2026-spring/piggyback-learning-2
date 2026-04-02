use loco_rs::prelude::*;
use sea_orm::{EntityTrait, Set};
use std::{fs, process::Command};

use crate::models::_entities::frames;

async fn extract_frames(
    State(ctx): State<AppContext>,
    Path(video_id): Path<String>,
) -> Result<Response> {
    let input = format!("downloads/{}/{}.mp4", video_id, video_id);
    let output_dir = format!("downloads/{}/extracted_frames", video_id);

    if std::path::Path::new(&output_dir).exists() {
        return format::json(
            serde_json::json!({"success": true, "msg": "Frames already extracted!"}),
        );
    }

    fs::create_dir_all(&output_dir)?;

    let status = Command::new("ffmpeg")
        .args([
            "-i",
            &input,
            "-vf",
            "fps=1",
            &format!("{}/frame_%04d.jpg", output_dir),
        ])
        .status()?;

    if !status.success() {
        return Err(Error::BadRequest("FFMPEG failed".to_string()));
    }

    let mut files: Vec<_> = fs::read_dir(&output_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .collect();

    files.sort_by_key(|f| f.path());

    let mut frames_to_insert = Vec::new();

    let fps = 1.0;

    for (idx, entry) in files.iter().enumerate() {
        let path = entry.path();

        let filename = path.file_name().unwrap().to_string_lossy().to_string();

        let timestamp_seconds = (idx as f64 / fps) as i32;

        let timestamp_formatted = format!(
            "{:02}:{:02}",
            timestamp_seconds / 60,
            timestamp_seconds % 60
        );

        frames_to_insert.push(frames::ActiveModel {
            video_id: Set(video_id.clone()),
            frame_number: Set(idx as i32),
            timestamp_seconds: Set(timestamp_seconds),
            timestamp_formatted: Set(timestamp_formatted),
            filename: Set(filename),
            file_path: Set(path.to_string_lossy().to_string()),
            subtitle_text: Set(None),
            is_keyframe: Set(false),
            ..Default::default()
        });
    }

    if !frames_to_insert.is_empty() {
        frames::Entity::insert_many(frames_to_insert)
            .exec(&ctx.db)
            .await?;
    }

    format::json(serde_json::json!({"success": true, "msg": "Frames extracted!"}))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("frames")
        .add("/extract/{video_id}", get(extract_frames))
}
