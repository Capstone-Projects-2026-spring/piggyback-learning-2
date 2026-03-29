use crate::{models::videos, utils::download::download_video};
use loco_rs::prelude::*;
use serde::Deserialize;

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

pub fn routes() -> Routes {
    Routes::new()
        .prefix("videos")
        .add("/download", post(download_and_store))
}
