export type LibraryType = 'movies' | 'shows' | 'music' | 'books' | 'photos';
export type MediaItemType = 'movie' | 'series' | 'season' | 'episode' | 'album' | 'track' | 'book' | 'photo';

export interface User {
  id: string;
  username: string;
  email?: string;
  is_admin: boolean;
  created_at: string;
  updated_at: string;
}

export interface Library {
  id: string;
  name: string;
  library_type: LibraryType;
  paths: string[];
  created_at: string;
  updated_at: string;
}

export interface MediaItem {
  id: string;
  library_id: string;
  parent_id?: string;
  item_type: MediaItemType;
  title: string;
  sort_title?: string;
  original_title?: string;
  overview?: string;
  year?: number;
  runtime_seconds?: number;
  file_path?: string;
  container?: string;
  video_codec?: string;
  audio_codec?: string;
  resolution?: string;
  bitrate?: number;
  size_bytes?: number;
  community_rating?: number;
  content_rating?: string;
  tagline?: string;
  premiere_date?: string;
  season_number?: number;
  episode_number?: number;
  date_added: string;
  created_at: string;
  updated_at: string;
}

export interface PlaybackProgress {
  id: string;
  user_id: string;
  media_item_id: string;
  position_seconds: number;
  completed: boolean;
  updated_at: string;
}

export interface MediaStream {
  id: string;
  media_item_id: string;
  stream_index: number;
  stream_type: 'video' | 'audio' | 'subtitle' | 'attachment';
  codec?: string;
  language?: string;
  title?: string;
  is_default: boolean;
  is_forced: boolean;
  width?: number;
  height?: number;
  channels?: number;
  channel_layout?: string;
}

export interface Person {
  id: string;
  name: string;
  character_name?: string;
  role: string;
  thumb_path?: string;
}

export interface Genre {
  id: string;
  name: string;
  slug: string;
}

export interface SearchResult {
  items: MediaItem[];
  total: number;
}

export interface SemanticSearchResult extends SearchResult {
  items: (MediaItem & { similarity: number })[];
}

export interface AuthResponse {
  token: string;
}

export interface ApiError {
  error: string;
}
