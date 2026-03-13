use axum::{
    Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use super::AppState;
use super::errors::ApiError;
use super::extractors::{AdminUser, AuthUser};
use crate::media::scanner::MediaScanner;
use crate::models::Library;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{id}", get(get_library).put(update).delete(delete))
        .route("/{id}/scan", post(scan))
}

// --- Types ---

#[derive(Debug, Deserialize)]
struct CreateLibraryRequest {
    name: String,
    library_type: String,
    paths: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct UpdateLibraryRequest {
    name: Option<String>,
    library_type: Option<String>,
    paths: Option<Vec<String>>,
}

// --- Handlers ---

async fn list(
    _auth: AuthUser,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let libraries = sqlx::query_as::<_, Library>(
        "SELECT id, name, library_type, paths, created_at, updated_at
         FROM libraries
         ORDER BY name ASC",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(json!({ "libraries": libraries })))
}

async fn create(
    _admin: AdminUser,
    State(state): State<AppState>,
    Json(body): Json<CreateLibraryRequest>,
) -> Result<impl IntoResponse, ApiError> {
    if body.name.is_empty() {
        return Err(ApiError::BadRequest("name is required".to_string()));
    }

    if body.paths.is_empty() {
        return Err(ApiError::BadRequest("at least one path is required".to_string()));
    }

    // Validate library_type
    let library_type = match body.library_type.to_lowercase().as_str() {
        "movies" => "movies",
        "shows" => "shows",
        "music" => "music",
        "books" => "books",
        "photos" => "photos",
        _ => {
            return Err(ApiError::BadRequest(format!(
                "invalid library_type '{}'. Must be one of: movies, shows, music, books, photos",
                body.library_type
            )));
        }
    };

    // Validate that all paths exist on filesystem
    for path in &body.paths {
        match tokio::fs::metadata(path).await {
            Ok(meta) => {
                if !meta.is_dir() {
                    return Err(ApiError::BadRequest(format!(
                        "path '{}' exists but is not a directory",
                        path
                    )));
                }
            }
            Err(_) => {
                return Err(ApiError::BadRequest(format!(
                    "path '{}' does not exist or is not accessible",
                    path
                )));
            }
        }
    }

    let library = sqlx::query_as::<_, Library>(
        "INSERT INTO libraries (name, library_type, paths)
         VALUES ($1, $2::library_type, $3)
         RETURNING id, name, library_type, paths, created_at, updated_at",
    )
    .bind(&body.name)
    .bind(library_type)
    .bind(&body.paths)
    .fetch_one(&state.db)
    .await?;

    Ok((axum::http::StatusCode::CREATED, Json(json!(library))))
}

async fn get_library(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let library = sqlx::query_as::<_, Library>(
        "SELECT id, name, library_type, paths, created_at, updated_at
         FROM libraries WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ApiError::NotFound)?;

    Ok(Json(json!(library)))
}

async fn update(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateLibraryRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let existing = sqlx::query_as::<_, Library>(
        "SELECT id, name, library_type, paths, created_at, updated_at
         FROM libraries WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ApiError::NotFound)?;

    let name = body.name.unwrap_or(existing.name);

    // If paths are being updated, validate them
    let paths = if let Some(new_paths) = body.paths {
        if new_paths.is_empty() {
            return Err(ApiError::BadRequest("at least one path is required".to_string()));
        }
        for path in &new_paths {
            match tokio::fs::metadata(path).await {
                Ok(meta) => {
                    if !meta.is_dir() {
                        return Err(ApiError::BadRequest(format!(
                            "path '{}' exists but is not a directory",
                            path
                        )));
                    }
                }
                Err(_) => {
                    return Err(ApiError::BadRequest(format!(
                        "path '{}' does not exist or is not accessible",
                        path
                    )));
                }
            }
        }
        new_paths
    } else {
        existing.paths
    };

    // If library_type is being updated, validate it
    let library_type_str = if let Some(ref lt) = body.library_type {
        match lt.to_lowercase().as_str() {
            "movies" => "movies",
            "shows" => "shows",
            "music" => "music",
            "books" => "books",
            "photos" => "photos",
            _ => {
                return Err(ApiError::BadRequest(format!(
                    "invalid library_type '{}'. Must be one of: movies, shows, music, books, photos",
                    lt
                )));
            }
        }
    } else {
        match existing.library_type {
            crate::models::LibraryType::Movies => "movies",
            crate::models::LibraryType::Shows => "shows",
            crate::models::LibraryType::Music => "music",
            crate::models::LibraryType::Books => "books",
            crate::models::LibraryType::Photos => "photos",
        }
    };

    let library = sqlx::query_as::<_, Library>(
        "UPDATE libraries SET name = $1, library_type = $2::library_type, paths = $3, updated_at = NOW()
         WHERE id = $4
         RETURNING id, name, library_type, paths, created_at, updated_at",
    )
    .bind(&name)
    .bind(library_type_str)
    .bind(&paths)
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(json!(library)))
}

async fn delete(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let result = sqlx::query("DELETE FROM libraries WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }

    Ok(axum::http::StatusCode::NO_CONTENT)
}

async fn scan(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let library = sqlx::query_as::<_, Library>(
        "SELECT id, name, library_type, paths, created_at, updated_at
         FROM libraries WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ApiError::NotFound)?;

    let db = state.db.clone();
    tokio::spawn(async move {
        let scanner = MediaScanner::new(db);
        match scanner.scan_library(&library).await {
            Ok(result) => {
                tracing::info!(
                    library_id = %library.id,
                    library_name = %library.name,
                    items_found = result.items_found,
                    items_added = result.items_added,
                    items_updated = result.items_updated,
                    items_removed = result.items_removed,
                    "library scan completed"
                );
            }
            Err(err) => {
                tracing::error!(
                    library_id = %library.id,
                    library_name = %library.name,
                    error = %err,
                    "library scan failed"
                );
            }
        }
    });

    Ok((
        axum::http::StatusCode::ACCEPTED,
        Json(json!({ "message": "scan started" })),
    ))
}
