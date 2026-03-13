-- People, genres, studios, and tags with junction tables

-- People
CREATE TABLE people (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    sort_name TEXT,
    thumb_path TEXT,
    thumb_url TEXT,
    tmdb_id INTEGER,
    imdb_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_people_name_trgm ON people USING GIN (name gin_trgm_ops);

CREATE TRIGGER trg_people_updated_at
    BEFORE UPDATE ON people
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Media-People junction
CREATE TABLE media_people (
    media_item_id UUID NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    person_id UUID NOT NULL REFERENCES people(id) ON DELETE CASCADE,
    role person_role NOT NULL,
    character_name TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (media_item_id, person_id, role)
);

CREATE INDEX idx_media_people_person ON media_people(person_id);

-- Genres
CREATE TABLE genres (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    slug TEXT NOT NULL UNIQUE
);

CREATE TABLE media_genres (
    media_item_id UUID NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    genre_id UUID NOT NULL REFERENCES genres(id) ON DELETE CASCADE,
    PRIMARY KEY (media_item_id, genre_id)
);

CREATE INDEX idx_media_genres_genre ON media_genres(genre_id);

-- Studios
CREATE TABLE studios (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    logo_path TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE media_studios (
    media_item_id UUID NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    studio_id UUID NOT NULL REFERENCES studios(id) ON DELETE CASCADE,
    PRIMARY KEY (media_item_id, studio_id)
);

CREATE INDEX idx_media_studios_studio ON media_studios(studio_id);

-- Tags
CREATE TABLE tags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    slug TEXT NOT NULL UNIQUE
);

CREATE TABLE media_tags (
    media_item_id UUID NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    tag_id UUID NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (media_item_id, tag_id)
);

CREATE INDEX idx_media_tags_tag ON media_tags(tag_id);
