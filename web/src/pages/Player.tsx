import { createSignal, createResource, createEffect, onMount, onCleanup, Show } from "solid-js";
import { useParams, A } from "@solidjs/router";
import Hls from "hls.js";
import { fetchMediaItem, fetchMediaStreams, reportProgress } from "../lib/api";
import type { MediaItem, MediaStream } from "../types";
import { PlayerControls } from "../components/PlayerControls";
import styles from "./Player.module.css";

export function Player() {
  const params = useParams();
  let videoRef!: HTMLVideoElement;
  let hls: Hls | null = null;

  const [item] = createResource(() => params.id, fetchMediaItem);
  const [streams] = createResource(() => params.id, (id) => fetchMediaStreams(id).catch(() => []));
  const [error, setError] = createSignal<string | null>(null);

  const subtitleStreams = () => (streams() ?? []).filter((s: MediaStream) => s.stream_type === "subtitle");

  // Determine streaming URL
  const streamUrl = () => {
    const i = item();
    if (!i) return null;

    // Check if the container is directly playable
    const directPlayable = ["mp4", "webm", "mkv"].includes(i.container ?? "");
    if (directPlayable) {
      return `/api/v1/streaming/${i.id}/direct`;
    }
    // Fall back to HLS transcode
    return `/api/v1/streaming/${i.id}/hls/master.m3u8`;
  };

  // Setup video source
  createEffect(() => {
    const url = streamUrl();
    if (!url || !videoRef) return;

    // Cleanup previous HLS instance
    if (hls) {
      hls.destroy();
      hls = null;
    }

    if (url.endsWith(".m3u8") && Hls.isSupported()) {
      hls = new Hls({
        enableWorker: true,
        lowLatencyMode: false,
      });
      hls.loadSource(url);
      hls.attachMedia(videoRef);
      hls.on(Hls.Events.ERROR, (_event, data) => {
        if (data.fatal) {
          setError("Playback error: " + data.details);
        }
      });
    } else if (videoRef.canPlayType("application/vnd.apple.mpegurl")) {
      // Native HLS (Safari/iOS)
      videoRef.src = url;
    } else {
      videoRef.src = url;
    }
  });

  // Progress reporting
  onMount(() => {
    let progressInterval: ReturnType<typeof setInterval>;

    const startTracking = () => {
      progressInterval = setInterval(() => {
        if (videoRef && !videoRef.paused && params.id) {
          reportProgress(params.id, videoRef.currentTime).catch(() => {});
        }
      }, 5000);
    };

    startTracking();

    // MediaSession API for lock screen controls
    if ("mediaSession" in navigator) {
      createEffect(() => {
        const i = item();
        if (!i) return;
        navigator.mediaSession.metadata = new MediaMetadata({
          title: i.title,
          artist: i.year?.toString() ?? "",
          artwork: [
            { src: `/api/v1/media/${i.id}/poster`, sizes: "512x512", type: "image/jpeg" },
          ],
        });
      });

      navigator.mediaSession.setActionHandler("play", () => videoRef?.play());
      navigator.mediaSession.setActionHandler("pause", () => videoRef?.pause());
      navigator.mediaSession.setActionHandler("seekbackward", () => {
        if (videoRef) videoRef.currentTime = Math.max(0, videoRef.currentTime - 10);
      });
      navigator.mediaSession.setActionHandler("seekforward", () => {
        if (videoRef) videoRef.currentTime = Math.min(videoRef.duration, videoRef.currentTime + 10);
      });
    }

    onCleanup(() => {
      clearInterval(progressInterval);
      hls?.destroy();
    });
  });

  // Subtitle handling
  const handleSubtitleChange = (stream: MediaStream | null) => {
    if (!videoRef) return;
    // Remove existing subtitle tracks
    const tracks = videoRef.textTracks;
    for (let i = 0; i < tracks.length; i++) {
      tracks[i].mode = "disabled";
    }

    if (stream) {
      // Load subtitle from API
      const track = document.createElement("track");
      track.kind = "subtitles";
      track.label = stream.title ?? stream.language ?? "Subtitles";
      track.srclang = stream.language ?? "en";
      track.src = `/api/v1/media/${params.id}/subtitles/${stream.stream_index}`;
      track.default = true;
      videoRef.appendChild(track);
      if (videoRef.textTracks.length > 0) {
        videoRef.textTracks[videoRef.textTracks.length - 1].mode = "showing";
      }
    }
  };

  return (
    <div class={styles.page}>
      <Show when={error()}>
        <div class={styles.error}>
          <div class={styles.errorTitle}>Playback Error</div>
          <div class={styles.errorText}>{error()}</div>
          <A href="/" class={styles.backLink}>Go Home</A>
        </div>
      </Show>

      <div class={styles.videoContainer}>
        <video
          ref={videoRef}
          class={styles.video}
          playsinline
          webkit-playsinline
          x-webkit-airplay="allow"
          autoplay
        />
        <Show when={videoRef}>
          <PlayerControls
            videoRef={videoRef}
            title={item()?.title}
            subtitleStreams={subtitleStreams()}
            onSubtitleChange={handleSubtitleChange}
          />
        </Show>
      </div>
    </div>
  );
}
