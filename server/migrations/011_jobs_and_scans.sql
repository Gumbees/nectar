-- Transcode jobs and library scans

-- Transcode jobs
CREATE TABLE transcode_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    media_item_id UUID NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    status job_status NOT NULL DEFAULT 'pending',
    input_path TEXT NOT NULL,
    output_path TEXT,
    output_format TEXT,
    target_resolution TEXT,
    target_bitrate BIGINT,
    hardware_accel TEXT,
    worker_id TEXT,
    progress_percent REAL DEFAULT 0,
    error_message TEXT,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_transcode_jobs_status ON transcode_jobs(status);
CREATE INDEX idx_transcode_jobs_media ON transcode_jobs(media_item_id);
CREATE INDEX idx_transcode_jobs_user ON transcode_jobs(user_id);

-- Library scans
CREATE TABLE library_scans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    library_id UUID NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    status scan_status NOT NULL DEFAULT 'pending',
    items_found INTEGER DEFAULT 0,
    items_added INTEGER DEFAULT 0,
    items_updated INTEGER DEFAULT 0,
    items_removed INTEGER DEFAULT 0,
    error_message TEXT,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ
);

CREATE INDEX idx_library_scans_library ON library_scans(library_id);
CREATE INDEX idx_library_scans_status ON library_scans(status);
