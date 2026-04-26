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
    pub video_id: String,
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

/// Dispatcher entry point - parent views all answers for a named kid.
pub async fn get_kids_answers(args: &[String], session: &SharedSession) {
    let (transcript, role) = {
        let s = session.lock().unwrap();
        let role = s.role.clone().unwrap_or_default();
        let transcript = args.first().cloned().unwrap_or_default();
        (transcript, role)
    };

    // TODO: re-enable parent-only guard
    // if role != "parent" {
    //     eprintln!("[answers] non-parent attempted to view results - blocked");
    //     return;
    // }

    let kid_id = match resolve_kid_name_from_transcript(&transcript).await {
        Some(id) => id,
        None => {
            eprintln!("[answers] could not resolve kid name from: {transcript:?}");
            crate::utils::app_handle::emit(
                "orb://answers-error",
                serde_json::json!({ "message": "I couldn't figure out which kid you meant." }),
            );
            return;
        }
    };

    match fetch_all_answers(kid_id).await {
        Ok(answers) => crate::utils::app_handle::emit(
            "orb://answers",
            serde_json::json!({ "answers": answers, "kid_id": kid_id }),
        ),
        Err(e) => eprintln!("[answers] fetch failed: {e}"),
    }
}

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
        video_id: video_id.clone(),
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
            "video_id":         answer.video_id,
        }),
    );
}

// DB

async fn fetch_all_answers(kid_id: i32) -> Result<Vec<Answer>, String> {
    let db = get_db();
    let rows = sqlx::query(
        "SELECT a.transcript, a.is_correct, a.similarity_score, a.mood,
                a.segment_id, a.video_id, v.title
         FROM answers a
         LEFT JOIN videos v ON v.id = a.video_id
         WHERE a.kid_id = ?
         ORDER BY a.video_id, a.id ASC",
    )
    .bind(kid_id)
    .fetch_all(db)
    .await
    .map_err(|e| format!("fetch_all_answers: {e}"))?;

    use sqlx::Row;
    Ok(rows
        .iter()
        .map(|r| Answer {
            transcript: r.get("transcript"),
            is_correct: r.get::<i32, _>("is_correct") != 0,
            similarity_score: r.get::<f64, _>("similarity_score") as f32,
            mood: r
                .get::<Option<String>, _>("mood")
                .unwrap_or_else(|| "neutral".into()),
            segment_id: r.get("segment_id"),
            video_id: r.get("video_id"),
        })
        .collect())
}

async fn persist_answer(kid_id: i32, video_id: &str, a: &Answer) -> Result<(), String> {
    let db = get_db();
    sqlx::query(
        "INSERT INTO answers
            (kid_id, video_id, segment_id, transcript, is_correct, similarity_score, mood)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(kid_id)
    .bind(video_id)
    .bind(a.segment_id)
    .bind(&a.transcript)
    .bind(a.is_correct as i32)
    .bind(a.similarity_score)
    .bind(&a.mood)
    .execute(db)
    .await
    .map_err(|e| format!("persist_answer: {e}"))?;
    Ok(())
}

async fn capture_face_frame_for_mood() -> String {
    match crate::utils::gaze::request_snapshot().await {
        Some(jpeg_bytes) => detect_mood_from_frame(&jpeg_bytes),
        None => {
            eprintln!("[answers] no snapshot available - mood=neutral");
            "neutral".into()
        }
    }
}

async fn resolve_kid_name_from_transcript(transcript: &str) -> Option<i32> {
    let db = get_db();
    let rows = sqlx::query("SELECT id, name FROM users WHERE role = 'kid'")
        .fetch_all(db)
        .await
        .ok()?;

    let normalised = crate::utils::text::normalize(transcript);
    use sqlx::Row;
    rows.iter().find_map(|row| {
        let name: String = row.get("name");
        let id: i32 = row.get("id");
        if normalised.contains(&crate::utils::text::normalize(&name)) {
            Some(id)
        } else {
            None
        }
    })
}
