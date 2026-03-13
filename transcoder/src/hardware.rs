use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareCapabilities {
    pub encoders: Vec<HwEncoder>,
    pub decoders: Vec<HwDecoder>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HwEncoder {
    Nvenc,          // NVIDIA GPU
    Qsv,            // Intel Quick Sync
    Vaapi,          // VA-API (Intel/AMD on Linux)
    Amf,            // AMD AMF (Windows)
    V4l2M2m,        // Orange Pi / ARM SBCs
    Videotoolbox,   // macOS
    Software,       // libx264/libx265 fallback
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HwDecoder {
    Cuvid,          // NVIDIA
    Qsv,            // Intel
    Vaapi,          // VA-API
    V4l2M2m,        // ARM
    Videotoolbox,   // macOS
    Software,
}

/// Probe the system for available hardware acceleration
pub async fn detect() -> Result<HardwareCapabilities> {
    let mut encoders = vec![HwEncoder::Software];
    let mut decoders = vec![HwDecoder::Software];

    // Check NVIDIA
    if probe_command("nvidia-smi").await {
        encoders.push(HwEncoder::Nvenc);
        decoders.push(HwDecoder::Cuvid);
    }

    // Check Intel QSV (via vainfo or ls /dev/dri)
    if probe_command("vainfo").await {
        encoders.push(HwEncoder::Vaapi);
        decoders.push(HwDecoder::Vaapi);
        // QSV is a subset of VA-API on Intel
        encoders.push(HwEncoder::Qsv);
        decoders.push(HwDecoder::Qsv);
    }

    // Check V4L2 M2M (ARM SBCs)
    if tokio::fs::metadata("/dev/video0").await.is_ok() {
        encoders.push(HwEncoder::V4l2M2m);
        decoders.push(HwDecoder::V4l2M2m);
    }

    Ok(HardwareCapabilities { encoders, decoders })
}

async fn probe_command(cmd: &str) -> bool {
    tokio::process::Command::new(cmd)
        .arg("--version")
        .output()
        .await
        .is_ok_and(|o| o.status.success())
}
