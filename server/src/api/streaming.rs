use axum::{Json, Router, routing::get};
use serde_json::{Value, json};

use super::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/{id}/direct", get(direct_play))
        .route("/{id}/transcode", get(transcode))
        .route("/{id}/hls/{segment}", get(hls_segment))
}

/// Direct play — serve the file as-is when client supports the codec
async fn direct_play() -> Json<Value> {
    Json(json!({ "todo": "direct file streaming" }))
}

/// Request transcoded stream — dispatches job to transcoder worker via NATS
async fn transcode() -> Json<Value> {
    Json(json!({ "todo": "dispatch transcode job" }))
}

/// Serve HLS segments for adaptive bitrate streaming
async fn hls_segment() -> Json<Value> {
    Json(json!({ "todo": "serve hls segment" }))
}
