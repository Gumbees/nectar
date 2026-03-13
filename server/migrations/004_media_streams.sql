-- Media streams (video, audio, subtitle tracks within a file)

CREATE TABLE media_streams (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    media_item_id UUID NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    stream_index INTEGER NOT NULL,
    stream_type stream_type NOT NULL,
    codec TEXT,
    codec_long TEXT,
    profile TEXT,
    language TEXT,
    title TEXT,
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    is_forced BOOLEAN NOT NULL DEFAULT FALSE,
    is_external BOOLEAN NOT NULL DEFAULT FALSE,
    external_path TEXT,

    -- Video-specific fields
    width INTEGER,
    height INTEGER,
    bitrate BIGINT,
    framerate REAL,
    pixel_format TEXT,
    color_space TEXT,
    color_transfer TEXT,
    color_primaries TEXT,
    hdr_format TEXT,

    -- Audio-specific fields
    channels INTEGER,
    channel_layout TEXT,
    sample_rate INTEGER,

    -- Subtitle-specific fields
    subtitle_format TEXT
);

CREATE INDEX idx_media_streams_media_item ON media_streams(media_item_id);
CREATE INDEX idx_media_streams_type ON media_streams(stream_type);
CREATE INDEX idx_media_streams_language ON media_streams(language);
