use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub password_hash: Option<String>,
    pub oidc_subject: Option<String>,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Library {
    pub id: Uuid,
    pub name: String,
    pub library_type: LibraryType,
    pub paths: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "library_type", rename_all = "lowercase")]
pub enum LibraryType {
    Movies,
    Shows,
    Music,
    Books,
    Photos,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MediaItem {
    pub id: Uuid,
    pub library_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub item_type: MediaItemType,
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
    pub embedding: Option<Vec<f32>>,
    pub metadata_json: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "media_item_type", rename_all = "lowercase")]
pub enum MediaItemType {
    Movie,
    Series,
    Season,
    Episode,
    Album,
    Track,
    Book,
    Photo,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PlaybackProgress {
    pub id: Uuid,
    pub user_id: Uuid,
    pub media_item_id: Uuid,
    pub position_seconds: i64,
    pub completed: bool,
    pub updated_at: DateTime<Utc>,
}
