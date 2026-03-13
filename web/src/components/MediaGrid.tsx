import { For, Show, createSignal } from "solid-js";
import type { MediaItem } from "../types";
import { MediaCard } from "./MediaCard";
import styles from "./MediaGrid.module.css";

interface MediaGridProps {
  items: MediaItem[];
  viewMode?: "grid" | "list";
  onViewChange?: (mode: "grid" | "list") => void;
  showViewToggle?: boolean;
}

export function MediaGrid(props: MediaGridProps) {
  const mode = () => props.viewMode ?? "grid";

  return (
    <div class={styles.container}>
      <Show when={props.showViewToggle}>
        <div class={styles.header}>
          <div class={styles.viewToggle}>
            <button
              class={`${styles.viewBtn} ${mode() === "grid" ? styles.viewBtnActive : ""}`}
              onClick={() => props.onViewChange?.("grid")}
              title="Grid view"
            >
              Grid
            </button>
            <button
              class={`${styles.viewBtn} ${mode() === "list" ? styles.viewBtnActive : ""}`}
              onClick={() => props.onViewChange?.("list")}
              title="List view"
            >
              List
            </button>
          </div>
        </div>
      </Show>

      <Show when={props.items.length > 0} fallback={
        <div class={styles.empty}>No items found</div>
      }>
        <div class={mode() === "grid" ? styles.grid : styles.list}>
          <For each={props.items}>
            {(item) => <MediaCard item={item} viewMode={mode()} />}
          </For>
        </div>
      </Show>
    </div>
  );
}
