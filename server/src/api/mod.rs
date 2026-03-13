use axum::{Json, Router, extract::State};
use serde_json::{Value, json};

use crate::config::Config;
use crate::db::Pool;

pub mod auth;
pub mod errors;
pub mod extractors;
pub mod libraries;
pub mod media;
pub mod search;
pub mod streaming;
pub mod users;

#[derive(Clone)]
pub struct AppState {
    pub db: Pool,
    pub config: Config,
    pub nats: Option<async_nats::Client>,
}

impl AppState {
    pub fn new(db: Pool, config: Config, nats: Option<async_nats::Client>) -> Self {
        Self { db, config, nats }
    }
}

pub async fn health(State(state): State<AppState>) -> Result<Json<Value>, errors::ApiError> {
    let db_ok = sqlx::query("SELECT 1")
        .fetch_one(&state.db)
        .await
        .is_ok();

    let nats_ok = state.nats.is_some();

    Ok(Json(json!({
        "status": if db_ok { "healthy" } else { "degraded" },
        "version": env!("CARGO_PKG_VERSION"),
        "database": db_ok,
        "nats": nats_ok,
    })))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .nest("/auth", auth::routes())
        .nest("/libraries", libraries::routes())
        .nest("/media", media::routes())
        .nest("/search", search::routes())
        .nest("/streaming", streaming::routes())
        .nest("/users", users::routes())
}
