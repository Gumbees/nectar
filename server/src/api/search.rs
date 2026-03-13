use axum::{Json, Router, routing::get};
use serde_json::{Value, json};

use super::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(search))
        .route("/semantic", get(semantic_search))
}

/// Traditional text search
async fn search() -> Json<Value> {
    Json(json!({ "results": [] }))
}

/// Vector embedding search via Ollama or OpenAI
async fn semantic_search() -> Json<Value> {
    Json(json!({ "todo": "semantic vector search" }))
}
