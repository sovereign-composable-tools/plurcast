-- NIP-65 relay list tracking
-- Tracks when relay lists (kind 10002) were last published per pubkey
-- This enables auto-publishing relay lists with staleness detection

-- Relay list metadata per pubkey
-- pubkey is stored as hex (64 chars) for simplicity
CREATE TABLE IF NOT EXISTS relay_list_metadata (
    pubkey TEXT PRIMARY KEY,
    last_published_at INTEGER NOT NULL,
    relay_count INTEGER NOT NULL DEFAULT 0
);

-- Index for efficient staleness checks
CREATE INDEX IF NOT EXISTS idx_relay_list_published
ON relay_list_metadata(last_published_at);
