/// Parsed progress data from ffmpeg -progress pipe:1 output
#[derive(Debug, Default)]
pub struct FfmpegProgress {
    pub frame: u64,
    pub fps: f32,
    pub speed: f32,
    pub out_time_us: u64,
}

impl FfmpegProgress {
    /// Calculate completion percentage from output time vs total duration
    pub fn percent(&self, duration_seconds: f64) -> f32 {
        if duration_seconds <= 0.0 {
            return 0.0;
        }
        let out_seconds = self.out_time_us as f64 / 1_000_000.0;
        ((out_seconds / duration_seconds) * 100.0).min(100.0) as f32
    }

    /// Estimate remaining seconds
    pub fn eta_seconds(&self, duration_seconds: f64) -> Option<u64> {
        if self.speed <= 0.0 || duration_seconds <= 0.0 {
            return None;
        }
        let out_seconds = self.out_time_us as f64 / 1_000_000.0;
        let remaining_media = duration_seconds - out_seconds;
        if remaining_media <= 0.0 {
            return Some(0);
        }
        Some((remaining_media / self.speed as f64) as u64)
    }
}

/// Parse a line from ffmpeg -progress pipe:1 output.
/// Lines are key=value pairs. A "progress=continue" or "progress=end" line
/// marks the end of a progress block.
///
/// Returns Some(true) if this is the end of a progress block, Some(false) for
/// a regular key=value line, None if the line is invalid.
pub fn parse_progress_line(line: &str, current: &mut FfmpegProgress) -> Option<bool> {
    let (key, value) = line.split_once('=')?;
    let key = key.trim();
    let value = value.trim();

    match key {
        "frame" => {
            current.frame = value.parse().unwrap_or(0);
        }
        "fps" => {
            current.fps = value.parse().unwrap_or(0.0);
        }
        "speed" => {
            // Speed is like "1.5x" or "N/A"
            let speed_str = value.trim_end_matches('x');
            current.speed = speed_str.parse().unwrap_or(0.0);
        }
        "out_time_us" => {
            current.out_time_us = value.parse().unwrap_or(0);
        }
        "out_time_ms" => {
            // Some ffmpeg versions use ms instead of us
            if current.out_time_us == 0 {
                let ms: u64 = value.parse().unwrap_or(0);
                current.out_time_us = ms * 1000;
            }
        }
        "progress" => {
            return Some(value == "end");
        }
        _ => {}
    }

    Some(false)
}
