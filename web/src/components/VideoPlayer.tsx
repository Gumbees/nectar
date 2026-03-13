import { createSignal, onMount, onCleanup } from "solid-js";
import Hls from "hls.js";

interface VideoPlayerProps {
  src: string;
  poster?: string;
  onProgress?: (seconds: number) => void;
  startAt?: number;
}

export function VideoPlayer(props: VideoPlayerProps) {
  let videoRef!: HTMLVideoElement;
  let hls: Hls | null = null;

  const [isFullscreen, setIsFullscreen] = createSignal(false);

  onMount(() => {
    // HLS.js for adaptive bitrate streaming
    if (props.src.endsWith(".m3u8") && Hls.isSupported()) {
      hls = new Hls({
        enableWorker: true,
        lowLatencyMode: false,
      });
      hls.loadSource(props.src);
      hls.attachMedia(videoRef);
    } else if (videoRef.canPlayType("application/vnd.apple.mpegurl")) {
      // Native HLS (Safari, iOS)
      videoRef.src = props.src;
    } else {
      videoRef.src = props.src;
    }

    if (props.startAt) {
      videoRef.currentTime = props.startAt;
    }

    // Track playback progress
    const interval = setInterval(() => {
      if (!videoRef.paused && props.onProgress) {
        props.onProgress(videoRef.currentTime);
      }
    }, 5000);

    // Fullscreen change detection (for screen mirroring state tracking)
    const onFsChange = () => setIsFullscreen(!!document.fullscreenElement);
    document.addEventListener("fullscreenchange", onFsChange);

    onCleanup(() => {
      clearInterval(interval);
      document.removeEventListener("fullscreenchange", onFsChange);
      hls?.destroy();
    });
  });

  const toggleFullscreen = () => {
    if (document.fullscreenElement) {
      document.exitFullscreen();
    } else {
      // Use the video element directly for proper iPad/Android fullscreen
      // This ensures screen mirroring shows only the video
      videoRef.requestFullscreen?.() ??
        (videoRef as any).webkitRequestFullscreen?.() ??
        (videoRef as any).webkitEnterFullscreen?.();
    }
  };

  return (
    <div class="video-container">
      <video
        ref={videoRef}
        poster={props.poster}
        controls
        playsinline
        webkit-playsinline
        x-webkit-airplay="allow"
        onDblClick={toggleFullscreen}
        style={{
          width: "100%",
          height: "100%",
          "max-height": "100vh",
          background: "#000",
        }}
      />
    </div>
  );
}
