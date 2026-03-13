use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmbeddingProvider {
    Ollama { base_url: String, model: String },
    OpenAi { api_key: String, model: String },
}

pub struct EmbeddingService;

impl EmbeddingService {
    /// Generate an embedding vector for a text description (title, overview, genres, etc.)
    pub async fn embed(&self, _text: &str) -> anyhow::Result<Vec<f32>> {
        // TODO: call Ollama or OpenAI embeddings API
        Ok(vec![])
    }

    /// Find similar items using pgvector cosine similarity
    pub async fn find_similar(&self, _embedding: &[f32], _limit: i32) -> anyhow::Result<Vec<uuid::Uuid>> {
        // TODO: SELECT * FROM media_items ORDER BY embedding <=> $1 LIMIT $2
        Ok(vec![])
    }
}
