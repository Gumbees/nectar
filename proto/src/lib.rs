use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Job types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobType {
    LiveTranscode,
    OfflineTranscode,
    Trickplay,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobPriority {
    Low,
    Normal,
    High,
    Urgent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VideoCodec {
    H264,
    H265,
    Av1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioCodec {
    Aac,
    Opus,
    Flac,
    Copy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HdrFormat {
    Hdr10,
    Hdr10Plus,
    DolbyVision,
    Hlg,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HlsPlaylistType {
    Event,
    Vod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobState {
    Accepted,
    Running,
    Completed,
    Failed,
    Cancelled,
}

// Source media info (from ffprobe, sent by server)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInfo {
    pub video_codec: String,
    pub audio_codec: String,
    pub width: u32,
    pub height: u32,
    pub bitrate: u64,
    pub duration_seconds: f64,
    pub is_hdr: bool,
    pub hdr_format: Option<HdrFormat>,
    pub color_space: Option<String>,
    pub color_transfer: Option<String>,
    pub color_primaries: Option<String>,
    pub pixel_format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoParams {
    pub codec: VideoCodec,
    pub resolution: Option<String>,
    pub bitrate: Option<String>,
    pub crf: Option<u8>,
    pub preset: Option<String>,
    pub max_framerate: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioParams {
    pub codec: AudioCodec,
    pub bitrate: String,
    pub channels: Option<u8>,
    pub sample_rate: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HlsParams {
    pub segment_duration: u32,
    pub playlist_type: HlsPlaylistType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrickplayParams {
    pub interval_seconds: u32,
    pub width: u32,
    pub columns: u32,
    pub rows: u32,
}

// Main job request (server -> transcoder via NATS)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscodeRequest {
    pub id: Uuid,
    pub media_item_id: Uuid,
    pub job_type: JobType,
    pub input_path: String,
    pub output_dir: String,
    pub video: VideoParams,
    pub audio: AudioParams,
    pub source_info: SourceInfo,
    pub hls: Option<HlsParams>,
    pub trickplay: Option<TrickplayParams>,
    pub priority: JobPriority,
}

// Progress update (transcoder -> server)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobProgress {
    pub job_id: Uuid,
    pub worker_id: String,
    pub state: JobState,
    pub percent: f32,
    pub fps: Option<f32>,
    pub speed: Option<f32>,
    pub eta_seconds: Option<u64>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

// Worker heartbeat (transcoder -> server)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerHeartbeat {
    pub worker_id: String,
    pub hostname: String,
    pub active_jobs: u32,
    pub max_jobs: u32,
    pub timestamp: DateTime<Utc>,
}
