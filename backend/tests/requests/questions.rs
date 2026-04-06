use backend::app::App;
use loco_rs::testing::prelude::*;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn can_get_questions() {
    request::<App, _, _>(|request, _ctx| async move {
        let response = request.get("/api/questions/l2FQ8ni1MfM").await;
        assert_eq!(response.status_code(), 200);
    })
    .await;
}
