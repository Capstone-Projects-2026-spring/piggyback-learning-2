use std::sync::Arc;

use crate::utils::openai::{generate_and_store_questions, QuestionsResponse};
use async_openai::{config::OpenAIConfig, Client};
use axum::Extension;
use loco_rs::prelude::*;
use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Deserialize, IntoParams)]
pub struct SegmentQuery {
    /// Start of the segment in seconds
    #[param(example = 0)]
    start: i32,
    /// End of the segment in seconds
    #[param(example = 60)]
    end: i32,
}

#[utoipa::path(
    get,
    path = "/api/openai/{video_id}",
    tag = "openai",
    params(
        ("video_id" = String, Path, description = "Video ID", example = "l2FQ8ni1MfM"),
        SegmentQuery,
    ),
    responses(
        (status = 200, description = "Questions generated successfully", body = QuestionsResponse),
        (status = 500, description = "Unknown error occurred"),
    )
)]
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
