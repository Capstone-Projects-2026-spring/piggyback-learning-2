use loco_rs::prelude::*;
use serde::Deserialize;

use crate::models::_entities::tags;

#[derive(Deserialize)]
struct CreateTagRequest {
    name: String,
}

async fn create_tag(
    State(ctx): State<AppContext>,
    Json(data): Json<CreateTagRequest>,
) -> Result<Response> {
    let name = data.name.trim().to_lowercase();

    let existing = tags::Entity::find()
        .filter(tags::Column::Name.eq(name.clone()))
        .one(&ctx.db)
        .await?;
    if let Some(tag) = existing {
        return format::json(tag);
    }

    let tag = tags::ActiveModel {
        name: Set(name.clone()),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;

    format::json(tag)
}

async fn get_tags(State(ctx): State<AppContext>) -> Result<Response> {
    format::json(tags::Entity::find().all(&ctx.db).await?)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("tags")
        .add("/", get(get_tags))
        .add("/", post(create_tag))
}
