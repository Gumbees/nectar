use nectar_proto::{VideoCodec, VideoParams};

use crate::hardware::{HardwareCapabilities, HwEncoder};

/// Select the best available encoder from hardware capabilities
pub fn select_encoder(hw: &HardwareCapabilities) -> HwEncoder {
    let preference = [
        HwEncoder::Nvenc,
        HwEncoder::Qsv,
        HwEncoder::Vaapi,
        HwEncoder::Amf,
        HwEncoder::V4l2M2m,
    ];

    for pref in &preference {
        if hw.encoders.contains(pref) {
            return pref.clone();
        }
    }

    HwEncoder::Software
}

/// Return hwaccel input args for the chosen encoder
pub fn hwaccel_input_args(encoder: &HwEncoder, vaapi_device: &str) -> Vec<String> {
    match encoder {
        HwEncoder::Nvenc => vec![
            "-hwaccel".into(),
            "cuda".into(),
            "-hwaccel_output_format".into(),
            "cuda".into(),
        ],
        HwEncoder::Qsv => vec![
            "-hwaccel".into(),
            "qsv".into(),
            "-hwaccel_output_format".into(),
            "qsv".into(),
        ],
        HwEncoder::Vaapi => vec![
            "-hwaccel".into(),
            "vaapi".into(),
            "-hwaccel_device".into(),
            vaapi_device.into(),
            "-hwaccel_output_format".into(),
            "vaapi".into(),
        ],
        HwEncoder::Amf => vec!["-hwaccel".into(), "d3d11va".into()],
        _ => vec![],
    }
}

/// Return video encoder args for h264 on the given hardware
pub fn h264_encoder_args(encoder: &HwEncoder, video: &VideoParams) -> Vec<String> {
    let mut args = Vec::new();

    match encoder {
        HwEncoder::Nvenc => {
            args.extend(["-c:v".into(), "h264_nvenc".into()]);
            args.extend([
                "-preset".into(),
                video.preset.as_deref().unwrap_or("p4").into(),
            ]);
            if let Some(ref bitrate) = video.bitrate {
                args.extend(["-b:v".into(), bitrate.clone()]);
            } else if let Some(crf) = video.crf {
                args.extend(["-cq".into(), crf.to_string()]);
                args.extend(["-rc".into(), "vbr".into()]);
            }
        }
        HwEncoder::Qsv => {
            args.extend(["-c:v".into(), "h264_qsv".into()]);
            args.extend([
                "-preset".into(),
                video.preset.as_deref().unwrap_or("medium").into(),
            ]);
            if let Some(ref bitrate) = video.bitrate {
                args.extend(["-b:v".into(), bitrate.clone()]);
            } else if let Some(crf) = video.crf {
                args.extend(["-global_quality".into(), crf.to_string()]);
            }
        }
        HwEncoder::Vaapi => {
            args.extend(["-c:v".into(), "h264_vaapi".into()]);
            if let Some(ref bitrate) = video.bitrate {
                args.extend(["-b:v".into(), bitrate.clone()]);
            } else if let Some(crf) = video.crf {
                args.extend(["-qp".into(), crf.to_string()]);
            }
        }
        HwEncoder::Amf => {
            args.extend(["-c:v".into(), "h264_amf".into()]);
            args.extend(["-quality".into(), "balanced".into()]);
            if let Some(ref bitrate) = video.bitrate {
                args.extend(["-b:v".into(), bitrate.clone()]);
            } else if let Some(crf) = video.crf {
                args.extend(["-qp_i".into(), crf.to_string()]);
                args.extend(["-qp_p".into(), crf.to_string()]);
            }
        }
        HwEncoder::V4l2M2m => {
            args.extend(["-c:v".into(), "h264_v4l2m2m".into()]);
            if let Some(ref bitrate) = video.bitrate {
                args.extend(["-b:v".into(), bitrate.clone()]);
            } else {
                args.extend(["-b:v".into(), "4M".into()]);
            }
        }
        _ => {
            args.extend(["-c:v".into(), "libx264".into()]);
            args.extend([
                "-preset".into(),
                video.preset.as_deref().unwrap_or("medium").into(),
            ]);
            if let Some(ref bitrate) = video.bitrate {
                args.extend(["-b:v".into(), bitrate.clone()]);
            } else {
                let crf = video.crf.unwrap_or(23);
                args.extend(["-crf".into(), crf.to_string()]);
            }
        }
    }

    args
}

/// Return video encoder args for h265/HEVC on the given hardware
pub fn h265_encoder_args(
    encoder: &HwEncoder,
    hw: &HardwareCapabilities,
    video: &VideoParams,
) -> Vec<String> {
    let mut args = Vec::new();

    match encoder {
        HwEncoder::Nvenc if hw.supports_hevc => {
            args.extend(["-c:v".into(), "hevc_nvenc".into()]);
            args.extend([
                "-preset".into(),
                video.preset.as_deref().unwrap_or("p4").into(),
            ]);
            if let Some(ref bitrate) = video.bitrate {
                args.extend(["-b:v".into(), bitrate.clone()]);
            } else if let Some(crf) = video.crf {
                args.extend(["-cq".into(), crf.to_string()]);
                args.extend(["-rc".into(), "vbr".into()]);
            }
        }
        HwEncoder::Qsv if hw.supports_hevc => {
            args.extend(["-c:v".into(), "hevc_qsv".into()]);
            args.extend([
                "-preset".into(),
                video.preset.as_deref().unwrap_or("medium").into(),
            ]);
            if let Some(ref bitrate) = video.bitrate {
                args.extend(["-b:v".into(), bitrate.clone()]);
            } else if let Some(crf) = video.crf {
                args.extend(["-global_quality".into(), crf.to_string()]);
            }
        }
        HwEncoder::Vaapi if hw.supports_hevc => {
            args.extend(["-c:v".into(), "hevc_vaapi".into()]);
            if let Some(ref bitrate) = video.bitrate {
                args.extend(["-b:v".into(), bitrate.clone()]);
            } else if let Some(crf) = video.crf {
                args.extend(["-qp".into(), crf.to_string()]);
            }
        }
        HwEncoder::Amf if hw.supports_hevc => {
            args.extend(["-c:v".into(), "hevc_amf".into()]);
            args.extend(["-quality".into(), "balanced".into()]);
            if let Some(ref bitrate) = video.bitrate {
                args.extend(["-b:v".into(), bitrate.clone()]);
            }
        }
        _ => {
            // Software fallback
            args.extend(["-c:v".into(), "libx265".into()]);
            args.extend([
                "-preset".into(),
                video.preset.as_deref().unwrap_or("medium").into(),
            ]);
            if let Some(ref bitrate) = video.bitrate {
                args.extend(["-b:v".into(), bitrate.clone()]);
            } else {
                let crf = video.crf.unwrap_or(28);
                args.extend(["-crf".into(), crf.to_string()]);
            }
        }
    }

    args
}

/// Return video encoder args for AV1 on the given hardware
pub fn av1_encoder_args(
    encoder: &HwEncoder,
    hw: &HardwareCapabilities,
    video: &VideoParams,
) -> Vec<String> {
    let mut args = Vec::new();

    match encoder {
        HwEncoder::Nvenc if hw.supports_av1 => {
            args.extend(["-c:v".into(), "av1_nvenc".into()]);
            args.extend([
                "-preset".into(),
                video.preset.as_deref().unwrap_or("p4").into(),
            ]);
            if let Some(ref bitrate) = video.bitrate {
                args.extend(["-b:v".into(), bitrate.clone()]);
            } else if let Some(crf) = video.crf {
                args.extend(["-cq".into(), crf.to_string()]);
                args.extend(["-rc".into(), "vbr".into()]);
            }
        }
        HwEncoder::Qsv if hw.supports_av1 => {
            args.extend(["-c:v".into(), "av1_qsv".into()]);
            if let Some(ref bitrate) = video.bitrate {
                args.extend(["-b:v".into(), bitrate.clone()]);
            } else if let Some(crf) = video.crf {
                args.extend(["-global_quality".into(), crf.to_string()]);
            }
        }
        HwEncoder::Vaapi if hw.supports_av1 => {
            args.extend(["-c:v".into(), "av1_vaapi".into()]);
            if let Some(ref bitrate) = video.bitrate {
                args.extend(["-b:v".into(), bitrate.clone()]);
            } else if let Some(crf) = video.crf {
                args.extend(["-qp".into(), crf.to_string()]);
            }
        }
        _ => {
            // Software fallback: SVT-AV1
            args.extend(["-c:v".into(), "libsvtav1".into()]);
            args.extend([
                "-preset".into(),
                video.preset.as_deref().unwrap_or("6").into(),
            ]);
            if let Some(ref bitrate) = video.bitrate {
                args.extend(["-b:v".into(), bitrate.clone()]);
            } else {
                let crf = video.crf.unwrap_or(30);
                args.extend(["-crf".into(), crf.to_string()]);
            }
        }
    }

    args
}

/// Select and return the appropriate encoder args based on the requested codec
pub fn video_encoder_args(
    encoder: &HwEncoder,
    hw: &HardwareCapabilities,
    video: &VideoParams,
) -> Vec<String> {
    match video.codec {
        VideoCodec::H264 => h264_encoder_args(encoder, video),
        VideoCodec::H265 => h265_encoder_args(encoder, hw, video),
        VideoCodec::Av1 => av1_encoder_args(encoder, hw, video),
    }
}

/// Return audio encoder args
pub fn audio_encoder_args(audio: &nectar_proto::AudioParams) -> Vec<String> {
    let mut args = Vec::new();

    match audio.codec {
        nectar_proto::AudioCodec::Aac => {
            args.extend(["-c:a".into(), "aac".into()]);
            args.extend(["-b:a".into(), audio.bitrate.clone()]);
        }
        nectar_proto::AudioCodec::Opus => {
            args.extend(["-c:a".into(), "libopus".into()]);
            args.extend(["-b:a".into(), audio.bitrate.clone()]);
        }
        nectar_proto::AudioCodec::Flac => {
            args.extend(["-c:a".into(), "flac".into()]);
        }
        nectar_proto::AudioCodec::Copy => {
            args.extend(["-c:a".into(), "copy".into()]);
        }
    }

    if let Some(channels) = audio.channels {
        args.extend(["-ac".into(), channels.to_string()]);
    }
    if let Some(sample_rate) = audio.sample_rate {
        args.extend(["-ar".into(), sample_rate.to_string()]);
    }

    args
}
