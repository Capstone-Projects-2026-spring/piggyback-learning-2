use loco_rs::prelude::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, ActiveModelTrait, Set};
use serde::{Serialize, Deserialize};
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
    pub id: i32,
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

#[derive(Deserialize, ToSchema)]
pub struct UpdateQuestionPayload {
    pub question: Option<String>,
    pub answer: Option<String>,
    pub followup_enabled: Option<bool>,
    pub followup_correct_question: Option<String>,
    pub followup_correct_answer: Option<String>,
    pub followup_wrong_question: Option<String>,
    pub followup_wrong_answer: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateSegmentBestQuestionPayload {
    pub best_question: Option<String>,
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
            id: q.id,
            qtype: q.qtype,
            question: q.question,
            answer: q.answer,
            rank: q.rank,
            followup_enabled: Some(q.followup_enabled.unwrap_or(false)),
            followup_for_correct_answer: match (q.followup_correct_question, q.followup_correct_answer) {
                (Some(question), Some(answer)) => Some(FollowUp { question, answer }),
                _ => None,
            },
            followup_for_wrong_answer: match (q.followup_wrong_question, q.followup_wrong_answer) {
                (Some(question), Some(answer)) => Some(FollowUp { question, answer }),
                _ => None,
            },
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


#[utoipa::path(
    patch,
    path = "/api/questions/{id}",
    tag = "questions",
    params(
        ("id" = i32, Path, description = "Question ID"),
    ),
    request_body = UpdateQuestionPayload,
    responses(
        (status = 200, description = "Question updated successfully"),
        (status = 404, description = "Question not found"),
    )
)]
async fn update_question(
    State(ctx): State<AppContext>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateQuestionPayload>,
) -> Result<Response> {
    let question = questions::Entity::find_by_id(id)
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)?;

    let mut active: questions::ActiveModel = question.into();

    if let Some(v) = payload.question { active.question = Set(v); }
    if let Some(v) = payload.answer { active.answer = Set(v); }
    if let Some(v) = payload.followup_enabled { active.followup_enabled = Set(Some(v)); }
    if let Some(v) = payload.followup_correct_question { active.followup_correct_question = Set(Some(v)); }
    if let Some(v) = payload.followup_correct_answer { active.followup_correct_answer = Set(Some(v)); }
    if let Some(v) = payload.followup_wrong_question { active.followup_wrong_question = Set(Some(v)); }
    if let Some(v) = payload.followup_wrong_answer { active.followup_wrong_answer = Set(Some(v)); }

    active.update(&ctx.db).await?;

    format::json(())
}

#[utoipa::path(
    patch,
    path = "/api/questions/segment/{id}",
    tag = "questions",
    params(
        ("id" = i32, Path, description = "Segment ID"),
    ),
    request_body = UpdateSegmentBestQuestionPayload,
    responses(
        (status = 200, description = "Segment updated successfully"),
        (status = 404, description = "Segment not found"),
    )
)]
async fn update_segment_best_question(
    State(ctx): State<AppContext>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateSegmentBestQuestionPayload>,
) -> Result<Response> {
    let segment = segments::Entity::find_by_id(id)
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)?;

    let mut active: segments::ActiveModel = segment.into();

    if let Some(v) = payload.best_question { active.best_question = Set(Some(v)); }

    active.update(&ctx.db).await?;

    format::json(())
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("questions")
        .add("/{video_id}", get(get_questions_by_video).patch(update_question))
        .add("/segment/{id}", patch(update_segment_best_question))
}
