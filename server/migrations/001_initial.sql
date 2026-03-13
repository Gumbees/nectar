-- Enable pgvector for embedding similarity search
CREATE EXTENSION IF NOT EXISTS vector;

-- Custom enum types
CREATE TYPE library_type AS ENUM ('movies', 'shows', 'music', 'books', 'photos');
CREATE TYPE media_item_type AS ENUM ('movie', 'series', 'season', 'episode', 'album', 'track', 'book', 'photo');

-- Users
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username TEXT NOT NULL UNIQUE,
    email TEXT UNIQUE,
    password_hash TEXT,
    oidc_subject TEXT UNIQUE,
    is_admin BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Libraries
CREATE TABLE libraries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    library_type library_type NOT NULL,
    paths TEXT[] NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Media items (movies, episodes, tracks, etc.)
CREATE TABLE media_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    library_id UUID NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    parent_id UUID REFERENCES media_items(id) ON DELETE CASCADE,
    item_type media_item_type NOT NULL,
    title TEXT NOT NULL,
    sort_title TEXT,
    original_title TEXT,
    overview TEXT,
    year INTEGER,
    runtime_seconds INTEGER,
    file_path TEXT,
    container TEXT,
    video_codec TEXT,
    audio_codec TEXT,
    resolution TEXT,
    bitrate BIGINT,
    size_bytes BIGINT,
    embedding vector(1536),
    metadata_json JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Playback progress tracking
CREATE TABLE playback_progress (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    media_item_id UUID NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    position_seconds BIGINT NOT NULL DEFAULT 0,
    completed BOOLEAN NOT NULL DEFAULT FALSE,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (user_id, media_item_id)
);

-- Indexes
CREATE INDEX idx_media_items_library ON media_items(library_id);
CREATE INDEX idx_media_items_parent ON media_items(parent_id);
CREATE INDEX idx_media_items_type ON media_items(item_type);
CREATE INDEX idx_media_items_file_path ON media_items(file_path);
CREATE INDEX idx_playback_progress_user ON playback_progress(user_id);

-- Vector similarity index (IVFFlat for large collections)
CREATE INDEX idx_media_items_embedding ON media_items
    USING ivfflat (embedding vector_cosine_ops)
    WITH (lists = 100);
