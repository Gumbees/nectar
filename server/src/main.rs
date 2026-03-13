use anyhow::Result;
use axum::{Router, routing::get};
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod auth;
mod config;
mod db;
mod media;
mod models;
mod services;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "nectar_server=debug,tower_http=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = config::Config::load()?;
    let db = db::connect(&config.database_url).await?;

    sqlx::migrate!("./migrations").run(&db).await?;

    // Connect to NATS (non-fatal if unavailable)
    let nats = match async_nats::connect(&config.nats_url).await {
        Ok(client) => {
            tracing::info!("connected to nats at {}", config.nats_url);
            Some(client)
        }
        Err(err) => {
            tracing::warn!("failed to connect to nats at {}: {err} — running without nats", config.nats_url);
            None
        }
    };

    let state = api::AppState::new(db, config.clone(), nats);

    let app = Router::new()
        .route("/api/health", get(api::health))
        .nest("/api/v1", api::routes())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("{}:{}", config.host, config.port);
    tracing::info!("nectar server listening on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
