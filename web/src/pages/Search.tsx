import { createSignal, createResource, Show, For } from "solid-js";
import { searchMedia } from "../lib/api";
import type { MediaItem } from "../types";
import { SearchBar } from "../components/SearchBar";
import { MediaGrid } from "../components/MediaGrid";
import styles from "./Search.module.css";

const HISTORY_KEY = "nectar_search_history";
const MAX_HISTORY = 10;

function getSearchHistory(): string[] {
  try {
    return JSON.parse(localStorage.getItem(HISTORY_KEY) ?? "[]");
  } catch {
    return [];
  }
}

function addToHistory(query: string) {
  if (!query.trim()) return;
  const history = getSearchHistory().filter((h) => h !== query);
  history.unshift(query);
  localStorage.setItem(HISTORY_KEY, JSON.stringify(history.slice(0, MAX_HISTORY)));
}

export function Search() {
  const [query, setQuery] = createSignal("");
  const [semantic, setSemantic] = createSignal(false);
  const [viewMode, setViewMode] = createSignal<"grid" | "list">("grid");
  const [history, setHistory] = createSignal(getSearchHistory());

  const [results] = createResource(
    () => ({ q: query(), s: semantic() }),
    ({ q, s }) => {
      if (!q.trim()) return Promise.resolve({ items: [], total: 0 });
      return searchMedia(q, s);
    }
  );

  const items = () => results()?.items ?? [];
  const total = () => results()?.total ?? 0;

  const handleSearch = (q: string) => {
    setQuery(q);
    if (q.trim()) {
      addToHistory(q.trim());
      setHistory(getSearchHistory());
    }
  };

  const handleHistoryClick = (q: string) => {
    setQuery(q);
    handleSearch(q);
  };

  return (
    <div class={styles.page}>
      <div class={styles.header}>
        <h1 class={styles.title}>Search</h1>
        <SearchBar
          onSearch={handleSearch}
          semantic={semantic()}
          onSemanticChange={setSemantic}
        />
      </div>

      <Show when={semantic() && query().trim()}>
        <div class={styles.semanticBadge}>
          Semantic search active
        </div>
      </Show>

      <Show when={query().trim()}>
        <Show when={items().length > 0} fallback={
          <Show when={!results.loading}>
            <div class={styles.empty}>
              <div class={styles.emptyText}>No results found</div>
              <div class={styles.emptyHint}>Try a different search term{semantic() ? "" : " or enable semantic search"}</div>
            </div>
          </Show>
        }>
          <div class={styles.resultCount}>{total()} result{total() !== 1 ? "s" : ""}</div>
          <MediaGrid
            items={items()}
            viewMode={viewMode()}
            onViewChange={setViewMode}
            showViewToggle
          />
        </Show>
      </Show>

      <Show when={!query().trim() && history().length > 0}>
        <div class={styles.history}>
          <div class={styles.historyTitle}>Recent Searches</div>
          <div class={styles.historyList}>
            <For each={history()}>
              {(h) => (
                <button class={styles.historyItem} onClick={() => handleHistoryClick(h)}>
                  {h}
                </button>
              )}
            </For>
          </div>
        </div>
      </Show>

      <Show when={!query().trim() && history().length === 0}>
        <div class={styles.empty}>
          <div class={styles.emptyIcon}>{"\uD83D\uDD0D"}</div>
          <div class={styles.emptyText}>Search your library</div>
          <div class={styles.emptyHint}>Find movies, shows, music, and more</div>
        </div>
      </Show>
    </div>
  );
}
