use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use uuid::Uuid;

use super::AppState;
use super::errors::ApiError;
use crate::auth;

/// Authenticated user extracted from a Bearer token in the Authorization header.
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub username: String,
    pub is_admin: bool,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(ApiError::Unauthorized)?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(ApiError::Unauthorized)?;

        let claims = auth::verify_token(token, &state.config)
            .map_err(|_| ApiError::Unauthorized)?;

        let user_id = claims
            .sub
            .parse::<Uuid>()
            .map_err(|_| ApiError::Unauthorized)?;

        Ok(AuthUser {
            id: user_id,
            username: claims.username,
            is_admin: claims.is_admin,
        })
    }
}

/// Admin user extractor. Returns 403 if the authenticated user is not an admin.
#[derive(Debug, Clone)]
pub struct AdminUser(pub AuthUser);

impl FromRequestParts<AppState> for AdminUser {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let user = AuthUser::from_request_parts(parts, state).await?;

        if !user.is_admin {
            return Err(ApiError::Forbidden);
        }

        Ok(AdminUser(user))
    }
}
