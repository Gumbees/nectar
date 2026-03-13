use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareCapabilities {
    pub encoders: Vec<HwEncoder>,
    pub decoders: Vec<HwDecoder>,
    pub gpu_name: Option<String>,
    pub vram_mb: Option<u32>,
    pub max_encode_sessions: u32,
    pub supports_hevc: bool,
    pub supports_av1: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HwEncoder {
    Nvenc,
    Qsv,
    Vaapi,
    Amf,
    V4l2M2m,
    Videotoolbox,
    Software,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HwDecoder {
    Cuvid,
    Qsv,
    Vaapi,
    V4l2M2m,
    Videotoolbox,
    Software,
}

/// Probe the system for available hardware acceleration
pub async fn detect() -> Result<HardwareCapabilities> {
    let mut encoders = vec![HwEncoder::Software];
    let mut decoders = vec![HwDecoder::Software];
    let mut gpu_name: Option<String> = None;
    let mut vram_mb: Option<u32> = None;
    let mut max_encode_sessions: u32 = 2;
    let mut supports_hevc = false;
    let mut supports_av1 = false;

    // Query available ffmpeg encoders once
    let available_encoders = query_ffmpeg_encoders().await;

    // Check NVIDIA via nvidia-smi CSV output
    if let Some(nvidia_info) = probe_nvidia().await {
        gpu_name = Some(nvidia_info.name);
        vram_mb = Some(nvidia_info.vram_mb);
        // Consumer GPUs typically support 3-5 simultaneous NVENC sessions
        max_encode_sessions = if nvidia_info.vram_mb >= 8192 { 5 } else { 3 };

        if available_encoders.contains(&"h264_nvenc".to_string()) {
            encoders.push(HwEncoder::Nvenc);
            decoders.push(HwDecoder::Cuvid);
        }
        if available_encoders.contains(&"hevc_nvenc".to_string()) {
            supports_hevc = true;
        }
        if available_encoders.contains(&"av1_nvenc".to_string()) {
            supports_av1 = true;
        }
    }

    // Check Intel QSV / VA-API
    if probe_command("vainfo").await {
        if available_encoders.contains(&"h264_vaapi".to_string()) {
            encoders.push(HwEncoder::Vaapi);
            decoders.push(HwDecoder::Vaapi);
        }
        if available_encoders.contains(&"h264_qsv".to_string()) {
            encoders.push(HwEncoder::Qsv);
            decoders.push(HwDecoder::Qsv);
        }
        if available_encoders.contains(&"hevc_vaapi".to_string())
            || available_encoders.contains(&"hevc_qsv".to_string())
        {
            supports_hevc = true;
        }
        if available_encoders.contains(&"av1_vaapi".to_string())
            || available_encoders.contains(&"av1_qsv".to_string())
        {
            supports_av1 = true;
        }
    }

    // Check AMD AMF (Windows)
    if available_encoders.contains(&"h264_amf".to_string()) {
        encoders.push(HwEncoder::Amf);
        if available_encoders.contains(&"hevc_amf".to_string()) {
            supports_hevc = true;
        }
        if available_encoders.contains(&"av1_amf".to_string()) {
            supports_av1 = true;
        }
    }

    // Check V4L2 M2M (ARM SBCs) — look for M2M devices specifically
    if probe_v4l2_m2m().await {
        if available_encoders.contains(&"h264_v4l2m2m".to_string()) {
            encoders.push(HwEncoder::V4l2M2m);
            decoders.push(HwDecoder::V4l2M2m);
        }
    }

    // Software HEVC/AV1 fallback checks
    if !supports_hevc && available_encoders.contains(&"libx265".to_string()) {
        supports_hevc = true;
    }
    if !supports_av1 && available_encoders.contains(&"libsvtav1".to_string()) {
        supports_av1 = true;
    }

    Ok(HardwareCapabilities {
        encoders,
        decoders,
        gpu_name,
        vram_mb,
        max_encode_sessions,
        supports_hevc,
        supports_av1,
    })
}

struct NvidiaInfo {
    name: String,
    vram_mb: u32,
}

async fn probe_nvidia() -> Option<NvidiaInfo> {
    let output = tokio::process::Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,memory.total",
            "--format=csv,noheader,nounits",
        ])
        .output()
        .await
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.lines().next()?;
    let mut parts = line.splitn(2, ',');
    let name = parts.next()?.trim().to_string();
    let vram_str = parts.next()?.trim();
    let vram_mb = vram_str.parse::<f64>().ok()? as u32;

    Some(NvidiaInfo { name, vram_mb })
}

/// Query ffmpeg for all available encoders
async fn query_ffmpeg_encoders() -> Vec<String> {
    let output = tokio::process::Command::new("ffmpeg")
        .args(["-hide_banner", "-encoders"])
        .output()
        .await;

    match output {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            stdout
                .lines()
                .filter_map(|line| {
                    // Encoder lines look like: " V..... h264_nvenc ..."
                    let trimmed = line.trim();
                    if trimmed.len() > 8 && trimmed.chars().nth(1) == Some('.') {
                        // Format: FLAGS name description
                        let after_flags = &trimmed[7..];
                        after_flags.split_whitespace().next().map(|s| s.to_string())
                    } else {
                        None
                    }
                })
                .collect()
        }
        _ => Vec::new(),
    }
}

/// Check for V4L2 M2M devices specifically (not just any /dev/video*)
async fn probe_v4l2_m2m() -> bool {
    // Check if any /dev/video* devices exist and are M2M type
    let output = tokio::process::Command::new("v4l2-ctl")
        .args(["--list-devices"])
        .output()
        .await;

    if let Ok(o) = output {
        if o.status.success() {
            let stdout = String::from_utf8_lossy(&o.stdout);
            return stdout.contains("m2m") || stdout.contains("M2M") || stdout.contains("codec");
        }
    }

    // Fallback: check for common M2M device nodes
    for i in 0..4 {
        let path = format!("/dev/video{i}");
        if let Ok(output) = tokio::process::Command::new("v4l2-ctl")
            .args(["-d", &path, "--all"])
            .output()
            .await
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.contains("m2m") || stdout.contains("Video Memory-To-Memory") {
                    return true;
                }
            }
        }
    }

    false
}

async fn probe_command(cmd: &str) -> bool {
    tokio::process::Command::new(cmd)
        .arg("--version")
        .output()
        .await
        .is_ok_and(|o| o.status.success())
}
