use backend::app::App;
use loco_rs::testing::prelude::*;
use serial_test::serial;
use backend::models::_entities::videos;
use sea_orm::{ActiveModelTrait, Set};

#[tokio::test]
#[serial]
async fn video_update_timestamp_logic() {
    let boot = boot_test::<App>().await.unwrap();
    let ctx = &boot.app_context;

    let video = videos::ActiveModel {
        id: Set("vid_1".to_string()),
        title: Set(Some("Initial".to_string())),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .unwrap();

    let first_time = video.updated_at;
    let mut active: videos::ActiveModel = video.into();
    active.title = Set(Some("Updated".to_string()));
    let updated = active.update(&ctx.db).await.unwrap();

    assert!(updated.updated_at > first_time);
}