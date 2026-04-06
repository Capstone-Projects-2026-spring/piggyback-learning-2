use backend::app::App;
use loco_rs::testing::prelude::*;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn can_get_recommendations() {
    request::<App, _, _>(|request, _ctx| async move {
        let response = request.get("/api/kids/1/recommendations").await;
        assert_eq!(response.status_code(), 200);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_get_kid_tags() {
    request::<App, _, _>(|request, _ctx| async move {
        let response = request.get("/api/kids/1/tags").await;
        assert_eq!(response.status_code(), 200);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_get_videos_assigned() {
    request::<App, _, _>(|request, _ctx| async move {
        let response = request.get("/api/kids/1/videos_assigned").await;
        assert_eq!(response.status_code(), 200);
    })
    .await;
}
