-- Images (posters, backdrops, trickplay tiles, etc.)

CREATE TABLE images (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    media_item_id UUID REFERENCES media_items(id) ON DELETE CASCADE,
    library_id UUID REFERENCES libraries(id) ON DELETE CASCADE,
    image_type image_type NOT NULL,
    path TEXT NOT NULL,
    source_url TEXT,
    width INTEGER,
    height INTEGER,
    format TEXT,
    blurhash TEXT,
    language TEXT,

    -- Trickplay-specific fields
    tile_width INTEGER,
    tile_height INTEGER,
    tiles_per_row INTEGER,
    interval_ms INTEGER
);

CREATE INDEX idx_images_media_item ON images(media_item_id);
CREATE INDEX idx_images_library ON images(library_id);
CREATE INDEX idx_images_type ON images(image_type);
