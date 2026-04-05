use backend::app::App;
use loco_rs::testing::prelude::*;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn can_get_parent_kids() {
    request::<App, _, _>(|request, _ctx| async move {
        let response = request.get("/api/parents/1/kids").await;
        assert_eq!(response.status_code(), 200);
    })
    .await;
}
