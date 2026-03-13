import { createResource } from "solid-js";
import { useParams } from "@solidjs/router";
import { VideoPlayer } from "../components/VideoPlayer";
import { api } from "../lib/api";

export function Player() {
  const params = useParams();
  const [item] = createResource(() => params.id, (id) => api(`/api/v1/media/${id}`));

  const reportProgress = async (seconds: number) => {
    await api(`/api/v1/media/${params.id}/progress`, {
      method: "POST",
      body: JSON.stringify({ position_seconds: Math.floor(seconds) }),
    });
  };

  return (
    <div class="player-page" style={{ height: "100vh", background: "#000" }}>
      <VideoPlayer
        src={`/api/v1/streaming/${params.id}/direct`}
        onProgress={reportProgress}
      />
    </div>
  );
}
