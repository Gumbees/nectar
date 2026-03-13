use axum::{Json, Router, extract::State};
use serde_json::{Value, json};

use crate::config::Config;
use crate::db::Pool;

pub mod auth;
pub mod libraries;
pub mod media;
pub mod search;
pub mod streaming;
pub mod users;

#[derive(Clone)]
pub struct AppState {
    pub db: Pool,
    pub config: Config,
}

impl AppState {
    pub fn new(db: Pool, config: Config) -> Self {
        Self { db, config }
    }
}

pub async fn health(State(state): State<AppState>) -> Json<Value> {
    let db_ok = sqlx::query("SELECT 1")
        .fetch_one(&state.db)
        .await
        .is_ok();

    Json(json!({
        "status": if db_ok { "healthy" } else { "degraded" },
        "version": env!("CARGO_PKG_VERSION"),
        "database": db_ok,
    }))
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
