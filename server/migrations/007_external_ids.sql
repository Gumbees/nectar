-- External IDs (TMDB, IMDB, TVDB, etc.)

CREATE TABLE external_ids (
    media_item_id UUID NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    provider external_provider NOT NULL,
    external_id TEXT NOT NULL,
    PRIMARY KEY (media_item_id, provider)
);

CREATE INDEX idx_external_ids_lookup ON external_ids(provider, external_id);
