use axum::{
    Json, Router,
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::get,
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use super::AppState;
use super::errors::ApiError;
use super::extractors::AuthUser;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list))
        .route("/{id}", get(get_item))
        .route("/{id}/metadata", get(metadata))
        .route("/{id}/trickplay", get(trickplay))
        .route("/{id}/similar", get(similar))
}

// --- Column list (everything except embedding) ---

const MEDIA_ITEM_COLUMNS: &str = "\
    m.id, m.library_id, m.parent_id, m.item_type, m.title, m.sort_title, \
    m.original_title, m.overview, m.year, m.runtime_seconds, m.file_path, \
    m.container, m.video_codec, m.audio_codec, m.resolution, m.bitrate, \
    m.size_bytes, m.metadata_json, m.created_at, m.updated_at, \
    m.community_rating, m.critic_rating, m.content_rating, m.tagline, \
    m.premiere_date, m.end_date, m.season_number, m.episode_number, \
    m.absolute_episode_number, m.track_number, m.disc_number, m.album_artist, \
    m.date_added, m.last_scanned_at, m.file_mtime";

// --- Types ---

#[derive(Debug, Deserialize)]
struct ListParams {
    library: Option<Uuid>,
    parent: Option<Uuid>,
    #[serde(rename = "type")]
    item_type: Option<String>,
    sort: Option<String>,
    order: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
}

/// MediaItem row without the embedding vector column (which needs pgvector decoding).
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
struct MediaItemRow {
    pub id: Uuid,
    pub library_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub item_type: crate::models::MediaItemType,
    pub title: String,
    pub sort_title: Option<String>,
    pub original_title: Option<String>,
    pub overview: Option<String>,
    pub year: Option<i32>,
    pub runtime_seconds: Option<i32>,
    pub file_path: Option<String>,
    pub container: Option<String>,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub resolution: Option<String>,
    pub bitrate: Option<i64>,
    pub size_bytes: Option<i64>,
    pub metadata_json: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub community_rating: Option<f32>,
    pub critic_rating: Option<f32>,
    pub content_rating: Option<String>,
    pub tagline: Option<String>,
    pub premiere_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub season_number: Option<i32>,
    pub episode_number: Option<i32>,
    pub absolute_episode_number: Option<i32>,
    pub track_number: Option<i32>,
    pub disc_number: Option<i32>,
    pub album_artist: Option<String>,
    pub date_added: Option<DateTime<Utc>>,
    pub last_scanned_at: Option<DateTime<Utc>>,
    pub file_mtime: Option<DateTime<Utc>>,
}

// --- Handlers ---

async fn list(
    _auth: AuthUser,
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> Result<impl IntoResponse, ApiError> {
    let limit = params.limit.unwrap_or(50).min(200);
    let offset = params.offset.unwrap_or(0);

    // Validate sort/order
    let sort_column = match params.sort.as_deref() {
        Some("title") | None => "m.sort_title",
        Some("year") => "m.year",
        Some("date_added") => "m.date_added",
        Some("rating") => "m.community_rating",
        Some(other) => {
            return Err(ApiError::BadRequest(format!(
                "invalid sort field '{}'. Must be one of: title, year, date_added, rating",
                other
            )));
        }
    };

    let sort_order = match params.order.as_deref() {
        Some("desc") => "DESC NULLS LAST",
        Some("asc") | None => "ASC NULLS LAST",
        Some(other) => {
            return Err(ApiError::BadRequest(format!(
                "invalid order '{}'. Must be 'asc' or 'desc'",
                other
            )));
        }
    };

    // Use separate queries per filter combination to keep sqlx type-safe binding
    let (items, total): (Vec<MediaItemRow>, i64) = if let Some(library_id) = params.library {
        if let Some(parent_id) = params.parent {
            if let Some(ref item_type) = params.item_type {
                let total: (i64,) = sqlx::query_as(
                    "SELECT COUNT(*) FROM media_items m \
                     WHERE m.library_id = $1 AND m.parent_id = $2 AND m.item_type = $3::media_item_type",
                )
                .bind(library_id).bind(parent_id).bind(item_type)
                .fetch_one(&state.db).await?;

                let items = sqlx::query_as::<_, MediaItemRow>(&format!(
                    "SELECT {MEDIA_ITEM_COLUMNS} FROM media_items m \
                     WHERE m.library_id = $1 AND m.parent_id = $2 AND m.item_type = $3::media_item_type \
                     ORDER BY {sort_column} {sort_order} LIMIT $4 OFFSET $5"
                ))
                .bind(library_id).bind(parent_id).bind(item_type)
                .bind(limit).bind(offset)
                .fetch_all(&state.db).await?;

                (items, total.0)
            } else {
                let total: (i64,) = sqlx::query_as(
                    "SELECT COUNT(*) FROM media_items m \
                     WHERE m.library_id = $1 AND m.parent_id = $2",
                )
                .bind(library_id).bind(parent_id)
                .fetch_one(&state.db).await?;

                let items = sqlx::query_as::<_, MediaItemRow>(&format!(
                    "SELECT {MEDIA_ITEM_COLUMNS} FROM media_items m \
                     WHERE m.library_id = $1 AND m.parent_id = $2 \
                     ORDER BY {sort_column} {sort_order} LIMIT $3 OFFSET $4"
                ))
                .bind(library_id).bind(parent_id)
                .bind(limit).bind(offset)
                .fetch_all(&state.db).await?;

                (items, total.0)
            }
        } else if let Some(ref item_type) = params.item_type {
            let total: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM media_items m \
                 WHERE m.library_id = $1 AND m.item_type = $2::media_item_type",
            )
            .bind(library_id).bind(item_type)
            .fetch_one(&state.db).await?;

            let items = sqlx::query_as::<_, MediaItemRow>(&format!(
                "SELECT {MEDIA_ITEM_COLUMNS} FROM media_items m \
                 WHERE m.library_id = $1 AND m.item_type = $2::media_item_type \
                 ORDER BY {sort_column} {sort_order} LIMIT $3 OFFSET $4"
            ))
            .bind(library_id).bind(item_type)
            .bind(limit).bind(offset)
            .fetch_all(&state.db).await?;

            (items, total.0)
        } else {
            let total: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM media_items m WHERE m.library_id = $1",
            )
            .bind(library_id)
            .fetch_one(&state.db).await?;

            let items = sqlx::query_as::<_, MediaItemRow>(&format!(
                "SELECT {MEDIA_ITEM_COLUMNS} FROM media_items m \
                 WHERE m.library_id = $1 \
                 ORDER BY {sort_column} {sort_order} LIMIT $2 OFFSET $3"
            ))
            .bind(library_id)
            .bind(limit).bind(offset)
            .fetch_all(&state.db).await?;

            (items, total.0)
        }
    } else if let Some(parent_id) = params.parent {
        if let Some(ref item_type) = params.item_type {
            let total: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM media_items m \
                 WHERE m.parent_id = $1 AND m.item_type = $2::media_item_type",
            )
            .bind(parent_id).bind(item_type)
            .fetch_one(&state.db).await?;

            let items = sqlx::query_as::<_, MediaItemRow>(&format!(
                "SELECT {MEDIA_ITEM_COLUMNS} FROM media_items m \
                 WHERE m.parent_id = $1 AND m.item_type = $2::media_item_type \
                 ORDER BY {sort_column} {sort_order} LIMIT $3 OFFSET $4"
            ))
            .bind(parent_id).bind(item_type)
            .bind(limit).bind(offset)
            .fetch_all(&state.db).await?;

            (items, total.0)
        } else {
            let total: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM media_items m WHERE m.parent_id = $1",
            )
            .bind(parent_id)
            .fetch_one(&state.db).await?;

            let items = sqlx::query_as::<_, MediaItemRow>(&format!(
                "SELECT {MEDIA_ITEM_COLUMNS} FROM media_items m \
                 WHERE m.parent_id = $1 \
                 ORDER BY {sort_column} {sort_order} LIMIT $2 OFFSET $3"
            ))
            .bind(parent_id)
            .bind(limit).bind(offset)
            .fetch_all(&state.db).await?;

            (items, total.0)
        }
    } else if let Some(ref item_type) = params.item_type {
        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM media_items m \
             WHERE m.item_type = $1::media_item_type",
        )
        .bind(item_type)
        .fetch_one(&state.db).await?;

        let items = sqlx::query_as::<_, MediaItemRow>(&format!(
            "SELECT {MEDIA_ITEM_COLUMNS} FROM media_items m \
             WHERE m.item_type = $1::media_item_type \
             ORDER BY {sort_column} {sort_order} LIMIT $2 OFFSET $3"
        ))
        .bind(item_type)
        .bind(limit).bind(offset)
        .fetch_all(&state.db).await?;

        (items, total.0)
    } else {
        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM media_items m",
        )
        .fetch_one(&state.db).await?;

        let items = sqlx::query_as::<_, MediaItemRow>(&format!(
            "SELECT {MEDIA_ITEM_COLUMNS} FROM media_items m \
             ORDER BY {sort_column} {sort_order} LIMIT $1 OFFSET $2"
        ))
        .bind(limit).bind(offset)
        .fetch_all(&state.db).await?;

        (items, total.0)
    };

    Ok(Json(json!({ "items": items, "total": total })))
}

async fn get_item(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let item = sqlx::query_as::<_, MediaItemRow>(&format!(
        "SELECT {MEDIA_ITEM_COLUMNS} FROM media_items m WHERE m.id = $1"
    ))
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ApiError::NotFound)?;

    // Get playback progress for this user
    let progress = sqlx::query_as::<_, crate::models::PlaybackProgress>(
        "SELECT * FROM playback_progress WHERE user_id = $1 AND media_item_id = $2",
    )
    .bind(auth.id)
    .bind(id)
    .fetch_optional(&state.db)
    .await?;

    Ok(Json(json!({
        "item": item,
        "progress": progress,
    })))
}

async fn metadata(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let row: (Option<serde_json::Value>,) = sqlx::query_as(
        "SELECT metadata_json FROM media_items WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ApiError::NotFound)?;

    Ok(Json(json!({ "metadata": row.0 })))
}

async fn trickplay(
    _auth: AuthUser,
    Path(_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    Err(ApiError::NotFound)
}

async fn similar(
    _auth: AuthUser,
    Path(_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(json!({ "items": [], "total": 0 })))
}
