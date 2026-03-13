use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::get,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use super::AppState;
use super::errors::ApiError;
use super::extractors::{AdminUser, AuthUser};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/me", get(me))
        .route("/{id}", get(get_user).put(update).delete(delete))
}

// --- Types ---

#[derive(Debug, Deserialize)]
struct CreateUserRequest {
    username: String,
    password: String,
    email: Option<String>,
    is_admin: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct UpdateUserRequest {
    username: Option<String>,
    email: Option<String>,
    password: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PaginationParams {
    limit: Option<i64>,
    offset: Option<i64>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct UserResponse {
    id: Uuid,
    username: String,
    email: Option<String>,
    is_admin: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
struct PublicUserResponse {
    id: Uuid,
    username: String,
}

// --- Handlers ---

async fn list(
    _admin: AdminUser,
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, ApiError> {
    let limit = params.limit.unwrap_or(50).min(200);
    let offset = params.offset.unwrap_or(0);

    let users = sqlx::query_as::<_, UserResponse>(
        "SELECT id, username, email, is_admin, created_at, updated_at
         FROM users
         ORDER BY created_at ASC
         LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(json!({ "users": users })))
}

async fn create(
    _admin: AdminUser,
    State(state): State<AppState>,
    Json(body): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, ApiError> {
    if body.username.is_empty() || body.password.is_empty() {
        return Err(ApiError::BadRequest(
            "username and password are required".to_string(),
        ));
    }

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(body.password.as_bytes(), &salt)
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("failed to hash password: {e}")))?
        .to_string();

    let is_admin = body.is_admin.unwrap_or(false);

    let user = sqlx::query_as::<_, UserResponse>(
        "INSERT INTO users (username, email, password_hash, is_admin)
         VALUES ($1, $2, $3, $4)
         RETURNING id, username, email, is_admin, created_at, updated_at",
    )
    .bind(&body.username)
    .bind(&body.email)
    .bind(&password_hash)
    .bind(is_admin)
    .fetch_one(&state.db)
    .await?;

    Ok((axum::http::StatusCode::CREATED, Json(json!(user))))
}

async fn me(
    auth_user: AuthUser,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let user = sqlx::query_as::<_, UserResponse>(
        "SELECT id, username, email, is_admin, created_at, updated_at
         FROM users WHERE id = $1",
    )
    .bind(auth_user.id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(json!(user)))
}

async fn get_user(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    if auth_user.is_admin {
        let user = sqlx::query_as::<_, UserResponse>(
            "SELECT id, username, email, is_admin, created_at, updated_at
             FROM users WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&state.db)
        .await?;

        Ok(Json(json!(user)))
    } else {
        let user = sqlx::query_as::<_, PublicUserResponse>(
            "SELECT id, username FROM users WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(ApiError::NotFound)?;

        Ok(Json(json!(user)))
    }
}

async fn update(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateUserRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Non-admins can only update themselves
    if !auth_user.is_admin && auth_user.id != id {
        return Err(ApiError::Forbidden);
    }

    // Verify target user exists
    let existing = sqlx::query_as::<_, UserResponse>(
        "SELECT id, username, email, is_admin, created_at, updated_at
         FROM users WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ApiError::NotFound)?;

    let username = body.username.unwrap_or(existing.username);
    let email = body.email.or(existing.email);

    let password_hash = if let Some(ref new_password) = body.password {
        if new_password.is_empty() {
            return Err(ApiError::BadRequest("password cannot be empty".to_string()));
        }
        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default()
            .hash_password(new_password.as_bytes(), &salt)
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("failed to hash password: {e}")))?
            .to_string();
        Some(hash)
    } else {
        None
    };

    let user = if let Some(hash) = password_hash {
        sqlx::query_as::<_, UserResponse>(
            "UPDATE users SET username = $1, email = $2, password_hash = $3, updated_at = NOW()
             WHERE id = $4
             RETURNING id, username, email, is_admin, created_at, updated_at",
        )
        .bind(&username)
        .bind(&email)
        .bind(&hash)
        .bind(id)
        .fetch_one(&state.db)
        .await?
    } else {
        sqlx::query_as::<_, UserResponse>(
            "UPDATE users SET username = $1, email = $2, updated_at = NOW()
             WHERE id = $3
             RETURNING id, username, email, is_admin, created_at, updated_at",
        )
        .bind(&username)
        .bind(&email)
        .bind(id)
        .fetch_one(&state.db)
        .await?
    };

    Ok(Json(json!(user)))
}

async fn delete(
    admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    if admin.0.id == id {
        return Err(ApiError::BadRequest("cannot delete yourself".to_string()));
    }

    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }

    Ok(axum::http::StatusCode::NO_CONTENT)
}
