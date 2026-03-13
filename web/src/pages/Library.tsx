import { createResource } from "solid-js";
import { useParams } from "@solidjs/router";
import { api } from "../lib/api";

export function Library() {
  const params = useParams();
  const [items] = createResource(() => params.id, (id) => api(`/api/v1/media?library=${id}`));

  return (
    <div class="library">
      <div class="media-grid">
        {/* TODO: render media cards */}
      </div>
    </div>
  );
}
