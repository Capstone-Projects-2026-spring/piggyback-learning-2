use std::env;

use loco_rs::prelude::*;
use loco_rs::{auth::jwt, hash};
use serde::Deserialize;
use serde_json::Map;

use crate::models::_entities::{kids, parents};

#[derive(Deserialize)]
struct SignupData {
    name: String,
    username: String,
    password: String,
    role: String,           // "parent" or "kid"
    parent_id: Option<i32>, // required if role is "kid"
}

#[derive(Deserialize)]
struct LoginData {
    username: String,
    password: String,
    role: String, // "parent" or "kid"
}

fn hash_password(password: &str) -> String {
    hash::hash_password(password).expect("Failed to hash password")
}

fn verify_password(hash: &str, password: &str) -> bool {
    hash::verify_password(password, hash)
}

pub fn generate_jwt(id: i32) -> ModelResult<String> {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set.");
    let expiration = env::var("JWT_EXPIRATION").expect("JWT_EXPIRATION must be set.");
    jwt::JWT::new(&secret)
        .generate_token(
            expiration.parse::<u64>().unwrap(),
            id.to_string(),
            Map::new(),
        )
        .map_err(ModelError::from)
}

async fn signup(State(ctx): State<AppContext>, Json(data): Json<SignupData>) -> Result<Response> {
    let password_hash = hash_password(&data.password);

    match data.role.as_str() {
        "parent" => {
            let _ = parents::ActiveModel {
                name: Set(data.name),
                username: Set(data.username),
                password_hash: Set(password_hash),
                ..Default::default()
            }
            .save(&ctx.db)
            .await;

            return format::json(serde_json::json!({"success": true}));
        }
        "kid" => {
            if data.parent_id.is_none() {
                return Err(Error::BadRequest(
                    "Kid doesn't have a parent id".to_string(),
                ));
            }

            let _ = kids::ActiveModel {
                name: Set(data.name),
                username: Set(data.username),
                password_hash: Set(password_hash),
                parent_id: Set(data.parent_id.unwrap()),
                ..Default::default()
            }
            .save(&ctx.db)
            .await;

            return format::json(serde_json::json!({"success": true}));
        }
        _ => Err(Error::BadRequest("Invalid role".to_string())),
    }
}

async fn login(State(ctx): State<AppContext>, Json(data): Json<LoginData>) -> Result<Response> {
    match data.role.as_str() {
        "parent" => {
            let parent = parents::Entity::find()
                .filter(parents::Column::Username.eq(data.username.clone()))
                .one(&ctx.db)
                .await?
                .ok_or_else(|| Error::BadRequest("Parent not found".to_string()))?;

            if !verify_password(&parent.password_hash, &data.password) {
                return Err(Error::BadRequest("Invalid password".to_string()));
            }

            let token = generate_jwt(parent.id)?;

            format::json(serde_json::json!({
                "success": true,
                "token": token,
                "role": "parent",
                "account": parent
            }))
        }

        "kid" => {
            let kid = kids::Entity::find()
                .filter(kids::Column::Username.eq(data.username.clone()))
                .one(&ctx.db)
                .await?
                .ok_or_else(|| Error::BadRequest("Kid not found".to_string()))?;

            if !verify_password(&kid.password_hash, &data.password) {
                return Err(Error::BadRequest("Invalid password".to_string()));
            }

            let token = generate_jwt(kid.id)?;

            format::json(serde_json::json!({
                "success": true,
                "token": token,
                "role": "kid",
                "account": kid
            }))
        }

        _ => Err(Error::BadRequest("Invalid role".to_string())),
    }
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("auth")
        .add("/signup", post(signup))
        .add("/login", post(login))
}
