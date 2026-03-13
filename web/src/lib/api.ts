import type {
  AuthResponse,
  User,
  Library,
  MediaItem,
  MediaStream,
  SearchResult,
  SemanticSearchResult,
  Person,
  Genre,
} from "../types";

export async function api<T = any>(url: string, options?: RequestInit): Promise<T> {
  const token = localStorage.getItem("nectar_token");

  const res = await fetch(url, {
    ...options,
    headers: {
      "Content-Type": "application/json",
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
      ...options?.headers,
    },
  });

  if (res.status === 401) {
    localStorage.removeItem("nectar_token");
    if (window.location.pathname !== "/login") {
      window.location.href = "/login";
    }
    throw new Error("Unauthorized");
  }

  if (!res.ok) {
    const body = await res.json().catch(() => ({ error: `API error: ${res.status}` }));
    throw new Error(body.error || `API error: ${res.status}`);
  }

  // Handle 204 No Content
  if (res.status === 204) return undefined as T;

  return res.json();
}

// --- Auth ---

export function login(username: string, password: string): Promise<AuthResponse> {
  return api<AuthResponse>("/api/v1/auth/login", {
    method: "POST",
    body: JSON.stringify({ username, password }),
  });
}

export function register(username: string, password: string, email?: string): Promise<AuthResponse> {
  return api<AuthResponse>("/api/v1/auth/register", {
    method: "POST",
    body: JSON.stringify({ username, password, email }),
  });
}

export function fetchMe(): Promise<User> {
  return api<User>("/api/v1/auth/me");
}

// --- Libraries ---

export function fetchLibraries(): Promise<{ libraries: Library[] }> {
  return api<{ libraries: Library[] }>("/api/v1/libraries");
}

// --- Media ---

export interface FetchMediaParams {
  library?: string;
  parent?: string;
  type?: string;
  sort?: string;
  order?: "asc" | "desc";
  limit?: number;
  offset?: number;
  genre?: string;
}

export function fetchMediaItems(params: FetchMediaParams = {}): Promise<{ items: MediaItem[]; total: number }> {
  const qs = new URLSearchParams();
  if (params.library) qs.set("library", params.library);
  if (params.parent) qs.set("parent", params.parent);
  if (params.type) qs.set("type", params.type);
  if (params.sort) qs.set("sort", params.sort);
  if (params.order) qs.set("order", params.order);
  if (params.limit) qs.set("limit", String(params.limit));
  if (params.offset) qs.set("offset", String(params.offset));
  if (params.genre) qs.set("genre", params.genre);
  return api<{ items: MediaItem[]; total: number }>(`/api/v1/media?${qs.toString()}`);
}

export function fetchMediaItem(id: string): Promise<MediaItem> {
  return api<MediaItem>(`/api/v1/media/${id}`);
}

export function fetchMediaStreams(id: string): Promise<MediaStream[]> {
  return api<MediaStream[]>(`/api/v1/media/${id}/streams`);
}

export function fetchMediaPeople(id: string): Promise<{ people: Person[] }> {
  return api<{ people: Person[] }>(`/api/v1/media/${id}/people`);
}

export function fetchMediaGenres(id: string): Promise<{ genres: Genre[] }> {
  return api<{ genres: Genre[] }>(`/api/v1/media/${id}/genres`);
}

// --- Search ---

export function searchMedia(query: string, semantic?: boolean): Promise<SearchResult | SemanticSearchResult> {
  const endpoint = semantic ? "/api/v1/search/semantic" : "/api/v1/search";
  return api<SearchResult | SemanticSearchResult>(`${endpoint}?q=${encodeURIComponent(query)}`);
}

// --- Playback ---

export function reportProgress(mediaId: string, seconds: number): Promise<void> {
  return api<void>(`/api/v1/media/${mediaId}/progress`, {
    method: "POST",
    body: JSON.stringify({ position_seconds: Math.floor(seconds) }),
  });
}

export function fetchContinueWatching(): Promise<{ items: MediaItem[] }> {
  return api<{ items: MediaItem[] }>("/api/v1/media/continue-watching");
}

// --- Admin ---

export function createLibrary(name: string, library_type: string, paths: string[]): Promise<Library> {
  return api<Library>("/api/v1/libraries", {
    method: "POST",
    body: JSON.stringify({ name, library_type, paths }),
  });
}

export function deleteLibrary(id: string): Promise<void> {
  return api<void>(`/api/v1/libraries/${id}`, { method: "DELETE" });
}

export function scanLibrary(id: string): Promise<void> {
  return api<void>(`/api/v1/libraries/${id}/scan`, { method: "POST" });
}
