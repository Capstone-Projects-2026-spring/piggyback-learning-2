use loco_rs::prelude::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde::Serialize;
use std::collections::HashMap;
use utoipa::ToSchema;

use crate::models::_entities::{questions, segments};

#[derive(Serialize, ToSchema)]
pub struct FollowUp {
    pub question: String,
    pub answer: String,
}

#[derive(Serialize, ToSchema)]
pub struct QuestionItem {
    pub qtype: String,
    pub question: String,
    pub answer: String,
    pub rank: Option<i32>,
    pub followup_enabled: Option<bool>,
    pub followup_for_correct_answer: Option<FollowUp>,
    pub followup_for_wrong_answer: Option<FollowUp>,
}

#[derive(Serialize, ToSchema)]
pub struct SegmentWithQuestions {
    pub id: i32,
    pub start_seconds: i32,
    pub end_seconds: i32,
    pub best_question: Option<String>,
    pub questions: Vec<QuestionItem>,
}

#[derive(Serialize, ToSchema)]
pub struct VideoQuestionsResponse {
    pub video_id: String,
    pub segments: Vec<SegmentWithQuestions>,
}

#[utoipa::path(
    get,
    path = "/api/questions/{video_id}",
    tag = "questions",
    params(
        ("video_id" = String, Path, description = "Video ID", example = "l2FQ8ni1MfM"),
    ),
    responses(
        (status = 200, description = "Questions grouped by segment for the video", body = VideoQuestionsResponse),
    )
)]
async fn get_questions_by_video(
    State(ctx): State<AppContext>,
    Path(video_id): Path<String>,
) -> Result<Response> {
    let segments = segments::Entity::find()
        .filter(segments::Column::VideoId.eq(&video_id))
        .order_by_asc(segments::Column::StartSeconds)
        .all(&ctx.db)
        .await?;

    if segments.is_empty() {
        return format::json(VideoQuestionsResponse {
            video_id,
            segments: vec![],
        });
    }

    let segment_ids: Vec<i32> = segments.iter().map(|s| s.id).collect();

    let questions = questions::Entity::find()
        .filter(questions::Column::SegmentId.is_in(segment_ids.clone()))
        .all(&ctx.db)
        .await?;

    let mut grouped: HashMap<i32, Vec<QuestionItem>> = HashMap::new();

    for q in questions {
        grouped.entry(q.segment_id).or_default().push(QuestionItem {
            qtype: q.qtype,
            question: q.question,
            answer: q.answer,
            rank: q.rank,
            // followup_enabled: q.followup_enabled.unwrap_or(false),
            // followup_for_correct_answer: q.followup_for_correct_answer.map(|f| FollowUp {
            //     question: f.question,
            //     answer: f.answer,
            // }),
            // followup_for_wrong_answer: q.followup_for_wrong_answer.map(|f| FollowUp {
            //     question: f.question,
            //     answer: f.answer,
            // }),
            followup_enabled: Some(true),
            followup_for_correct_answer: Some(FollowUp {
                question: "Bob question".to_string(),
                answer: "Bob answer".to_string(),
            }),
            followup_for_wrong_answer: Some(FollowUp {
                question: "Tom question".to_string(),
                answer: "Tom answer".to_string(),
            }),
        });
    }

    let segments_with_questions = segments
        .into_iter()
        .map(|s| SegmentWithQuestions {
            id: s.id,
            start_seconds: s.start_seconds,
            end_seconds: s.end_seconds,
            best_question: s.best_question,
            questions: grouped.remove(&s.id).unwrap_or_default(),
        })
        .collect();

    format::json(VideoQuestionsResponse {
        video_id,
        segments: segments_with_questions,
    })
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("questions")
        .add("/{video_id}", get(get_questions_by_video))
}
