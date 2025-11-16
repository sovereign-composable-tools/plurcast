-- Multi-account tracking and query optimization
-- Adds account tracking to post records and improves query performance

-- Add account_name column to track which account was used for posting
-- Default to 'default' for backward compatibility with existing records
ALTER TABLE post_records ADD COLUMN account_name TEXT NOT NULL DEFAULT 'default';

-- Index for querying posts by account
-- Useful for filtering history by specific account
CREATE INDEX IF NOT EXISTS idx_post_records_account
ON post_records(account_name, platform);

-- Index for querying failed posts
-- Used by plur-queue failed list and error analysis
CREATE INDEX IF NOT EXISTS idx_post_records_success
ON post_records(success, platform)
WHERE success = 0;

-- Index for querying by platform and success status
-- Optimizes queries that filter by both platform and success
CREATE INDEX IF NOT EXISTS idx_post_records_platform_success
ON post_records(platform, success, posted_at);

-- Note: The account_name field tracks which account was used
-- This allows users to:
-- 1. Filter post history by account
-- 2. Analyze success rates per account
-- 3. Identify which account posted what content
-- 4. Support account-specific retry logic
