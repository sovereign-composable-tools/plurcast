-- Initial database schema for Plurcast

-- Posts authored by user
CREATE TABLE IF NOT EXISTS posts (
    id TEXT PRIMARY KEY,
    content TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    scheduled_at INTEGER,
    status TEXT DEFAULT 'pending',
    metadata TEXT
);

CREATE INDEX IF NOT EXISTS idx_posts_created_at ON posts(created_at);
CREATE INDEX IF NOT EXISTS idx_posts_status ON posts(status);

-- Platform-specific post records
CREATE TABLE IF NOT EXISTS post_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    post_id TEXT NOT NULL,
    platform TEXT NOT NULL,
    platform_post_id TEXT,
    posted_at INTEGER,
    success INTEGER DEFAULT 0,
    error_message TEXT,
    FOREIGN KEY (post_id) REFERENCES posts(id)
);

CREATE INDEX IF NOT EXISTS idx_post_records_post_id ON post_records(post_id);
CREATE INDEX IF NOT EXISTS idx_post_records_platform ON post_records(platform);

-- Platform configurations
CREATE TABLE IF NOT EXISTS platforms (
    name TEXT PRIMARY KEY,
    enabled INTEGER DEFAULT 1,
    config TEXT
);
