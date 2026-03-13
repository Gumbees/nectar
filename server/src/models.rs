use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ── Enums (existing) ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "library_type", rename_all = "lowercase")]
pub enum LibraryType {
    Movies,
    Shows,
    Music,
    Books,
    Photos,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
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

// ── Enums (new) ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "stream_type", rename_all = "lowercase")]
pub enum StreamType {
    Video,
    Audio,
    Subtitle,
    Attachment,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "image_type", rename_all = "lowercase")]
pub enum ImageType {
    Primary,
    Backdrop,
    Banner,
    Logo,
    Thumb,
    Disc,
    Clearart,
    Landscape,
    Trickplay,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "job_status", rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "scan_status", rename_all = "lowercase")]
pub enum ScanStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "external_provider", rename_all = "lowercase")]
pub enum ExternalProvider {
    Tmdb,
    Imdb,
    Tvdb,
    Musicbrainz,
    Discogs,
    Anidb,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "person_role", rename_all = "snake_case")]
pub enum PersonRole {
    Actor,
    Director,
    Writer,
    Producer,
    Composer,
    Creator,
    GuestStar,
    Artist,
}

// ── Core tables ───────────────────────────────────────────────────────────────

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

/// User without sensitive fields, safe for API responses.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            username: u.username,
            email: u.email,
            is_admin: u.is_admin,
            created_at: u.created_at,
            updated_at: u.updated_at,
        }
    }
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
    // Added in 008
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

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PlaybackProgress {
    pub id: Uuid,
    pub user_id: Uuid,
    pub media_item_id: Uuid,
    pub position_seconds: i64,
    pub completed: bool,
    pub updated_at: DateTime<Utc>,
}

// ── Media streams ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MediaStream {
    pub id: Uuid,
    pub media_item_id: Uuid,
    pub stream_index: i32,
    pub stream_type: StreamType,
    pub codec: Option<String>,
    pub codec_long: Option<String>,
    pub profile: Option<String>,
    pub language: Option<String>,
    pub title: Option<String>,
    pub is_default: bool,
    pub is_forced: bool,
    pub is_external: bool,
    pub external_path: Option<String>,
    // Video
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub bitrate: Option<i64>,
    pub framerate: Option<f32>,
    pub pixel_format: Option<String>,
    pub color_space: Option<String>,
    pub color_transfer: Option<String>,
    pub color_primaries: Option<String>,
    pub hdr_format: Option<String>,
    // Audio
    pub channels: Option<i32>,
    pub channel_layout: Option<String>,
    pub sample_rate: Option<i32>,
    // Subtitle
    pub subtitle_format: Option<String>,
}

// ── Images ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Image {
    pub id: Uuid,
    pub media_item_id: Option<Uuid>,
    pub library_id: Option<Uuid>,
    pub image_type: ImageType,
    pub path: String,
    pub source_url: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub format: Option<String>,
    pub blurhash: Option<String>,
    pub language: Option<String>,
    // Trickplay
    pub tile_width: Option<i32>,
    pub tile_height: Option<i32>,
    pub tiles_per_row: Option<i32>,
    pub interval_ms: Option<i32>,
}

// ── People, genres, studios, tags ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Person {
    pub id: Uuid,
    pub name: String,
    pub sort_name: Option<String>,
    pub thumb_path: Option<String>,
    pub thumb_url: Option<String>,
    pub tmdb_id: Option<i32>,
    pub imdb_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MediaPerson {
    pub media_item_id: Uuid,
    pub person_id: Uuid,
    pub role: PersonRole,
    pub character_name: Option<String>,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Genre {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Studio {
    pub id: Uuid,
    pub name: String,
    pub logo_path: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
}

// ── External IDs ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExternalId {
    pub media_item_id: Uuid,
    pub provider: ExternalProvider,
    pub external_id: String,
}

// ── Auth & sessions ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub device_name: Option<String>,
    // INET maps to String in sqlx postgres
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_used_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ApiKey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub key_hash: String,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserPreferences {
    pub user_id: Uuid,
    pub preferred_audio_language: Option<String>,
    pub preferred_subtitle_language: Option<String>,
    pub subtitle_mode: Option<String>,
    pub max_streaming_bitrate: Option<i32>,
    pub transcode_preference: Option<String>,
    pub theme: Option<String>,
    pub home_sections: Option<serde_json::Value>,
    pub updated_at: DateTime<Utc>,
}

// ── Watch history & collections ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WatchHistoryEntry {
    pub id: Uuid,
    pub user_id: Uuid,
    pub media_item_id: Uuid,
    pub watched_at: DateTime<Utc>,
    pub duration_seconds: Option<i32>,
    pub play_method: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Collection {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub is_smart: bool,
    pub filter_rules: Option<serde_json::Value>,
    pub sort_order: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CollectionItem {
    pub collection_id: Uuid,
    pub media_item_id: Uuid,
    pub sort_index: i32,
    pub added_at: DateTime<Utc>,
}

// ── Jobs & scans ──────────────────────────────────────────────────────────────

/// Database-backed transcode job record. Distinct from `nectar_proto::TranscodeRequest`
/// which is the NATS message sent to workers.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TranscodeJobRecord {
    pub id: Uuid,
    pub media_item_id: Uuid,
    pub user_id: Option<Uuid>,
    pub status: JobStatus,
    pub input_path: String,
    pub output_path: Option<String>,
    pub output_format: Option<String>,
    pub target_resolution: Option<String>,
    pub target_bitrate: Option<i64>,
    pub hardware_accel: Option<String>,
    pub worker_id: Option<String>,
    pub progress_percent: Option<f32>,
    pub error_message: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LibraryScan {
    pub id: Uuid,
    pub library_id: Uuid,
    pub status: ScanStatus,
    pub items_found: Option<i32>,
    pub items_added: Option<i32>,
    pub items_updated: Option<i32>,
    pub items_removed: Option<i32>,
    pub error_message: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}
