use loco_rs::prelude::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::models::_entities::kids;

#[utoipa::path(
    get,
    path = "/api/parents/{parent_id}/kids",
    tag = "parents",
    params(
        ("parent_id" = i32, Path, description = "Parent ID", example = 1),
    ),
    responses(
        (status = 200, description = "Kids belonging to the parent", body = Vec<kids::Model>),
    )
)]
pub async fn get_kids(
    State(ctx): State<AppContext>,
    Path(parent_id): Path<i32>,
) -> Result<Response> {
    let kids = kids::Entity::find()
        .filter(kids::Column::ParentId.eq(parent_id))
        .all(&ctx.db)
        .await?;

    format::json(kids)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("parents")
        .add("/{parent_id}/kids", get(get_kids))
}
