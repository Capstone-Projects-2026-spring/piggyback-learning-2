use loco_rs::prelude::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

use crate::{
    models::{
        _entities::{tags, video_tags},
        videos,
    },
    utils::download::download_video,
};

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

#[derive(Serialize)]
struct TagResponse {
    id: i32,
    name: String,
}

#[derive(Deserialize)]
struct AddTagsRequest {
    tag_ids: Vec<i32>,
}
async fn get_video_tags(
    State(ctx): State<AppContext>,
    Path(video_id): Path<String>,
) -> Result<Response> {
    let results = video_tags::Entity::find()
        .filter(video_tags::Column::VideoId.eq(video_id))
        .find_also_related(tags::Entity)
        .all(&ctx.db)
        .await?;

    let tags: Vec<TagResponse> = results
        .into_iter()
        .filter_map(|(_, tag)| tag)
        .map(|t| TagResponse {
            id: t.id,
            name: t.name,
        })
        .collect();

    format::json(tags)
}

async fn add_video_tags(
    State(ctx): State<AppContext>,
    Path(video_id): Path<String>,
    Json(data): Json<AddTagsRequest>,
) -> Result<Response> {
    for tag_id in data.tag_ids {
        // Prevent duplicates
        let exists = video_tags::Entity::find()
            .filter(video_tags::Column::VideoId.eq(video_id.clone()))
            .filter(video_tags::Column::TagId.eq(tag_id))
            .one(&ctx.db)
            .await?;

        if exists.is_none() {
            video_tags::Entity::insert(video_tags::ActiveModel {
                video_id: Set(video_id.clone()),
                tag_id: Set(tag_id),
                ..Default::default()
            })
            .exec(&ctx.db)
            .await?;
        }
    }

    format::json(serde_json::json!({
        "success": true
    }))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("videos")
        .add("/download", post(download_and_store))
        .add("/{video_id}/tags", get(get_video_tags))
        .add("/{video_id}/tags", post(add_video_tags))
}
