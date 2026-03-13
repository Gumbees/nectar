use axum::{Json, Router, routing::get};
use serde_json::{Value, json};

use super::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/me", get(me))
        .route("/{id}", get(get_user).put(update).delete(delete))
}

async fn list() -> Json<Value> { Json(json!({ "users": [] })) }
async fn create() -> Json<Value> { Json(json!({ "todo": "create user" })) }
async fn me() -> Json<Value> { Json(json!({ "todo": "current user profile" })) }
async fn get_user() -> Json<Value> { Json(json!({ "todo": "get user" })) }
async fn update() -> Json<Value> { Json(json!({ "todo": "update user" })) }
async fn delete() -> Json<Value> { Json(json!({ "todo": "delete user" })) }
