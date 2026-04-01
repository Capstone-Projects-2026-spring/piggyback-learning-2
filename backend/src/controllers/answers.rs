use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::utils::voice::{
    audio_processor::parse_wav, matching::compute_similarity, mood::detect_mood, stt::transcribe,
};

#[derive(Deserialize)]
pub struct VoiceAnalyzeRequest {
    pub expected_answer: String,
}

#[derive(Serialize)]
pub struct VoiceAnalyzeResponse {
    pub transcript: String,
    pub is_correct: bool,
    pub similarity_score: f32,
    pub mood: String,
    pub energy: f32,
}

pub async fn analyze_voice(mut multipart: Multipart) -> Result<Response> {
    let mut audio_bytes: Option<Vec<u8>> = None;
    let mut expected_answer: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?
    {
        let name = field.name().unwrap_or("");

        match name {
            "audio" => {
                audio_bytes = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|e| Error::BadRequest(e.to_string()))?
                        .to_vec(),
                );
            }
            "expected_answer" => {
                expected_answer = Some(
                    field
                        .text()
                        .await
                        .map_err(|e| Error::BadRequest(e.to_string()))?,
                );
            }
            _ => {}
        }
    }

    let audio_bytes = audio_bytes.ok_or_else(|| Error::BadRequest("Missing audio".into()))?;

    let expected_answer =
        expected_answer.ok_or_else(|| Error::BadRequest("Missing expected_answer".into()))?;

    let audio =
        parse_wav(&audio_bytes).map_err(|e| Error::BadRequest(format!("Invalid audio: {}", e)))?;

    let transcript =
        transcribe(&audio.samples).map_err(|e| Error::BadRequest(format!("STT failed: {}", e)))?;

    let (is_correct, similarity_score) = compute_similarity(&transcript, &expected_answer);

    let (mood, energy) = detect_mood(&audio.samples);

    format::json(VoiceAnalyzeResponse {
        transcript,
        is_correct,
        similarity_score,
        mood,
        energy,
    })
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("answers")
        .add("/analyze", post(analyze_voice))
}
