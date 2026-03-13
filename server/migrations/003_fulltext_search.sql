-- Full-text search on media_items

-- Add tsvector column for full-text search
ALTER TABLE media_items ADD COLUMN search_vector tsvector;

-- GIN index on the search vector
CREATE INDEX idx_media_items_search_vector ON media_items USING GIN (search_vector);

-- Trigram GIN index on title for fuzzy matching
CREATE INDEX idx_media_items_title_trgm ON media_items USING GIN (title gin_trgm_ops);

-- Auto-update search_vector on insert/update
CREATE OR REPLACE FUNCTION media_items_search_vector_update()
RETURNS TRIGGER AS $$
BEGIN
    NEW.search_vector :=
        setweight(to_tsvector('english', COALESCE(NEW.title, '')), 'A') ||
        setweight(to_tsvector('english', COALESCE(NEW.original_title, '')), 'B') ||
        setweight(to_tsvector('english', COALESCE(NEW.overview, '')), 'C');
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_media_items_search_vector
    BEFORE INSERT OR UPDATE OF title, original_title, overview ON media_items
    FOR EACH ROW EXECUTE FUNCTION media_items_search_vector_update();

-- Backfill existing rows
UPDATE media_items SET search_vector =
    setweight(to_tsvector('english', COALESCE(title, '')), 'A') ||
    setweight(to_tsvector('english', COALESCE(original_title, '')), 'B') ||
    setweight(to_tsvector('english', COALESCE(overview, '')), 'C');
