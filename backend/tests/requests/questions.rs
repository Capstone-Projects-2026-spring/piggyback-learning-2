use backend::app::App;
use loco_rs::testing::prelude::*;
use loco_rs::app::AppContext;
use serial_test::serial;
use backend::models::_entities::{questions, segments, videos};
use sea_orm::{ActiveModelTrait, Set};

async fn seed_video(ctx: &AppContext) {
    use sea_orm::EntityTrait;
    videos::Entity::insert(videos::ActiveModel {
        id: Set("test_video".to_string()),
        title: Set(Some("Test Video".to_string())),
        thumbnail_url: Set(None),
        duration_seconds: Set(None),
        local_video_path: Set(None),
        ..Default::default()
    })
    .exec(&ctx.db)
    .await
    .unwrap();
}

async fn seed_segment(ctx: &AppContext) -> segments::Model {
    seed_video(ctx).await; 
    segments::ActiveModel {
        video_id: Set("test_video".to_string()),
        start_seconds: Set(0),
        end_seconds: Set(10),
        best_question: Set(Some("Who is speaking?".to_string())),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .unwrap()
}

async fn seed_question(ctx: &AppContext, segment_id: i32) -> questions::Model {
    questions::ActiveModel {
        segment_id: Set(segment_id),
        qtype: Set("character".to_string()),
        question: Set("Who is speaking?".to_string()),
        answer: Set("actor".to_string()),
        rank: Set(Some(1)),
        followup_enabled: Set(Some(true)),
        followup_correct_question: Set(Some("What else did they say?".to_string())),
        followup_correct_answer: Set(Some("hello".to_string())),
        followup_wrong_question: Set(Some("Can you point to the speaker?".to_string())),
        followup_wrong_answer: Set(Some("actor".to_string())),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .unwrap()
}

#[tokio::test]
#[serial]
async fn can_get_questions() {
    request::<App, _, _>(|request, ctx| async move {
        let segment = seed_segment(&ctx).await;
        seed_question(&ctx, segment.id).await;

        let response = request
            .get(&format!("/api/questions/{}", segment.video_id))
            .await;
        assert_eq!(response.status_code(), 200);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn get_questions_returns_correct_shape() {
    request::<App, _, _>(|request, ctx| async move {
        let segment = seed_segment(&ctx).await;
        seed_question(&ctx, segment.id).await;

        let response = request
            .get(&format!("/api/questions/{}", segment.video_id))
            .await;
        assert_eq!(response.status_code(), 200);

        let body: serde_json::Value = response.json();
        assert!(body.get("video_id").is_some());
        assert!(body.get("segments").is_some());
        assert!(body["segments"].is_array());
    })
    .await;
}

#[tokio::test]
#[serial]
async fn get_questions_unknown_video_returns_empty_segments() {
    request::<App, _, _>(|request, _ctx| async move {
        let response = request.get("/api/questions/nonexistent_video_id").await;
        assert_eq!(response.status_code(), 200);

        let body: serde_json::Value = response.json();
        assert_eq!(body["segments"].as_array().unwrap().len(), 0);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_update_question() {
    request::<App, _, _>(|request, ctx| async move {
        let segment = seed_segment(&ctx).await;
        let question = seed_question(&ctx, segment.id).await;

        let response = request
            .patch(&format!("/api/questions/{}", question.id))
            .json(&serde_json::json!({
                "question": "Updated question?",
                "answer": "updated"
            }))
            .await;
        assert_eq!(response.status_code(), 200);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn update_question_not_found() {
    request::<App, _, _>(|request, _ctx| async move {
        let response = request
            .patch("/api/questions/999999")
            .json(&serde_json::json!({
                "question": "Does not exist"
            }))
            .await;
        assert_eq!(response.status_code(), 404);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_update_segment_best_question() {
    request::<App, _, _>(|request, ctx| async move {
        let segment = seed_segment(&ctx).await;

        let response = request
            .patch(&format!("/api/questions/segment/{}", segment.id))
            .json(&serde_json::json!({
                "best_question": "Who is speaking?"
            }))
            .await;
        assert_eq!(response.status_code(), 200);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn update_segment_best_question_not_found() {
    request::<App, _, _>(|request, _ctx| async move {
        let response = request
            .patch("/api/questions/segment/999999")
            .json(&serde_json::json!({
                "best_question": "Does not exist"
            }))
            .await;
        assert_eq!(response.status_code(), 404);
    })
    .await;
}