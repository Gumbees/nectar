-- Watch history and collections

-- Watch history
CREATE TABLE watch_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    media_item_id UUID NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    watched_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    duration_seconds INTEGER,
    play_method TEXT
);

CREATE INDEX idx_watch_history_user ON watch_history(user_id);
CREATE INDEX idx_watch_history_media ON watch_history(media_item_id);
CREATE INDEX idx_watch_history_watched_at ON watch_history(watched_at);

-- Collections
CREATE TABLE collections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    is_smart BOOLEAN NOT NULL DEFAULT FALSE,
    filter_rules JSONB,
    sort_order TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_collections_user ON collections(user_id);

CREATE TRIGGER trg_collections_updated_at
    BEFORE UPDATE ON collections
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Collection items
CREATE TABLE collection_items (
    collection_id UUID NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
    media_item_id UUID NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    sort_index INTEGER NOT NULL DEFAULT 0,
    added_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (collection_id, media_item_id)
);

CREATE INDEX idx_collection_items_media ON collection_items(media_item_id);
