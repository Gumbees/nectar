use anyhow::Result;
use sqlx::postgres::{PgPool, PgPoolOptions};

pub type Pool = PgPool;

pub async fn connect(database_url: &str) -> Result<Pool> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(database_url)
        .await?;

    tracing::info!("connected to postgresql");
    Ok(pool)
}
