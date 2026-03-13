use axum::{Json, Router, routing::get};
use serde_json::{Value, json};

use super::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list))
        .route("/{id}", get(get_item))
        .route("/{id}/metadata", get(metadata))
        .route("/{id}/trickplay", get(trickplay))
        .route("/{id}/similar", get(similar))
}

async fn list() -> Json<Value> { Json(json!({ "items": [] })) }
async fn get_item() -> Json<Value> { Json(json!({ "todo": "get media item" })) }
async fn metadata() -> Json<Value> { Json(json!({ "todo": "get metadata" })) }
async fn trickplay() -> Json<Value> { Json(json!({ "todo": "trickplay images" })) }
async fn similar() -> Json<Value> { Json(json!({ "todo": "vector similarity search" })) }
