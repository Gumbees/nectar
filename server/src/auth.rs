use anyhow::Result;
use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::Config;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// User UUID as string
    pub sub: String,
    pub username: String,
    pub is_admin: bool,
    /// Expiration (UTC timestamp)
    pub exp: usize,
    /// Issued at (UTC timestamp)
    pub iat: usize,
}

/// Sign a JWT for the given user.
pub fn sign_token(user_id: Uuid, username: &str, is_admin: bool, config: &Config) -> Result<String> {
    let now = Utc::now().timestamp() as usize;
    let expiry = now + (config.jwt_expiry_hours * 3600) as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        username: username.to_string(),
        is_admin,
        exp: expiry,
        iat: now,
    };

    let token = jsonwebtoken::encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )?;

    Ok(token)
}

/// Verify and decode a JWT, returning the claims.
pub fn verify_token(token: &str, config: &Config) -> Result<Claims> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_required_spec_claims(&["exp", "iat", "sub"]);

    let token_data = jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &validation,
    )?;

    Ok(token_data.claims)
}
