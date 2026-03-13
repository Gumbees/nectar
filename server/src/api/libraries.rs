use axum::{Json, Router, routing::get};
use serde_json::{Value, json};

use super::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{id}", get(get_library).put(update).delete(delete))
        .route("/{id}/scan", post(scan))
}

async fn list() -> Json<Value> { Json(json!({ "libraries": [] })) }
async fn create() -> Json<Value> { Json(json!({ "todo": "create library" })) }
async fn get_library() -> Json<Value> { Json(json!({ "todo": "get library" })) }
async fn update() -> Json<Value> { Json(json!({ "todo": "update library" })) }
async fn delete() -> Json<Value> { Json(json!({ "todo": "delete library" })) }

use axum::routing::post;
async fn scan() -> Json<Value> { Json(json!({ "todo": "trigger library scan" })) }
