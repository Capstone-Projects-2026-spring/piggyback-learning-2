use loco_rs::prelude::*;
use sea_orm::{sea_query::OnConflict, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};

use crate::models::_entities::{kid_tags, tags, video_assignments, videos};

#[derive(Serialize)]
struct TagResponse {
    id: i32,
    name: String,
}

#[derive(Deserialize)]
struct AddTagsRequest {
    tags: Vec<i32>,
}

async fn get_kid_tags(State(ctx): State<AppContext>, Path(kid_id): Path<i32>) -> Result<Response> {
    let results = kid_tags::Entity::find()
        .filter(kid_tags::Column::KidId.eq(kid_id))
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

async fn add_kid_tags(
    State(ctx): State<AppContext>,
    Path(kid_id): Path<i32>,
    Json(data): Json<AddTagsRequest>,
) -> Result<Response> {
    let models = data.tags.into_iter().map(|tag_id| kid_tags::ActiveModel {
        kid_id: Set(kid_id),
        tag_id: Set(tag_id),
        ..Default::default()
    });

    match kid_tags::Entity::insert_many(models)
        .on_conflict(
            OnConflict::columns([kid_tags::Column::KidId, kid_tags::Column::TagId])
                .do_nothing()
                .to_owned(),
        )
        .exec(&ctx.db)
        .await
    {
        Ok(_) => {}
        Err(sea_orm::DbErr::RecordNotInserted) => {
            // This means all rows already existed totally fine
        }
        Err(_) => {
            return format::json(
                serde_json::json!({"success": false, "msg": "Unknown error occurred"}),
            )
        }
    }

    format::json(serde_json::json!({
        "success": true
    }))
}

#[derive(Deserialize)]
pub struct CreateVideoAssignmentRequest {
    pub video_id: i32,
}

async fn create_video_assignment(
    State(ctx): State<AppContext>,
    Path(kid_id): Path<i32>,
    Json(data): Json<CreateVideoAssignmentRequest>,
) -> Result<Response> {
    let model = video_assignments::ActiveModel {
        kid_id: Set(kid_id),
        video_id: Set(data.video_id),
        ..Default::default()
    };

    match video_assignments::Entity::insert(model)
        .on_conflict(
            OnConflict::columns([
                video_assignments::Column::KidId,
                video_assignments::Column::VideoId,
            ])
            .do_nothing()
            .to_owned(),
        )
        .exec(&ctx.db)
        .await
    {
        Ok(_) => {}
        Err(DbErr::RecordNotInserted) => {
            // already exists → fine
        }
        Err(_) => {
            return format::json(
                serde_json::json!({"success": false, "msg": "Unknown error occurred"}),
            )
        }
    }

    format::json(serde_json::json!({
        "success": true
    }))
}

#[derive(Serialize)]
pub struct GetVideosResponse {
    pub success: bool,
    pub data: Vec<videos::Model>,
}

async fn get_video_assignments(
    State(ctx): State<AppContext>,
    Path(kid_id): Path<i32>,
) -> Result<Response> {
    let results = video_assignments::Entity::find()
        .filter(video_assignments::Column::KidId.eq(kid_id))
        .find_also_related(videos::Entity)
        .all(&ctx.db)
        .await?;

    // Extract only videos (skip None just in case)
    let videos: Vec<videos::Model> = results.into_iter().filter_map(|(_, video)| video).collect();

    format::json(GetVideosResponse {
        success: true,
        data: videos,
    })
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("kids")
        .add("/{kid_id}/tags", get(get_kid_tags))
        .add("/{kid_id}/tags", post(add_kid_tags))
        .add("/{kid_id}/videos_assigned", get(get_video_assignments))
        .add("/{kid_id}/videos_assigned", post(create_video_assignment))
}
