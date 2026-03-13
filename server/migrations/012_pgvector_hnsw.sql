-- Switch from IVFFlat to HNSW index, change embedding dimension to 768

-- Drop the old IVFFlat index
DROP INDEX IF EXISTS idx_media_items_embedding;

-- Change embedding dimension from 1536 to 768
ALTER TABLE media_items DROP COLUMN embedding;
ALTER TABLE media_items ADD COLUMN embedding vector(768);

-- Create HNSW index for better recall and no training requirement
CREATE INDEX idx_media_items_embedding ON media_items
    USING hnsw (embedding vector_cosine_ops)
    WITH (m = 16, ef_construction = 200);
