use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmbeddingProvider {
    Ollama { base_url: String, model: String },
    OpenAi { api_key: String, model: String },
}

pub struct EmbeddingService {
    provider: EmbeddingProvider,
    client: reqwest::Client,
}

// ── Ollama response types ────────────────────────────────────────────────────

#[derive(Deserialize)]
struct OllamaEmbeddingResponse {
    embedding: Vec<f32>,
}

#[derive(Serialize)]
struct OllamaEmbeddingRequest<'a> {
    model: &'a str,
    prompt: &'a str,
}

// ── OpenAI response types ────────────────────────────────────────────────────

#[derive(Serialize)]
struct OpenAiEmbeddingRequest<'a> {
    model: &'a str,
    input: &'a str,
}

#[derive(Deserialize)]
struct OpenAiEmbeddingResponse {
    data: Vec<OpenAiEmbeddingData>,
}

#[derive(Deserialize)]
struct OpenAiEmbeddingData {
    embedding: Vec<f32>,
}

impl EmbeddingService {
    pub fn new(provider: EmbeddingProvider) -> Self {
        Self {
            provider,
            client: reqwest::Client::new(),
        }
    }

    /// Generate an embedding vector for a text description (title, overview, genres, etc.)
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        match &self.provider {
            EmbeddingProvider::Ollama { base_url, model } => {
                let url = format!("{}/api/embeddings", base_url.trim_end_matches('/'));
                let body = OllamaEmbeddingRequest {
                    model,
                    prompt: text,
                };
                let resp = self
                    .client
                    .post(&url)
                    .json(&body)
                    .send()
                    .await
                    .context("failed to call Ollama embeddings API")?;

                if !resp.status().is_success() {
                    let status = resp.status();
                    let body_text = resp.text().await.unwrap_or_default();
                    bail!("Ollama returned {status}: {body_text}");
                }

                let parsed: OllamaEmbeddingResponse = resp
                    .json()
                    .await
                    .context("failed to parse Ollama embedding response")?;
                Ok(parsed.embedding)
            }
            EmbeddingProvider::OpenAi { api_key, model } => {
                let body = OpenAiEmbeddingRequest {
                    model,
                    input: text,
                };
                let resp = self
                    .client
                    .post("https://api.openai.com/v1/embeddings")
                    .header("Authorization", format!("Bearer {api_key}"))
                    .json(&body)
                    .send()
                    .await
                    .context("failed to call OpenAI embeddings API")?;

                if !resp.status().is_success() {
                    let status = resp.status();
                    let body_text = resp.text().await.unwrap_or_default();
                    bail!("OpenAI returned {status}: {body_text}");
                }

                let parsed: OpenAiEmbeddingResponse = resp
                    .json()
                    .await
                    .context("failed to parse OpenAI embedding response")?;
                parsed
                    .data
                    .into_iter()
                    .next()
                    .map(|d| d.embedding)
                    .ok_or_else(|| anyhow::anyhow!("OpenAI returned empty embedding data"))
            }
        }
    }

    /// Find similar items using pgvector cosine similarity.
    /// Returns Vec<(item_id, similarity_score)>.
    pub async fn find_similar(
        &self,
        pool: &PgPool,
        embedding: &[f32],
        limit: i32,
    ) -> Result<Vec<(Uuid, f32)>> {
        let embedding_str = format!(
            "[{}]",
            embedding
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );

        let rows: Vec<(Uuid, f32)> = sqlx::query_as(
            r#"
            SELECT id, (1.0 - (embedding <=> $1::vector))::real as similarity
            FROM media_items
            WHERE embedding IS NOT NULL
            ORDER BY embedding <=> $1::vector
            LIMIT $2
            "#,
        )
        .bind(&embedding_str)
        .bind(limit)
        .fetch_all(pool)
        .await
        .context("pgvector similarity query failed")?;

        Ok(rows)
    }
}
