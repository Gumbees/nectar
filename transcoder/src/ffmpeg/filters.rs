use nectar_proto::{SourceInfo, TrickplayParams, VideoParams};

use crate::hardware::HwEncoder;

/// Build scale filter args per hardware type
pub fn scale_filter(
    encoder: &HwEncoder,
    video: &VideoParams,
    source: &SourceInfo,
) -> Option<String> {
    let resolution = video.resolution.as_deref()?;

    // Parse target resolution (e.g. "1920x1080" or "1280x720")
    let (target_w, target_h) = parse_resolution(resolution)?;

    // Skip scaling if source is already at or below target
    if source.width <= target_w && source.height <= target_h {
        return None;
    }

    let filter = match encoder {
        HwEncoder::Nvenc => {
            format!("scale_cuda={target_w}:{target_h}:interp_algo=lanczos")
        }
        HwEncoder::Qsv => {
            format!("scale_qsv=w={target_w}:h={target_h}")
        }
        HwEncoder::Vaapi => {
            format!("scale_vaapi=w={target_w}:h={target_h}")
        }
        _ => {
            // Software scale
            format!("scale={target_w}:{target_h}:flags=lanczos")
        }
    };

    Some(filter)
}

/// Build HDR to SDR tonemap filter chain per hardware type
pub fn hdr_tonemap_filter(encoder: &HwEncoder, source: &SourceInfo) -> Option<Vec<String>> {
    if !source.is_hdr {
        return None;
    }

    let filters = match encoder {
        HwEncoder::Nvenc => {
            // CUDA tonemap: download from GPU, tonemap in software, upload back
            vec![
                "hwdownload".into(),
                "format=nv12".into(),
                "zscale=t=linear:npl=100,format=gbrpf32le,zscale=p=bt709,tonemap=tonemap=hable:desat=0,zscale=t=bt709:m=bt709:r=tv,format=yuv420p".into(),
                "hwupload_cuda".into(),
            ]
        }
        HwEncoder::Vaapi => {
            vec![
                "hwdownload".into(),
                "format=nv12".into(),
                "zscale=t=linear:npl=100,format=gbrpf32le,zscale=p=bt709,tonemap=tonemap=hable:desat=0,zscale=t=bt709:m=bt709:r=tv,format=nv12".into(),
                "hwupload".into(),
            ]
        }
        HwEncoder::Qsv => {
            vec![
                "hwdownload".into(),
                "format=nv12".into(),
                "zscale=t=linear:npl=100,format=gbrpf32le,zscale=p=bt709,tonemap=tonemap=hable:desat=0,zscale=t=bt709:m=bt709:r=tv,format=nv12".into(),
                "hwupload=extra_hw_frames=64".into(),
            ]
        }
        _ => {
            // Full software tonemap pipeline
            vec![
                "zscale=t=linear:npl=100,format=gbrpf32le,zscale=p=bt709,tonemap=tonemap=hable:desat=0,zscale=t=bt709:m=bt709:r=tv,format=yuv420p".into(),
            ]
        }
    };

    Some(filters)
}

/// Build trickplay (thumbnail sprite) filter chain
pub fn trickplay_filter(params: &TrickplayParams) -> String {
    // fps=1/{interval}, scale to width:-1, tile into grid
    format!(
        "fps=1/{interval},scale={width}:-1,tile={cols}x{rows}",
        interval = params.interval_seconds,
        width = params.width,
        cols = params.columns,
        rows = params.rows,
    )
}

/// Build framerate filter if max_framerate is set
pub fn framerate_filter(video: &VideoParams) -> Option<String> {
    video.max_framerate.map(|fps| format!("fps={fps}"))
}

/// Combine all video filters into a single -vf or -filter_complex string
pub fn build_video_filter_chain(
    encoder: &HwEncoder,
    video: &VideoParams,
    source: &SourceInfo,
) -> Option<Vec<String>> {
    let mut filters: Vec<String> = Vec::new();

    // HDR tonemap (must come first)
    if let Some(tonemap) = hdr_tonemap_filter(encoder, source) {
        filters.extend(tonemap);
    }

    // Scale
    if let Some(scale) = scale_filter(encoder, video, source) {
        filters.push(scale);
    }

    // Framerate
    if let Some(fps) = framerate_filter(video) {
        // For hardware encoders, fps filter needs software frames
        match encoder {
            HwEncoder::Nvenc | HwEncoder::Vaapi | HwEncoder::Qsv => {
                // fps filter works on software frames, skip for hw pipelines
                // unless we already have a tonemap that goes through software
                if source.is_hdr {
                    // Already in software path from tonemap, insert before hwupload
                    if let Some(last) = filters.last() {
                        if last.starts_with("hwupload") {
                            let upload = filters.pop().unwrap();
                            filters.push(fps);
                            filters.push(upload);
                        } else {
                            filters.push(fps);
                        }
                    }
                }
                // If not HDR and using pure HW path, skip fps filter
                // (would need hwdownload/hwupload pair which is expensive)
            }
            _ => {
                filters.push(fps);
            }
        }
    }

    if filters.is_empty() {
        None
    } else {
        Some(vec!["-vf".into(), filters.join(",")])
    }
}

fn parse_resolution(s: &str) -> Option<(u32, u32)> {
    let parts: Vec<&str> = s.split('x').collect();
    if parts.len() == 2 {
        let w = parts[0].parse().ok()?;
        let h = parts[1].parse().ok()?;
        Some((w, h))
    } else {
        None
    }
}
