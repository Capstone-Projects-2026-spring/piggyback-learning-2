use loco_rs::prelude::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::models::_entities::video_assignments;
use crate::utils::voice::{
    audio_processor::parse_wav, matching::compute_similarity, mood::detect_mood, stt::transcribe,
};

#[derive(Deserialize)]
pub struct AnswerAnalyzeRequest {
    pub expected_answer: String,
    pub kid_id: i32,
    pub video_id: String,
    pub segment_id: i32,
}

#[derive(Serialize, Clone)]
pub struct AnswerAnalyzeResponse {
    pub transcript: String,
    pub is_correct: bool,
    pub similarity_score: f32,
    pub mood: String,
    pub energy: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Answer {
    pub transcript: String,
    pub is_correct: bool,
    pub similarity_score: f32,
    pub mood: String,
    pub energy: f32,
    pub segment_id: i32,
}

pub async fn analyze_answer(
    State(ctx): State<AppContext>,
    mut multipart: Multipart,
) -> Result<Response> {
    let mut audio_bytes: Option<Vec<u8>> = None;
    let mut expected_answer: Option<String> = None;
    let mut kid_id: Option<i32> = None;
    let mut video_id: Option<String> = None;
    let mut segment_id: Option<i32> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?
    {
        match field.name() {
            Some("audio") => {
                audio_bytes = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|e| Error::BadRequest(e.to_string()))?
                        .to_vec(),
                );
            }
            Some("expected_answer") => {
                expected_answer = Some(
                    field
                        .text()
                        .await
                        .map_err(|e| Error::BadRequest(e.to_string()))?,
                );
            }
            Some("kid_id") => {
                kid_id = Some(
                    field
                        .text()
                        .await
                        .map_err(|e| Error::BadRequest(e.to_string()))?
                        .parse()
                        .map_err(|_| Error::BadRequest("Invalid kid_id".into()))?,
                );
            }
            Some("video_id") => {
                video_id = Some(
                    field
                        .text()
                        .await
                        .map_err(|e| Error::BadRequest(e.to_string()))?,
                );
            }
            Some("segment_id") => {
                segment_id = Some(
                    field
                        .text()
                        .await
                        .map_err(|e| Error::BadRequest(e.to_string()))?
                        .parse()
                        .map_err(|_| Error::BadRequest("Invalid segment_id".into()))?,
                );
            }
            _ => {}
        }
    }

    let audio_bytes = audio_bytes.ok_or_else(|| Error::BadRequest("Missing audio".into()))?;
    let expected_answer =
        expected_answer.ok_or_else(|| Error::BadRequest("Missing expected_answer".into()))?;
    let kid_id = kid_id.ok_or_else(|| Error::BadRequest("Missing kid_id".into()))?;
    let video_id = video_id.ok_or_else(|| Error::BadRequest("Missing video_id".into()))?;
    let segment_id = segment_id.ok_or_else(|| Error::BadRequest("Missing segment_id".into()))?;

    let audio =
        parse_wav(&audio_bytes).map_err(|e| Error::BadRequest(format!("Invalid audio: {}", e)))?;
    let transcript =
        transcribe(&audio.samples).map_err(|e| Error::BadRequest(format!("STT failed: {}", e)))?;
    let (is_correct, similarity_score) = compute_similarity(&transcript, &expected_answer);
    let (mood, energy) = detect_mood(&audio.samples);

    let answer = Answer {
        transcript: transcript.clone(),
        is_correct,
        similarity_score,
        mood: mood.clone(),
        energy,
        segment_id,
    };

    let record = video_assignments::Entity::find()
        .filter(video_assignments::Column::KidId.eq(kid_id))
        .filter(video_assignments::Column::VideoId.eq(video_id.clone()))
        .one(&ctx.db)
        .await?
        .unwrap();

    let mut answers: Vec<Answer> = record
        .answers
        .map(|json| serde_json::from_value(json).unwrap_or_default())
        .unwrap_or_default();

    answers.push(answer.clone());

    let answers_json: JsonValue = serde_json::to_value(answers)
        .map_err(|e| Error::BadRequest(format!("Serialization error: {}", e)))?;

    video_assignments::ActiveModel {
        kid_id: Set(record.kid_id),
        video_id: Set(record.video_id.clone()),
        answers: Set(Some(answers_json)),
        ..Default::default()
    }
    .update(&ctx.db)
    .await?;

    format::json(AnswerAnalyzeResponse {
        transcript,
        is_correct,
        similarity_score,
        mood,
        energy,
    })
}

pub async fn get_answers(
    State(ctx): State<AppContext>,
    Path((kid_id, video_id)): Path<(i32, String)>,
) -> Result<Response> {
    let record = video_assignments::Entity::find()
        .filter(video_assignments::Column::KidId.eq(kid_id))
        .filter(video_assignments::Column::VideoId.eq(video_id))
        .one(&ctx.db)
        .await?;

    if let Some(record) = record {
        format::json(record)
    } else {
        format::json(serde_json::json!({"success": false, "msg": "Unknown error occurred"}))
    }
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("answers")
        .add("/analyze", post(analyze_answer))
        .add("/{kid_id}/{video_id}", get(get_answers))
}
