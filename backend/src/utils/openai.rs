use async_openai::{config::OpenAIConfig, Client as OpenAIClient};
use base64::{engine::general_purpose, Engine as _};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, Insert, QueryFilter,
    QueryOrder, Set,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{fs, sync::Arc};
use utoipa::ToSchema;

use crate::models::_entities::{
    frames::{Column as FrameColumn, Entity as Frames, Model as Frame},
    questions::{ActiveModel as QuestionActive, Column as QuestionColumn, Entity as Questions},
    segments::{
        ActiveModel as SegmentActive, Column as SegmentColumn, Entity as Segments,
        Model as SegmentModel,
    },
};

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct QuestionItem {
    pub qtype: String,
    pub question: String,
    pub answer: String,
    pub rank: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct SegmentResponse {
    pub id: i32,
    pub video_id: String,
    pub start_seconds: i32,
    pub end_seconds: i32,
    pub best_question: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct QuestionsResponse {
    pub segment: SegmentResponse,
    pub questions: Vec<QuestionItem>,
}

#[derive(Deserialize, ToSchema)]
struct OpenAIQuestionItem {
    q: String,
    a: String,
    rank: Option<i32>,
}

#[derive(Deserialize)]
struct OpenAIQuestions {
    character: OpenAIQuestionItem,
    setting: OpenAIQuestionItem,
    feeling: OpenAIQuestionItem,
    action: OpenAIQuestionItem,
    causal: OpenAIQuestionItem,
    outcome: OpenAIQuestionItem,
    prediction: OpenAIQuestionItem,
}

#[derive(Deserialize)]
pub struct OpenAIResponse {
    questions: OpenAIQuestions,
    best_question: String,
}

fn build_prompt(transcript: &str, duration: i32, start: i32, end: i32) -> String {
    format!(
        r#"You are an early childhood educator designing comprehension questions for children ages 6–8.

COMPLETE TRANSCRIPT:
==========================================
{}
==========================================

TASK:
Frames from {}s–{}s ({} seconds)

Provide ONE question for EACH:
- Character
- Setting
- Feeling
- Action
- Causal Relationship
- Outcome
- Prediction

IMPORTANT:
- Every answer must be a SINGLE WORD only
- Do NOT include spaces
- Do NOT include punctuation
- Example:
  BAD: "in the park"
  GOOD: "park"

Return JSON:
{{
"questions": {{
    "character": {{ "q": "...", "a": "ONE_WORD_ONLY", "rank": "ANY_ONE_DIGIT_NUMBER" }},
    "setting": {{ "q": "...", "a": "ONE_WORD_ONLY", "rank": "ANY_ONE_DIGIT_NUMBER" }},
    "feeling": {{ "q": "...", "a": "ONE_WORD_ONLY", "rank": "ANY_ONE_DIGIT_NUMBER" }},
    "action": {{ "q": "...", "a": "ONE_WORD_ONLY", "rank": "ANY_ONE_DIGIT_NUMBER" }},
    "causal": {{ "q": "...", "a": "ONE_WORD_ONLY", "rank": "ANY_ONE_DIGIT_NUMBER" }},
    "outcome": {{ "q": "...", "a": "ONE_WORD_ONLY", "rank": "ANY_ONE_DIGIT_NUMBER" }},
    "prediction": {{ "q": "...", "a": "ONE_WORD_ONLY", "rank": "ANY_ONE_DIGIT_NUMBER" }}
}},
"best_question": "..."
}}"#,
        transcript, start, end, duration
    )
}

fn build_transcript(frames: &[Frame]) -> String {
    let parts: Vec<String> = frames
        .iter()
        .filter_map(|f| {
            f.subtitle_text.as_ref().and_then(|txt| {
                if txt.trim().is_empty() {
                    None
                } else {
                    Some(format!("[{}] {}", f.timestamp_formatted, txt))
                }
            })
        })
        .collect();

    if parts.is_empty() {
        "No transcript available.".to_string()
    } else {
        parts.join("\n")
    }
}

fn sample_frames(frames: &[Frame], max: usize) -> Vec<Frame> {
    if frames.len() <= max {
        return frames.to_vec();
    }

    let step = (frames.len() / max).max(1);
    frames.iter().step_by(step).take(max).cloned().collect()
}

pub async fn call_openai(
    client: &Arc<OpenAIClient<OpenAIConfig>>,
    prompt: String,
    image_paths: Vec<String>,
) -> Result<OpenAIResponse, String> {
    let mut content = vec![json!({
        "type": "text",
        "text": prompt
    })];

    for path in image_paths {
        if let Ok(bytes) = fs::read(&path) {
            let b64 = general_purpose::STANDARD.encode(bytes);

            content.push(json!({
                "type": "image_url",
                "image_url": {
                    "url": format!("data:image/jpeg;base64,{}", b64),
                    "detail": "low"
                }
            }));
        }
    }

    let request_body = json!({
        "model": "gpt-4o-mini",
        "temperature": 0.2,
        "max_tokens": 1500,
        "response_format": { "type": "json_object" },
        "messages": [
            {
                "role": "system",
                "content": "You are a safe, child-focused educational assistant."
            },
            {
                "role": "user",
                "content": content
            }
        ]
    });

    let response: serde_json::Value = client
        .chat()
        .create_byot(request_body)
        .await
        .map_err(|e| format!("OpenAI error: {}", e))?;

    let text = response["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("Invalid OpenAI response")?;

    let parsed: OpenAIResponse =
        serde_json::from_str(text).map_err(|_| "Invalid JSON structure".to_string())?;

    Ok(parsed)
}

pub async fn generate_and_store_questions(
    db: &DatabaseConnection,
    client: &Arc<OpenAIClient<OpenAIConfig>>,
    video_id: String,
    start: i32,
    end: i32,
) -> Result<QuestionsResponse, String> {
    if let Some(segment) = Segments::find()
        .filter(SegmentColumn::VideoId.eq(&video_id))
        .filter(SegmentColumn::StartSeconds.eq(start))
        .filter(SegmentColumn::EndSeconds.eq(end))
        .one(db)
        .await
        .map_err(|e| format!("DB error: {}", e))?
    {
        let questions = Questions::find()
            .filter(QuestionColumn::SegmentId.eq(segment.id))
            .all(db)
            .await
            .map_err(|e| format!("DB error: {}", e))?;

        return Ok(QuestionsResponse {
            segment: SegmentResponse {
                id: segment.id,
                video_id: segment.video_id,
                start_seconds: segment.start_seconds,
                end_seconds: segment.end_seconds,
                best_question: segment.best_question,
            },
            questions: questions
                .into_iter()
                .map(|q| QuestionItem {
                    qtype: q.qtype,
                    question: q.question,
                    answer: q.answer,
                    rank: q.rank,
                })
                .collect(),
        });
    }

    let frames = Frames::find()
        .filter(FrameColumn::VideoId.eq(&video_id))
        .filter(FrameColumn::TimestampSeconds.gte(start))
        .filter(FrameColumn::TimestampSeconds.lte(end))
        .order_by_asc(FrameColumn::TimestampSeconds)
        .all(db)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    if frames.is_empty() {
        return Err("no_frames".into());
    }

    let transcript = build_transcript(&frames);
    let sampled = sample_frames(&frames, 5);

    let prompt = build_prompt(&transcript, end - start, start, end);
    let image_paths: Vec<String> = sampled.iter().map(|f| f.file_path.clone()).collect();

    let parsed = call_openai(client, prompt, image_paths).await?;

    let segment: SegmentModel = SegmentActive {
        video_id: Set(video_id.clone()),
        start_seconds: Set(start),
        end_seconds: Set(end),
        best_question: Set(Some(parsed.best_question.clone())),
        ..Default::default()
    }
    .insert(db)
    .await
    .map_err(|e| format!("Insert segment failed: {}", e))?;

    let question_models = vec![
        ("character", parsed.questions.character),
        ("setting", parsed.questions.setting),
        ("feeling", parsed.questions.feeling),
        ("action", parsed.questions.action),
        ("causal", parsed.questions.causal),
        ("outcome", parsed.questions.outcome),
        ("prediction", parsed.questions.prediction),
    ];

    let active_models: Vec<QuestionActive> = question_models
        .iter()
        .map(|(qtype, item)| QuestionActive {
            segment_id: Set(segment.id),
            qtype: Set(qtype.to_string()),
            question: Set(item.q.clone()),
            answer: Set(item.a.clone()),
            rank: Set(item.rank),
            ..Default::default()
        })
        .collect();

    Insert::many(active_models)
        .exec(db)
        .await
        .map_err(|e| format!("Batch insert failed: {}", e))?;

    let questions = question_models
        .into_iter()
        .map(|(qtype, item)| QuestionItem {
            qtype: qtype.to_string(),
            question: item.q,
            answer: item.a,
            rank: item.rank,
        })
        .collect();

    Ok(QuestionsResponse {
        segment: SegmentResponse {
            id: segment.id,
            video_id,
            start_seconds: start,
            end_seconds: end,
            best_question: Some(parsed.best_question),
        },
        questions,
    })
}
