use backend::app::App;
use loco_rs::testing::prelude::*;
use serial_test::serial;
use backend::models::_entities::{parents, kids};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};

#[tokio::test]
#[serial]
async fn kid_parent_relationship() {
    let boot = boot_test::<App>().await.unwrap();
    let ctx = &boot.app_context;

    let parent = parents::ActiveModel {
        name: Set("Parent Name".to_string()),
        username: Set("parent_1".to_string()),
        password_hash: Set("hash".to_string()),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .unwrap();

    let kid = kids::ActiveModel {
        name: Set("Kid Name".to_string()),
        username: Set("kid_1".to_string()),
        password_hash: Set("hash".to_string()),
        parent_id: Set(parent.id),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .unwrap();

    assert_eq!(kid.parent_id, parent.id);
    let found_kid = kids::Entity::find_by_id(kid.id).one(&ctx.db).await.unwrap().unwrap();
    assert_eq!(found_kid.username, "kid_1");
}