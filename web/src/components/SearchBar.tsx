import { createSignal, createEffect, onCleanup } from "solid-js";
import styles from "./SearchBar.module.css";

interface SearchBarProps {
  onSearch: (query: string) => void;
  semantic: boolean;
  onSemanticChange: (enabled: boolean) => void;
  placeholder?: string;
}

export function SearchBar(props: SearchBarProps) {
  const [localQuery, setLocalQuery] = createSignal("");
  let debounceTimer: ReturnType<typeof setTimeout>;

  createEffect(() => {
    const q = localQuery();
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      props.onSearch(q);
    }, 300);
  });

  onCleanup(() => clearTimeout(debounceTimer));

  return (
    <div class={styles.container}>
      <div class={styles.inputWrap}>
        <span class={styles.icon}>&#x1F50D;</span>
        <input
          class={styles.input}
          type="text"
          placeholder={props.placeholder ?? "Search your library..."}
          value={localQuery()}
          onInput={(e) => setLocalQuery(e.currentTarget.value)}
        />
      </div>

      <div class={styles.toggle}>
        <label class={styles.toggleLabel}>
          Semantic
          <input
            type="checkbox"
            class={styles.hiddenCheckbox}
            checked={props.semantic}
            onChange={(e) => props.onSemanticChange(e.currentTarget.checked)}
          />
          <div
            class={`${styles.switch} ${props.semantic ? styles.switchActive : ""}`}
            role="switch"
            aria-checked={props.semantic}
          >
            <div class={styles.switchKnob} />
          </div>
        </label>
      </div>
    </div>
  );
}
