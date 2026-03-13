import { createResource, For, Show } from "solid-js";
import { A } from "@solidjs/router";
import { fetchContinueWatching } from "../lib/api";
import type { MediaItem } from "../types";
import styles from "./ContinueWatching.module.css";

function formatProgress(item: MediaItem & { position_seconds?: number }): number {
  if (!item.runtime_seconds || !(item as any).position_seconds) return 0;
  return (item as any).position_seconds / item.runtime_seconds;
}

export function ContinueWatching() {
  const [data] = createResource(fetchContinueWatching);

  const items = () => data()?.items ?? [];

  return (
    <Show when={items().length > 0}>
      <div class={styles.container}>
        <h2 class={styles.header}>Continue Watching</h2>
        <div class={styles.row}>
          <For each={items()}>
            {(item) => (
              <div class={styles.item}>
                <A href={`/play/${item.id}`} class={styles.card}>
                  <div class={styles.thumb}>
                    <img
                      class={styles.thumbImg}
                      src={`/api/v1/media/${item.id}/backdrop`}
                      alt=""
                      loading="lazy"
                      onError={(e) => {
                        (e.currentTarget as HTMLImageElement).src = `/api/v1/media/${item.id}/poster`;
                      }}
                    />
                    <div class={styles.progressBar}>
                      <div
                        class={styles.progressFill}
                        style={{ width: `${formatProgress(item) * 100}%` }}
                      />
                    </div>
                  </div>
                  <div class={styles.meta}>
                    <div class={styles.title}>{item.title}</div>
                    <Show when={item.episode_number !== undefined}>
                      <div class={styles.subtitle}>
                        S{item.season_number} E{item.episode_number}
                      </div>
                    </Show>
                  </div>
                </A>
              </div>
            )}
          </For>
        </div>
      </div>
    </Show>
  );
}
