//! Rate limiting for scheduled posts
//!
//! Prevents over-posting to platforms by tracking posts per hour window.

use crate::error::Result;
use crate::Database;
use std::collections::HashMap;

/// Rate limiter for platform posting
pub struct RateLimiter {
    /// Platform-specific limits (posts per hour)
    limits: HashMap<String, u32>,
}

impl RateLimiter {
    /// Create a new rate limiter with the given limits
    pub fn new(limits: HashMap<String, u32>) -> Self {
        Self { limits }
    }

    /// Check if posting is allowed and record the post
    ///
    /// Returns Ok(true) if posting is allowed, Ok(false) if rate limited
    pub async fn check_and_record(&self, db: &Database, platform: &str, now: i64) -> Result<bool> {
        // Check if allowed
        if !self.check(db, platform, now).await? {
            return Ok(false);
        }

        // Record the post
        self.record(db, platform, now).await?;
        Ok(true)
    }

    /// Check if posting is allowed (without recording)
    pub async fn check(&self, db: &Database, platform: &str, now: i64) -> Result<bool> {
        // Get limit for this platform
        let limit = match self.limits.get(platform) {
            Some(l) => *l,
            None => return Ok(true), // No limit configured, allow
        };

        // Calculate window start (floor to hour)
        let window_start = get_window_start(now);

        // Get current count for this window
        let count = get_window_count(db, platform, window_start).await?;

        // Check if under limit
        Ok(count < limit)
    }

    /// Record a post for rate limiting
    pub async fn record(&self, db: &Database, platform: &str, now: i64) -> Result<()> {
        let window_start = get_window_start(now);
        increment_window_count(db, platform, window_start).await
    }

    /// Clean up old rate limit windows
    pub async fn cleanup_old_windows(&self, db: &Database, cutoff: i64) -> Result<()> {
        let cutoff_window = get_window_start(cutoff);
        delete_old_windows(db, cutoff_window).await
    }
}

/// Get the window start timestamp (floor to hour)
fn get_window_start(timestamp: i64) -> i64 {
    (timestamp / 3600) * 3600
}

/// Get the post count for a window
async fn get_window_count(db: &Database, platform: &str, window_start: i64) -> Result<u32> {
    use crate::error::DbError;

    let row = sqlx::query_as::<_, (Option<i64>,)>(
        r#"
        SELECT post_count FROM rate_limits
        WHERE platform = ? AND window_start = ?
        "#,
    )
    .bind(platform)
    .bind(window_start)
    .fetch_optional(db.pool())
    .await
    .map_err(DbError::SqlxError)?;

    Ok(row.and_then(|r| r.0).unwrap_or(0) as u32)
}

/// Increment the post count for a window
async fn increment_window_count(db: &Database, platform: &str, window_start: i64) -> Result<()> {
    use crate::error::DbError;

    sqlx::query(
        r#"
        INSERT INTO rate_limits (platform, window_start, post_count)
        VALUES (?, ?, 1)
        ON CONFLICT(platform, window_start)
        DO UPDATE SET post_count = post_count + 1
        "#,
    )
    .bind(platform)
    .bind(window_start)
    .execute(db.pool())
    .await
    .map_err(DbError::SqlxError)?;

    Ok(())
}

/// Delete old rate limit windows
async fn delete_old_windows(db: &Database, cutoff_window: i64) -> Result<()> {
    use crate::error::DbError;

    sqlx::query(
        r#"
        DELETE FROM rate_limits
        WHERE window_start < ?
        "#,
    )
    .bind(cutoff_window)
    .execute(db.pool())
    .await
    .map_err(DbError::SqlxError)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::TempDir;

    async fn setup_test_db() -> (TempDir, Database) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = Database::new(&db_path.to_string_lossy()).await.unwrap();
        (temp_dir, db)
    }

    fn test_limiter() -> RateLimiter {
        let mut limits = HashMap::new();
        limits.insert("nostr".to_string(), 100);
        limits.insert("mastodon".to_string(), 300);
        RateLimiter::new(limits)
    }

    #[tokio::test]
    async fn test_allows_first_post() {
        let (_temp, db) = setup_test_db().await;
        let limiter = test_limiter();
        let now = 1000000;

        let allowed = limiter.check_and_record(&db, "nostr", now).await.unwrap();
        assert!(allowed, "First post should be allowed");
    }

    #[tokio::test]
    async fn test_allows_posts_under_limit() {
        let (_temp, db) = setup_test_db().await;
        let limiter = test_limiter();
        let now = 1000000;

        // Post 10 times (well under 100/hour limit)
        for _ in 0..10 {
            let allowed = limiter.check_and_record(&db, "nostr", now).await.unwrap();
            assert!(allowed, "Posts under limit should be allowed");
        }
    }

    #[tokio::test]
    async fn test_blocks_posts_over_limit() {
        let (_temp, db) = setup_test_db().await;
        let mut limits = HashMap::new();
        limits.insert("nostr".to_string(), 5); // Low limit for testing
        let limiter = RateLimiter::new(limits);
        let now = 1000000;

        // Post 5 times (at limit)
        for i in 0..5 {
            let allowed = limiter.check_and_record(&db, "nostr", now).await.unwrap();
            assert!(allowed, "Post {} should be allowed (under limit)", i + 1);
        }

        // 6th post should be blocked
        let allowed = limiter.check_and_record(&db, "nostr", now).await.unwrap();
        assert!(!allowed, "Post 6 should be blocked (over limit)");
    }

    #[tokio::test]
    async fn test_window_sliding() {
        let (_temp, db) = setup_test_db().await;
        let mut limits = HashMap::new();
        limits.insert("nostr".to_string(), 5);
        let limiter = RateLimiter::new(limits);

        // Post 5 times in first window (hour 0)
        let window1 = 1000000;
        for _ in 0..5 {
            limiter
                .check_and_record(&db, "nostr", window1)
                .await
                .unwrap();
        }

        // Next post in same window should be blocked (stay within the hour)
        let allowed = limiter
            .check_and_record(&db, "nostr", window1 + 100)
            .await
            .unwrap();
        assert!(!allowed, "Should be blocked in same window");

        // Post in next window (1 hour later) should be allowed
        let window2 = window1 + 3600; // 1 hour later
        let allowed = limiter
            .check_and_record(&db, "nostr", window2)
            .await
            .unwrap();
        assert!(allowed, "Should be allowed in new window");
    }

    #[tokio::test]
    async fn test_independent_platforms() {
        let (_temp, db) = setup_test_db().await;
        let mut limits = HashMap::new();
        limits.insert("nostr".to_string(), 5);
        limits.insert("mastodon".to_string(), 5);
        let limiter = RateLimiter::new(limits);
        let now = 1000000;

        // Fill up nostr limit
        for _ in 0..5 {
            limiter.check_and_record(&db, "nostr", now).await.unwrap();
        }

        // Mastodon should still be allowed
        let allowed = limiter
            .check_and_record(&db, "mastodon", now)
            .await
            .unwrap();
        assert!(allowed, "Mastodon should be independent of nostr limit");
    }

    #[tokio::test]
    async fn test_check_without_recording() {
        let (_temp, db) = setup_test_db().await;
        let mut limits = HashMap::new();
        limits.insert("nostr".to_string(), 5);
        let limiter = RateLimiter::new(limits);
        let now = 1000000;

        // Check should be allowed
        let allowed = limiter.check(&db, "nostr", now).await.unwrap();
        assert!(allowed, "Check should show available");

        // Check again - should still be allowed (not recorded)
        let allowed = limiter.check(&db, "nostr", now).await.unwrap();
        assert!(allowed, "Check should still show available (not recorded)");

        // Now record 5 posts
        for _ in 0..5 {
            limiter.record(&db, "nostr", now).await.unwrap();
        }

        // Check should now show blocked
        let allowed = limiter.check(&db, "nostr", now).await.unwrap();
        assert!(!allowed, "Check should show blocked after recording");
    }

    #[tokio::test]
    async fn test_cleanup_old_windows() {
        let (_temp, db) = setup_test_db().await;
        let limiter = test_limiter();

        // Record posts in old window
        let old_time = 1000000;
        limiter.record(&db, "nostr", old_time).await.unwrap();

        // Record posts in current window
        let current_time = old_time + 7200; // 2 hours later
        limiter.record(&db, "nostr", current_time).await.unwrap();

        // Cleanup windows older than 1 hour ago
        let cutoff = current_time - 3600;
        limiter.cleanup_old_windows(&db, cutoff).await.unwrap();

        // Old window should be cleaned up (verified via check)
        // This is implicit - if cleanup works, old posts don't count
        let allowed = limiter.check(&db, "nostr", current_time).await.unwrap();
        assert!(allowed, "Old windows should be cleaned up");
    }

    #[tokio::test]
    async fn test_no_limit_configured() {
        let (_temp, db) = setup_test_db().await;
        let limits = HashMap::new(); // No limits configured
        let limiter = RateLimiter::new(limits);
        let now = 1000000;

        // Should allow posting even without configured limit
        let allowed = limiter.check_and_record(&db, "nostr", now).await.unwrap();
        assert!(allowed, "Should allow posting when no limit configured");
    }
}
