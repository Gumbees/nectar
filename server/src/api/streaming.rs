use axum::{
    Router,
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use serde_json::json;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use super::AppState;
use super::errors::ApiError;
use super::extractors::AuthUser;
use crate::services::{TranscodeJob, OutputFormat};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/{id}/direct", get(direct_play))
        .route("/{id}/transcode", get(transcode))
        .route("/{id}/hls/{segment}", get(hls_segment))
}

/// Map container format to MIME type
fn content_type_for_container(container: &str) -> &'static str {
    match container.to_lowercase().as_str() {
        "mkv" | "matroska" => "video/x-matroska",
        "mp4" | "m4v" => "video/mp4",
        "avi" => "video/x-msvideo",
        "webm" => "video/webm",
        "mov" => "video/quicktime",
        "ts" | "m2ts" => "video/mp2t",
        "wmv" => "video/x-ms-wmv",
        "flv" => "video/x-flv",
        "ogv" => "video/ogg",
        "mp3" => "audio/mpeg",
        "flac" => "audio/flac",
        "ogg" | "oga" => "audio/ogg",
        "m4a" | "aac" => "audio/mp4",
        "wav" => "audio/wav",
        "wma" => "audio/x-ms-wma",
        _ => "application/octet-stream",
    }
}

/// Parse a Range header value like "bytes=0-1023" into (start, optional end).
fn parse_range(range_header: &str) -> Option<(u64, Option<u64>)> {
    let s = range_header.strip_prefix("bytes=")?;
    let mut parts = s.splitn(2, '-');
    let start_str = parts.next()?;
    let end_str = parts.next()?;
    let start: u64 = start_str.parse().ok()?;
    let end: Option<u64> = if end_str.is_empty() {
        None
    } else {
        Some(end_str.parse().ok()?)
    };
    Some((start, end))
}

/// Direct play — serve the file as-is when client supports the codec
async fn direct_play(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _user: AuthUser,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    // Look up media item
    let row: (Option<String>, Option<String>) = sqlx::query_as(
        "SELECT file_path, container FROM media_items WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ApiError::NotFound)?;

    let file_path = row.0.ok_or(ApiError::NotFound)?;
    let container = row.1.unwrap_or_default();

    // Get file metadata
    let metadata = tokio::fs::metadata(&file_path)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("cannot stat file: {e}")))?;
    let file_size = metadata.len();

    let content_type = content_type_for_container(&container);

    // Check for Range header
    let range_value = headers
        .get(header::RANGE)
        .and_then(|v| v.to_str().ok())
        .and_then(parse_range);

    match range_value {
        Some((start, end_opt)) => {
            let end = end_opt.unwrap_or(file_size - 1).min(file_size - 1);
            let content_length = end - start + 1;

            let mut file = tokio::fs::File::open(&file_path)
                .await
                .map_err(|e| ApiError::Internal(anyhow::anyhow!("cannot open file: {e}")))?;
            file.seek(std::io::SeekFrom::Start(start))
                .await
                .map_err(|e| ApiError::Internal(anyhow::anyhow!("seek failed: {e}")))?;

            let limited = file.take(content_length);
            let stream = ReaderStream::new(limited);
            let body = Body::from_stream(stream);

            Ok(Response::builder()
                .status(StatusCode::PARTIAL_CONTENT)
                .header(header::CONTENT_TYPE, content_type)
                .header(header::ACCEPT_RANGES, "bytes")
                .header(header::CONTENT_LENGTH, content_length)
                .header(
                    header::CONTENT_RANGE,
                    format!("bytes {start}-{end}/{file_size}"),
                )
                .body(body)
                .unwrap())
        }
        None => {
            let file = tokio::fs::File::open(&file_path)
                .await
                .map_err(|e| ApiError::Internal(anyhow::anyhow!("cannot open file: {e}")))?;

            let stream = ReaderStream::new(file);
            let body = Body::from_stream(stream);

            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, content_type)
                .header(header::ACCEPT_RANGES, "bytes")
                .header(header::CONTENT_LENGTH, file_size)
                .body(body)
                .unwrap())
        }
    }
}

/// Request transcoded stream — dispatches job to transcoder worker via NATS
async fn transcode(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _user: AuthUser,
) -> Result<Response, ApiError> {
    // Look up media item
    let row: (Option<String>,) = sqlx::query_as(
        "SELECT file_path FROM media_items WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ApiError::NotFound)?;

    let file_path = row.0.ok_or(ApiError::NotFound)?;

    let nats = state
        .nats
        .as_ref()
        .ok_or_else(|| ApiError::Internal(anyhow::anyhow!("NATS not available — cannot dispatch transcode job")))?;

    let job_id = Uuid::new_v4();
    let job = TranscodeJob {
        id: job_id,
        media_item_id: id,
        input_path: file_path,
        output_format: OutputFormat::Hls,
        target_resolution: None,
        target_bitrate: None,
        hardware_accel: None,
    };

    let payload = serde_json::to_vec(&job)
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("serialize failed: {e}")))?;

    nats.publish("nectar.transcode.jobs.live", payload.into())
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("NATS publish failed: {e}")))?;

    let playlist_url = format!("/api/v1/streaming/{id}/hls/stream.m3u8");

    Ok(axum::Json(json!({
        "job_id": job_id,
        "playlist_url": playlist_url,
    }))
    .into_response())
}

/// Serve HLS segments for adaptive bitrate streaming
async fn hls_segment(
    State(state): State<AppState>,
    Path((id, segment)): Path<(Uuid, String)>,
    _user: AuthUser,
) -> Result<Response, ApiError> {
    let output_dir = std::path::Path::new(&state.config.transcode_output_dir)
        .join(id.to_string());
    let segment_path = output_dir.join(&segment);

    if !segment_path.exists() {
        return Err(ApiError::NotFound);
    }

    let content_type = if segment.ends_with(".m3u8") {
        "application/vnd.apple.mpegurl"
    } else if segment.ends_with(".ts") {
        "video/mp2t"
    } else {
        "application/octet-stream"
    };

    let file = tokio::fs::File::open(&segment_path)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("cannot open segment: {e}")))?;

    let metadata = file
        .metadata()
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("cannot stat segment: {e}")))?;

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONTENT_LENGTH, metadata.len())
        .body(body)
        .unwrap())
}
