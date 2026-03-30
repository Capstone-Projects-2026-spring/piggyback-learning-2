use std::{fs, process::Command};

use loco_rs::prelude::*;

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
        .prefix("frames")
        .add("/extract/{video_id}", get(extract_frames))
}
