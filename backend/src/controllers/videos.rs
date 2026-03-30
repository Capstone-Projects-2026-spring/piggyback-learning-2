use loco_rs::prelude::*;
use serde::Deserialize;
use std::fs;
use std::process::Command;

use crate::{models::videos, utils::download::download_video};

#[derive(Deserialize)]
struct DownloadRequest {
    url: String,
}

async fn download_and_store(
    State(ctx): State<AppContext>,
    Json(data): Json<DownloadRequest>,
) -> Result<Response> {
    if let Some((video_id, title, thumbnail, duration, path)) = download_video(&data.url) {
        let _ = videos::Entity::insert(videos::ActiveModel {
            id: Set(video_id.clone()),
            title: Set(Some(title)),
            thumbnail_url: Set(Some(thumbnail)),
            duration_seconds: Set(Some(duration)),
            local_video_path: Set(Some(path)),
            ..Default::default()
        })
        .exec(&ctx.db)
        .await?;

        return format::json(serde_json::json!({
            "success": true,
            "video_id": video_id
        }));
    } else {
        return format::empty();
    }
}

async fn extract_frames(
    State(_ctx): State<AppContext>,
    Path(video_id): Path<String>,
) -> Result<Response> {
    let input = format!("downloads/{}/{}.mp4", video_id, video_id);
    let output_dir = format!("downloads/{}/extracted_frames", video_id);

    fs::create_dir_all(&output_dir)?;

    let status = Command::new("ffmpeg")
        .arg("-i")
        .arg(&input)
        .arg("-vf")
        .arg("fps=1") // 1 frame per second
        .arg(format!("{}/frame_%04d.jpg", output_dir))
        .status()?;

    if !status.success() {
        return Err(Error::BadRequest("FFMPEG failed".to_string()));
    }

    format::empty()
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("videos")
        .add("/download", post(download_and_store))
        .add("/extract_frames/{video_id}", get(extract_frames))
}
