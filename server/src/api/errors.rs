use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

#[derive(Debug)]
pub enum ApiError {
    NotFound,
    Unauthorized,
    Forbidden,
    BadRequest(String),
    Conflict(String),
    Internal(anyhow::Error),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "not found"),
            Self::Unauthorized => write!(f, "unauthorized"),
            Self::Forbidden => write!(f, "forbidden"),
            Self::BadRequest(msg) => write!(f, "bad request: {msg}"),
            Self::Conflict(msg) => write!(f, "conflict: {msg}"),
            Self::Internal(err) => write!(f, "internal error: {err}"),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            Self::NotFound => (StatusCode::NOT_FOUND, "not found".to_string()),
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized".to_string()),
            Self::Forbidden => (StatusCode::FORBIDDEN, "forbidden".to_string()),
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            Self::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            Self::Internal(err) => {
                tracing::error!("internal server error: {err:?}");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal server error".to_string())
            }
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        match &err {
            sqlx::Error::RowNotFound => Self::NotFound,
            sqlx::Error::Database(db_err) => {
                // Unique constraint violation
                if db_err.code().as_deref() == Some("23505") {
                    Self::Conflict(db_err.message().to_string())
                } else {
                    Self::Internal(err.into())
                }
            }
            _ => Self::Internal(err.into()),
        }
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        Self::Internal(err)
    }
}
