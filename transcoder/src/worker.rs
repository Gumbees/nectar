use anyhow::Result;
use async_nats::Client;
use tokio::sync::Semaphore;
use std::sync::Arc;

use crate::config::Config;
use crate::hardware::HardwareCapabilities;
use crate::ffmpeg;

pub async fn run(client: Client, hw: HardwareCapabilities, config: Config) -> Result<()> {
    let semaphore = Arc::new(Semaphore::new(config.max_concurrent_jobs));
    let mut subscriber = client.subscribe("nectar.transcode.jobs").await?;

    tracing::info!(
        "transcoder worker started, max concurrent jobs: {}",
        config.max_concurrent_jobs
    );

    while let Some(msg) = subscriber.next().await {
        let permit = semaphore.clone().acquire_owned().await?;
        let hw = hw.clone();
        let config = config.clone();
        let client = client.clone();

        tokio::spawn(async move {
            match serde_json::from_slice::<serde_json::Value>(&msg.payload) {
                Ok(job) => {
                    let job_id = job["id"].as_str().unwrap_or("unknown");
                    tracing::info!(job_id, "processing transcode job");

                    match ffmpeg::run_transcode(&job, &hw, &config).await {
                        Ok(()) => {
                            tracing::info!(job_id, "transcode complete");
                            let _ = client
                                .publish("nectar.transcode.complete", job_id.into())
                                .await;
                        }
                        Err(e) => {
                            tracing::error!(job_id, error = %e, "transcode failed");
                            let _ = client
                                .publish("nectar.transcode.failed", format!("{job_id}:{e}").into())
                                .await;
                        }
                    }
                }
                Err(e) => tracing::error!(error = %e, "invalid job payload"),
            }

            drop(permit);
        });
    }

    Ok(())
}
