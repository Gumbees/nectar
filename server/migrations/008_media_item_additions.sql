-- Additional columns on media_items

-- Ratings and metadata
ALTER TABLE media_items ADD COLUMN community_rating REAL;
ALTER TABLE media_items ADD COLUMN critic_rating REAL;
ALTER TABLE media_items ADD COLUMN content_rating TEXT;
ALTER TABLE media_items ADD COLUMN tagline TEXT;
ALTER TABLE media_items ADD COLUMN premiere_date DATE;
ALTER TABLE media_items ADD COLUMN end_date DATE;

-- Season/episode fields
ALTER TABLE media_items ADD COLUMN season_number INTEGER;
ALTER TABLE media_items ADD COLUMN episode_number INTEGER;
ALTER TABLE media_items ADD COLUMN absolute_episode_number INTEGER;

-- Music fields
ALTER TABLE media_items ADD COLUMN track_number INTEGER;
ALTER TABLE media_items ADD COLUMN disc_number INTEGER;
ALTER TABLE media_items ADD COLUMN album_artist TEXT;

-- File tracking
ALTER TABLE media_items ADD COLUMN date_added TIMESTAMPTZ DEFAULT now();
ALTER TABLE media_items ADD COLUMN last_scanned_at TIMESTAMPTZ;
ALTER TABLE media_items ADD COLUMN file_mtime TIMESTAMPTZ;

-- Indexes for new columns
CREATE INDEX idx_media_items_year ON media_items(year);
CREATE INDEX idx_media_items_community_rating ON media_items(community_rating);
CREATE INDEX idx_media_items_premiere_date ON media_items(premiere_date);
CREATE INDEX idx_media_items_date_added ON media_items(date_added);
CREATE INDEX idx_media_items_season ON media_items(parent_id, season_number);
CREATE INDEX idx_media_items_episode ON media_items(parent_id, episode_number);
CREATE INDEX idx_media_items_type_date ON media_items(item_type, date_added DESC);
CREATE INDEX idx_media_items_library_type_sort ON media_items(library_id, item_type, sort_title);
