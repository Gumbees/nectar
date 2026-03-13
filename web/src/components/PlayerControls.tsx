import { createSignal, createEffect, onCleanup, Show } from "solid-js";
import { useNavigate } from "@solidjs/router";
import type { MediaStream } from "../types";
import { SubtitleSelector } from "./SubtitleSelector";
import styles from "./PlayerControls.module.css";

interface PlayerControlsProps {
  videoRef: HTMLVideoElement;
  title?: string;
  subtitleStreams?: MediaStream[];
  onSubtitleChange?: (stream: MediaStream | null) => void;
}

function formatTime(seconds: number): string {
  if (!isFinite(seconds)) return "0:00";
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = Math.floor(seconds % 60);
  const pad = (n: number) => n.toString().padStart(2, "0");
  return h > 0 ? `${h}:${pad(m)}:${pad(s)}` : `${m}:${pad(s)}`;
}

export function PlayerControls(props: PlayerControlsProps) {
  const navigate = useNavigate();
  const [isPlaying, setIsPlaying] = createSignal(false);
  const [currentTime, setCurrentTime] = createSignal(0);
  const [duration, setDuration] = createSignal(0);
  const [volume, setVolume] = createSignal(1);
  const [isMuted, setIsMuted] = createSignal(false);
  const [visible, setVisible] = createSignal(true);
  const [showSubtitles, setShowSubtitles] = createSignal(false);
  let hideTimer: ReturnType<typeof setTimeout>;

  const resetHideTimer = () => {
    setVisible(true);
    clearTimeout(hideTimer);
    hideTimer = setTimeout(() => {
      if (isPlaying()) setVisible(false);
    }, 3000);
  };

  createEffect(() => {
    const video = props.videoRef;
    if (!video) return;

    const onPlay = () => setIsPlaying(true);
    const onPause = () => { setIsPlaying(false); setVisible(true); };
    const onTimeUpdate = () => setCurrentTime(video.currentTime);
    const onDurationChange = () => setDuration(video.duration);
    const onVolumeChange = () => {
      setVolume(video.volume);
      setIsMuted(video.muted);
    };

    video.addEventListener("play", onPlay);
    video.addEventListener("pause", onPause);
    video.addEventListener("timeupdate", onTimeUpdate);
    video.addEventListener("durationchange", onDurationChange);
    video.addEventListener("volumechange", onVolumeChange);

    // Mouse/touch events on the container
    const container = video.parentElement;
    if (container) {
      container.addEventListener("mousemove", resetHideTimer);
      container.addEventListener("touchstart", resetHideTimer);
    }

    onCleanup(() => {
      video.removeEventListener("play", onPlay);
      video.removeEventListener("pause", onPause);
      video.removeEventListener("timeupdate", onTimeUpdate);
      video.removeEventListener("durationchange", onDurationChange);
      video.removeEventListener("volumechange", onVolumeChange);
      clearTimeout(hideTimer);
      if (container) {
        container.removeEventListener("mousemove", resetHideTimer);
        container.removeEventListener("touchstart", resetHideTimer);
      }
    });
  });

  const togglePlay = () => {
    if (props.videoRef.paused) {
      props.videoRef.play();
    } else {
      props.videoRef.pause();
    }
  };

  const skip = (seconds: number) => {
    props.videoRef.currentTime = Math.max(0, Math.min(props.videoRef.currentTime + seconds, duration()));
  };

  const seek = (e: MouseEvent) => {
    const bar = e.currentTarget as HTMLElement;
    const rect = bar.getBoundingClientRect();
    const pct = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
    props.videoRef.currentTime = pct * duration();
  };

  const toggleMute = () => {
    props.videoRef.muted = !props.videoRef.muted;
  };

  const handleVolume = (e: Event) => {
    const val = parseFloat((e.target as HTMLInputElement).value);
    props.videoRef.volume = val;
    if (val > 0) props.videoRef.muted = false;
  };

  const toggleFullscreen = () => {
    if (document.fullscreenElement) {
      document.exitFullscreen();
    } else {
      const container = props.videoRef.parentElement;
      container?.requestFullscreen?.();
    }
  };

  const goBack = () => {
    if (window.history.length > 1) {
      navigate(-1);
    } else {
      navigate("/");
    }
  };

  const progress = () => duration() > 0 ? (currentTime() / duration()) * 100 : 0;

  return (
    <div
      class={`${styles.overlay} ${visible() ? styles.overlayVisible : styles.overlayHidden}`}
      onClick={(e) => {
        // Only toggle play if clicking the overlay itself, not buttons
        if (e.target === e.currentTarget) togglePlay();
      }}
    >
      {/* Top Bar */}
      <div class={styles.topBar}>
        <button class={styles.backBtn} onClick={goBack} title="Back">
          &#x2190;
        </button>
        <Show when={props.title}>
          <div class={styles.mediaTitle}>{props.title}</div>
        </Show>
      </div>

      {/* Center Play Controls */}
      <div class={styles.center}>
        <button class={styles.centerBtn} onClick={() => skip(-10)} title="Rewind 10s">
          &#x23EA;
        </button>
        <button class={`${styles.centerBtn} ${styles.playBtn}`} onClick={togglePlay}>
          {isPlaying() ? "\u23F8" : "\u25B6"}
        </button>
        <button class={styles.centerBtn} onClick={() => skip(10)} title="Forward 10s">
          &#x23E9;
        </button>
      </div>

      {/* Bottom Bar */}
      <div class={styles.bottomBar}>
        {/* Seek Bar */}
        <div class={styles.seekRow}>
          <span class={styles.timeDisplay}>{formatTime(currentTime())}</span>
          <div class={styles.seekBarWrap} onClick={seek}>
            <div class={styles.seekBar}>
              <div class={styles.seekFill} style={{ width: `${progress()}%` }} />
              <div class={styles.seekThumb} style={{ left: `${progress()}%` }} />
            </div>
          </div>
          <span class={styles.timeDisplay}>{formatTime(duration())}</span>
        </div>

        {/* Bottom Controls */}
        <div class={styles.controls}>
          <div class={styles.controlsLeft}>
            <div class={styles.volumeGroup}>
              <button class={styles.controlBtn} onClick={toggleMute} title={isMuted() ? "Unmute" : "Mute"}>
                {isMuted() || volume() === 0 ? "\uD83D\uDD07" : volume() < 0.5 ? "\uD83D\uDD09" : "\uD83D\uDD0A"}
              </button>
              <input
                type="range"
                class={styles.volumeSlider}
                min="0"
                max="1"
                step="0.05"
                value={isMuted() ? 0 : volume()}
                onInput={handleVolume}
              />
            </div>
          </div>

          <div class={styles.controlsRight}>
            <Show when={props.subtitleStreams && props.subtitleStreams.length > 0}>
              <div style={{ position: "relative" }}>
                <button
                  class={styles.controlBtn}
                  onClick={() => setShowSubtitles(!showSubtitles())}
                  title="Subtitles"
                >
                  CC
                </button>
                <Show when={showSubtitles()}>
                  <SubtitleSelector
                    streams={props.subtitleStreams!}
                    onSelect={(stream) => {
                      props.onSubtitleChange?.(stream);
                      setShowSubtitles(false);
                    }}
                    onClose={() => setShowSubtitles(false)}
                  />
                </Show>
              </div>
            </Show>

            <button class={styles.controlBtn} onClick={toggleFullscreen} title="Fullscreen">
              &#x26F6;
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
