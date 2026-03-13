use anyhow::Result;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod hardware;
mod worker;
mod ffmpeg;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "nectar_transcoder=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = config::Config::load()?;

    // Detect available hardware acceleration
    let hw_caps = hardware::detect().await?;
    tracing::info!("detected hardware: {:?}", hw_caps);

    // Connect to NATS and start processing jobs
    let client = async_nats::connect(&config.nats_url).await?;
    tracing::info!("connected to nats at {}", config.nats_url);

    worker::run(client, hw_caps, config).await
}
