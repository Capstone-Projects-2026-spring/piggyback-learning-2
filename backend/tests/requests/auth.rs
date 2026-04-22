use backend::app::App;
use loco_rs::testing::prelude::*;
use serial_test::serial;

fn setup_env() {
    std::env::set_var("JWT_SECRET", "oyFnOV0jDlUo6umgptSE");
    std::env::set_var("JWT_EXPIRATION", "604800");
}

#[tokio::test]
#[serial]
async fn can_signup_parent() {
    request::<App, _, _>(|request, _ctx| async move {
        let response = request
            .post("/api/auth/signup")
            .json(&serde_json::json!({
                "name": "Test Parent",
                "username": "testparent",
                "password": "password123",
                "role": "parent"
            }))
            .await;
        assert_eq!(response.status_code(), 200);

        let body: serde_json::Value = response.json();
        assert_eq!(body["success"], true);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_signup_kid() {
    request::<App, _, _>(|request, ctx| async move {
        // seed a parent first
        let parent_response = request
            .post("/api/auth/signup")
            .json(&serde_json::json!({
                "name": "Test Parent",
                "username": "testparent",
                "password": "password123",
                "role": "parent"
            }))
            .await;
        assert_eq!(parent_response.status_code(), 200);

        // get the parent id from the db
        use backend::models::_entities::parents;
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
        let parent = parents::Entity::find()
            .filter(parents::Column::Username.eq("testparent"))
            .one(&ctx.db)
            .await
            .unwrap()
            .unwrap();

        let response = request
            .post("/api/auth/signup")
            .json(&serde_json::json!({
                "name": "Test Kid",
                "username": "testkid",
                "password": "password123",
                "role": "kid",
                "parent_id": parent.id
            }))
            .await;
        assert_eq!(response.status_code(), 200);

        let body: serde_json::Value = response.json();
        assert_eq!(body["success"], true);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn duplicate_parent_username_rejected() {
    request::<App, _, _>(|request, _ctx| async move {
        let payload = serde_json::json!({
            "name": "Test Parent",
            "username": "dupeparent",
            "password": "password123",
            "role": "parent"
        });

        let first = request.post("/api/auth/signup").json(&payload).await;
        assert_eq!(first.status_code(), 200);

        let second = request.post("/api/auth/signup").json(&payload).await;
        assert_eq!(second.status_code(), 400);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn duplicate_kid_username_rejected() {
    request::<App, _, _>(|request, ctx| async move {
        use backend::models::_entities::parents;
        use sea_orm::Set;
        use sea_orm::ActiveModelTrait;

        // seed parent directly
        let parent = parents::ActiveModel {
            name: Set("Test Parent".to_string()),
            username: Set("parentforkid".to_string()),
            password_hash: Set("dummy".to_string()),
            ..Default::default()
        }
        .insert(&ctx.db)
        .await
        .unwrap();

        let payload = serde_json::json!({
            "name": "Test Kid",
            "username": "dupekid",
            "password": "password123",
            "role": "kid",
            "parent_id": parent.id
        });

        let first = request.post("/api/auth/signup").json(&payload).await;
        assert_eq!(first.status_code(), 200);

        let second = request.post("/api/auth/signup").json(&payload).await;
        assert_eq!(second.status_code(), 400);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn kid_signup_without_parent_id_rejected() {
    request::<App, _, _>(|request, _ctx| async move {
        let response = request
            .post("/api/auth/signup")
            .json(&serde_json::json!({
                "name": "Test Kid",
                "username": "testkid",
                "password": "password123",
                "role": "kid"
            }))
            .await;
        assert_eq!(response.status_code(), 400);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn invalid_role_rejected() {
    request::<App, _, _>(|request, _ctx| async move {
        let response = request
            .post("/api/auth/signup")
            .json(&serde_json::json!({
                "name": "Test",
                "username": "test",
                "password": "password123",
                "role": "admin"
            }))
            .await;
        assert_eq!(response.status_code(), 400);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_login_parent() {
    setup_env();
    request::<App, _, _>(|request, _ctx| async move {
        request
            .post("/api/auth/signup")
            .json(&serde_json::json!({
                "name": "Test Parent",
                "username": "loginparent",
                "password": "password123",
                "role": "parent"
            }))
            .await;

        let response = request
            .post("/api/auth/login")
            .json(&serde_json::json!({
                "username": "loginparent",
                "password": "password123",
                "role": "parent"
            }))
            .await;
        assert_eq!(response.status_code(), 200);

        let body: serde_json::Value = response.json();
        assert_eq!(body["success"], true);
        assert!(body.get("token").is_some());
        assert_eq!(body["role"], "parent");
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_login_kid() {
    setup_env();
    request::<App, _, _>(|request, ctx| async move {
        use backend::models::_entities::parents;
        use sea_orm::Set;
        use sea_orm::ActiveModelTrait;

        let parent = parents::ActiveModel {
            name: Set("Test Parent".to_string()),
            username: Set("parentforkidlogin".to_string()),
            password_hash: Set("dummy".to_string()),
            ..Default::default()
        }
        .insert(&ctx.db)
        .await
        .unwrap();

        request
            .post("/api/auth/signup")
            .json(&serde_json::json!({
                "name": "Test Kid",
                "username": "loginkid",
                "password": "password123",
                "role": "kid",
                "parent_id": parent.id
            }))
            .await;

        let response = request
            .post("/api/auth/login")
            .json(&serde_json::json!({
                "username": "loginkid",
                "password": "password123",
                "role": "kid"
            }))
            .await;
        assert_eq!(response.status_code(), 200);

        let body: serde_json::Value = response.json();
        assert_eq!(body["success"], true);
        assert!(body.get("token").is_some());
        assert_eq!(body["role"], "kid");
        assert!(body.get("parent_username").is_some());
    })
    .await;
}

#[tokio::test]
#[serial]
async fn login_wrong_password_rejected() {
    request::<App, _, _>(|request, _ctx| async move {
        request
            .post("/api/auth/signup")
            .json(&serde_json::json!({
                "name": "Test Parent",
                "username": "wrongpassparent",
                "password": "password123",
                "role": "parent"
            }))
            .await;

        let response = request
            .post("/api/auth/login")
            .json(&serde_json::json!({
                "username": "wrongpassparent",
                "password": "wrongpassword",
                "role": "parent"
            }))
            .await;
        assert_eq!(response.status_code(), 400);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn login_nonexistent_user_rejected() {
    request::<App, _, _>(|request, _ctx| async move {
        let response = request
            .post("/api/auth/login")
            .json(&serde_json::json!({
                "username": "doesnotexist",
                "password": "password123",
                "role": "parent"
            }))
            .await;
        assert_eq!(response.status_code(), 400);
    })
    .await;
}