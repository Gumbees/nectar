use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_nats_url")]
    pub nats_url: String,
    #[serde(default = "default_ffmpeg_path")]
    pub ffmpeg_path: String,
    #[serde(default = "default_ffprobe_path")]
    pub ffprobe_path: String,
    #[serde(default = "default_work_dir")]
    pub work_dir: String,
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent_jobs: usize,
    #[serde(default = "default_worker_id")]
    pub worker_id: String,
    #[serde(default = "default_heartbeat_interval")]
    pub heartbeat_interval_secs: u64,
    #[serde(default = "default_progress_interval")]
    pub progress_interval_secs: u64,
    #[serde(default = "default_vaapi_device")]
    pub vaapi_device: String,
}

fn default_nats_url() -> String {
    "nats://localhost:4222".into()
}
fn default_ffmpeg_path() -> String {
    "ffmpeg".into()
}
fn default_ffprobe_path() -> String {
    "ffprobe".into()
}
fn default_work_dir() -> String {
    "/tmp/nectar-transcode".into()
}
fn default_max_concurrent() -> usize {
    2
}
fn default_worker_id() -> String {
    uuid::Uuid::new_v4().to_string()
}
fn default_heartbeat_interval() -> u64 {
    15
}
fn default_progress_interval() -> u64 {
    2
}
fn default_vaapi_device() -> String {
    "/dev/dri/renderD128".into()
}

impl Config {
    pub fn load() -> Result<Self> {
        let config = config::Config::builder()
            .add_source(config::Environment::with_prefix("NECTAR_TRANSCODER").separator("__"))
            .build()?;

        Ok(config.try_deserialize()?)
    }
}
