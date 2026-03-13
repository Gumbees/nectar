-- Core infrastructure: extensions, enums, trigger function

-- Trigram extension for fuzzy text search
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- Enum types
CREATE TYPE image_type AS ENUM (
    'primary', 'backdrop', 'banner', 'logo', 'thumb', 'disc',
    'clearart', 'landscape', 'trickplay'
);

CREATE TYPE stream_type AS ENUM ('video', 'audio', 'subtitle', 'attachment');

CREATE TYPE job_status AS ENUM ('pending', 'running', 'completed', 'failed', 'cancelled');

CREATE TYPE scan_status AS ENUM ('pending', 'running', 'completed', 'failed');

CREATE TYPE external_provider AS ENUM ('tmdb', 'imdb', 'tvdb', 'musicbrainz', 'discogs', 'anidb');

CREATE TYPE person_role AS ENUM (
    'actor', 'director', 'writer', 'producer', 'composer',
    'creator', 'guest_star', 'artist'
);

-- Generic updated_at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply trigger to existing tables
CREATE TRIGGER trg_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER trg_libraries_updated_at
    BEFORE UPDATE ON libraries
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER trg_media_items_updated_at
    BEFORE UPDATE ON media_items
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER trg_playback_progress_updated_at
    BEFORE UPDATE ON playback_progress
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
