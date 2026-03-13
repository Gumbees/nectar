import { For, Show } from "solid-js";
import type { Person } from "../types";
import styles from "./CastRow.module.css";

interface CastRowProps {
  people: Person[];
}

export function CastRow(props: CastRowProps) {
  return (
    <Show when={props.people.length > 0}>
      <div class={styles.container}>
        <h3 class={styles.header}>Cast</h3>
        <div class={styles.row}>
          <For each={props.people}>
            {(person) => (
              <div class={styles.person}>
                <div class={styles.avatar}>
                  <Show when={person.thumb_path} fallback={
                    <div class={styles.avatarPlaceholder}>
                      {person.name.charAt(0).toUpperCase()}
                    </div>
                  }>
                    <img
                      class={styles.avatarImg}
                      src={person.thumb_path}
                      alt={person.name}
                      loading="lazy"
                      onError={(e) => {
                        const el = e.currentTarget as HTMLImageElement;
                        el.style.display = "none";
                        const placeholder = el.parentElement?.querySelector(`.${styles.avatarPlaceholder}`) as HTMLElement;
                        if (placeholder) placeholder.style.display = "flex";
                      }}
                    />
                  </Show>
                </div>
                <div class={styles.name}>{person.name}</div>
                <Show when={person.character_name}>
                  <div class={styles.character}>{person.character_name}</div>
                </Show>
              </div>
            )}
          </For>
        </div>
      </div>
    </Show>
  );
}
