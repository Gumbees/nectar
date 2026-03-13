import { createSignal, createResource, For, Show, createMemo } from "solid-js";
import { A } from "@solidjs/router";
import { fetchMediaItems } from "../lib/api";
import type { MediaItem } from "../types";
import styles from "./EpisodeList.module.css";

interface EpisodeListProps {
  seriesId: string;
  seasons: MediaItem[];
}

function formatRuntime(seconds?: number): string {
  if (!seconds) return "";
  const m = Math.floor(seconds / 60);
  return `${m} min`;
}

export function EpisodeList(props: EpisodeListProps) {
  const [selectedSeason, setSelectedSeason] = createSignal<string | null>(null);

  const activeSeason = createMemo(() => {
    const sel = selectedSeason();
    if (sel) return sel;
    // Default to first season
    return props.seasons.length > 0 ? props.seasons[0].id : null;
  });

  const [episodes] = createResource(activeSeason, (seasonId) => {
    if (!seasonId) return Promise.resolve({ items: [], total: 0 });
    return fetchMediaItems({ parent: seasonId, type: "episode", sort: "episode_number", order: "asc" });
  });

  const episodeList = () => episodes()?.items ?? [];

  return (
    <div class={styles.container}>
      <div class={styles.seasonSelector}>
        <For each={props.seasons}>
          {(season) => (
            <button
              class={`${styles.seasonBtn} ${activeSeason() === season.id ? styles.seasonBtnActive : ""}`}
              onClick={() => setSelectedSeason(season.id)}
            >
              {season.season_number !== undefined ? `Season ${season.season_number}` : season.title}
            </button>
          )}
        </For>
      </div>

      <Show when={episodeList().length > 0} fallback={
        <div class={styles.empty}>No episodes found</div>
      }>
        <div class={styles.episodeList}>
          <For each={episodeList()}>
            {(ep) => (
              <A href={`/play/${ep.id}`} class={styles.episode}>
                <div class={styles.episodeNumber}>{ep.episode_number ?? ""}</div>

                <div class={styles.episodeThumb}>
                  <img
                    class={styles.episodeThumbImg}
                    src={`/api/v1/media/${ep.id}/backdrop`}
                    alt=""
                    loading="lazy"
                    onError={(e) => {
                      (e.currentTarget as HTMLImageElement).style.visibility = "hidden";
                    }}
                  />
                </div>

                <div class={styles.episodeInfo}>
                  <div class={styles.episodeTitle}>{ep.title}</div>
                  <Show when={ep.overview}>
                    <div class={styles.episodeOverview}>{ep.overview}</div>
                  </Show>
                  <Show when={ep.runtime_seconds}>
                    <div class={styles.episodeRuntime}>{formatRuntime(ep.runtime_seconds)}</div>
                  </Show>
                </div>
              </A>
            )}
          </For>
        </div>
      </Show>
    </div>
  );
}
