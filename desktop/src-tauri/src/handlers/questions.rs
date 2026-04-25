use crate::db::init::get_db;
use crate::utils::app_handle::emit;
use crate::utils::openai::{
    build_prompt, call_openai_vision, sample_frame_paths, QuestionItem, QuestionsResponse,
    SegmentResponse,
};

pub async fn generate_questions_for_video(video_id: &str) -> Result<(), String> {
    let pool = get_db();

    let (existing,) =
        sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM segments WHERE video_id = ?")
            .bind(video_id)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("[questions] count failed: {e}"))?;

    if existing > 0 {
        eprintln!("[questions] already generated for {video_id} — skipping");
        return Ok(());
    }

    emit(
        "orb://processing-status",
        serde_json::json!({
            "video_id": video_id,
            "stage": "generating_questions",
            "progress": { "current": 0, "total": 0 }
        }),
    );

    let boundaries = get_keyframe_boundaries(video_id).await?;
    eprintln!("[questions] keyframe boundaries for {video_id}: {boundaries:?}");

    let segments = build_segments(&boundaries, video_id).await?;
    let total = segments.len();
    eprintln!("[questions] {total} segments to process");

    let mut all_existing_questions: Vec<String> = Vec::new();
    let mut all_responses: Vec<QuestionsResponse> = Vec::new();

    for (i, (start, end)) in segments.iter().enumerate() {
        eprintln!("[questions] generating for {video_id} [{start}s → {end}s]");

        emit(
            "orb://processing-status",
            serde_json::json!({
                "video_id": video_id,
                "stage": "generating_questions",
                "progress": { "current": i + 1, "total": total }
            }),
        );

        match generate_for_segment(video_id, *start, *end, &all_existing_questions).await {
            Ok(response) => {
                for q in &response.questions {
                    all_existing_questions.push(q.question.clone());
                    if let Some(ref fq) = q.followup_correct_question {
                        all_existing_questions.push(fq.clone());
                    }
                    if let Some(ref fq) = q.followup_wrong_question {
                        all_existing_questions.push(fq.clone());
                    }
                }
                all_responses.push(response);
                eprintln!("[questions] done [{start}s → {end}s]");
            }
            Err(e) => eprintln!("[questions] failed [{start}s → {end}s]: {e}"),
        }
    }

    emit(
        "orb://questions-ready",
        serde_json::json!({
            "video_id": video_id,
            "segments": all_responses,
        }),
    );

    Ok(())
}

// ── Internals ────────────────────────────────────────────────────────────────

/// Reads keyframe timestamps from DB instead of re-running ffmpeg.
async fn get_keyframe_boundaries(video_id: &str) -> Result<Vec<i32>, String> {
    let pool = get_db();

    let mut boundaries: Vec<i32> = sqlx::query_as::<_, (i32,)>(
        "SELECT timestamp_seconds FROM frames
         WHERE video_id = ? AND is_keyframe = 1
         ORDER BY timestamp_seconds ASC",
    )
    .bind(video_id)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("[questions] keyframe query failed: {e}"))?
    .into_iter()
    .map(|(ts,)| ts)
    .collect();

    // Always start at 0
    if boundaries.first() != Some(&0) {
        boundaries.insert(0, 0);
    }

    Ok(boundaries)
}

async fn get_video_duration(video_id: &str) -> i32 {
    let pool = get_db();
    sqlx::query_as::<_, (i32,)>("SELECT duration_seconds FROM videos WHERE id = ?")
        .bind(video_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .map(|(d,)| d)
        .unwrap_or(0)
}

async fn build_segments(boundaries: &[i32], video_id: &str) -> Result<Vec<(i32, i32)>, String> {
    let duration = get_video_duration(video_id).await;
    if duration == 0 {
        return Err("[questions] video duration unknown".to_string());
    }

    const MIN_SEGMENT: i32 = 30;
    const MAX_SEGMENT: i32 = 90;

    let mut segments: Vec<(i32, i32)> = Vec::new();
    let mut seg_start = 0i32;

    for &boundary in boundaries.iter().skip(1) {
        let len = boundary - seg_start;
        if len < MIN_SEGMENT {
            continue;
        }
        if len > MAX_SEGMENT {
            let mut cursor = seg_start;
            while cursor + MAX_SEGMENT < boundary {
                segments.push((cursor, cursor + MAX_SEGMENT));
                cursor += MAX_SEGMENT;
            }
            seg_start = cursor;
        } else {
            segments.push((seg_start, boundary));
            seg_start = boundary;
        }
    }

    if seg_start < duration {
        segments.push((seg_start, duration));
    }

    Ok(segments)
}

async fn generate_for_segment(
    video_id: &str,
    start: i32,
    end: i32,
    existing_questions: &[String],
) -> Result<QuestionsResponse, String> {
    let pool = get_db();

    let (existing,) = sqlx::query_as::<_, (i64,)>(
        "SELECT COUNT(*) FROM segments
         WHERE video_id = ? AND start_seconds = ? AND end_seconds = ?",
    )
    .bind(video_id)
    .bind(start)
    .bind(end)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("[questions] segment check failed: {e}"))?;

    if existing > 0 {
        return Err("already_exists".to_string());
    }

    let frame_paths: Vec<String> = sqlx::query_as::<_, (String,)>(
        "SELECT file_path FROM frames
         WHERE video_id = ? AND timestamp_seconds >= ? AND timestamp_seconds <= ?
         ORDER BY timestamp_seconds ASC",
    )
    .bind(video_id)
    .bind(start)
    .bind(end)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("[questions] frame fetch failed: {e}"))?
    .into_iter()
    .map(|(p,)| p)
    .collect();

    if frame_paths.is_empty() {
        return Err(format!("[questions] no frames for [{start}→{end}]"));
    }

    let sampled = sample_frame_paths(&frame_paths, 5);
    let prompt = build_prompt(start, end, existing_questions);
    let parsed = call_openai_vision(prompt, sampled).await?;

    let row = sqlx::query(
        "INSERT INTO segments (video_id, start_seconds, end_seconds, best_question)
         VALUES (?, ?, ?, ?) RETURNING id",
    )
    .bind(video_id)
    .bind(start)
    .bind(end)
    .bind(&parsed.best_question)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("[questions] segment insert failed: {e}"))?;

    let segment_id: i64 =
        sqlx::Row::try_get(&row, "id").map_err(|e| format!("[questions] segment id fetch: {e}"))?;

    let question_types = [
        ("character", &parsed.questions.character),
        ("setting", &parsed.questions.setting),
        ("feeling", &parsed.questions.feeling),
        ("action", &parsed.questions.action),
        ("causal", &parsed.questions.causal),
        ("outcome", &parsed.questions.outcome),
        ("prediction", &parsed.questions.prediction),
    ];

    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("[questions] tx begin failed: {e}"))?;

    for (qtype, item) in &question_types {
        sqlx::query(
            "INSERT INTO questions
             (segment_id, qtype, question, answer, rank,
              followup_correct_question, followup_correct_answer,
              followup_wrong_question,   followup_wrong_answer)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(segment_id)
        .bind(qtype)
        .bind(&item.q)
        .bind(&item.a)
        .bind(item.rank)
        .bind(item.followup_for_correct_answer.as_ref().map(|f| &f.q))
        .bind(item.followup_for_correct_answer.as_ref().map(|f| &f.a))
        .bind(item.followup_for_wrong_answer.as_ref().map(|f| &f.q))
        .bind(item.followup_for_wrong_answer.as_ref().map(|f| &f.a))
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("[questions] insert failed for {qtype}: {e}"))?;
    }

    tx.commit()
        .await
        .map_err(|e| format!("[questions] tx commit failed: {e}"))?;

    eprintln!(
        "[questions] saved segment_id={segment_id} with {} questions",
        question_types.len()
    );

    let questions = question_types
        .iter()
        .map(|(qtype, item)| QuestionItem {
            qtype: qtype.to_string(),
            question: item.q.clone(),
            answer: item.a.clone(),
            rank: item.rank,
            followup_correct_question: item
                .followup_for_correct_answer
                .as_ref()
                .map(|f| f.q.clone()),
            followup_correct_answer: item
                .followup_for_correct_answer
                .as_ref()
                .map(|f| f.a.clone()),
            followup_wrong_question: item.followup_for_wrong_answer.as_ref().map(|f| f.q.clone()),
            followup_wrong_answer: item.followup_for_wrong_answer.as_ref().map(|f| f.a.clone()),
        })
        .collect();

    Ok(QuestionsResponse {
        segment: SegmentResponse {
            id: segment_id,
            video_id: video_id.to_string(),
            start_seconds: start,
            end_seconds: end,
            best_question: Some(parsed.best_question),
        },
        questions,
    })
}

#[derive(serde::Deserialize)]
pub struct QuestionUpdate {
    pub _segment_id: i64,
    pub qtype: String,
    pub question: String,
    pub answer: String,
    pub followup_correct_question: Option<String>,
    pub followup_correct_answer: Option<String>,
    pub followup_wrong_question: Option<String>,
    pub followup_wrong_answer: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct SegmentUpdate {
    pub segment_id: i64,
    pub best_question: String,
    pub questions: Vec<QuestionUpdate>,
}

#[tauri::command]
pub async fn save_questions(updates: Vec<SegmentUpdate>) -> Result<(), String> {
    let pool = get_db();

    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("[questions] tx begin failed: {e}"))?;

    for seg in &updates {
        // Update best question on segment
        sqlx::query("UPDATE segments SET best_question = ? WHERE id = ?")
            .bind(&seg.best_question)
            .bind(seg.segment_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("[questions] segment update failed: {e}"))?;

        // Update each question
        for q in &seg.questions {
            sqlx::query(
                "UPDATE questions SET
                     question                  = ?,
                     answer                    = ?,
                     followup_correct_question = ?,
                     followup_correct_answer   = ?,
                     followup_wrong_question   = ?,
                     followup_wrong_answer     = ?
                 WHERE segment_id = ? AND qtype = ?",
            )
            .bind(&q.question)
            .bind(&q.answer)
            .bind(&q.followup_correct_question)
            .bind(&q.followup_correct_answer)
            .bind(&q.followup_wrong_question)
            .bind(&q.followup_wrong_answer)
            .bind(seg.segment_id)
            .bind(&q.qtype)
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("[questions] question update failed for {}: {e}", q.qtype))?;
        }
    }

    tx.commit()
        .await
        .map_err(|e| format!("[questions] tx commit failed: {e}"))?;

    eprintln!("[questions] saved {} segments", updates.len());
    Ok(())
}

#[derive(serde::Serialize, Clone)]
pub struct QuestionRow {
    pub id: i64,
    pub segment_id: i64,
    pub qtype: String,
    pub question: String,
    pub answer: String,
    pub rank: Option<i32>,
    pub followup_correct_question: Option<String>,
    pub followup_correct_answer: Option<String>,
    pub followup_wrong_question: Option<String>,
    pub followup_wrong_answer: Option<String>,
}

#[derive(serde::Serialize, Clone)]
pub struct SegmentWithQuestions {
    pub id: i64,
    pub video_id: String,
    pub local_video_path: Option<String>,
    pub start_seconds: i32,
    pub end_seconds: i32,
    pub best_question: Option<String>,
    pub questions: Vec<QuestionRow>,
}

#[tauri::command]
pub async fn get_segments(video_id: String) -> Result<Vec<SegmentWithQuestions>, String> {
    let pool = get_db();

    // Fetch video path once
    let local_video_path: Option<String> =
        sqlx::query_as::<_, (Option<String>,)>("SELECT local_video_path FROM videos WHERE id = ?")
            .bind(&video_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| format!("[questions] video path fetch failed: {e}"))?
            .and_then(|(p,)| p.map(|v| Some(v)))
            .flatten();

    let segs = sqlx::query(
        "SELECT id, video_id, start_seconds, end_seconds, best_question
         FROM segments WHERE video_id = ? ORDER BY start_seconds ASC",
    )
    .bind(&video_id)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("[questions] get_segments failed: {e}"))?;

    let mut result = Vec::new();

    for seg in segs {
        let seg_id: i64 = sqlx::Row::get(&seg, "id");
        let start: i32 = sqlx::Row::get(&seg, "start_seconds");
        let end: i32 = sqlx::Row::get(&seg, "end_seconds");
        let best: Option<String> = sqlx::Row::try_get(&seg, "best_question").ok().flatten();

        let q_rows = sqlx::query(
            "SELECT id, segment_id, qtype, question, answer, rank,
                    followup_correct_question, followup_correct_answer,
                    followup_wrong_question, followup_wrong_answer
             FROM questions WHERE segment_id = ? ORDER BY rank ASC",
        )
        .bind(seg_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("[questions] fetch questions failed: {e}"))?;

        let questions: Vec<QuestionRow> = q_rows
            .into_iter()
            .map(|r| QuestionRow {
                id: sqlx::Row::get(&r, "id"),
                segment_id: sqlx::Row::get(&r, "segment_id"),
                qtype: sqlx::Row::get(&r, "qtype"),
                question: sqlx::Row::get(&r, "question"),
                answer: sqlx::Row::get(&r, "answer"),
                rank: sqlx::Row::try_get(&r, "rank").ok(),
                followup_correct_question: sqlx::Row::try_get(&r, "followup_correct_question")
                    .ok()
                    .flatten(),
                followup_correct_answer: sqlx::Row::try_get(&r, "followup_correct_answer")
                    .ok()
                    .flatten(),
                followup_wrong_question: sqlx::Row::try_get(&r, "followup_wrong_question")
                    .ok()
                    .flatten(),
                followup_wrong_answer: sqlx::Row::try_get(&r, "followup_wrong_answer")
                    .ok()
                    .flatten(),
            })
            .collect();

        result.push(SegmentWithQuestions {
            id: seg_id,
            video_id: video_id.clone(),
            local_video_path: local_video_path.clone(),
            start_seconds: start,
            end_seconds: end,
            best_question: best,
            questions,
        });
    }

    eprintln!(
        "[questions] get_segments → {} segments for {video_id}",
        result.len()
    );
    Ok(result)
}
