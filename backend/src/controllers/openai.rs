use crate::utils::openai::generate_and_store_questions;
use loco_rs::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SegmentQuery {
    start: i32,
    end: i32,
}

pub async fn generate_questions(
    State(ctx): State<AppContext>,
    Path(video_id): Path<String>,
    Query(params): Query<SegmentQuery>,
) -> Result<Response> {
    let result = generate_and_store_questions(&ctx.db, video_id, params.start, params.end).await;

    format::json(result)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("openai")
        .add("/{video_id}", get(generate_questions))
}
