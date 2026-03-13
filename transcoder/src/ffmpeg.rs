use anyhow::Result;

use crate::config::Config;
use crate::hardware::{HardwareCapabilities, HwEncoder};

/// Build and execute an FFmpeg command for the given job
pub async fn run_transcode(
    job: &serde_json::Value,
    hw: &HardwareCapabilities,
    config: &Config,
) -> Result<()> {
    let input = job["input_path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("missing input_path"))?;

    let output_dir = format!("{}/{}", config.work_dir, job["id"].as_str().unwrap_or("out"));
    tokio::fs::create_dir_all(&output_dir).await?;

    let mut cmd = tokio::process::Command::new(&config.ffmpeg_path);
    cmd.arg("-y");

    // Select best available hardware acceleration
    match select_encoder(hw) {
        HwEncoder::Nvenc => {
            cmd.args(["-hwaccel", "cuda", "-hwaccel_output_format", "cuda"]);
            cmd.args(["-i", input]);
            cmd.args(["-c:v", "h264_nvenc", "-preset", "p4", "-b:v", "8M"]);
        }
        HwEncoder::Qsv => {
            cmd.args(["-hwaccel", "qsv", "-hwaccel_output_format", "qsv"]);
            cmd.args(["-i", input]);
            cmd.args(["-c:v", "h264_qsv", "-preset", "medium", "-b:v", "8M"]);
        }
        HwEncoder::Vaapi => {
            cmd.args(["-hwaccel", "vaapi", "-hwaccel_device", "/dev/dri/renderD128"]);
            cmd.args(["-hwaccel_output_format", "vaapi"]);
            cmd.args(["-i", input]);
            cmd.args(["-c:v", "h264_vaapi", "-b:v", "8M"]);
        }
        HwEncoder::V4l2M2m => {
            cmd.args(["-i", input]);
            cmd.args(["-c:v", "h264_v4l2m2m", "-b:v", "4M"]);
        }
        HwEncoder::Amf => {
            cmd.args(["-hwaccel", "d3d11va"]);
            cmd.args(["-i", input]);
            cmd.args(["-c:v", "h264_amf", "-quality", "balanced", "-b:v", "8M"]);
        }
        _ => {
            cmd.args(["-i", input]);
            cmd.args(["-c:v", "libx264", "-preset", "medium", "-crf", "23"]);
        }
    }

    cmd.args(["-c:a", "aac", "-b:a", "192k"]);

    // HLS output
    let output_path = format!("{output_dir}/stream.m3u8");
    cmd.args([
        "-f", "hls",
        "-hls_time", "6",
        "-hls_list_size", "0",
        "-hls_segment_filename", &format!("{output_dir}/segment_%03d.ts"),
        &output_path,
    ]);

    tracing::debug!(cmd = ?cmd, "running ffmpeg");

    let status = cmd.status().await?;
    if !status.success() {
        anyhow::bail!("ffmpeg exited with status {status}");
    }

    Ok(())
}

fn select_encoder(hw: &HardwareCapabilities) -> HwEncoder {
    // Prefer dedicated GPU encoders, fall back to software
    let preference = [
        HwEncoder::Nvenc,
        HwEncoder::Qsv,
        HwEncoder::Vaapi,
        HwEncoder::Amf,
        HwEncoder::V4l2M2m,
    ];

    for pref in &preference {
        if hw.encoders.iter().any(|e| std::mem::discriminant(e) == std::mem::discriminant(pref)) {
            return pref.clone();
        }
    }

    HwEncoder::Software
}
