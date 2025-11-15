-- Scheduling enhancements for Phase 5
-- Adds indexes and rate limiting support for scheduled posts

-- Index for efficient scheduled post queries
-- This index allows the daemon to quickly find posts that are due to be sent
-- WHERE clause ensures index only covers relevant rows (scheduled posts)
CREATE INDEX IF NOT EXISTS idx_posts_scheduled_at
ON posts(scheduled_at)
WHERE status = 'scheduled' AND scheduled_at IS NOT NULL;

-- Additional index for querying scheduled posts (for plur-queue list)
-- Composite index on status and scheduled_at for sorting scheduled posts
CREATE INDEX IF NOT EXISTS idx_posts_status_scheduled
ON posts(status, scheduled_at)
WHERE status = 'scheduled';

-- Rate limiting tracking table
-- Tracks how many posts have been made per platform per time window
-- Used by plur-send daemon to enforce rate limits
CREATE TABLE IF NOT EXISTS rate_limits (
    platform TEXT NOT NULL,
    window_start INTEGER NOT NULL,
    post_count INTEGER DEFAULT 0,
    PRIMARY KEY (platform, window_start)
);

-- Index for efficient rate limit queries
-- Daemon needs to quickly look up current count for a platform+window
CREATE INDEX IF NOT EXISTS idx_rate_limits_platform
ON rate_limits(platform, window_start);

-- Add metadata to track retry attempts (if needed)
-- Note: The posts table already has a 'metadata' TEXT column
-- We'll use JSON in that column to store retry information:
-- {
--   "retry_count": 0,
--   "last_retry_at": 1731747600,
--   "scheduled_by": "user",
--   "original_schedule": 1731747600
-- }
