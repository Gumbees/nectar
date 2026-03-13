pub mod encoder;
pub mod filters;
pub mod progress;

use anyhow::Result;
use nectar_proto::TranscodeRequest;
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::config::Config;
use crate::hardware::HardwareCapabilities;

use self::encoder::{audio_encoder_args, hwaccel_input_args, select_encoder, video_encoder_args};
use self::filters::{build_video_filter_chain, trickplay_filter};
use self::progress::{parse_progress_line, FfmpegProgress};

pub struct FfmpegCommand {
    args: Vec<String>,
}

impl FfmpegCommand {
    /// Build command for live (HLS streaming) transcode
    pub fn for_live_transcode(
        job: &TranscodeRequest,
        hw: &HardwareCapabilities,
        config: &Config,
    ) -> Self {
        let encoder = select_encoder(hw);
        let mut args: Vec<String> = Vec::new();

        // Global options
        args.extend(["-y".into(), "-progress".into(), "pipe:1".into()]);

        // Hardware acceleration input args
        args.extend(hwaccel_input_args(&encoder, &config.vaapi_device));

        // Input
        args.extend(["-i".into(), job.input_path.clone()]);

        // Video filter chain
        if let Some(vf_args) = build_video_filter_chain(&encoder, &job.video, &job.source_info) {
            args.extend(vf_args);
        }

        // Video encoder
        args.extend(video_encoder_args(&encoder, hw, &job.video));

        // Audio encoder
        args.extend(audio_encoder_args(&job.audio));

        // HLS output
        let hls = job.hls.as_ref();
        let seg_duration = hls.map_or(6, |h| h.segment_duration);
        let playlist_type = hls.map_or("event", |h| match h.playlist_type {
            nectar_proto::HlsPlaylistType::Event => "event",
            nectar_proto::HlsPlaylistType::Vod => "vod",
        });

        let output_dir = &job.output_dir;
        args.extend([
            "-f".into(),
            "hls".into(),
            "-hls_time".into(),
            seg_duration.to_string(),
            "-hls_list_size".into(),
            "0".into(),
            "-hls_playlist_type".into(),
            playlist_type.into(),
            "-hls_segment_filename".into(),
            format!("{output_dir}/segment_%03d.ts"),
            format!("{output_dir}/stream.m3u8"),
        ]);

        Self { args }
    }

    /// Build command for offline (file-to-file) transcode
    pub fn for_offline_transcode(
        job: &TranscodeRequest,
        hw: &HardwareCapabilities,
        config: &Config,
    ) -> Self {
        let encoder = select_encoder(hw);
        let mut args: Vec<String> = Vec::new();

        // Global options
        args.extend(["-y".into(), "-progress".into(), "pipe:1".into()]);

        // Hardware acceleration input args
        args.extend(hwaccel_input_args(&encoder, &config.vaapi_device));

        // Input
        args.extend(["-i".into(), job.input_path.clone()]);

        // Video filter chain
        if let Some(vf_args) = build_video_filter_chain(&encoder, &job.video, &job.source_info) {
            args.extend(vf_args);
        }

        // Video encoder
        args.extend(video_encoder_args(&encoder, hw, &job.video));

        // Audio encoder
        args.extend(audio_encoder_args(&job.audio));

        // If HLS params provided, output as HLS, otherwise as MP4
        if let Some(hls) = &job.hls {
            let output_dir = &job.output_dir;
            let playlist_type = match hls.playlist_type {
                nectar_proto::HlsPlaylistType::Event => "event",
                nectar_proto::HlsPlaylistType::Vod => "vod",
            };
            args.extend([
                "-f".into(),
                "hls".into(),
                "-hls_time".into(),
                hls.segment_duration.to_string(),
                "-hls_list_size".into(),
                "0".into(),
                "-hls_playlist_type".into(),
                playlist_type.into(),
                "-hls_segment_filename".into(),
                format!("{output_dir}/segment_%03d.ts"),
                format!("{output_dir}/stream.m3u8"),
            ]);
        } else {
            // MP4 output
            args.extend([
                "-movflags".into(),
                "+faststart".into(),
                format!("{}/output.mp4", job.output_dir),
            ]);
        }

        Self { args }
    }

    /// Build command for trickplay (thumbnail sprite sheet) generation
    pub fn for_trickplay(
        job: &TranscodeRequest,
        _hw: &HardwareCapabilities,
        _config: &Config,
    ) -> Self {
        let mut args: Vec<String> = Vec::new();

        let trickplay = job
            .trickplay
            .as_ref()
            .expect("trickplay params required for trickplay job");

        // Global options
        args.extend(["-y".into(), "-progress".into(), "pipe:1".into()]);

        // Input
        args.extend(["-i".into(), job.input_path.clone()]);

        // Trickplay filter chain (always software — thumbnails don't need GPU)
        let filter = trickplay_filter(trickplay);
        args.extend(["-vf".into(), filter]);

        // Output as image sequence
        args.extend([
            "-an".into(), // no audio
            format!("{}/trickplay_%04d.jpg", job.output_dir),
        ]);

        Self { args }
    }

    /// Execute the ffmpeg command, parsing progress and sending updates
    pub async fn execute(
        &self,
        config: &Config,
        duration_seconds: f64,
        progress_tx: tokio::sync::mpsc::Sender<f32>,
    ) -> Result<()> {
        let mut cmd = tokio::process::Command::new(&config.ffmpeg_path);
        cmd.args(&self.args);
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        tracing::debug!(args = ?self.args, "running ffmpeg");

        let mut child = cmd.spawn()?;
        let stdout = child.stdout.take().expect("stdout piped");
        let stderr = child.stderr.take().expect("stderr piped");

        // Read stderr in background to capture error messages
        let stderr_handle = tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            let mut last_lines: Vec<String> = Vec::new();
            while let Ok(Some(line)) = lines.next_line().await {
                tracing::trace!(target: "ffmpeg_stderr", "{}", line);
                if last_lines.len() >= 20 {
                    last_lines.remove(0);
                }
                last_lines.push(line);
            }
            last_lines
        });

        // Parse progress from stdout
        let progress_interval_us = (config.progress_interval_secs as f64 * 1_000_000.0) as u64;
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        let mut current = FfmpegProgress::default();
        let mut last_report_time_us: u64 = 0;

        while let Ok(Some(line)) = lines.next_line().await {
            if let Some(is_end) = parse_progress_line(&line, &mut current) {
                if is_end || current.out_time_us >= last_report_time_us + progress_interval_us {
                    let pct = current.percent(duration_seconds);
                    let _ = progress_tx.send(pct).await;
                    last_report_time_us = current.out_time_us;
                }
            }
        }

        let status = child.wait().await?;
        let stderr_lines = stderr_handle.await.unwrap_or_default();

        if !status.success() {
            let error_context = stderr_lines.join("\n");
            anyhow::bail!(
                "ffmpeg exited with status {status}. Last stderr:\n{error_context}"
            );
        }

        // Send final 100%
        let _ = progress_tx.send(100.0).await;

        Ok(())
    }

    /// Get the args for logging/debugging
    pub fn args(&self) -> &[String] {
        &self.args
    }
}
