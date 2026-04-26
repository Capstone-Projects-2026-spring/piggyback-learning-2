use async_openai::{config::OpenAIConfig, Client as OpenAIClient};
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{fs, sync::OnceLock};

static OPENAI_CLIENT: OnceLock<OpenAIClient<OpenAIConfig>> = OnceLock::new();

pub fn init_openai() {
    OPENAI_CLIENT.get_or_init(|| {
        let config = OpenAIConfig::new().with_api_key(env!("OPENAI_API_KEY"));
        eprintln!("[openai] client ready");
        OpenAIClient::with_config(config)
    });
}

fn get_client() -> &'static OpenAIClient<OpenAIConfig> {
    OPENAI_CLIENT
        .get()
        .expect("[openai] not initialised - call init_openai() at startup")
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QuestionItem {
    pub qtype: String,
    pub question: String,
    pub answer: String,
    pub rank: Option<i32>,
    pub followup_correct_question: Option<String>,
    pub followup_correct_answer: Option<String>,
    pub followup_wrong_question: Option<String>,
    pub followup_wrong_answer: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SegmentResponse {
    pub id: i64,
    pub video_id: String,
    pub start_seconds: i32,
    pub end_seconds: i32,
    pub best_question: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QuestionsResponse {
    pub segment: SegmentResponse,
    pub questions: Vec<QuestionItem>,
}

// Separate from the public types above so the API shape stays internal.

#[derive(Deserialize)]
pub struct OpenAIFollowUp {
    pub q: String,
    pub a: String,
}

#[derive(Deserialize)]
pub struct OpenAIQuestionItem {
    pub q: String,
    pub a: String,
    pub rank: Option<i32>,
    pub followup_for_correct_answer: Option<OpenAIFollowUp>,
    pub followup_for_wrong_answer: Option<OpenAIFollowUp>,
}

#[derive(Deserialize)]
pub struct OpenAIQuestions {
    pub character: OpenAIQuestionItem,
    pub setting: OpenAIQuestionItem,
    pub feeling: OpenAIQuestionItem,
    pub action: OpenAIQuestionItem,
    pub causal: OpenAIQuestionItem,
    pub outcome: OpenAIQuestionItem,
    pub prediction: OpenAIQuestionItem,
}

#[derive(Deserialize)]
pub struct OpenAIResponse {
    pub questions: OpenAIQuestions,
    pub best_question: String,
}

// API calls

/// Sample up to `max` evenly-spaced paths from a frame list.
pub fn sample_frame_paths(paths: &[String], max: usize) -> Vec<String> {
    if paths.len() <= max {
        return paths.to_vec();
    }
    let step = (paths.len() / max).max(1);
    paths.iter().step_by(step).take(max).cloned().collect()
}

/// Send frames to GPT-4o and parse the structured question response.
pub async fn call_openai_vision(
    prompt: String,
    image_paths: Vec<String>,
) -> Result<OpenAIResponse, String> {
    let mut content = vec![json!({ "type": "text", "text": prompt })];

    for path in &image_paths {
        match fs::read(path) {
            Ok(bytes) => content.push(json!({
                "type": "image_url",
                "image_url": {
                    "url":    format!("data:image/jpeg;base64,{}", general_purpose::STANDARD.encode(&bytes)),
                    "detail": "low"
                }
            })),
            Err(e) => eprintln!("[openai] could not read frame {path}: {e}"),
        }
    }

    let request_body = json!({
        "model":           "gpt-4o",
        "temperature":     0.2,
        "max_tokens":      2000,
        "response_format": { "type": "json_object" },
        "messages": [
            {
                "role":    "system",
                "content": "You are a safe, child-focused educational assistant. Always return valid JSON."
            },
            {
                "role":    "user",
                "content": content
            }
        ]
    });

    let response: serde_json::Value = get_client()
        .chat()
        .create_byot(request_body)
        .await
        .map_err(|e| format!("[openai] API error: {e}"))?;

    let text = response["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("[openai] missing content in response")?;

    serde_json::from_str(text).map_err(|e| format!("[openai] JSON parse failed: {e}\nraw: {text}"))
}

/// Build the GPT-4o prompt for a video segment.
/// `existing_questions` prevents the model from repeating questions
/// already generated for earlier segments of the same video.
pub fn build_prompt(start: i32, end: i32, existing_questions: &[String]) -> String {
    let repeat_warning = if existing_questions.is_empty() {
        String::new()
    } else {
        let list = existing_questions
            .iter()
            .map(|q| format!("- \"{q}\""))
            .collect::<Vec<_>>()
            .join("\n");
        format!("PREVIOUSLY ASKED QUESTIONS (do not repeat or closely resemble these):\n{list}\n\n")
    };

    format!(
        r#"You are an early childhood educator designing visual comprehension questions for children ages 6–8.

You will be shown frames from a children's video between {start}s and {end}s.
Base your questions ONLY on what is visually present in the frames.
Do not assume any dialogue or narration.

{repeat_warning}Generate ONE question for EACH of these types:
- Character (who is in the scene)
- Setting (where does it take place)
- Feeling (how does a character appear to feel)
- Action (what is happening)
- Causal Relationship (why did something happen)
- Outcome (what was the result)
- Prediction (what might happen next)

RULES:
- Every answer must be a SINGLE WORD only
- No spaces, no punctuation in answers
- Questions must be answerable from the frames alone
- Use simple language a 6 year old can understand
- For each question provide TWO follow-up questions:
  1. followup_for_correct_answer: a slightly harder question on the same topic for when the child answers correctly
  2. followup_for_wrong_answer: an easier guiding question to help the child reach the original answer when they answer incorrectly

Good answers: "happy", "forest", "running", "spider", "scared"
Bad answers: "in the park", "very happy", "spider-man"

Return ONLY this JSON:
{{
  "questions": {{
    "character":  {{ "q": "...", "a": "ONE_WORD", "rank": 1, "followup_for_correct_answer": {{ "q": "...", "a": "ONE_WORD" }}, "followup_for_wrong_answer": {{ "q": "...", "a": "ONE_WORD" }} }},
    "setting":    {{ "q": "...", "a": "ONE_WORD", "rank": 2, "followup_for_correct_answer": {{ "q": "...", "a": "ONE_WORD" }}, "followup_for_wrong_answer": {{ "q": "...", "a": "ONE_WORD" }} }},
    "feeling":    {{ "q": "...", "a": "ONE_WORD", "rank": 3, "followup_for_correct_answer": {{ "q": "...", "a": "ONE_WORD" }}, "followup_for_wrong_answer": {{ "q": "...", "a": "ONE_WORD" }} }},
    "action":     {{ "q": "...", "a": "ONE_WORD", "rank": 4, "followup_for_correct_answer": {{ "q": "...", "a": "ONE_WORD" }}, "followup_for_wrong_answer": {{ "q": "...", "a": "ONE_WORD" }} }},
    "causal":     {{ "q": "...", "a": "ONE_WORD", "rank": 5, "followup_for_correct_answer": {{ "q": "...", "a": "ONE_WORD" }}, "followup_for_wrong_answer": {{ "q": "...", "a": "ONE_WORD" }} }},
    "outcome":    {{ "q": "...", "a": "ONE_WORD", "rank": 6, "followup_for_correct_answer": {{ "q": "...", "a": "ONE_WORD" }}, "followup_for_wrong_answer": {{ "q": "...", "a": "ONE_WORD" }} }},
    "prediction": {{ "q": "...", "a": "ONE_WORD", "rank": 7, "followup_for_correct_answer": {{ "q": "...", "a": "ONE_WORD" }}, "followup_for_wrong_answer": {{ "q": "...", "a": "ONE_WORD" }} }}
  }},
  "best_question": "the single most engaging question from above"
}}"#
    )
}
