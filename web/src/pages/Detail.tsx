import { createResource, Show, For, createMemo } from "solid-js";
import { useParams, A } from "@solidjs/router";
import { fetchMediaItem, fetchMediaItems, fetchMediaStreams, fetchMediaPeople, fetchMediaGenres } from "../lib/api";
import type { MediaItem, Person, Genre, MediaStream } from "../types";
import { EpisodeList } from "../components/EpisodeList";
import { CastRow } from "../components/CastRow";
import { MediaCard } from "../components/MediaCard";
import styles from "./Detail.module.css";

function formatRuntime(seconds?: number): string {
  if (!seconds) return "";
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  return h > 0 ? `${h}h ${m}m` : `${m}m`;
}

function formatBytes(bytes?: number): string {
  if (!bytes) return "";
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

export function Detail() {
  const params = useParams();

  const [item] = createResource(() => params.id, fetchMediaItem);
  const [peopleData] = createResource(() => params.id, (id) => fetchMediaPeople(id).catch(() => ({ people: [] })));
  const [genresData] = createResource(() => params.id, (id) => fetchMediaGenres(id).catch(() => ({ genres: [] })));
  const [streamsData] = createResource(() => params.id, (id) => fetchMediaStreams(id).catch(() => []));

  // For series: load seasons
  const isSeries = createMemo(() => item()?.item_type === "series");
  const [seasonsData] = createResource(
    () => isSeries() ? params.id : null,
    (id) => id ? fetchMediaItems({ parent: id, type: "season", sort: "season_number", order: "asc" }) : Promise.resolve({ items: [], total: 0 })
  );

  const seasons = () => seasonsData()?.items ?? [];
  const people = () => peopleData()?.people ?? [];
  const genres = () => genresData()?.genres ?? [];

  // Similar items (same library, same type, excluding current)
  const [similar] = createResource(
    () => item() ? { libraryId: item()!.library_id, type: item()!.item_type } : null,
    (params) => params
      ? fetchMediaItems({ library: params.libraryId, type: params.type, limit: 10 })
      : Promise.resolve({ items: [], total: 0 })
  );

  const similarItems = createMemo(() =>
    (similar()?.items ?? []).filter((i: MediaItem) => i.id !== params.id).slice(0, 8)
  );

  const playTarget = createMemo(() => {
    const i = item();
    if (!i) return "#";
    if (i.item_type === "movie" || i.item_type === "episode") return `/play/${i.id}`;
    // For series, link to first episode of first season
    const firstSeason = seasons()[0];
    return firstSeason ? `/detail/${firstSeason.id}` : "#";
  });

  return (
    <div class={styles.page}>
      <Show when={item()} fallback={
        <div class={styles.loading}>Loading...</div>
      }>
        {(itemData) => {
          const i = itemData();
          return (
            <>
              {/* Hero */}
              <div class={styles.hero}>
                <div class={styles.backdrop}>
                  <img
                    class={styles.backdropImg}
                    src={`/api/v1/media/${i.id}/backdrop`}
                    alt=""
                    onError={(e) => { (e.currentTarget as HTMLImageElement).style.display = "none"; }}
                  />
                  <div class={styles.backdropGradient} />
                </div>

                <div class={styles.heroContent}>
                  <div class={styles.poster}>
                    <img
                      class={styles.posterImg}
                      src={`/api/v1/media/${i.id}/poster`}
                      alt={i.title}
                      onError={(e) => { (e.currentTarget as HTMLImageElement).style.visibility = "hidden"; }}
                    />
                  </div>

                  <div class={styles.heroInfo}>
                    <h1 class={styles.title}>{i.title}</h1>

                    <Show when={i.tagline}>
                      <div class={styles.tagline}>{i.tagline}</div>
                    </Show>

                    <div class={styles.meta}>
                      <Show when={i.year}><span>{i.year}</span></Show>
                      <Show when={i.runtime_seconds}>
                        <span>{formatRuntime(i.runtime_seconds)}</span>
                      </Show>
                      <Show when={i.community_rating}>
                        <span class={styles.rating}>
                          {i.community_rating?.toFixed(1)}
                        </span>
                      </Show>
                      <Show when={i.content_rating}>
                        <span class={styles.contentRating}>{i.content_rating}</span>
                      </Show>
                    </div>

                    <Show when={genres().length > 0}>
                      <div class={styles.genreChips}>
                        <For each={genres()}>
                          {(genre) => <span class={styles.genreChip}>{genre.name}</span>}
                        </For>
                      </div>
                    </Show>

                    <Show when={i.overview}>
                      <div class={styles.overview}>{i.overview}</div>
                    </Show>

                    <div class={styles.actions}>
                      <A href={playTarget()} class={styles.playBtn}>
                        {"\u25B6"} Play
                      </A>
                    </div>
                  </div>
                </div>
              </div>

              {/* Body */}
              <div class={styles.body}>
                {/* Episodes (for series) */}
                <Show when={isSeries() && seasons().length > 0}>
                  <div class={styles.section}>
                    <EpisodeList seriesId={i.id} seasons={seasons()} />
                  </div>
                </Show>

                {/* Cast */}
                <Show when={people().length > 0}>
                  <div class={styles.section}>
                    <CastRow people={people()} />
                  </div>
                </Show>

                {/* Technical Info */}
                <Show when={i.file_path || i.video_codec || i.audio_codec || i.resolution}>
                  <div class={styles.section}>
                    <h2 class={styles.sectionTitle}>Technical Details</h2>
                    <div class={styles.techGrid}>
                      <Show when={i.video_codec}>
                        <div class={styles.techItem}>
                          <div class={styles.techLabel}>Video Codec</div>
                          <div class={styles.techValue}>{i.video_codec}</div>
                        </div>
                      </Show>
                      <Show when={i.audio_codec}>
                        <div class={styles.techItem}>
                          <div class={styles.techLabel}>Audio Codec</div>
                          <div class={styles.techValue}>{i.audio_codec}</div>
                        </div>
                      </Show>
                      <Show when={i.resolution}>
                        <div class={styles.techItem}>
                          <div class={styles.techLabel}>Resolution</div>
                          <div class={styles.techValue}>{i.resolution}</div>
                        </div>
                      </Show>
                      <Show when={i.container}>
                        <div class={styles.techItem}>
                          <div class={styles.techLabel}>Container</div>
                          <div class={styles.techValue}>{i.container}</div>
                        </div>
                      </Show>
                      <Show when={i.bitrate}>
                        <div class={styles.techItem}>
                          <div class={styles.techLabel}>Bitrate</div>
                          <div class={styles.techValue}>{((i.bitrate ?? 0) / 1000).toFixed(0)} kbps</div>
                        </div>
                      </Show>
                      <Show when={i.size_bytes}>
                        <div class={styles.techItem}>
                          <div class={styles.techLabel}>File Size</div>
                          <div class={styles.techValue}>{formatBytes(i.size_bytes)}</div>
                        </div>
                      </Show>
                    </div>
                  </div>
                </Show>

                {/* Similar Items */}
                <Show when={similarItems().length > 0}>
                  <div class={styles.section}>
                    <h2 class={styles.sectionTitle}>More Like This</h2>
                    <div style={{
                      display: "grid",
                      "grid-template-columns": "repeat(auto-fill, minmax(var(--card-width-md), 1fr))",
                      gap: "var(--space-4)"
                    }}>
                      <For each={similarItems()}>
                        {(sim) => <MediaCard item={sim} />}
                      </For>
                    </div>
                  </div>
                </Show>
              </div>
            </>
          );
        }}
      </Show>
    </div>
  );
}
