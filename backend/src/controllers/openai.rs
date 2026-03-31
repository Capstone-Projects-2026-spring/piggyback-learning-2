use std::sync::Arc;

use crate::utils::openai::generate_and_store_questions;
use async_openai::{config::OpenAIConfig, Client};
use axum::Extension;
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
    Extension(openai_client): Extension<Arc<Client<OpenAIConfig>>>,
) -> Result<Response> {
    let result =
        generate_and_store_questions(&ctx.db, &openai_client, video_id, params.start, params.end)
            .await;
    if result.is_err() {
        println!("{:#?}", result);
        return format::json(
            serde_json::json!({"success": false, "msg": "Unknown error occurred"}),
        );
    }

    format::json(result.unwrap())
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("openai")
        .add("/{video_id}", get(generate_questions))
}
