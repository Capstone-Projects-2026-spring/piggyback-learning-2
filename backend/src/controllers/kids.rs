use loco_rs::prelude::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect, Set};
use serde::{Deserialize, Serialize};

use crate::models::_entities::{kid_tags, tags};

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
    let tag_ids = data.tags;

    let existing: Vec<i32> = kid_tags::Entity::find()
        .select_only()
        .column(kid_tags::Column::TagId)
        .filter(kid_tags::Column::KidId.eq(kid_id))
        .filter(kid_tags::Column::TagId.is_in(tag_ids.clone()))
        .into_tuple()
        .all(&ctx.db)
        .await?;

    let new_tags: Vec<i32> = tag_ids
        .into_iter()
        .filter(|id| !existing.contains(id))
        .collect();

    if !new_tags.is_empty() {
        let models = new_tags.into_iter().map(|tag_id| kid_tags::ActiveModel {
            kid_id: Set(kid_id),
            tag_id: Set(tag_id),
            ..Default::default()
        });

        kid_tags::Entity::insert_many(models).exec(&ctx.db).await?;
    }

    format::json(serde_json::json!({
        "success": true
    }))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("kids")
        .add("/{kid_id}/tags", get(get_kid_tags))
        .add("/{kid_id}/tags", post(add_kid_tags))
}
