use loco_rs::prelude::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::models::_entities::kids;

pub async fn get_kids_by_parent(
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
        .add("/{parent_id}/kids", get(get_kids_by_parent))
}
