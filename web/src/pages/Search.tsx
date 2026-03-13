import { createSignal, createResource, Show } from "solid-js";
import { api } from "../lib/api";

export function Search() {
  const [query, setQuery] = createSignal("");
  const [useSemanticSearch, setUseSemantic] = createSignal(false);

  const [results] = createResource(
    () => ({ q: query(), semantic: useSemanticSearch() }),
    ({ q, semantic }) => {
      if (!q.trim()) return { results: [] };
      const endpoint = semantic ? "/api/v1/search/semantic" : "/api/v1/search";
      return api(`${endpoint}?q=${encodeURIComponent(q)}`);
    }
  );

  return (
    <div class="search-page">
      <div class="search-bar">
        <input
          type="text"
          placeholder="Search your library..."
          value={query()}
          onInput={(e) => setQuery(e.currentTarget.value)}
        />
        <label>
          <input
            type="checkbox"
            checked={useSemanticSearch()}
            onChange={(e) => setUseSemantic(e.currentTarget.checked)}
          />
          Semantic search
        </label>
      </div>
      <div class="results">
        {/* TODO: render search results */}
      </div>
    </div>
  );
}
