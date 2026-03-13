use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use axum::{Json, Router, extract::State, routing::{get, post}};
use serde::Deserialize;
use serde_json::{Value, json};

use super::AppState;
use super::errors::ApiError;
use crate::auth;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
        .route("/logout", post(logout))
        .route("/oidc/callback", post(oidc_callback))
        .route("/oidc/authorize", get(oidc_authorize))
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<Value>, ApiError> {
    let user = sqlx::query_as::<_, crate::models::User>(
        "SELECT * FROM users WHERE username = $1"
    )
    .bind(&body.username)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ApiError::Unauthorized)?;

    let password_hash = user.password_hash.as_deref().ok_or(ApiError::BadRequest(
        "account uses OIDC login, not password".to_string(),
    ))?;

    let parsed_hash = PasswordHash::new(password_hash)
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("invalid stored hash: {e}")))?;

    Argon2::default()
        .verify_password(body.password.as_bytes(), &parsed_hash)
        .map_err(|_| ApiError::Unauthorized)?;

    let token = auth::sign_token(user.id, &user.username, user.is_admin, &state.config)
        .map_err(|e| ApiError::Internal(e))?;

    Ok(Json(json!({ "token": token })))
}

#[derive(Debug, Deserialize)]
struct RegisterRequest {
    username: String,
    password: String,
    email: Option<String>,
}

async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> Result<Json<Value>, ApiError> {
    if body.username.is_empty() || body.password.is_empty() {
        return Err(ApiError::BadRequest("username and password are required".to_string()));
    }

    // Hash password with argon2id
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(body.password.as_bytes(), &salt)
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("failed to hash password: {e}")))?
        .to_string();

    // First user auto-becomes admin
    let user_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&state.db)
        .await?;
    let is_admin = user_count.0 == 0;

    let user = sqlx::query_as::<_, crate::models::User>(
        r#"INSERT INTO users (username, email, password_hash, is_admin)
           VALUES ($1, $2, $3, $4)
           RETURNING *"#,
    )
    .bind(&body.username)
    .bind(&body.email)
    .bind(&password_hash)
    .bind(is_admin)
    .fetch_one(&state.db)
    .await?;

    let token = auth::sign_token(user.id, &user.username, user.is_admin, &state.config)
        .map_err(|e| ApiError::Internal(e))?;

    Ok(Json(json!({ "token": token })))
}

async fn logout() -> Json<Value> {
    Json(json!({ "ok": true }))
}

async fn oidc_callback() -> Json<Value> {
    Json(json!({ "todo": "oidc callback" }))
}

async fn oidc_authorize() -> Json<Value> {
    Json(json!({ "todo": "oidc authorize" }))
}
