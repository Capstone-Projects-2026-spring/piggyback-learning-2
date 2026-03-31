use async_openai::Client as OpenAIClient;
use base64::{engine::general_purpose, Engine as _};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use serde_json::{json, Value};
use std::fs;

use crate::models::_entities::{
    frames::{Column as FrameColumn, Entity as Frames, Model as Frame},
    questions::{ActiveModel as QuestionActive, Column as QuestionColumn, Entity as Questions},
    segments::{
        ActiveModel as SegmentActive, Column as SegmentColumn, Entity as Segments,
        Model as SegmentModel,
    },
};

use loco_rs::prelude::*;

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
    "character": {{ "q": "...", "a": "ONE_WORD_ONLY", "rank": "" }},
    "setting": {{ "q": "...", "a": "ONE_WORD_ONLY", "rank": "" }},
    "feeling": {{ "q": "...", "a": "ONE_WORD_ONLY", "rank": "" }},
    "action": {{ "q": "...", "a": "ONE_WORD_ONLY", "rank": "" }},
    "causal": {{ "q": "...", "a": "ONE_WORD_ONLY", "rank": "" }},
    "outcome": {{ "q": "...", "a": "ONE_WORD_ONLY", "rank": "" }},
    "prediction": {{ "q": "...", "a": "ONE_WORD_ONLY", "rank": "" }}
}},
"best_question": "..."
}}"#,
        transcript, start, end, duration
    )
}

fn build_transcript(frames: &[Frame]) -> String {
    let mut parts = Vec::new();

    for f in frames {
        if let Some(txt) = &f.subtitle_text {
            if !txt.trim().is_empty() {
                parts.push(format!("[{}] {}", f.timestamp_formatted, txt));
            }
        }
    }

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

pub async fn call_openai(prompt: String, image_paths: Vec<String>) -> Result<String, String> {
    let client = OpenAIClient::new();

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
        "messages": [
            {
                "role": "system",
                "content": "You are a safe, child-focused educational assistant."
            },
            {
                "role": "user",
                "content": content
            }
        ],
        "temperature": 0.2,
        "max_tokens": 1500,
        "response_format": { "type": "json_object" }
    });

    let response: Value = client.chat().create_byot(request_body).await.unwrap();

    let text = response["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string();

    Ok(text)
}

pub async fn generate_and_store_questions(
    db: &DatabaseConnection,
    video_id: String,
    start: i32,
    end: i32,
) -> Result<String, String> {
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

        let response = json!({
            "cached": true,
            "segment_id": segment.id,
            "best_question": segment.best_question,
            "questions": questions
        });

        return Ok(response.to_string());
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
        return Ok(r#"{"error":"no_frames"}"#.to_string());
    }

    let transcript = build_transcript(&frames);
    let sampled = sample_frames(&frames, 5);
    let prompt = build_prompt(&transcript, end - start, start, end);
    let image_paths: Vec<String> = sampled.iter().map(|f| f.file_path.clone()).collect();

    let raw = call_openai(prompt, image_paths).await?;

    let parsed: Value =
        serde_json::from_str(&raw).unwrap_or_else(|_| json!({ "error": "invalid_json" }));

    let best_question = parsed["best_question"].as_str().unwrap_or("").to_string();

    let segment: SegmentModel = SegmentActive {
        video_id: Set(video_id.clone()),
        start_seconds: Set(start),
        end_seconds: Set(end),
        best_question: Set(Some(best_question)),
        ..Default::default()
    }
    .insert(db)
    .await
    .map_err(|e| format!("Failed to insert segment: {}", e))?;

    if let Some(qmap) = parsed["questions"].as_object() {
        for (qtype, val) in qmap {
            let q = val["q"].as_str().unwrap_or("");
            let a = val["a"].as_str().unwrap_or("");
            let rank = val["rank"].as_str().and_then(|r| r.parse::<i32>().ok());

            let _ = QuestionActive {
                segment_id: Set(segment.id),
                qtype: Set(qtype.clone()),
                question: Set(q.to_string()),
                answer: Set(a.to_string()),
                rank: Set(rank),
                ..Default::default()
            }
            .insert(db)
            .await
            .map_err(|e| format!("Failed to insert question: {}", e))?;
        }
    }

    Ok(raw.to_string())
}
