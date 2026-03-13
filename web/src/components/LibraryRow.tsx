import { createResource, For, Show } from "solid-js";
import { A } from "@solidjs/router";
import { fetchMediaItems } from "../lib/api";
import type { Library } from "../types";
import { MediaCard } from "./MediaCard";
import styles from "./LibraryRow.module.css";

interface LibraryRowProps {
  library: Library;
}

export function LibraryRow(props: LibraryRowProps) {
  const [data] = createResource(
    () => props.library.id,
    (id) => fetchMediaItems({ library: id, sort: "date_added", order: "desc", limit: 20 })
  );

  const items = () => data()?.items ?? [];

  return (
    <Show when={items().length > 0}>
      <div class={styles.container}>
        <div class={styles.header}>
          <A href={`/library/${props.library.id}`} class={styles.headerLink}>
            {props.library.name}
          </A>
          <A href={`/library/${props.library.id}`} class={styles.seeAll}>
            See all
          </A>
        </div>
        <div class={styles.row}>
          <For each={items()}>
            {(item) => (
              <div class={styles.item}>
                <MediaCard item={item} />
              </div>
            )}
          </For>
        </div>
      </div>
    </Show>
  );
}
