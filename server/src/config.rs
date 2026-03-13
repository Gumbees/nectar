use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    pub database_url: String,
    #[serde(default = "default_nats_url")]
    pub nats_url: String,
    #[serde(default)]
    pub media_paths: Vec<String>,
    pub oidc: Option<OidcConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OidcConfig {
    pub issuer_url: String,
    pub client_id: String,
    pub client_secret: String,
}

fn default_host() -> String { "0.0.0.0".into() }
fn default_port() -> u16 { 8096 }
fn default_nats_url() -> String { "nats://localhost:4222".into() }

impl Config {
    pub fn load() -> Result<Self> {
        let config = config::Config::builder()
            .add_source(config::File::with_name("nectar").required(false))
            .add_source(config::Environment::with_prefix("NECTAR").separator("__"))
            .build()?;

        Ok(config.try_deserialize()?)
    }
}
