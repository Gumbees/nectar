import { createSignal, For } from "solid-js";
import type { MediaStream } from "../types";
import styles from "./SubtitleSelector.module.css";

interface SubtitleSelectorProps {
  streams: MediaStream[];
  onSelect: (stream: MediaStream | null) => void;
  onClose: () => void;
  selectedId?: string;
}

function languageLabel(stream: MediaStream): string {
  if (stream.title) return stream.title;
  if (stream.language) {
    // Common language code to name mapping
    const langs: Record<string, string> = {
      eng: "English", spa: "Spanish", fre: "French", fra: "French",
      deu: "German", ger: "German", ita: "Italian", por: "Portuguese",
      jpn: "Japanese", kor: "Korean", zho: "Chinese", chi: "Chinese",
      rus: "Russian", ara: "Arabic", hin: "Hindi", und: "Unknown",
    };
    return langs[stream.language] ?? stream.language;
  }
  return `Track ${stream.stream_index}`;
}

export function SubtitleSelector(props: SubtitleSelectorProps) {
  const [selected, setSelected] = createSignal<string | null>(props.selectedId ?? null);

  const handleSelect = (stream: MediaStream | null) => {
    setSelected(stream?.id ?? null);
    props.onSelect(stream);
  };

  return (
    <div class={styles.panel}>
      <div class={styles.header}>Subtitles</div>
      <ul class={styles.list}>
        <li>
          <button
            class={`${styles.item} ${selected() === null ? styles.itemActive : ""}`}
            onClick={() => handleSelect(null)}
          >
            <span class={`${styles.indicator} ${selected() !== null ? styles.indicatorHidden : ""}`} />
            <span class={styles.label}>Off</span>
          </button>
        </li>
        <For each={props.streams}>
          {(stream) => (
            <li>
              <button
                class={`${styles.item} ${selected() === stream.id ? styles.itemActive : ""}`}
                onClick={() => handleSelect(stream)}
              >
                <span class={`${styles.indicator} ${selected() !== stream.id ? styles.indicatorHidden : ""}`} />
                <span class={styles.label}>{languageLabel(stream)}</span>
                {stream.is_forced && <span class={styles.tag}>Forced</span>}
                {stream.codec && <span class={styles.tag}>{stream.codec.toUpperCase()}</span>}
              </button>
            </li>
          )}
        </For>
      </ul>
    </div>
  );
}
