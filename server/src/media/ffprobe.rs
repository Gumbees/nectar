use anyhow::Result;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct FfprobeOutput {
    streams: Vec<FfprobeStream>,
    format: FfprobeFormat,
}

#[derive(Debug, Deserialize)]
struct FfprobeStream {
    #[allow(dead_code)]
    index: i32,
    codec_type: String,
    codec_name: Option<String>,
    #[allow(dead_code)]
    codec_long_name: Option<String>,
    #[allow(dead_code)]
    profile: Option<String>,
    width: Option<i32>,
    height: Option<i32>,
    bit_rate: Option<String>,
    channels: Option<i32>,
    #[allow(dead_code)]
    channel_layout: Option<String>,
    #[allow(dead_code)]
    sample_rate: Option<String>,
    #[allow(dead_code)]
    tags: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
struct FfprobeFormat {
    duration: Option<String>,
    bit_rate: Option<String>,
    format_name: Option<String>,
    #[allow(dead_code)]
    size: Option<String>,
}

#[derive(Debug)]
pub struct ProbeResult {
    pub duration_seconds: Option<f64>,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub bitrate: Option<i64>,
    pub container: Option<String>,
    pub channels: Option<i32>,
}

pub async fn probe(path: &Path) -> Result<ProbeResult> {
    let output = tokio::process::Command::new("ffprobe")
        .args([
            "-v", "quiet",
            "-print_format", "json",
            "-show_format",
            "-show_streams",
        ])
        .arg(path)
        .output()
        .await?;

    if !output.status.success() {
        anyhow::bail!(
            "ffprobe failed for '{}': {}",
            path.display(),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let data: FfprobeOutput = serde_json::from_slice(&output.stdout)?;

    // Find primary video stream (first video stream)
    let video_stream = data
        .streams
        .iter()
        .find(|s| s.codec_type == "video");

    // Find primary audio stream (first audio stream)
    let audio_stream = data
        .streams
        .iter()
        .find(|s| s.codec_type == "audio");

    let video_codec = video_stream.and_then(|s| s.codec_name.clone());
    let audio_codec = audio_stream.and_then(|s| s.codec_name.clone());
    let width = video_stream.and_then(|s| s.width);
    let height = video_stream.and_then(|s| s.height);
    let channels = audio_stream.and_then(|s| s.channels);

    // Parse duration from format
    let duration_seconds = data
        .format
        .duration
        .as_deref()
        .and_then(|d| d.parse::<f64>().ok());

    // Parse bitrate: prefer format-level, fall back to video stream
    let bitrate = data
        .format
        .bit_rate
        .as_deref()
        .and_then(|b| b.parse::<i64>().ok())
        .or_else(|| {
            video_stream
                .and_then(|s| s.bit_rate.as_deref())
                .and_then(|b| b.parse::<i64>().ok())
        });

    let container = data.format.format_name.clone();

    Ok(ProbeResult {
        duration_seconds,
        video_codec,
        audio_codec,
        width,
        height,
        bitrate,
        container,
        channels,
    })
}

/// Map a resolution (width x height) to a common name.
pub fn resolution_string(width: i32, height: i32) -> String {
    if width >= 7680 || height >= 4320 {
        "8K".to_string()
    } else if width >= 3840 || height >= 2160 {
        "4K".to_string()
    } else if width >= 2560 || height >= 1440 {
        "1440p".to_string()
    } else if width >= 1920 || height >= 1080 {
        "1080p".to_string()
    } else if width >= 1280 || height >= 720 {
        "720p".to_string()
    } else if width >= 854 || height >= 480 {
        "480p".to_string()
    } else {
        format!("{height}p")
    }
}
