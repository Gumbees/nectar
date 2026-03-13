use axum::{Json, Router, routing::post};
use serde_json::{Value, json};

use super::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/oidc/callback", post(oidc_callback))
}

async fn login() -> Json<Value> {
    Json(json!({ "todo": "local auth" }))
}

async fn logout() -> Json<Value> {
    Json(json!({ "ok": true }))
}

async fn oidc_callback() -> Json<Value> {
    Json(json!({ "todo": "oidc callback" }))
}
