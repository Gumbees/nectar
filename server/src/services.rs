/// Business logic services
pub mod transcoding;
pub mod embeddings;

/// Transcoding service — dispatches jobs to worker nodes via NATS
pub use transcoding::{TranscodingService, TranscodeJob, OutputFormat};

/// Embedding service — generates vector embeddings via Ollama or OpenAI
pub use embeddings::{EmbeddingService, EmbeddingProvider};
