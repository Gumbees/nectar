use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use anyhow::Result;
use async_nats::Client;
use chrono::Utc;
use nectar_proto::{JobProgress, JobState, JobType, TranscodeRequest, WorkerHeartbeat};
use tokio::sync::Semaphore;
use uuid::Uuid;

use crate::config::Config;
use crate::ffmpeg::FfmpegCommand;
use crate::hardware::HardwareCapabilities;

pub async fn run(
    client: Client,
    hw: HardwareCapabilities,
    config: Config,
    hostname: String,
) -> Result<()> {
    let semaphore = Arc::new(Semaphore::new(config.max_concurrent_jobs));
    let active_jobs = Arc::new(AtomicU32::new(0));
    let mut subscriber = client.subscribe("nectar.transcode.jobs").await?;

    // Start heartbeat background task
    let heartbeat_client = client.clone();
    let heartbeat_config = config.clone();
    let heartbeat_active = active_jobs.clone();
    let heartbeat_hostname = hostname.clone();
    tokio::spawn(async move {
        heartbeat_loop(
            heartbeat_client,
            &heartbeat_config,
            heartbeat_active,
            &heartbeat_hostname,
        )
        .await;
    });

    tracing::info!(
        worker_id = %config.worker_id,
        max_concurrent = config.max_concurrent_jobs,
        "transcoder worker started, listening for jobs"
    );

    while let Some(msg) = subscriber.next().await {
        let permit = semaphore.clone().acquire_owned().await?;
        let hw = hw.clone();
        let config = config.clone();
        let client = client.clone();
        let active = active_jobs.clone();

        active.fetch_add(1, Ordering::Relaxed);

        tokio::spawn(async move {
            match serde_json::from_slice::<TranscodeRequest>(&msg.payload) {
                Ok(job) => {
                    let job_id = job.id;
                    tracing::info!(%job_id, job_type = ?job.job_type, "processing transcode job");

                    // Send accepted status
                    publish_progress(
                        &client,
                        &config,
                        job_id,
                        JobState::Accepted,
                        0.0,
                        None,
                        None,
                        None,
                        None,
                    )
                    .await;

                    // Ensure output directory exists
                    if let Err(e) = tokio::fs::create_dir_all(&job.output_dir).await {
                        tracing::error!(%job_id, error = %e, "failed to create output dir");
                        publish_progress(
                            &client,
                            &config,
                            job_id,
                            JobState::Failed,
                            0.0,
                            None,
                            None,
                            None,
                            Some(e.to_string()),
                        )
                        .await;
                        active.fetch_sub(1, Ordering::Relaxed);
                        drop(permit);
                        return;
                    }

                    // Build the appropriate ffmpeg command
                    let ffmpeg_cmd = match job.job_type {
                        JobType::LiveTranscode => {
                            FfmpegCommand::for_live_transcode(&job, &hw, &config)
                        }
                        JobType::OfflineTranscode => {
                            FfmpegCommand::for_offline_transcode(&job, &hw, &config)
                        }
                        JobType::Trickplay => FfmpegCommand::for_trickplay(&job, &hw, &config),
                    };

                    // Progress channel
                    let (progress_tx, mut progress_rx) = tokio::sync::mpsc::channel::<f32>(32);

                    // Spawn progress reporter
                    let progress_client = client.clone();
                    let progress_config = config.clone();
                    let progress_handle = tokio::spawn(async move {
                        while let Some(pct) = progress_rx.recv().await {
                            publish_progress(
                                &progress_client,
                                &progress_config,
                                job_id,
                                JobState::Running,
                                pct,
                                None,
                                None,
                                None,
                                None,
                            )
                            .await;
                        }
                    });

                    // Execute
                    let duration = job.source_info.duration_seconds;
                    match ffmpeg_cmd.execute(&config, duration, progress_tx).await {
                        Ok(()) => {
                            tracing::info!(%job_id, "transcode complete");

                            // Generate WebVTT manifest for trickplay jobs
                            if matches!(job.job_type, JobType::Trickplay) {
                                if let Some(ref trickplay) = job.trickplay {
                                    if let Err(e) = generate_trickplay_webvtt(
                                        &job.output_dir,
                                        trickplay,
                                        duration,
                                    )
                                    .await
                                    {
                                        tracing::warn!(
                                            %job_id,
                                            error = %e,
                                            "failed to generate trickplay WebVTT"
                                        );
                                    }
                                }
                            }

                            publish_progress(
                                &client,
                                &config,
                                job_id,
                                JobState::Completed,
                                100.0,
                                None,
                                None,
                                None,
                                None,
                            )
                            .await;
                        }
                        Err(e) => {
                            tracing::error!(%job_id, error = %e, "transcode failed");
                            publish_progress(
                                &client,
                                &config,
                                job_id,
                                JobState::Failed,
                                0.0,
                                None,
                                None,
                                None,
                                Some(e.to_string()),
                            )
                            .await;
                        }
                    }

                    // Wait for progress reporter to finish
                    let _ = progress_handle.await;
                }
                Err(e) => tracing::error!(error = %e, "invalid job payload"),
            }

            active.fetch_sub(1, Ordering::Relaxed);
            drop(permit);
        });
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn publish_progress(
    client: &Client,
    config: &Config,
    job_id: Uuid,
    state: JobState,
    percent: f32,
    fps: Option<f32>,
    speed: Option<f32>,
    eta_seconds: Option<u64>,
    error: Option<String>,
) {
    let progress = JobProgress {
        job_id,
        worker_id: config.worker_id.clone(),
        state,
        percent,
        fps,
        speed,
        eta_seconds,
        error,
        timestamp: Utc::now(),
    };

    match serde_json::to_vec(&progress) {
        Ok(payload) => {
            if let Err(e) = client
                .publish("nectar.transcode.progress", payload.into())
                .await
            {
                tracing::warn!(error = %e, "failed to publish progress");
            }
        }
        Err(e) => tracing::warn!(error = %e, "failed to serialize progress"),
    }
}

async fn heartbeat_loop(
    client: Client,
    config: &Config,
    active_jobs: Arc<AtomicU32>,
    hostname: &str,
) {
    let mut interval =
        tokio::time::interval(std::time::Duration::from_secs(config.heartbeat_interval_secs));

    loop {
        interval.tick().await;

        let heartbeat = WorkerHeartbeat {
            worker_id: config.worker_id.clone(),
            hostname: hostname.to_string(),
            active_jobs: active_jobs.load(Ordering::Relaxed),
            max_jobs: config.max_concurrent_jobs as u32,
            timestamp: Utc::now(),
        };

        match serde_json::to_vec(&heartbeat) {
            Ok(payload) => {
                if let Err(e) = client
                    .publish("nectar.transcoder.heartbeat", payload.into())
                    .await
                {
                    tracing::warn!(error = %e, "failed to publish heartbeat");
                }
            }
            Err(e) => tracing::warn!(error = %e, "failed to serialize heartbeat"),
        }
    }
}

async fn generate_trickplay_webvtt(
    output_dir: &str,
    trickplay: &nectar_proto::TrickplayParams,
    duration_seconds: f64,
) -> Result<()> {
    let interval = trickplay.interval_seconds as f64;
    let tiles_per_image = (trickplay.columns * trickplay.rows) as f64;
    let tile_width = trickplay.width;
    // Estimate tile height based on 16:9 aspect ratio
    let tile_height = (tile_width as f64 * 9.0 / 16.0) as u32;

    let mut webvtt = String::from("WEBVTT\n\n");
    let mut time = 0.0;
    let mut image_index = 1u32;
    let mut tile_index = 0u32;

    while time < duration_seconds {
        let start = format_vtt_time(time);
        let end_time = (time + interval).min(duration_seconds);
        let end = format_vtt_time(end_time);

        let col = tile_index % trickplay.columns;
        let row = (tile_index / trickplay.columns) % trickplay.rows;
        let x = col * tile_width;
        let y = row * tile_height;

        webvtt.push_str(&format!(
            "{start} --> {end}\ntrickplay_{image_index:04}.jpg#xywh={x},{y},{tile_width},{tile_height}\n\n"
        ));

        time += interval;
        tile_index += 1;

        if tile_index as f64 >= tiles_per_image {
            tile_index = 0;
            image_index += 1;
        }
    }

    tokio::fs::write(format!("{output_dir}/trickplay.vtt"), webvtt).await?;
    Ok(())
}

fn format_vtt_time(seconds: f64) -> String {
    let total_secs = seconds as u64;
    let hours = total_secs / 3600;
    let mins = (total_secs % 3600) / 60;
    let secs = total_secs % 60;
    let millis = ((seconds - total_secs as f64) * 1000.0) as u64;
    format!("{hours:02}:{mins:02}:{secs:02}.{millis:03}")
}
