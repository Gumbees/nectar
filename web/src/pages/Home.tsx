import { createResource, For, Show, Suspense } from "solid-js";
import { fetchLibraries } from "../lib/api";
import { ContinueWatching } from "../components/ContinueWatching";
import { LibraryRow } from "../components/LibraryRow";
import styles from "./Home.module.css";

export function Home() {
  const [librariesData] = createResource(fetchLibraries);

  const libraries = () => librariesData()?.libraries ?? [];

  return (
    <div class={styles.page}>
      <ContinueWatching />

      <Show when={!librariesData.loading} fallback={
        <div class={styles.loading}>Loading libraries...</div>
      }>
        <Show when={libraries().length > 0} fallback={
          <div class={styles.empty}>
            <div class={styles.emptyTitle}>No libraries yet</div>
            <div class={styles.emptyText}>Add a library in Settings to get started.</div>
          </div>
        }>
          <For each={libraries()}>
            {(lib) => <LibraryRow library={lib} />}
          </For>
        </Show>
      </Show>
    </div>
  );
}
