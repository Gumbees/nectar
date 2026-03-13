use axum::{
    Json, Router,
    extract::{Query, State},
    response::IntoResponse,
    routing::get,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::AppState;
use super::errors::ApiError;
use super::extractors::AuthUser;
use crate::services::{EmbeddingProvider, EmbeddingService};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(search))
        .route("/semantic", get(semantic_search))
}

#[derive(Debug, Deserialize)]
struct SearchParams {
    q: String,
    library_id: Option<Uuid>,
    item_type: Option<String>,
    #[serde(default = "default_limit_i64")]
    limit: i64,
    #[serde(default)]
    offset: i64,
}

fn default_limit_i64() -> i64 {
    20
}

fn default_limit_i32() -> i32 {
    20
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct SearchResult {
    id: Uuid,
    library_id: Uuid,
    title: String,
    item_type: crate::models::MediaItemType,
    overview: Option<String>,
    year: Option<i32>,
    file_path: Option<String>,
    container: Option<String>,
}

#[derive(Debug, Serialize)]
struct SearchResponse {
    results: Vec<SearchResult>,
    total: i64,
}

/// Traditional text search — full-text search with ILIKE fallback
async fn search(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(params): Query<SearchParams>,
) -> Result<impl IntoResponse, ApiError> {
    if params.q.is_empty() {
        return Err(ApiError::BadRequest("query parameter 'q' is required".into()));
    }

    // Try full-text search first; fall back to ILIKE if search_vector column doesn't exist.
    let fts_result = try_fulltext_search(&state, &params).await;

    let response = match fts_result {
        Ok(resp) => resp,
        Err(_) => ilike_search(&state, &params).await?,
    };

    Ok(Json(response))
}

async fn try_fulltext_search(
    state: &AppState,
    params: &SearchParams,
) -> Result<SearchResponse, ApiError> {
    let (results, total) = if let (Some(lib_id), Some(ref itype)) =
        (params.library_id, &params.item_type)
    {
        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM media_items \
             WHERE search_vector @@ plainto_tsquery('english', $1) \
             AND library_id = $2 AND item_type = $3",
        )
        .bind(&params.q)
        .bind(lib_id)
        .bind(itype)
        .fetch_one(&state.db)
        .await?;

        let rows: Vec<SearchResult> = sqlx::query_as(
            "SELECT id, library_id, title, item_type, overview, year, file_path, container \
             FROM media_items \
             WHERE search_vector @@ plainto_tsquery('english', $1) \
             AND library_id = $2 AND item_type = $3 \
             ORDER BY ts_rank(search_vector, plainto_tsquery('english', $1)) DESC \
             LIMIT $4 OFFSET $5",
        )
        .bind(&params.q)
        .bind(lib_id)
        .bind(itype)
        .bind(params.limit)
        .bind(params.offset)
        .fetch_all(&state.db)
        .await?;

        (rows, total.0)
    } else if let Some(lib_id) = params.library_id {
        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM media_items \
             WHERE search_vector @@ plainto_tsquery('english', $1) \
             AND library_id = $2",
        )
        .bind(&params.q)
        .bind(lib_id)
        .fetch_one(&state.db)
        .await?;

        let rows: Vec<SearchResult> = sqlx::query_as(
            "SELECT id, library_id, title, item_type, overview, year, file_path, container \
             FROM media_items \
             WHERE search_vector @@ plainto_tsquery('english', $1) \
             AND library_id = $2 \
             ORDER BY ts_rank(search_vector, plainto_tsquery('english', $1)) DESC \
             LIMIT $3 OFFSET $4",
        )
        .bind(&params.q)
        .bind(lib_id)
        .bind(params.limit)
        .bind(params.offset)
        .fetch_all(&state.db)
        .await?;

        (rows, total.0)
    } else if let Some(ref itype) = params.item_type {
        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM media_items \
             WHERE search_vector @@ plainto_tsquery('english', $1) \
             AND item_type = $2",
        )
        .bind(&params.q)
        .bind(itype)
        .fetch_one(&state.db)
        .await?;

        let rows: Vec<SearchResult> = sqlx::query_as(
            "SELECT id, library_id, title, item_type, overview, year, file_path, container \
             FROM media_items \
             WHERE search_vector @@ plainto_tsquery('english', $1) \
             AND item_type = $2 \
             ORDER BY ts_rank(search_vector, plainto_tsquery('english', $1)) DESC \
             LIMIT $3 OFFSET $4",
        )
        .bind(&params.q)
        .bind(itype)
        .bind(params.limit)
        .bind(params.offset)
        .fetch_all(&state.db)
        .await?;

        (rows, total.0)
    } else {
        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM media_items \
             WHERE search_vector @@ plainto_tsquery('english', $1)",
        )
        .bind(&params.q)
        .fetch_one(&state.db)
        .await?;

        let rows: Vec<SearchResult> = sqlx::query_as(
            "SELECT id, library_id, title, item_type, overview, year, file_path, container \
             FROM media_items \
             WHERE search_vector @@ plainto_tsquery('english', $1) \
             ORDER BY ts_rank(search_vector, plainto_tsquery('english', $1)) DESC \
             LIMIT $2 OFFSET $3",
        )
        .bind(&params.q)
        .bind(params.limit)
        .bind(params.offset)
        .fetch_all(&state.db)
        .await?;

        (rows, total.0)
    };

    Ok(SearchResponse { results, total })
}

async fn ilike_search(
    state: &AppState,
    params: &SearchParams,
) -> Result<SearchResponse, ApiError> {
    let pattern = format!("%{}%", params.q);

    let (results, total) = if let (Some(lib_id), Some(ref itype)) =
        (params.library_id, &params.item_type)
    {
        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM media_items \
             WHERE title ILIKE $1 AND library_id = $2 AND item_type = $3",
        )
        .bind(&pattern)
        .bind(lib_id)
        .bind(itype)
        .fetch_one(&state.db)
        .await?;

        let rows: Vec<SearchResult> = sqlx::query_as(
            "SELECT id, library_id, title, item_type, overview, year, file_path, container \
             FROM media_items \
             WHERE title ILIKE $1 AND library_id = $2 AND item_type = $3 \
             ORDER BY title LIMIT $4 OFFSET $5",
        )
        .bind(&pattern)
        .bind(lib_id)
        .bind(itype)
        .bind(params.limit)
        .bind(params.offset)
        .fetch_all(&state.db)
        .await?;

        (rows, total.0)
    } else if let Some(lib_id) = params.library_id {
        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM media_items \
             WHERE title ILIKE $1 AND library_id = $2",
        )
        .bind(&pattern)
        .bind(lib_id)
        .fetch_one(&state.db)
        .await?;

        let rows: Vec<SearchResult> = sqlx::query_as(
            "SELECT id, library_id, title, item_type, overview, year, file_path, container \
             FROM media_items \
             WHERE title ILIKE $1 AND library_id = $2 \
             ORDER BY title LIMIT $3 OFFSET $4",
        )
        .bind(&pattern)
        .bind(lib_id)
        .bind(params.limit)
        .bind(params.offset)
        .fetch_all(&state.db)
        .await?;

        (rows, total.0)
    } else if let Some(ref itype) = params.item_type {
        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM media_items \
             WHERE title ILIKE $1 AND item_type = $2",
        )
        .bind(&pattern)
        .bind(itype)
        .fetch_one(&state.db)
        .await?;

        let rows: Vec<SearchResult> = sqlx::query_as(
            "SELECT id, library_id, title, item_type, overview, year, file_path, container \
             FROM media_items \
             WHERE title ILIKE $1 AND item_type = $2 \
             ORDER BY title LIMIT $3 OFFSET $4",
        )
        .bind(&pattern)
        .bind(itype)
        .bind(params.limit)
        .bind(params.offset)
        .fetch_all(&state.db)
        .await?;

        (rows, total.0)
    } else {
        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM media_items WHERE title ILIKE $1",
        )
        .bind(&pattern)
        .fetch_one(&state.db)
        .await?;

        let rows: Vec<SearchResult> = sqlx::query_as(
            "SELECT id, library_id, title, item_type, overview, year, file_path, container \
             FROM media_items WHERE title ILIKE $1 \
             ORDER BY title LIMIT $2 OFFSET $3",
        )
        .bind(&pattern)
        .bind(params.limit)
        .bind(params.offset)
        .fetch_all(&state.db)
        .await?;

        (rows, total.0)
    };

    Ok(SearchResponse { results, total })
}

// ── Semantic search ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct SemanticSearchParams {
    q: String,
    #[serde(default = "default_limit_i32")]
    limit: i32,
}

#[derive(Debug, Serialize)]
struct SemanticResult {
    id: Uuid,
    title: String,
    similarity: f32,
}

#[derive(Debug, Serialize)]
struct SemanticSearchResponse {
    results: Vec<SemanticResult>,
}

/// Vector embedding search via Ollama or OpenAI
async fn semantic_search(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(params): Query<SemanticSearchParams>,
) -> Result<impl IntoResponse, ApiError> {
    if params.q.is_empty() {
        return Err(ApiError::BadRequest("query parameter 'q' is required".into()));
    }

    // Build the embedding service from config
    let provider = match state.config.embedding_provider.as_deref() {
        Some("ollama") => {
            let base_url = state
                .config
                .ollama_url
                .clone()
                .unwrap_or_else(|| "http://localhost:11434".into());
            let model = state
                .config
                .ollama_model
                .clone()
                .unwrap_or_else(|| "nomic-embed-text".into());
            EmbeddingProvider::Ollama { base_url, model }
        }
        Some("openai") => {
            let api_key = state
                .config
                .openai_api_key
                .clone()
                .ok_or_else(|| ApiError::BadRequest("OpenAI API key not configured".into()))?;
            let model = state
                .config
                .openai_model
                .clone()
                .unwrap_or_else(|| "text-embedding-3-small".into());
            EmbeddingProvider::OpenAi { api_key, model }
        }
        Some(other) => {
            return Err(ApiError::BadRequest(format!(
                "unknown embedding provider: {other}"
            )));
        }
        None => {
            return Err(ApiError::BadRequest(
                "embedding provider not configured — set NECTAR__EMBEDDING_PROVIDER to 'ollama' or 'openai'"
                    .into(),
            ));
        }
    };

    let service = EmbeddingService::new(provider);

    // Generate embedding for query text
    let query_embedding = service
        .embed(&params.q)
        .await
        .map_err(|e| ApiError::Internal(e))?;

    // Find similar items
    let similar = service
        .find_similar(&state.db, &query_embedding, params.limit)
        .await
        .map_err(|e| ApiError::Internal(e))?;

    // Fetch titles for the results
    let ids: Vec<Uuid> = similar.iter().map(|(id, _)| *id).collect();
    let similarity_map: std::collections::HashMap<Uuid, f32> =
        similar.into_iter().collect();

    if ids.is_empty() {
        return Ok(Json(SemanticSearchResponse { results: vec![] }));
    }

    let rows: Vec<(Uuid, String)> = sqlx::query_as(
        "SELECT id, title FROM media_items WHERE id = ANY($1)",
    )
    .bind(&ids)
    .fetch_all(&state.db)
    .await?;

    let mut results: Vec<SemanticResult> = rows
        .into_iter()
        .map(|(id, title)| SemanticResult {
            id,
            title,
            similarity: similarity_map.get(&id).copied().unwrap_or(0.0),
        })
        .collect();

    // Sort by similarity descending
    results.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap_or(std::cmp::Ordering::Equal));

    Ok(Json(SemanticSearchResponse { results }))
}
