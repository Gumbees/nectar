import { createResource, For } from "solid-js";
import { api } from "../lib/api";

export function Home() {
  const [libraries] = createResource(() => api("/api/v1/libraries"));

  return (
    <div class="home">
      <h1>Libraries</h1>
      <div class="library-grid">
        <For each={libraries()?.libraries ?? []}>
          {(lib: any) => (
            <a href={`/library/${lib.id}`} class="library-card">
              <h2>{lib.name}</h2>
              <span>{lib.library_type}</span>
            </a>
          )}
        </For>
      </div>
    </div>
  );
}
