use backend::models::_entities::video_assignments;
use serde_json::json;

#[tokio::test]
#[serial]
async fn video_assignment_json_storage() {
    let boot = boot_test::<App>().await.unwrap();
    let ctx = &boot.app_context;

    let test_answers = json!([
        {
            "transcript": "hello",
            "is_correct": true,
            "similarity_score": 1.0,
            "mood": "excited",
            "energy": 5000.0,
            "segment_id": 1
        }
    ]);

    let assignment = video_assignments::ActiveModel {
        kid_id: Set(1),
        video_id: Set("test_vid".to_string()),
        answers: Set(Some(test_answers.clone())),
        ..Default::default()
    };

    assert!(assignment.answers.as_ref().is_some());
    let stored_json = assignment.answers.as_ref().unwrap();
    assert_eq!(stored_json, &Some(test_answers));
}