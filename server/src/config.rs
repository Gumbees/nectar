use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    #[serde(default = "default_jwt_expiry_hours")]
    pub jwt_expiry_hours: u64,
    #[serde(default = "default_nats_url")]
    pub nats_url: String,
    #[serde(default)]
    pub media_paths: Vec<String>,
    pub oidc: Option<OidcConfig>,
    #[serde(default = "default_transcode_output_dir")]
    pub transcode_output_dir: String,
    pub embedding_provider: Option<String>,
    #[serde(default = "default_ollama_url")]
    pub ollama_url: Option<String>,
    #[serde(default = "default_ollama_model")]
    pub ollama_model: Option<String>,
    pub openai_api_key: Option<String>,
    #[serde(default = "default_openai_model")]
    pub openai_model: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OidcConfig {
    pub issuer_url: String,
    pub client_id: String,
    pub client_secret: String,
}

fn default_host() -> String { "0.0.0.0".into() }
fn default_port() -> u16 { 8096 }
fn default_jwt_expiry_hours() -> u64 { 24 }
fn default_nats_url() -> String { "nats://localhost:4222".into() }
fn default_transcode_output_dir() -> String { "/tmp/nectar-transcode".into() }
fn default_ollama_url() -> Option<String> { Some("http://localhost:11434".into()) }
fn default_ollama_model() -> Option<String> { Some("nomic-embed-text".into()) }
fn default_openai_model() -> Option<String> { Some("text-embedding-3-small".into()) }

impl Config {
    pub fn load() -> Result<Self> {
        let config = config::Config::builder()
            .add_source(config::File::with_name("nectar").required(false))
            .add_source(config::Environment::with_prefix("NECTAR").separator("__"))
            .build()?;

        Ok(config.try_deserialize()?)
    }
}
