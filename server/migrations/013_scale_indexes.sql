-- Scale-focused covering and partial indexes

-- Covering index on playback_progress for fast user+item lookup
CREATE INDEX idx_playback_progress_covering
    ON playback_progress(user_id, media_item_id)
    INCLUDE (completed, position_seconds);

-- Partial index for in-progress items (not completed)
CREATE INDEX idx_playback_progress_in_progress
    ON playback_progress(user_id, media_item_id)
    WHERE completed = FALSE;

-- Partial index on media_items for items with files (scannable)
CREATE INDEX idx_media_items_with_file
    ON media_items(library_id, item_type)
    WHERE file_path IS NOT NULL;

-- Subtitle stream lookup (common query pattern)
CREATE INDEX idx_media_streams_subtitle_lookup
    ON media_streams(media_item_id, stream_type, language)
    WHERE stream_type = 'subtitle';

-- Watch history popularity (for "most watched" queries)
CREATE INDEX idx_watch_history_popularity
    ON watch_history(media_item_id, watched_at DESC);
