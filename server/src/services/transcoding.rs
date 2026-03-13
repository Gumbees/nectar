use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscodeJob {
    pub id: Uuid,
    pub media_item_id: Uuid,
    pub input_path: String,
    pub output_format: OutputFormat,
    pub target_resolution: Option<String>,
    pub target_bitrate: Option<String>,
    pub hardware_accel: Option<HardwareAccel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    Hls,
    Mp4,
    Webm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HardwareAccel {
    Nvenc,         // NVIDIA
    Qsv,           // Intel Quick Sync
    Vaapi,         // Linux VA-API (Intel/AMD)
    Amf,           // AMD AMF
    V4l2,          // Video4Linux (Orange Pi, RPi)
    Videotoolbox,  // macOS
}

pub struct TranscodingService {
    nats: async_nats::Client,
}

impl TranscodingService {
    pub fn new(nats: async_nats::Client) -> Self {
        Self { nats }
    }

    /// Publish a transcode job to NATS for a worker to pick up
    pub async fn dispatch(&self, job: TranscodeJob) -> Result<()> {
        let payload = serde_json::to_vec(&job)
            .context("failed to serialize transcode job")?;

        self.nats
            .publish("nectar.transcode.jobs.live", payload.into())
            .await
            .context("failed to publish transcode job to NATS")?;

        tracing::info!(
            job_id = %job.id,
            media_item_id = %job.media_item_id,
            "dispatched transcode job to NATS"
        );

        Ok(())
    }
}
