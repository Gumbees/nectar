use anyhow::Result;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod ffmpeg;
mod hardware;
mod worker;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "nectar_transcoder=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = config::Config::load()?;

    // Get system hostname
    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".into());

    tracing::info!(
        worker_id = %config.worker_id,
        hostname = %hostname,
        "nectar transcoder starting"
    );

    // Detect available hardware acceleration
    let hw_caps = hardware::detect().await?;
    tracing::info!(
        encoders = ?hw_caps.encoders,
        gpu = ?hw_caps.gpu_name,
        vram_mb = ?hw_caps.vram_mb,
        hevc = hw_caps.supports_hevc,
        av1 = hw_caps.supports_av1,
        max_sessions = hw_caps.max_encode_sessions,
        "hardware detection complete"
    );

    // Connect to NATS and start processing jobs
    let client = async_nats::connect(&config.nats_url).await?;
    tracing::info!(nats_url = %config.nats_url, "connected to NATS");

    worker::run(client, hw_caps, config, hostname).await
}
