import { Show } from "solid-js";
import { A } from "@solidjs/router";
import type { MediaItem } from "../types";
import styles from "./MediaCard.module.css";

interface MediaCardProps {
  item: MediaItem;
  viewMode?: "grid" | "list";
  progress?: number; // 0-1
}

function formatRuntime(seconds?: number): string {
  if (!seconds) return "";
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  return h > 0 ? `${h}h ${m}m` : `${m}m`;
}

function posterUrl(item: MediaItem): string {
  return `/api/v1/media/${item.id}/poster`;
}

function linkTarget(item: MediaItem): string {
  if (item.item_type === "movie" || item.item_type === "episode") {
    return `/play/${item.id}`;
  }
  return `/detail/${item.id}`;
}

export function MediaCard(props: MediaCardProps) {
  const mode = () => props.viewMode ?? "grid";

  return (
    <Show when={mode() === "grid"} fallback={
      <A href={linkTarget(props.item)} class={styles.listCard}>
        <img
          class={styles.listPoster}
          src={posterUrl(props.item)}
          alt=""
          loading="lazy"
          onError={(e) => { (e.currentTarget as HTMLImageElement).style.display = "none"; }}
        />
        <div class={styles.listInfo}>
          <div class={styles.listTitle}>{props.item.title}</div>
          <div class={styles.listMeta}>
            <Show when={props.item.year}><span>{props.item.year}</span></Show>
            <Show when={props.item.runtime_seconds}>
              <span>{formatRuntime(props.item.runtime_seconds)}</span>
            </Show>
            <Show when={props.item.community_rating}>
              <span>{props.item.community_rating?.toFixed(1)}</span>
            </Show>
          </div>
        </div>
      </A>
    }>
      <A href={linkTarget(props.item)} class={styles.card}>
        <div class={styles.posterWrap}>
          <img
            class={styles.poster}
            src={posterUrl(props.item)}
            alt={props.item.title}
            loading="lazy"
            onError={(e) => {
              (e.currentTarget as HTMLImageElement).style.display = "none";
              const placeholder = e.currentTarget.nextElementSibling as HTMLElement;
              if (placeholder) placeholder.style.display = "flex";
            }}
          />
          <div class={styles.posterPlaceholder} style="display: none;">
            {props.item.title.charAt(0).toUpperCase()}
          </div>

          <div class={styles.overlay}>
            <div class={styles.overlayTitle}>{props.item.title}</div>
            <Show when={props.item.year}>
              <div class={styles.overlayYear}>{props.item.year}</div>
            </Show>
            <Show when={props.item.community_rating}>
              <div class={styles.overlayRating}>
                {props.item.community_rating?.toFixed(1)}
              </div>
            </Show>
          </div>

          <Show when={props.progress && props.progress > 0}>
            <div class={styles.progressBar}>
              <div
                class={styles.progressFill}
                style={{ width: `${(props.progress ?? 0) * 100}%` }}
              />
            </div>
          </Show>
        </div>

        <div class={styles.info}>
          <div class={styles.title}>{props.item.title}</div>
          <Show when={props.item.year}>
            <div class={styles.year}>{props.item.year}</div>
          </Show>
        </div>
      </A>
    </Show>
  );
}
