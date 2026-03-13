use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_nats_url")]
    pub nats_url: String,
    #[serde(default = "default_ffmpeg_path")]
    pub ffmpeg_path: String,
    #[serde(default = "default_work_dir")]
    pub work_dir: String,
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent_jobs: usize,
}

fn default_nats_url() -> String { "nats://localhost:4222".into() }
fn default_ffmpeg_path() -> String { "ffmpeg".into() }
fn default_work_dir() -> String { "/tmp/nectar-transcode".into() }
fn default_max_concurrent() -> usize { 2 }

impl Config {
    pub fn load() -> Result<Self> {
        let config = config::Config::builder()
            .add_source(config::Environment::with_prefix("NECTAR_TRANSCODER").separator("__"))
            .build()?;

        Ok(config.try_deserialize()?)
    }
}
