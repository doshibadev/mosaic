-- Add download count column
ALTER TABLE packages ADD COLUMN download_count BIGINT NOT NULL DEFAULT 0;
