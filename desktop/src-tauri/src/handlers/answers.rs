use crate::{
    db::init::get_db,
    utils::{matching::compute_similarity, mood::detect_mood_from_frame},
    voice::session::SharedSession,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Answer {
    pub transcript: String,
    pub is_correct: bool,
    pub similarity_score: f32,
    pub mood: String,
    pub segment_id: i32,
}

/// Frontend calls this when a question appears.
/// Puts the pipeline into AnswerMode so the next utterance is scored, not classified.
#[tauri::command]
pub fn set_answer_context(
    expected_answer: String,
    video_id: String,
    segment_id: i32,
    session: tauri::State<SharedSession>,
) {
    session
        .lock()
        .unwrap()
        .enter_answer_mode(expected_answer, video_id, segment_id);
}

/// Frontend calls this on skip / close / unmount.
#[tauri::command]
pub fn clear_answer_context(session: tauri::State<SharedSession>) {
    session.lock().unwrap().exit_answer_mode();
}

/// Tauri command - called from frontend with explicit kid_id + video_id
#[tauri::command]
pub async fn get_answers(kid_id: i32, video_id: String) -> Result<Vec<Answer>, String> {
    let answers = fetch_answers(kid_id, &video_id).await?;
    crate::utils::app_handle::emit("orb://answers", serde_json::json!(answers));
    Ok(answers)
}

/// Dispatcher entry point - reads kid_id + video_id from session
pub async fn get_answers_for_session(args: &[String], session: &SharedSession) {
    let (kid_id, video_id) = {
        let s = session.lock().unwrap();
        let kid_id = match s.user_id {
            Some(id) => id,
            None => {
                eprintln!("[answers] no user in session");
                return;
            }
        };
        let video_id = args
            .first()
            .cloned()
            .or_else(|| s.current_video.clone())
            .unwrap_or_default();
        (kid_id, video_id)
    };
    match fetch_answers(kid_id, &video_id).await {
        Ok(answers) => crate::utils::app_handle::emit("orb://answers", serde_json::json!(answers)),
        Err(e) => eprintln!("[answers] fetch failed: {e}"),
    }
}

// Dispatcher entry point

/// Called directly from capture loop when SessionMode::Answer.
/// Reads last_transcript + session context, scores, detects mood, persists, emits.
pub async fn analyze_answer(_args: &[String], session: &SharedSession) {
    let (kid_id, video_id, segment_id, transcript, expected_answer) = {
        let s = session.lock().unwrap();
        let kid_id = match s.user_id {
            Some(id) => id,
            None => {
                eprintln!("[answers] no user in session");
                return;
            }
        };
        let video_id = match &s.current_video {
            Some(v) => v.clone(),
            None => {
                eprintln!("[answers] no current_video in session");
                return;
            }
        };
        let segment_id = match s.current_segment {
            Some(id) => id,
            None => {
                eprintln!("[answers] no current_segment in session");
                return;
            }
        };
        let transcript = match &s.last_transcript {
            Some(t) => t.clone(),
            None => {
                eprintln!("[answers] no transcript in session");
                return;
            }
        };
        let expected = s.expected_answer.clone().unwrap_or_default();
        (kid_id, video_id, segment_id, transcript, expected)
    };

    // Exit answer mode immediately - next utterance goes back through classifier
    session.lock().unwrap().exit_answer_mode();

    eprintln!("[answers] transcript={transcript:?} expected={expected_answer:?}");

    let (is_correct, similarity_score) = compute_similarity(&transcript, &expected_answer);

    let mood = capture_face_frame_for_mood().await;

    let answer = Answer {
        transcript,
        is_correct,
        similarity_score,
        mood,
        segment_id,
    };

    if let Err(e) = persist_answer(kid_id, &video_id, &answer).await {
        eprintln!("[answers] persist failed: {e}");
    }

    crate::utils::app_handle::emit(
        "orb://answer-result",
        serde_json::json!({
            "is_correct":       answer.is_correct,
            "similarity_score": answer.similarity_score,
            "mood":             answer.mood,
            "segment_id":       answer.segment_id,
            "transcript":       answer.transcript,
        }),
    );
}

// Camera snapshot

async fn capture_face_frame_for_mood() -> String {
    match crate::utils::gaze::request_snapshot().await {
        Some(jpeg_bytes) => detect_mood_from_frame(&jpeg_bytes),
        None => {
            eprintln!("[answers] no snapshot available - mood=neutral");
            "neutral".into()
        }
    }
}

// DB

async fn persist_answer(kid_id: i32, video_id: &str, a: &Answer) -> Result<(), String> {
    let db = get_db();
    let is_correct_int = a.is_correct as i32;
    sqlx::query(
        "INSERT INTO answers
            (kid_id, video_id, segment_id, transcript, is_correct, similarity_score, mood)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(kid_id)
    .bind(video_id)
    .bind(a.segment_id)
    .bind(&a.transcript)
    .bind(is_correct_int)
    .bind(a.similarity_score)
    .bind(&a.mood)
    .execute(db)
    .await
    .map_err(|e| format!("persist_answer: {e}"))?;
    Ok(())
}

async fn fetch_answers(kid_id: i32, video_id: &str) -> Result<Vec<Answer>, String> {
    let db = get_db();
    let rows = sqlx::query(
        "SELECT transcript, is_correct, similarity_score, mood, segment_id
         FROM answers WHERE kid_id = ? AND video_id = ? ORDER BY id ASC",
    )
    .bind(kid_id)
    .bind(video_id)
    .fetch_all(db)
    .await
    .map_err(|e| format!("fetch_answers: {e}"))?;

    Ok(rows
        .iter()
        .map(|r| {
            use sqlx::Row;
            Answer {
                transcript: r.get("transcript"),
                is_correct: r.get::<i32, _>("is_correct") != 0,
                similarity_score: r.get::<f64, _>("similarity_score") as f32,
                mood: r
                    .get::<Option<String>, _>("mood")
                    .unwrap_or_else(|| "neutral".into()),
                segment_id: r.get("segment_id"),
            }
        })
        .collect())
}
