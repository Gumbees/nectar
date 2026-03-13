import { createSignal, createResource, createEffect, on, onMount, onCleanup, For, Show } from "solid-js";
import { useParams } from "@solidjs/router";
import { fetchMediaItems, fetchLibraries } from "../lib/api";
import type { MediaItem, Library as LibraryType } from "../types";
import { MediaGrid } from "../components/MediaGrid";
import styles from "./Library.module.css";

const SORT_OPTIONS = [
  { value: "title", label: "Title" },
  { value: "year", label: "Year" },
  { value: "date_added", label: "Date Added" },
  { value: "community_rating", label: "Rating" },
];

const PAGE_SIZE = 40;

export function Library() {
  const params = useParams();

  const [sort, setSort] = createSignal("date_added");
  const [order, setOrder] = createSignal<"asc" | "desc">("desc");
  const [viewMode, setViewMode] = createSignal<"grid" | "list">("grid");
  const [activeGenre, setActiveGenre] = createSignal<string | null>(null);
  const [items, setItems] = createSignal<MediaItem[]>([]);
  const [total, setTotal] = createSignal(0);
  const [offset, setOffset] = createSignal(0);
  const [loading, setLoading] = createSignal(false);

  // Genres are deduced from results or could be fetched
  const [genres, setGenres] = createSignal<string[]>([]);

  const [librariesData] = createResource(fetchLibraries);
  const currentLibrary = () => librariesData()?.libraries?.find((l: LibraryType) => l.id === params.id);

  const loadItems = async (append = false) => {
    setLoading(true);
    try {
      const result = await fetchMediaItems({
        library: params.id,
        sort: sort(),
        order: order(),
        genre: activeGenre() ?? undefined,
        limit: PAGE_SIZE,
        offset: append ? offset() : 0,
      });
      if (append) {
        setItems((prev) => [...prev, ...(result.items ?? [])]);
      } else {
        setItems(result.items ?? []);
      }
      setTotal(result.total ?? 0);
      setOffset((append ? offset() : 0) + (result.items?.length ?? 0));
    } catch (e) {
      console.error("Failed to load media items:", e);
    } finally {
      setLoading(false);
    }
  };

  // Reload when params/sort/genre change
  createEffect(on([() => params.id, sort, order, activeGenre], () => {
    loadItems(false);
  }));

  // Infinite scroll via IntersectionObserver
  let sentinelRef!: HTMLDivElement;

  onMount(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting && !loading() && items().length < total()) {
          loadItems(true);
        }
      },
      { rootMargin: "200px" }
    );

    // Small delay to let the ref be assigned
    setTimeout(() => {
      if (sentinelRef) observer.observe(sentinelRef);
    }, 100);

    onCleanup(() => observer.disconnect());
  });

  const hasMore = () => items().length < total();

  return (
    <div class={styles.page}>
      <div class={styles.header}>
        <h1 class={styles.title}>{currentLibrary()?.name ?? "Library"}</h1>
        <div class={styles.controls}>
          <select
            class={styles.sortSelect}
            value={sort()}
            onChange={(e) => setSort(e.currentTarget.value)}
          >
            <For each={SORT_OPTIONS}>
              {(opt) => <option value={opt.value}>{opt.label}</option>}
            </For>
          </select>

          <button
            class={styles.sortSelect}
            onClick={() => setOrder(order() === "asc" ? "desc" : "asc")}
            title={order() === "asc" ? "Ascending" : "Descending"}
          >
            {order() === "asc" ? "\u2191" : "\u2193"}
          </button>
        </div>
      </div>

      <Show when={genres().length > 0}>
        <div class={styles.genreChips}>
          <button
            class={`${styles.genreChip} ${activeGenre() === null ? styles.genreChipActive : ""}`}
            onClick={() => setActiveGenre(null)}
          >
            All
          </button>
          <For each={genres()}>
            {(genre) => (
              <button
                class={`${styles.genreChip} ${activeGenre() === genre ? styles.genreChipActive : ""}`}
                onClick={() => setActiveGenre(genre === activeGenre() ? null : genre)}
              >
                {genre}
              </button>
            )}
          </For>
        </div>
      </Show>

      <Show when={!loading() || items().length > 0} fallback={
        <div class={styles.loading}>Loading...</div>
      }>
        <MediaGrid
          items={items()}
          viewMode={viewMode()}
          onViewChange={setViewMode}
          showViewToggle
        />
      </Show>

      <Show when={hasMore() && !loading()}>
        <div class={styles.loadMore}>
          <button class={styles.loadMoreBtn} onClick={() => loadItems(true)}>
            Load more
          </button>
        </div>
      </Show>

      <div ref={sentinelRef} class={styles.sentinel} />
    </div>
  );
}
