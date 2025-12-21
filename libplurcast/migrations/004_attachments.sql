-- Attachments support for image uploads
-- Migration 004: Add tables for tracking attachments and their platform-specific upload status

-- Attachments table: stores metadata about attached image files
-- Files are stored on disk, not in the database (file_path references the local file)
CREATE TABLE IF NOT EXISTS attachments (
    id TEXT PRIMARY KEY,                   -- UUID v4 (same format as posts.id)
    post_id TEXT NOT NULL,                 -- FK to posts.id
    file_path TEXT NOT NULL,               -- Absolute path to the image file on disk
    mime_type TEXT NOT NULL,               -- image/jpeg, image/png, image/gif, image/webp
    file_size INTEGER NOT NULL,            -- Size in bytes
    file_hash TEXT NOT NULL,               -- SHA-256 hash of file content (hex encoded)
    alt_text TEXT,                         -- Optional accessibility description
    created_at INTEGER NOT NULL,           -- Unix timestamp
    FOREIGN KEY (post_id) REFERENCES posts(id) ON DELETE CASCADE
);

-- Index for querying attachments by post (most common query pattern)
CREATE INDEX IF NOT EXISTS idx_attachments_post_id ON attachments(post_id);

-- Index for finding duplicate files by hash (for potential deduplication)
CREATE INDEX IF NOT EXISTS idx_attachments_hash ON attachments(file_hash);

-- Attachment uploads table: tracks platform-specific upload status
-- A single attachment may have multiple upload records (one per platform)
-- This allows tracking which platforms have received the attachment
CREATE TABLE IF NOT EXISTS attachment_uploads (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    attachment_id TEXT NOT NULL,           -- FK to attachments.id
    platform TEXT NOT NULL,                -- Platform name: nostr, mastodon
    platform_attachment_id TEXT,           -- Platform-specific ID (e.g., Mastodon media_id)
    remote_url TEXT,                       -- URL after upload (for Nostr imeta tags)
    uploaded_at INTEGER,                   -- Unix timestamp when upload completed
    status TEXT DEFAULT 'pending',         -- pending, uploaded, failed
    error_message TEXT,                    -- Error details if upload failed
    FOREIGN KEY (attachment_id) REFERENCES attachments(id) ON DELETE CASCADE,
    UNIQUE(attachment_id, platform)        -- Only one upload record per attachment per platform
);

-- Index for querying uploads by attachment
CREATE INDEX IF NOT EXISTS idx_attachment_uploads_attachment_id
    ON attachment_uploads(attachment_id);

-- Index for querying uploads by platform and status (useful for retry logic)
CREATE INDEX IF NOT EXISTS idx_attachment_uploads_platform_status
    ON attachment_uploads(platform, status);
