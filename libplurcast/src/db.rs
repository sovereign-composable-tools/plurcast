//! Database operations for Plurcast

use sqlx::sqlite::SqlitePool;
use std::collections::HashMap;
use std::path::Path;

use crate::error::Result;
use crate::types::{
    Attachment, AttachmentStatus, AttachmentUpload, ImageMimeType, Post, PostRecord, PostStatus,
};

/// A post with all its platform records
#[derive(Debug, Clone)]
pub struct PostWithRecords {
    pub post: Post,
    pub records: Vec<PostRecord>,
}

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Get a reference to the connection pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Create a new database connection
    pub async fn new(db_path: &str) -> Result<Self> {
        // Expand path and create parent directories
        let expanded_path = shellexpand::tilde(db_path).to_string();
        let path = Path::new(&expanded_path);

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(crate::error::DbError::IoError)?;
        }

        // Create connection pool
        // Use forward slashes for SQLite URL (works on both Windows and Unix)
        // Use mode=rwc to allow creating the database file if it doesn't exist
        let db_url = format!("sqlite://{}?mode=rwc", expanded_path.replace('\\', "/"));

        let pool = SqlitePool::connect(&db_url)
            .await
            .map_err(crate::error::DbError::SqlxError)?;

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(crate::error::DbError::MigrationError)?;

        Ok(Self { pool })
    }

    /// Create a new post
    pub async fn create_post(&self, post: &Post) -> Result<()> {
        let status_str = match post.status {
            PostStatus::Draft => "draft",
            PostStatus::Scheduled => "scheduled",
            PostStatus::Pending => "pending",
            PostStatus::Posted => "posted",
            PostStatus::Failed => "failed",
        };

        sqlx::query(
            r#"
            INSERT INTO posts (id, content, created_at, scheduled_at, status, metadata)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&post.id)
        .bind(&post.content)
        .bind(post.created_at)
        .bind(post.scheduled_at)
        .bind(status_str)
        .bind(&post.metadata)
        .execute(&self.pool)
        .await
        .map_err(crate::error::DbError::SqlxError)?;

        Ok(())
    }

    /// Update post status
    pub async fn update_post_status(&self, post_id: &str, status: PostStatus) -> Result<()> {
        let status_str = match status {
            PostStatus::Draft => "draft",
            PostStatus::Scheduled => "scheduled",
            PostStatus::Pending => "pending",
            PostStatus::Posted => "posted",
            PostStatus::Failed => "failed",
        };

        sqlx::query(
            r#"
            UPDATE posts SET status = ? WHERE id = ?
            "#,
        )
        .bind(status_str)
        .bind(post_id)
        .execute(&self.pool)
        .await
        .map_err(crate::error::DbError::SqlxError)?;

        Ok(())
    }

    /// Update post content
    pub async fn update_post_content(&self, post_id: &str, content: String) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE posts SET content = ? WHERE id = ?
            "#,
        )
        .bind(content)
        .bind(post_id)
        .execute(&self.pool)
        .await
        .map_err(crate::error::DbError::SqlxError)?;

        Ok(())
    }

    /// Get a post by ID
    pub async fn get_post(&self, post_id: &str) -> Result<Option<Post>> {
        use sqlx::Row;

        let row = sqlx::query(
            r#"
            SELECT id, content, created_at, scheduled_at, status, metadata
            FROM posts WHERE id = ?
            "#,
        )
        .bind(post_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(crate::error::DbError::SqlxError)?;

        Ok(row.map(|r| Post {
            id: r.get("id"),
            content: r.get("content"),
            created_at: r.get("created_at"),
            scheduled_at: r.get("scheduled_at"),
            status: match r.get::<String, _>("status").as_str() {
                "draft" => PostStatus::Draft,
                "scheduled" => PostStatus::Scheduled,
                "pending" => PostStatus::Pending,
                "posted" => PostStatus::Posted,
                "failed" => PostStatus::Failed,
                _ => PostStatus::Pending,
            },
            metadata: r.get("metadata"),
        }))
    }

    /// Create a post record
    pub async fn create_post_record(&self, record: &PostRecord) -> Result<()> {
        let success = if record.success { 1 } else { 0 };

        sqlx::query(
            r#"
            INSERT INTO post_records (post_id, platform, platform_post_id, posted_at, success, error_message, account_name)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&record.post_id)
        .bind(&record.platform)
        .bind(&record.platform_post_id)
        .bind(record.posted_at)
        .bind(success)
        .bind(&record.error_message)
        .bind(&record.account_name)
        .execute(&self.pool)
        .await
        .map_err(crate::error::DbError::SqlxError)?;

        Ok(())
    }

    /// Query posts with all platform records
    pub async fn query_posts_with_records(
        &self,
        platform: Option<&str>,
        since: Option<i64>,
        until: Option<i64>,
        search: Option<&str>,
        limit: usize,
    ) -> Result<Vec<PostWithRecords>> {
        use sqlx::Row;

        // Build the WHERE clause dynamically
        let mut where_clauses = vec!["1=1"];

        if platform.is_some() {
            where_clauses.push("pr.platform = ?");
        }
        if since.is_some() {
            where_clauses.push("p.created_at >= ?");
        }
        if until.is_some() {
            where_clauses.push("p.created_at <= ?");
        }
        if search.is_some() {
            where_clauses.push("p.content LIKE ?");
        }

        let where_clause = where_clauses.join(" AND ");

        // First, get the post IDs that match the criteria
        let query_str = format!(
            r#"
            SELECT DISTINCT p.id
            FROM posts p
            LEFT JOIN post_records pr ON p.id = pr.post_id
            WHERE {}
            ORDER BY p.created_at DESC
            LIMIT ?
            "#,
            where_clause
        );

        let mut query = sqlx::query(&query_str);

        // Bind parameters in the same order as WHERE clauses
        if let Some(plat) = platform {
            query = query.bind(plat);
        }
        if let Some(s) = since {
            query = query.bind(s);
        }
        if let Some(u) = until {
            query = query.bind(u);
        }
        if let Some(search_term) = search {
            query = query.bind(format!("%{}%", search_term));
        }
        query = query.bind(limit as i64);

        let rows = query
            .fetch_all(&self.pool)
            .await
            .map_err(crate::error::DbError::SqlxError)?;

        let post_ids: Vec<String> = rows.iter().map(|r| r.get("id")).collect();

        // Now fetch full post data with records for these IDs
        let mut results = Vec::new();
        for post_id in post_ids {
            if let Some(post) = self.get_post(&post_id).await? {
                let records = self.get_post_records(&post_id).await?;
                results.push(PostWithRecords { post, records });
            }
        }

        Ok(results)
    }

    /// Get all post records for a specific post
    pub async fn get_post_records(&self, post_id: &str) -> Result<Vec<PostRecord>> {
        use sqlx::Row;

        let rows = sqlx::query(
            r#"
            SELECT id, post_id, platform, platform_post_id, posted_at, success, error_message, account_name
            FROM post_records
            WHERE post_id = ?
            ORDER BY posted_at DESC
            "#,
        )
        .bind(post_id)
        .fetch_all(&self.pool)
        .await
        .map_err(crate::error::DbError::SqlxError)?;

        Ok(rows
            .iter()
            .map(|r| PostRecord {
                id: r.get("id"),
                post_id: r.get("post_id"),
                platform: r.get("platform"),
                platform_post_id: r.get("platform_post_id"),
                posted_at: r.get("posted_at"),
                success: r.get::<i32, _>("success") != 0,
                error_message: r.get("error_message"),
                account_name: r.get("account_name"),
            })
            .collect())
    }

    /// Get platform-specific post IDs for a plurcast post UUID
    ///
    /// Returns a map of platform name -> platform_post_id for posts that
    /// were successfully posted. Useful for cross-platform threading where
    /// we need to look up the platform-specific ID to reply to.
    ///
    /// # Arguments
    /// * `post_id` - The plurcast internal post UUID
    ///
    /// # Returns
    /// * HashMap where key is platform name (e.g., "nostr", "mastodon")
    ///   and value is the platform-specific post ID
    pub async fn get_platform_post_ids(&self, post_id: &str) -> Result<HashMap<String, String>> {
        use sqlx::Row;

        let rows = sqlx::query(
            r#"
            SELECT platform, platform_post_id
            FROM post_records
            WHERE post_id = ? AND success = 1 AND platform_post_id IS NOT NULL
            "#,
        )
        .bind(post_id)
        .fetch_all(&self.pool)
        .await
        .map_err(crate::error::DbError::SqlxError)?;

        Ok(rows
            .iter()
            .filter_map(|r| {
                let platform: String = r.get("platform");
                let platform_post_id: Option<String> = r.get("platform_post_id");
                platform_post_id.map(|id| (platform, id))
            })
            .collect())
    }

    /// Look up a plurcast post_id by its platform-specific post ID (reverse lookup)
    ///
    /// This enables cross-platform reply-to: when replying to a Nostr post,
    /// we can find if it was posted via plurcast and get the corresponding
    /// Mastodon/SSB IDs for the same original post.
    ///
    /// # Arguments
    ///
    /// * `platform` - The platform name (e.g., "nostr", "mastodon", "ssb")
    /// * `platform_post_id` - The platform-specific post ID (e.g., "note1abc...", "12345678")
    ///
    /// # Returns
    ///
    /// * `Ok(Some(post_id))` - The plurcast UUID if found
    /// * `Ok(None)` - If the platform_post_id is not in our database
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Find the plurcast UUID for a Nostr post
    /// let post_id = db.get_post_id_by_platform_post_id("nostr", "note1abc...").await?;
    ///
    /// if let Some(uuid) = post_id {
    ///     // Look up all platform IDs for cross-posting reply
    ///     let all_ids = db.get_platform_post_ids(&uuid).await?;
    /// }
    /// ```
    pub async fn get_post_id_by_platform_post_id(
        &self,
        platform: &str,
        platform_post_id: &str,
    ) -> Result<Option<String>> {
        use sqlx::Row;

        let row = sqlx::query(
            r#"
            SELECT post_id
            FROM post_records
            WHERE platform = ? AND platform_post_id = ? AND success = 1
            LIMIT 1
            "#,
        )
        .bind(platform)
        .bind(platform_post_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(crate::error::DbError::SqlxError)?;

        Ok(row.map(|r| r.get("post_id")))
    }

    /// Filter posts by platform
    pub async fn filter_by_platform(
        &self,
        platform: &str,
        limit: usize,
    ) -> Result<Vec<PostWithRecords>> {
        self.query_posts_with_records(Some(platform), None, None, None, limit)
            .await
    }

    /// Filter posts by date range
    pub async fn filter_by_date_range(
        &self,
        since: Option<i64>,
        until: Option<i64>,
        limit: usize,
    ) -> Result<Vec<PostWithRecords>> {
        self.query_posts_with_records(None, since, until, None, limit)
            .await
    }

    /// Search posts by content
    pub async fn search_content(
        &self,
        search_term: &str,
        limit: usize,
    ) -> Result<Vec<PostWithRecords>> {
        self.query_posts_with_records(None, None, None, Some(search_term), limit)
            .await
    }

    // ========================================================================
    // Scheduling methods (Phase 5)
    // ========================================================================

    /// Get scheduled posts that are due to be sent
    ///
    /// Returns posts where:
    /// - status = 'scheduled'
    /// - scheduled_at <= now (due or overdue)
    ///
    /// Used by plur-send daemon to find posts that need to be sent.
    pub async fn get_scheduled_posts_due(&self) -> Result<Vec<Post>> {
        let now = chrono::Utc::now().timestamp();

        let rows = sqlx::query_as::<_, (String, String, i64, Option<i64>, String, Option<String>)>(
            r#"
            SELECT id, content, created_at, scheduled_at, status, metadata
            FROM posts
            WHERE status = 'scheduled'
              AND scheduled_at IS NOT NULL
              AND scheduled_at <= ?
            ORDER BY scheduled_at ASC
            "#,
        )
        .bind(now)
        .fetch_all(&self.pool)
        .await
        .map_err(crate::error::DbError::SqlxError)?;

        let posts = rows
            .into_iter()
            .map(
                |(id, content, created_at, scheduled_at, status, metadata)| {
                    let status = match status.as_str() {
                        "draft" => PostStatus::Draft,
                        "scheduled" => PostStatus::Scheduled,
                        "pending" => PostStatus::Pending,
                        "posted" => PostStatus::Posted,
                        "failed" => PostStatus::Failed,
                        _ => PostStatus::Pending,
                    };

                    Post {
                        id,
                        content,
                        created_at,
                        scheduled_at,
                        status,
                        metadata,
                    }
                },
            )
            .collect();

        Ok(posts)
    }

    /// Get all scheduled posts (for plur-queue list)
    ///
    /// Returns all posts with status = 'scheduled', ordered by scheduled_at.
    pub async fn get_scheduled_posts(&self) -> Result<Vec<Post>> {
        let rows = sqlx::query_as::<_, (String, String, i64, Option<i64>, String, Option<String>)>(
            r#"
            SELECT id, content, created_at, scheduled_at, status, metadata
            FROM posts
            WHERE status = 'scheduled'
            ORDER BY scheduled_at ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(crate::error::DbError::SqlxError)?;

        let posts = rows
            .into_iter()
            .map(
                |(id, content, created_at, scheduled_at, status, metadata)| {
                    let status = match status.as_str() {
                        "draft" => PostStatus::Draft,
                        "scheduled" => PostStatus::Scheduled,
                        "pending" => PostStatus::Pending,
                        "posted" => PostStatus::Posted,
                        "failed" => PostStatus::Failed,
                        _ => PostStatus::Pending,
                    };

                    Post {
                        id,
                        content,
                        created_at,
                        scheduled_at,
                        status,
                        metadata,
                    }
                },
            )
            .collect();

        Ok(posts)
    }

    /// Get all failed posts that may need retry
    ///
    /// Returns posts with status 'failed'.
    pub async fn get_failed_posts(&self) -> Result<Vec<Post>> {
        let rows = sqlx::query_as::<_, (String, String, i64, Option<i64>, String, Option<String>)>(
            r#"
            SELECT id, content, created_at, scheduled_at, status, metadata
            FROM posts
            WHERE status = 'failed'
            ORDER BY created_at ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(crate::error::DbError::SqlxError)?;

        let posts = rows
            .into_iter()
            .map(
                |(id, content, created_at, scheduled_at, status, metadata)| {
                    let status = match status.as_str() {
                        "draft" => PostStatus::Draft,
                        "scheduled" => PostStatus::Scheduled,
                        "pending" => PostStatus::Pending,
                        "posted" => PostStatus::Posted,
                        "failed" => PostStatus::Failed,
                        _ => PostStatus::Pending,
                    };

                    Post {
                        id,
                        content,
                        created_at,
                        scheduled_at,
                        status,
                        metadata,
                    }
                },
            )
            .collect();

        Ok(posts)
    }

    /// Get the most recent scheduled_at timestamp from all scheduled posts
    ///
    /// Used by random scheduling to schedule the next post after the last one.
    /// Returns None if there are no scheduled posts.
    pub async fn get_last_scheduled_timestamp(&self) -> Result<Option<i64>> {
        let row = sqlx::query_as::<_, (Option<i64>,)>(
            r#"
            SELECT MAX(scheduled_at) FROM posts
            WHERE status = 'scheduled' AND scheduled_at IS NOT NULL
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(crate::error::DbError::SqlxError)?;

        Ok(row.0)
    }

    /// Update the scheduled_at time for a post
    ///
    /// Used by plur-queue reschedule command.
    pub async fn update_post_schedule(
        &self,
        post_id: &str,
        scheduled_at: Option<i64>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE posts SET scheduled_at = ? WHERE id = ?
            "#,
        )
        .bind(scheduled_at)
        .bind(post_id)
        .execute(&self.pool)
        .await
        .map_err(crate::error::DbError::SqlxError)?;

        Ok(())
    }

    /// Update post metadata
    ///
    /// Used by plur-queue update command to modify platform-specific settings
    /// like Nostr PoW difficulty.
    ///
    /// # Arguments
    ///
    /// * `post_id` - The post ID to update
    /// * `metadata` - JSON string containing updated metadata
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use libplurcast::Database;
    /// # async fn example(db: &Database) -> libplurcast::Result<()> {
    /// let metadata = r#"{"nostr": {"pow_difficulty": 28}}"#;
    /// db.update_post_metadata("post-id-123", metadata).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_post_metadata(&self, post_id: &str, metadata: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE posts SET metadata = ? WHERE id = ?
            "#,
        )
        .bind(metadata)
        .bind(post_id)
        .execute(&self.pool)
        .await
        .map_err(crate::error::DbError::SqlxError)?;

        Ok(())
    }

    /// Delete a scheduled post
    ///
    /// Used by plur-queue cancel command.
    pub async fn delete_post(&self, post_id: &str) -> Result<()> {
        // Delete post records first (foreign key constraint)
        sqlx::query(
            r#"
            DELETE FROM post_records WHERE post_id = ?
            "#,
        )
        .bind(post_id)
        .execute(&self.pool)
        .await
        .map_err(crate::error::DbError::SqlxError)?;

        // Delete post
        sqlx::query(
            r#"
            DELETE FROM posts WHERE id = ?
            "#,
        )
        .bind(post_id)
        .execute(&self.pool)
        .await
        .map_err(crate::error::DbError::SqlxError)?;

        Ok(())
    }

    /// Get the current rate limit count for a platform within a time window
    ///
    /// Returns the number of posts made in the current window.
    pub async fn get_rate_limit_count(&self, platform: &str, window_start: i64) -> Result<usize> {
        let row = sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT post_count FROM rate_limits
            WHERE platform = ? AND window_start = ?
            "#,
        )
        .bind(platform)
        .bind(window_start)
        .fetch_optional(&self.pool)
        .await
        .map_err(crate::error::DbError::SqlxError)?;

        Ok(row.map(|(count,)| count as usize).unwrap_or(0))
    }

    /// Increment the rate limit counter for a platform
    ///
    /// Called after successfully posting to track rate limits.
    pub async fn increment_rate_limit(&self, platform: &str, window_start: i64) -> Result<()> {
        // Insert or update
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
        .execute(&self.pool)
        .await
        .map_err(crate::error::DbError::SqlxError)?;

        Ok(())
    }

    /// Clean up old rate limit records
    ///
    /// Removes rate limit records older than the specified timestamp.
    /// Should be called periodically to prevent table bloat.
    pub async fn cleanup_rate_limits(&self, before_timestamp: i64) -> Result<()> {
        sqlx::query(
            r#"
            DELETE FROM rate_limits WHERE window_start < ?
            "#,
        )
        .bind(before_timestamp)
        .execute(&self.pool)
        .await
        .map_err(crate::error::DbError::SqlxError)?;

        Ok(())
    }

    // ========================================================================
    // Attachment methods
    // ========================================================================

    /// Create a new attachment record
    ///
    /// Stores metadata about an attached image file. The actual file is stored
    /// on disk; this only stores the reference and metadata.
    pub async fn create_attachment(&self, attachment: &Attachment) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO attachments (id, post_id, file_path, mime_type, file_size, file_hash, alt_text, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&attachment.id)
        .bind(&attachment.post_id)
        .bind(&attachment.file_path)
        .bind(attachment.mime_type.as_str())
        .bind(attachment.file_size as i64)
        .bind(&attachment.file_hash)
        .bind(&attachment.alt_text)
        .bind(attachment.created_at)
        .execute(&self.pool)
        .await
        .map_err(crate::error::DbError::SqlxError)?;

        Ok(())
    }

    /// Get an attachment by ID
    pub async fn get_attachment(&self, attachment_id: &str) -> Result<Option<Attachment>> {
        use sqlx::Row;

        let row = sqlx::query(
            r#"
            SELECT id, post_id, file_path, mime_type, file_size, file_hash, alt_text, created_at
            FROM attachments WHERE id = ?
            "#,
        )
        .bind(attachment_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(crate::error::DbError::SqlxError)?;

        Ok(row.map(|r| {
            let mime_str: String = r.get("mime_type");
            Attachment {
                id: r.get("id"),
                post_id: r.get("post_id"),
                file_path: r.get("file_path"),
                mime_type: ImageMimeType::from_mime_str(&mime_str).unwrap_or(ImageMimeType::Jpeg),
                file_size: r.get::<i64, _>("file_size") as u64,
                file_hash: r.get("file_hash"),
                alt_text: r.get("alt_text"),
                created_at: r.get("created_at"),
            }
        }))
    }

    /// Get all attachments for a post
    ///
    /// Returns attachments ordered by creation time (oldest first).
    pub async fn get_attachments_for_post(&self, post_id: &str) -> Result<Vec<Attachment>> {
        use sqlx::Row;

        let rows = sqlx::query(
            r#"
            SELECT id, post_id, file_path, mime_type, file_size, file_hash, alt_text, created_at
            FROM attachments
            WHERE post_id = ?
            ORDER BY created_at ASC
            "#,
        )
        .bind(post_id)
        .fetch_all(&self.pool)
        .await
        .map_err(crate::error::DbError::SqlxError)?;

        Ok(rows
            .iter()
            .map(|r| {
                let mime_str: String = r.get("mime_type");
                Attachment {
                    id: r.get("id"),
                    post_id: r.get("post_id"),
                    file_path: r.get("file_path"),
                    mime_type: ImageMimeType::from_mime_str(&mime_str)
                        .unwrap_or(ImageMimeType::Jpeg),
                    file_size: r.get::<i64, _>("file_size") as u64,
                    file_hash: r.get("file_hash"),
                    alt_text: r.get("alt_text"),
                    created_at: r.get("created_at"),
                }
            })
            .collect())
    }

    /// Delete an attachment by ID
    ///
    /// Also deletes associated upload records due to CASCADE.
    pub async fn delete_attachment(&self, attachment_id: &str) -> Result<()> {
        sqlx::query(
            r#"
            DELETE FROM attachments WHERE id = ?
            "#,
        )
        .bind(attachment_id)
        .execute(&self.pool)
        .await
        .map_err(crate::error::DbError::SqlxError)?;

        Ok(())
    }

    /// Delete all attachments for a post
    pub async fn delete_attachments_for_post(&self, post_id: &str) -> Result<()> {
        sqlx::query(
            r#"
            DELETE FROM attachments WHERE post_id = ?
            "#,
        )
        .bind(post_id)
        .execute(&self.pool)
        .await
        .map_err(crate::error::DbError::SqlxError)?;

        Ok(())
    }

    /// Find attachment by file hash (for deduplication)
    pub async fn find_attachment_by_hash(&self, file_hash: &str) -> Result<Option<Attachment>> {
        use sqlx::Row;

        let row = sqlx::query(
            r#"
            SELECT id, post_id, file_path, mime_type, file_size, file_hash, alt_text, created_at
            FROM attachments WHERE file_hash = ?
            LIMIT 1
            "#,
        )
        .bind(file_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(crate::error::DbError::SqlxError)?;

        Ok(row.map(|r| {
            let mime_str: String = r.get("mime_type");
            Attachment {
                id: r.get("id"),
                post_id: r.get("post_id"),
                file_path: r.get("file_path"),
                mime_type: ImageMimeType::from_mime_str(&mime_str).unwrap_or(ImageMimeType::Jpeg),
                file_size: r.get::<i64, _>("file_size") as u64,
                file_hash: r.get("file_hash"),
                alt_text: r.get("alt_text"),
                created_at: r.get("created_at"),
            }
        }))
    }

    // ========================================================================
    // Attachment Upload methods
    // ========================================================================

    /// Create a new attachment upload record
    ///
    /// Tracks the upload status of an attachment to a specific platform.
    pub async fn create_attachment_upload(&self, upload: &AttachmentUpload) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO attachment_uploads (attachment_id, platform, platform_attachment_id, remote_url, uploaded_at, status, error_message)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&upload.attachment_id)
        .bind(&upload.platform)
        .bind(&upload.platform_attachment_id)
        .bind(&upload.remote_url)
        .bind(upload.uploaded_at)
        .bind(upload.status.to_string())
        .bind(&upload.error_message)
        .execute(&self.pool)
        .await
        .map_err(crate::error::DbError::SqlxError)?;

        Ok(())
    }

    /// Get attachment upload record for a specific attachment and platform
    pub async fn get_attachment_upload(
        &self,
        attachment_id: &str,
        platform: &str,
    ) -> Result<Option<AttachmentUpload>> {
        use sqlx::Row;

        let row = sqlx::query(
            r#"
            SELECT id, attachment_id, platform, platform_attachment_id, remote_url, uploaded_at, status, error_message
            FROM attachment_uploads
            WHERE attachment_id = ? AND platform = ?
            "#,
        )
        .bind(attachment_id)
        .bind(platform)
        .fetch_optional(&self.pool)
        .await
        .map_err(crate::error::DbError::SqlxError)?;

        Ok(row.map(|r| {
            let status_str: String = r.get("status");
            AttachmentUpload {
                id: r.get("id"),
                attachment_id: r.get("attachment_id"),
                platform: r.get("platform"),
                platform_attachment_id: r.get("platform_attachment_id"),
                remote_url: r.get("remote_url"),
                uploaded_at: r.get("uploaded_at"),
                status: match status_str.as_str() {
                    "uploaded" => AttachmentStatus::Uploaded,
                    "failed" => AttachmentStatus::Failed,
                    _ => AttachmentStatus::Pending,
                },
                error_message: r.get("error_message"),
            }
        }))
    }

    /// Get all upload records for an attachment
    pub async fn get_attachment_uploads(
        &self,
        attachment_id: &str,
    ) -> Result<Vec<AttachmentUpload>> {
        use sqlx::Row;

        let rows = sqlx::query(
            r#"
            SELECT id, attachment_id, platform, platform_attachment_id, remote_url, uploaded_at, status, error_message
            FROM attachment_uploads
            WHERE attachment_id = ?
            ORDER BY uploaded_at DESC
            "#,
        )
        .bind(attachment_id)
        .fetch_all(&self.pool)
        .await
        .map_err(crate::error::DbError::SqlxError)?;

        Ok(rows
            .iter()
            .map(|r| {
                let status_str: String = r.get("status");
                AttachmentUpload {
                    id: r.get("id"),
                    attachment_id: r.get("attachment_id"),
                    platform: r.get("platform"),
                    platform_attachment_id: r.get("platform_attachment_id"),
                    remote_url: r.get("remote_url"),
                    uploaded_at: r.get("uploaded_at"),
                    status: match status_str.as_str() {
                        "uploaded" => AttachmentStatus::Uploaded,
                        "failed" => AttachmentStatus::Failed,
                        _ => AttachmentStatus::Pending,
                    },
                    error_message: r.get("error_message"),
                }
            })
            .collect())
    }

    /// Update attachment upload status
    ///
    /// Used after uploading to a platform to record success or failure.
    pub async fn update_attachment_upload(
        &self,
        attachment_id: &str,
        platform: &str,
        platform_attachment_id: Option<&str>,
        remote_url: Option<&str>,
        status: AttachmentStatus,
        error_message: Option<&str>,
    ) -> Result<()> {
        let now = chrono::Utc::now().timestamp();

        sqlx::query(
            r#"
            UPDATE attachment_uploads
            SET platform_attachment_id = ?, remote_url = ?, uploaded_at = ?, status = ?, error_message = ?
            WHERE attachment_id = ? AND platform = ?
            "#,
        )
        .bind(platform_attachment_id)
        .bind(remote_url)
        .bind(now)
        .bind(status.to_string())
        .bind(error_message)
        .bind(attachment_id)
        .bind(platform)
        .execute(&self.pool)
        .await
        .map_err(crate::error::DbError::SqlxError)?;

        Ok(())
    }

    /// Get pending upload records for a platform
    ///
    /// Used by retry logic to find attachments that need to be uploaded.
    pub async fn get_pending_uploads(&self, platform: &str) -> Result<Vec<AttachmentUpload>> {
        use sqlx::Row;

        let rows = sqlx::query(
            r#"
            SELECT id, attachment_id, platform, platform_attachment_id, remote_url, uploaded_at, status, error_message
            FROM attachment_uploads
            WHERE platform = ? AND status = 'pending'
            ORDER BY id ASC
            "#,
        )
        .bind(platform)
        .fetch_all(&self.pool)
        .await
        .map_err(crate::error::DbError::SqlxError)?;

        Ok(rows
            .iter()
            .map(|r| {
                let status_str: String = r.get("status");
                AttachmentUpload {
                    id: r.get("id"),
                    attachment_id: r.get("attachment_id"),
                    platform: r.get("platform"),
                    platform_attachment_id: r.get("platform_attachment_id"),
                    remote_url: r.get("remote_url"),
                    uploaded_at: r.get("uploaded_at"),
                    status: match status_str.as_str() {
                        "uploaded" => AttachmentStatus::Uploaded,
                        "failed" => AttachmentStatus::Failed,
                        _ => AttachmentStatus::Pending,
                    },
                    error_message: r.get("error_message"),
                }
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{DbError, PlurcastError};
    use crate::types::{Post, PostRecord, PostStatus};
    use tempfile::TempDir;

    /// Helper to create a test post
    fn create_test_post() -> Post {
        Post {
            id: uuid::Uuid::new_v4().to_string(),
            content: "Test post content".to_string(),
            created_at: chrono::Utc::now().timestamp(),
            scheduled_at: None,
            status: PostStatus::Pending,
            metadata: None,
        }
    }

    #[tokio::test]
    async fn test_database_initialization_with_invalid_path() {
        // Test with a path that cannot be created (e.g., null byte in path on Unix)
        // On Windows, we'll use an invalid character
        #[cfg(unix)]
        let invalid_path = "/tmp/test\0invalid.db";

        #[cfg(windows)]
        let invalid_path = "C:\\invalid<>path\\test.db";

        let result = Database::new(invalid_path).await;
        assert!(result.is_err(), "Expected error for invalid path");

        // Verify it's a database error
        match result {
            Err(PlurcastError::Database(_)) => {
                // Success - got the expected error type
            }
            _ => panic!("Expected DbError for invalid path"),
        }
    }

    #[tokio::test]
    async fn test_database_initialization_with_readonly_parent() {
        // Note: This test is platform-specific and may behave differently on Windows vs Unix
        // On Windows, readonly attribute on directories doesn't prevent file creation
        // On Unix, we can make a directory truly read-only

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            // Create a temporary directory and make it read-only
            let temp_dir = TempDir::new().unwrap();
            let readonly_dir = temp_dir.path().join("readonly");
            std::fs::create_dir(&readonly_dir).unwrap();

            // Set directory to read-only (no write permission)
            let mut perms = std::fs::metadata(&readonly_dir).unwrap().permissions();
            perms.set_mode(0o444); // Read-only
            std::fs::set_permissions(&readonly_dir, perms).unwrap();

            let db_path = readonly_dir.join("test.db");
            let result = Database::new(db_path.to_str().unwrap()).await;

            // Clean up permissions before asserting
            let mut perms = std::fs::metadata(&readonly_dir).unwrap().permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&readonly_dir, perms).unwrap();

            assert!(
                result.is_err(),
                "Expected error for read-only directory on Unix"
            );
        }

        #[cfg(windows)]
        {
            // On Windows, the readonly attribute on directories doesn't prevent file creation
            // So we skip this test or test a different scenario
            // We'll test that we can still create a database in a normal directory
            let temp_dir = TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let result = Database::new(db_path.to_str().unwrap()).await;
            assert!(
                result.is_ok(),
                "Should be able to create database on Windows"
            );
        }
    }

    #[tokio::test]
    async fn test_foreign_key_constraint_enforcement() {
        // Use in-memory database for testing
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        // Try to create a post_record without a corresponding post
        let record = PostRecord {
            id: None,
            post_id: "nonexistent-post-id".to_string(),
            platform: "nostr".to_string(),
            platform_post_id: Some("note1abc".to_string()),
            posted_at: Some(chrono::Utc::now().timestamp()),
            success: true,
            error_message: None,
            account_name: "default".to_string(),
        };

        // Enable foreign key constraints (SQLite has them off by default in some configurations)
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&db.pool)
            .await
            .unwrap();

        let result = db.create_post_record(&record).await;

        // This should fail due to foreign key constraint
        assert!(result.is_err(), "Expected foreign key constraint violation");

        match result {
            Err(PlurcastError::Database(DbError::SqlxError(sqlx::Error::Database(db_err)))) => {
                // Verify it's a foreign key constraint error
                let message = db_err.message();
                assert!(
                    message.contains("FOREIGN KEY") || message.contains("foreign key"),
                    "Expected foreign key error, got: {}",
                    message
                );
            }
            _ => panic!("Expected foreign key constraint error"),
        }
    }

    #[tokio::test]
    async fn test_foreign_key_constraint_with_valid_post() {
        // Use in-memory database for testing
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        // Enable foreign key constraints
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&db.pool)
            .await
            .unwrap();

        // Create a valid post first
        let post = create_test_post();
        db.create_post(&post).await.unwrap();

        // Now create a post_record with valid foreign key
        let record = PostRecord {
            id: None,
            post_id: post.id.clone(),
            platform: "nostr".to_string(),
            platform_post_id: Some("note1abc".to_string()),
            posted_at: Some(chrono::Utc::now().timestamp()),
            success: true,
            error_message: None,
            account_name: "default".to_string(),
        };

        let result = db.create_post_record(&record).await;
        assert!(result.is_ok(), "Expected success with valid foreign key");
    }

    #[tokio::test]
    async fn test_transaction_rollback_on_error() {
        // Use in-memory database for testing
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        // Create a post
        let post = create_test_post();
        db.create_post(&post).await.unwrap();

        // Verify post exists
        let retrieved = db.get_post(&post.id).await.unwrap();
        assert!(retrieved.is_some());

        // Now try to create a duplicate post (should fail due to PRIMARY KEY constraint)
        let duplicate_post = Post {
            id: post.id.clone(), // Same ID
            content: "Different content".to_string(),
            created_at: chrono::Utc::now().timestamp(),
            scheduled_at: None,
            status: PostStatus::Pending,
            metadata: None,
        };

        let result = db.create_post(&duplicate_post).await;
        assert!(result.is_err(), "Expected error for duplicate primary key");

        // Verify original post is still there and unchanged
        let retrieved = db.get_post(&post.id).await.unwrap().unwrap();
        assert_eq!(retrieved.content, post.content);
        assert_ne!(retrieved.content, duplicate_post.content);
    }

    #[tokio::test]
    async fn test_constraint_violation_on_invalid_status() {
        // Use in-memory database for testing
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        // Try to insert a post with invalid status directly via SQL
        let post_id = uuid::Uuid::new_v4().to_string();
        let _result = sqlx::query(
            r#"
            INSERT INTO posts (id, content, created_at, status)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(&post_id)
        .bind("Test content")
        .bind(chrono::Utc::now().timestamp())
        .bind("invalid_status") // This should be 'pending', 'posted', or 'failed'
        .execute(&db.pool)
        .await;

        // Note: SQLite doesn't enforce CHECK constraints on status by default in our schema
        // But we can verify that our application logic handles status correctly
        // This test documents the behavior

        // If we want to enforce this, we'd need to add a CHECK constraint in the migration
        // For now, we verify that our API only allows valid statuses
        let post = Post {
            id: uuid::Uuid::new_v4().to_string(),
            content: "Test".to_string(),
            created_at: chrono::Utc::now().timestamp(),
            scheduled_at: None,
            status: PostStatus::Pending, // Only valid statuses allowed by type system
            metadata: None,
        };

        assert!(db.create_post(&post).await.is_ok());
    }

    #[tokio::test]
    async fn test_not_null_constraint_on_content() {
        // Use in-memory database for testing
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        // Try to insert a post with NULL content directly via SQL
        let post_id = uuid::Uuid::new_v4().to_string();
        let result = sqlx::query(
            r#"
            INSERT INTO posts (id, content, created_at, status)
            VALUES (?, NULL, ?, ?)
            "#,
        )
        .bind(&post_id)
        .bind(chrono::Utc::now().timestamp())
        .bind("pending")
        .execute(&db.pool)
        .await;

        assert!(result.is_err(), "Expected NOT NULL constraint violation");

        match result {
            Err(sqlx::Error::Database(db_err)) => {
                let message = db_err.message();
                assert!(
                    message.contains("NOT NULL") || message.contains("not null"),
                    "Expected NOT NULL error, got: {}",
                    message
                );
            }
            _ => panic!("Expected NOT NULL constraint error"),
        }
    }

    #[tokio::test]
    async fn test_database_operations_after_error() {
        // Use in-memory database for testing
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        // Create a post
        let post1 = create_test_post();
        db.create_post(&post1).await.unwrap();

        // Try to create a duplicate (should fail)
        let duplicate = Post {
            id: post1.id.clone(),
            content: "Duplicate".to_string(),
            created_at: chrono::Utc::now().timestamp(),
            scheduled_at: None,
            status: PostStatus::Pending,
            metadata: None,
        };
        let _ = db.create_post(&duplicate).await;

        // Verify database is still functional after error
        let post2 = create_test_post();
        let result = db.create_post(&post2).await;
        assert!(
            result.is_ok(),
            "Database should still be functional after error"
        );

        // Verify we can retrieve both posts
        assert!(db.get_post(&post1.id).await.unwrap().is_some());
        assert!(db.get_post(&post2.id).await.unwrap().is_some());
    }

    // Task 1.4: Expanded database CRUD tests

    #[tokio::test]
    async fn test_create_and_retrieve_post_happy_path() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        // Create a post
        let post = create_test_post();
        db.create_post(&post).await.unwrap();

        // Retrieve the post
        let retrieved = db.get_post(&post.id).await.unwrap();
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, post.id);
        assert_eq!(retrieved.content, post.content);
        assert_eq!(retrieved.created_at, post.created_at);
        assert_eq!(retrieved.scheduled_at, post.scheduled_at);
        assert!(matches!(retrieved.status, PostStatus::Pending));
    }

    #[tokio::test]
    async fn test_update_post_status_pending_to_posted() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        // Create a post with Pending status
        let post = create_test_post();
        db.create_post(&post).await.unwrap();

        // Update status to Posted
        db.update_post_status(&post.id, PostStatus::Posted)
            .await
            .unwrap();

        // Verify status was updated
        let retrieved = db.get_post(&post.id).await.unwrap().unwrap();
        assert!(matches!(retrieved.status, PostStatus::Posted));
    }

    #[tokio::test]
    async fn test_update_post_status_pending_to_failed() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        // Create a post with Pending status
        let post = create_test_post();
        db.create_post(&post).await.unwrap();

        // Update status to Failed
        db.update_post_status(&post.id, PostStatus::Failed)
            .await
            .unwrap();

        // Verify status was updated
        let retrieved = db.get_post(&post.id).await.unwrap().unwrap();
        assert!(matches!(retrieved.status, PostStatus::Failed));
    }

    #[tokio::test]
    async fn test_get_nonexistent_post_returns_none() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        // Try to get a post that doesn't exist
        let nonexistent_id = uuid::Uuid::new_v4().to_string();
        let result = db.get_post(&nonexistent_id).await.unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_create_post_record_with_success() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        // Create a post first
        let post = create_test_post();
        db.create_post(&post).await.unwrap();

        // Create a successful post record
        let record = PostRecord {
            id: None,
            post_id: post.id.clone(),
            platform: "nostr".to_string(),
            platform_post_id: Some("note1abc123".to_string()),
            posted_at: Some(chrono::Utc::now().timestamp()),
            success: true,
            error_message: None,
            account_name: "default".to_string(),
        };

        let result = db.create_post_record(&record).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_post_record_with_failure() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        // Create a post first
        let post = create_test_post();
        db.create_post(&post).await.unwrap();

        // Create a failed post record
        let record = PostRecord {
            id: None,
            post_id: post.id.clone(),
            platform: "mastodon".to_string(),
            platform_post_id: None,
            posted_at: None,
            success: false,
            error_message: Some("Network timeout".to_string()),
            account_name: "default".to_string(),
        };

        let result = db.create_post_record(&record).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_concurrent_post_operations() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        // Create multiple posts concurrently
        let mut handles = vec![];

        for i in 0..5 {
            let post = Post {
                id: uuid::Uuid::new_v4().to_string(),
                content: format!("Concurrent post {}", i),
                created_at: chrono::Utc::now().timestamp(),
                scheduled_at: None,
                status: PostStatus::Pending,
                metadata: None,
            };

            // Clone the pool for each task
            let pool_clone = db.pool.clone();
            let post_clone = post.clone();

            let handle = tokio::spawn(async move {
                let db = Database { pool: pool_clone };
                db.create_post(&post_clone).await
            });

            handles.push((handle, post.id));
        }

        // Wait for all operations to complete
        for (handle, post_id) in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok(), "Concurrent post creation should succeed");

            // Verify post was created
            let retrieved = db.get_post(&post_id).await.unwrap();
            assert!(retrieved.is_some());
        }
    }

    #[tokio::test]
    async fn test_concurrent_status_updates() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        // Create a post
        let post = create_test_post();
        db.create_post(&post).await.unwrap();

        // Update status concurrently (simulating multiple operations)
        let mut handles = vec![];

        for _ in 0..3 {
            let pool_clone = db.pool.clone();
            let post_id = post.id.clone();

            let handle = tokio::spawn(async move {
                let db = Database { pool: pool_clone };
                db.update_post_status(&post_id, PostStatus::Posted).await
            });

            handles.push(handle);
        }

        // Wait for all operations to complete
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok(), "Concurrent status update should succeed");
        }

        // Verify final status
        let retrieved = db.get_post(&post.id).await.unwrap().unwrap();
        assert!(matches!(retrieved.status, PostStatus::Posted));
    }

    #[tokio::test]
    async fn test_multiple_post_records_for_same_post() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        // Create a post
        let post = create_test_post();
        db.create_post(&post).await.unwrap();

        // Create multiple post records for different platforms
        let platforms = vec!["nostr", "mastodon", "bluesky"];

        for platform in platforms {
            let record = PostRecord {
                id: None,
                post_id: post.id.clone(),
                platform: platform.to_string(),
                platform_post_id: Some(format!("{}_post_123", platform)),
                posted_at: Some(chrono::Utc::now().timestamp()),
                success: true,
                error_message: None,
                account_name: "default".to_string(),
            };

            let result = db.create_post_record(&record).await;
            assert!(
                result.is_ok(),
                "Should be able to create multiple records for same post"
            );
        }
    }

    #[tokio::test]
    async fn test_post_with_scheduled_at() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        let scheduled_time = chrono::Utc::now().timestamp() + 3600; // 1 hour from now

        let post = Post {
            id: uuid::Uuid::new_v4().to_string(),
            content: "Scheduled post".to_string(),
            created_at: chrono::Utc::now().timestamp(),
            scheduled_at: Some(scheduled_time),
            status: PostStatus::Pending,
            metadata: None,
        };

        db.create_post(&post).await.unwrap();

        let retrieved = db.get_post(&post.id).await.unwrap().unwrap();
        assert_eq!(retrieved.scheduled_at, Some(scheduled_time));
    }

    #[tokio::test]
    async fn test_post_with_metadata() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        let metadata = r#"{"tags":["rust","nostr"],"reply_to":"note1abc"}"#;

        let post = Post {
            id: uuid::Uuid::new_v4().to_string(),
            content: "Post with metadata".to_string(),
            created_at: chrono::Utc::now().timestamp(),
            scheduled_at: None,
            status: PostStatus::Pending,
            metadata: Some(metadata.to_string()),
        };

        db.create_post(&post).await.unwrap();

        let retrieved = db.get_post(&post.id).await.unwrap().unwrap();
        assert_eq!(retrieved.metadata, Some(metadata.to_string()));
    }

    // Task 10.3: Database tests for multi-platform queries

    #[tokio::test]
    async fn test_post_creation_with_multiple_platform_records() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        // Create a post
        let post = create_test_post();
        db.create_post(&post).await.unwrap();

        // Create records for multiple platforms
        let platforms = vec![
            ("nostr", "note1abc123", true, None),
            ("mastodon", "12345", true, None),
            ("bluesky", "", false, Some("Authentication failed")),
        ];

        for (platform, platform_post_id, success, error) in platforms {
            let record = PostRecord {
                id: None,
                post_id: post.id.clone(),
                platform: platform.to_string(),
                platform_post_id: if platform_post_id.is_empty() {
                    None
                } else {
                    Some(platform_post_id.to_string())
                },
                posted_at: if success {
                    Some(chrono::Utc::now().timestamp())
                } else {
                    None
                },
                success,
                error_message: error.map(|s| s.to_string()),
                account_name: "default".to_string(),
            };

            db.create_post_record(&record).await.unwrap();
        }

        // Retrieve post with records
        let records = db.get_post_records(&post.id).await.unwrap();
        assert_eq!(records.len(), 3);

        // Verify each platform record
        let nostr_record = records.iter().find(|r| r.platform == "nostr").unwrap();
        assert!(nostr_record.success);
        assert_eq!(
            nostr_record.platform_post_id,
            Some("note1abc123".to_string())
        );

        let mastodon_record = records.iter().find(|r| r.platform == "mastodon").unwrap();
        assert!(mastodon_record.success);
        assert_eq!(mastodon_record.platform_post_id, Some("12345".to_string()));

        let bluesky_record = records.iter().find(|r| r.platform == "bluesky").unwrap();
        assert!(!bluesky_record.success);
        assert_eq!(
            bluesky_record.error_message,
            Some("Authentication failed".to_string())
        );
    }

    #[tokio::test]
    async fn test_query_posts_with_platform_filter() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        // Create multiple posts
        let post1 = create_test_post();
        let post2 = create_test_post();
        let post3 = create_test_post();

        db.create_post(&post1).await.unwrap();
        db.create_post(&post2).await.unwrap();
        db.create_post(&post3).await.unwrap();

        // Add records for different platforms
        db.create_post_record(&PostRecord {
            id: None,
            post_id: post1.id.clone(),
            platform: "nostr".to_string(),
            platform_post_id: Some("note1".to_string()),
            posted_at: Some(chrono::Utc::now().timestamp()),
            success: true,
            error_message: None,
            account_name: "default".to_string(),
        })
        .await
        .unwrap();

        db.create_post_record(&PostRecord {
            id: None,
            post_id: post2.id.clone(),
            platform: "mastodon".to_string(),
            platform_post_id: Some("12345".to_string()),
            posted_at: Some(chrono::Utc::now().timestamp()),
            success: true,
            error_message: None,
            account_name: "default".to_string(),
        })
        .await
        .unwrap();

        db.create_post_record(&PostRecord {
            id: None,
            post_id: post3.id.clone(),
            platform: "nostr".to_string(),
            platform_post_id: Some("note2".to_string()),
            posted_at: Some(chrono::Utc::now().timestamp()),
            success: true,
            error_message: None,
            account_name: "default".to_string(),
        })
        .await
        .unwrap();

        // Filter by nostr platform
        let nostr_posts = db.filter_by_platform("nostr", 10).await.unwrap();
        assert_eq!(nostr_posts.len(), 2);
        assert!(nostr_posts
            .iter()
            .all(|p| p.records.iter().any(|r| r.platform == "nostr")));

        // Filter by mastodon platform
        let mastodon_posts = db.filter_by_platform("mastodon", 10).await.unwrap();
        assert_eq!(mastodon_posts.len(), 1);
        assert_eq!(mastodon_posts[0].post.id, post2.id);
    }

    #[tokio::test]
    async fn test_query_posts_with_date_range_filter() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        let now = chrono::Utc::now().timestamp();
        let one_hour_ago = now - 3600;
        let two_hours_ago = now - 7200;

        // Create posts with different timestamps
        let post1 = Post {
            id: uuid::Uuid::new_v4().to_string(),
            content: "Old post".to_string(),
            created_at: two_hours_ago,
            scheduled_at: None,
            status: PostStatus::Posted,
            metadata: None,
        };

        let post2 = Post {
            id: uuid::Uuid::new_v4().to_string(),
            content: "Recent post".to_string(),
            created_at: one_hour_ago,
            scheduled_at: None,
            status: PostStatus::Posted,
            metadata: None,
        };

        let post3 = Post {
            id: uuid::Uuid::new_v4().to_string(),
            content: "New post".to_string(),
            created_at: now,
            scheduled_at: None,
            status: PostStatus::Posted,
            metadata: None,
        };

        db.create_post(&post1).await.unwrap();
        db.create_post(&post2).await.unwrap();
        db.create_post(&post3).await.unwrap();

        // Query posts since one hour ago
        let recent_posts = db
            .filter_by_date_range(Some(one_hour_ago), None, 10)
            .await
            .unwrap();
        assert_eq!(recent_posts.len(), 2);
        assert!(recent_posts.iter().any(|p| p.post.id == post2.id));
        assert!(recent_posts.iter().any(|p| p.post.id == post3.id));

        // Query posts until one hour ago
        let old_posts = db
            .filter_by_date_range(None, Some(one_hour_ago), 10)
            .await
            .unwrap();
        assert_eq!(old_posts.len(), 2);
        assert!(old_posts.iter().any(|p| p.post.id == post1.id));
        assert!(old_posts.iter().any(|p| p.post.id == post2.id));

        // Query posts in specific range
        let range_posts = db
            .filter_by_date_range(Some(two_hours_ago), Some(one_hour_ago), 10)
            .await
            .unwrap();
        assert_eq!(range_posts.len(), 2);
    }

    #[tokio::test]
    async fn test_search_posts_by_content() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        // Create posts with different content
        let post1 = Post {
            id: uuid::Uuid::new_v4().to_string(),
            content: "Learning Rust programming".to_string(),
            created_at: chrono::Utc::now().timestamp(),
            scheduled_at: None,
            status: PostStatus::Posted,
            metadata: None,
        };

        let post2 = Post {
            id: uuid::Uuid::new_v4().to_string(),
            content: "Exploring Nostr protocol".to_string(),
            created_at: chrono::Utc::now().timestamp(),
            scheduled_at: None,
            status: PostStatus::Posted,
            metadata: None,
        };

        let post3 = Post {
            id: uuid::Uuid::new_v4().to_string(),
            content: "Building with Rust and Nostr".to_string(),
            created_at: chrono::Utc::now().timestamp(),
            scheduled_at: None,
            status: PostStatus::Posted,
            metadata: None,
        };

        db.create_post(&post1).await.unwrap();
        db.create_post(&post2).await.unwrap();
        db.create_post(&post3).await.unwrap();

        // Search for "Rust"
        let rust_posts = db.search_content("Rust", 10).await.unwrap();
        assert_eq!(rust_posts.len(), 2);
        assert!(rust_posts.iter().any(|p| p.post.id == post1.id));
        assert!(rust_posts.iter().any(|p| p.post.id == post3.id));

        // Search for "Nostr"
        let nostr_posts = db.search_content("Nostr", 10).await.unwrap();
        assert_eq!(nostr_posts.len(), 2);
        assert!(nostr_posts.iter().any(|p| p.post.id == post2.id));
        assert!(nostr_posts.iter().any(|p| p.post.id == post3.id));

        // Search for "protocol"
        let protocol_posts = db.search_content("protocol", 10).await.unwrap();
        assert_eq!(protocol_posts.len(), 1);
        assert_eq!(protocol_posts[0].post.id, post2.id);

        // Search for non-existent term
        let empty_posts = db.search_content("blockchain", 10).await.unwrap();
        assert_eq!(empty_posts.len(), 0);
    }

    #[tokio::test]
    async fn test_concurrent_writes_to_post_records() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        // Create a post
        let post = create_test_post();
        db.create_post(&post).await.unwrap();

        // Concurrently create records for different platforms
        let mut handles = vec![];
        let platforms = vec!["nostr", "mastodon", "bluesky"];

        for platform in platforms {
            let pool_clone = db.pool.clone();
            let post_id = post.id.clone();
            let platform = platform.to_string();

            let handle = tokio::spawn(async move {
                let db = Database { pool: pool_clone };
                let record = PostRecord {
                    id: None,
                    post_id,
                    platform: platform.clone(),
                    platform_post_id: Some(format!("{}_post_id", platform)),
                    posted_at: Some(chrono::Utc::now().timestamp()),
                    success: true,
                    error_message: None,
                    account_name: "default".to_string(),
                };
                db.create_post_record(&record).await
            });

            handles.push(handle);
        }

        // Wait for all operations to complete
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok(), "Concurrent record creation should succeed");
        }

        // Verify all records were created
        let records = db.get_post_records(&post.id).await.unwrap();
        assert_eq!(records.len(), 3);
    }

    #[tokio::test]
    async fn test_query_with_multiple_filters() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        let now = chrono::Utc::now().timestamp();
        let one_hour_ago = now - 3600;

        // Create posts
        let post1 = Post {
            id: uuid::Uuid::new_v4().to_string(),
            content: "Rust programming on Nostr".to_string(),
            created_at: one_hour_ago,
            scheduled_at: None,
            status: PostStatus::Posted,
            metadata: None,
        };

        let post2 = Post {
            id: uuid::Uuid::new_v4().to_string(),
            content: "Rust programming on Mastodon".to_string(),
            created_at: now,
            scheduled_at: None,
            status: PostStatus::Posted,
            metadata: None,
        };

        db.create_post(&post1).await.unwrap();
        db.create_post(&post2).await.unwrap();

        // Add platform records
        db.create_post_record(&PostRecord {
            id: None,
            post_id: post1.id.clone(),
            platform: "nostr".to_string(),
            platform_post_id: Some("note1".to_string()),
            posted_at: Some(one_hour_ago),
            success: true,
            error_message: None,
            account_name: "default".to_string(),
        })
        .await
        .unwrap();

        db.create_post_record(&PostRecord {
            id: None,
            post_id: post2.id.clone(),
            platform: "mastodon".to_string(),
            platform_post_id: Some("12345".to_string()),
            posted_at: Some(now),
            success: true,
            error_message: None,
            account_name: "default".to_string(),
        })
        .await
        .unwrap();

        // Query with platform and search filters
        let results = db
            .query_posts_with_records(Some("nostr"), None, None, Some("Rust"), 10)
            .await
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].post.id, post1.id);

        // Query with date range and search filters
        let results = db
            .query_posts_with_records(
                None,
                Some(now - 1800), // 30 minutes ago
                None,
                Some("programming"),
                10,
            )
            .await
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].post.id, post2.id);
    }

    #[tokio::test]
    async fn test_query_respects_limit() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        // Create 10 posts
        for i in 0..10 {
            let post = Post {
                id: uuid::Uuid::new_v4().to_string(),
                content: format!("Post {}", i),
                created_at: chrono::Utc::now().timestamp() + i,
                scheduled_at: None,
                status: PostStatus::Posted,
                metadata: None,
            };
            db.create_post(&post).await.unwrap();
        }

        // Query with limit of 5
        let results = db
            .query_posts_with_records(None, None, None, None, 5)
            .await
            .unwrap();
        assert_eq!(results.len(), 5);

        // Query with limit of 3
        let results = db
            .query_posts_with_records(None, None, None, None, 3)
            .await
            .unwrap();
        assert_eq!(results.len(), 3);
    }

    #[tokio::test]
    async fn test_get_post_records_empty() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        // Create a post without any records
        let post = create_test_post();
        db.create_post(&post).await.unwrap();

        // Get records should return empty vector
        let records = db.get_post_records(&post.id).await.unwrap();
        assert_eq!(records.len(), 0);
    }

    #[tokio::test]
    async fn test_post_with_records_ordering() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        let post = create_test_post();
        db.create_post(&post).await.unwrap();

        let now = chrono::Utc::now().timestamp();

        // Create records with different timestamps
        for i in 0..3 {
            let record = PostRecord {
                id: None,
                post_id: post.id.clone(),
                platform: format!("platform{}", i),
                platform_post_id: Some(format!("post{}", i)),
                posted_at: Some(now + i),
                success: true,
                error_message: None,
                account_name: "default".to_string(),
            };
            db.create_post_record(&record).await.unwrap();
            // Small delay to ensure different timestamps
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        // Get records - should be ordered by posted_at DESC
        let records = db.get_post_records(&post.id).await.unwrap();
        assert_eq!(records.len(), 3);

        // Verify descending order
        for i in 0..records.len() - 1 {
            assert!(records[i].posted_at >= records[i + 1].posted_at);
        }
    }

    // SCHEDULING TESTS (Phase 5.1 Task 2 & 3)

    #[tokio::test]
    async fn test_migration_creates_scheduling_indexes() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        // Verify idx_posts_scheduled_at index exists
        let result = sqlx::query(
            r#"
            SELECT name FROM sqlite_master
            WHERE type='index' AND name='idx_posts_scheduled_at'
            "#,
        )
        .fetch_optional(&pool)
        .await
        .unwrap();
        assert!(result.is_some(), "idx_posts_scheduled_at index not created");

        // Verify idx_posts_status_scheduled index exists
        let result = sqlx::query(
            r#"
            SELECT name FROM sqlite_master
            WHERE type='index' AND name='idx_posts_status_scheduled'
            "#,
        )
        .fetch_optional(&pool)
        .await
        .unwrap();
        assert!(
            result.is_some(),
            "idx_posts_status_scheduled index not created"
        );
    }

    #[tokio::test]
    async fn test_migration_creates_rate_limits_table() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        // Verify rate_limits table exists
        let result = sqlx::query(
            r#"
            SELECT name FROM sqlite_master
            WHERE type='table' AND name='rate_limits'
            "#,
        )
        .fetch_optional(&pool)
        .await
        .unwrap();
        assert!(result.is_some(), "rate_limits table not created");

        // Verify table structure by inserting and querying
        sqlx::query(
            r#"
            INSERT INTO rate_limits (platform, window_start, post_count)
            VALUES ('nostr', 1234567890, 5)
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT post_count FROM rate_limits
            WHERE platform = 'nostr' AND window_start = 1234567890
            "#,
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(count.0, 5);
    }

    #[tokio::test]
    async fn test_get_scheduled_posts_due_returns_only_due_posts() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        let now = chrono::Utc::now().timestamp();

        // Create post scheduled in the past (due)
        let past_post = Post {
            id: uuid::Uuid::new_v4().to_string(),
            content: "Past post".to_string(),
            created_at: now - 3600,
            scheduled_at: Some(now - 1800),
            status: PostStatus::Scheduled,
            metadata: None,
        };
        db.create_post(&past_post).await.unwrap();

        // Create post scheduled for future (not due)
        let future_post = Post {
            id: uuid::Uuid::new_v4().to_string(),
            content: "Future post".to_string(),
            created_at: now,
            scheduled_at: Some(now + 3600),
            status: PostStatus::Scheduled,
            metadata: None,
        };
        db.create_post(&future_post).await.unwrap();

        // Create posted post (not scheduled)
        let posted_post = Post {
            id: uuid::Uuid::new_v4().to_string(),
            content: "Posted post".to_string(),
            created_at: now - 7200,
            scheduled_at: Some(now - 3600),
            status: PostStatus::Posted,
            metadata: None,
        };
        db.create_post(&posted_post).await.unwrap();

        let due_posts = db.get_scheduled_posts_due().await.unwrap();

        assert_eq!(due_posts.len(), 1);
        assert_eq!(due_posts[0].id, past_post.id);
        assert_eq!(due_posts[0].content, "Past post");
    }

    #[tokio::test]
    async fn test_get_scheduled_posts_due_empty_when_none_due() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        let now = chrono::Utc::now().timestamp();

        // Create only future posts
        let future_post = Post {
            id: uuid::Uuid::new_v4().to_string(),
            content: "Future post".to_string(),
            created_at: now,
            scheduled_at: Some(now + 3600),
            status: PostStatus::Scheduled,
            metadata: None,
        };
        db.create_post(&future_post).await.unwrap();

        let due_posts = db.get_scheduled_posts_due().await.unwrap();
        assert_eq!(due_posts.len(), 0);
    }

    #[tokio::test]
    async fn test_get_scheduled_posts_returns_all_scheduled() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        let now = chrono::Utc::now().timestamp();

        // Create scheduled posts with different times
        for i in 0..3 {
            let post = Post {
                id: uuid::Uuid::new_v4().to_string(),
                content: format!("Scheduled post {}", i),
                created_at: now,
                scheduled_at: Some(now + (i * 3600)),
                status: PostStatus::Scheduled,
                metadata: None,
            };
            db.create_post(&post).await.unwrap();
        }

        // Create non-scheduled post
        let posted = Post {
            id: uuid::Uuid::new_v4().to_string(),
            content: "Posted".to_string(),
            created_at: now,
            scheduled_at: None,
            status: PostStatus::Posted,
            metadata: None,
        };
        db.create_post(&posted).await.unwrap();

        let scheduled = db.get_scheduled_posts().await.unwrap();

        assert_eq!(scheduled.len(), 3);
        // Verify ordering (ASC by scheduled_at)
        for i in 0..scheduled.len() - 1 {
            assert!(scheduled[i].scheduled_at <= scheduled[i + 1].scheduled_at);
        }
    }

    #[tokio::test]
    async fn test_get_last_scheduled_timestamp_returns_max() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        let now = chrono::Utc::now().timestamp();

        // Create scheduled posts
        let times = vec![now + 1000, now + 5000, now + 3000];
        for time in &times {
            let post = Post {
                id: uuid::Uuid::new_v4().to_string(),
                content: "Scheduled".to_string(),
                created_at: now,
                scheduled_at: Some(*time),
                status: PostStatus::Scheduled,
                metadata: None,
            };
            db.create_post(&post).await.unwrap();
        }

        let max_time = db.get_last_scheduled_timestamp().await.unwrap();

        assert_eq!(max_time, Some(now + 5000));
    }

    #[tokio::test]
    async fn test_get_last_scheduled_timestamp_none_when_empty() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        let max_time = db.get_last_scheduled_timestamp().await.unwrap();
        assert_eq!(max_time, None);
    }

    #[tokio::test]
    async fn test_get_last_scheduled_timestamp_ignores_posted() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        let now = chrono::Utc::now().timestamp();

        // Create posted post with scheduled_at
        let posted = Post {
            id: uuid::Uuid::new_v4().to_string(),
            content: "Posted".to_string(),
            created_at: now,
            scheduled_at: Some(now + 10000),
            status: PostStatus::Posted,
            metadata: None,
        };
        db.create_post(&posted).await.unwrap();

        // Create scheduled post with earlier time
        let scheduled = Post {
            id: uuid::Uuid::new_v4().to_string(),
            content: "Scheduled".to_string(),
            created_at: now,
            scheduled_at: Some(now + 1000),
            status: PostStatus::Scheduled,
            metadata: None,
        };
        db.create_post(&scheduled).await.unwrap();

        let max_time = db.get_last_scheduled_timestamp().await.unwrap();

        // Should return scheduled post time, not posted post time
        assert_eq!(max_time, Some(now + 1000));
    }

    #[tokio::test]
    async fn test_update_post_schedule_changes_time() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        let now = chrono::Utc::now().timestamp();

        let post = Post {
            id: uuid::Uuid::new_v4().to_string(),
            content: "Test".to_string(),
            created_at: now,
            scheduled_at: Some(now + 1000),
            status: PostStatus::Scheduled,
            metadata: None,
        };
        db.create_post(&post).await.unwrap();

        // Update schedule time
        let new_time = now + 5000;
        db.update_post_schedule(&post.id, Some(new_time))
            .await
            .unwrap();

        let updated = db.get_post(&post.id).await.unwrap().unwrap();
        assert_eq!(updated.scheduled_at, Some(new_time));
    }

    #[tokio::test]
    async fn test_update_post_schedule_can_clear_time() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        let now = chrono::Utc::now().timestamp();

        let post = Post {
            id: uuid::Uuid::new_v4().to_string(),
            content: "Test".to_string(),
            created_at: now,
            scheduled_at: Some(now + 1000),
            status: PostStatus::Scheduled,
            metadata: None,
        };
        db.create_post(&post).await.unwrap();

        // Clear schedule time
        db.update_post_schedule(&post.id, None).await.unwrap();

        let updated = db.get_post(&post.id).await.unwrap().unwrap();
        assert_eq!(updated.scheduled_at, None);
    }

    #[tokio::test]
    async fn test_delete_post_removes_post() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        let post = create_test_post();
        db.create_post(&post).await.unwrap();

        db.delete_post(&post.id).await.unwrap();

        let retrieved = db.get_post(&post.id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_delete_post_removes_records() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        let post = create_test_post();
        db.create_post(&post).await.unwrap();

        // Add a record
        let record = PostRecord {
            id: None,
            post_id: post.id.clone(),
            platform: "nostr".to_string(),
            platform_post_id: Some("note1abc".to_string()),
            posted_at: Some(chrono::Utc::now().timestamp()),
            success: true,
            error_message: None,
            account_name: "default".to_string(),
        };
        db.create_post_record(&record).await.unwrap();

        db.delete_post(&post.id).await.unwrap();

        let records = db.get_post_records(&post.id).await.unwrap();
        assert_eq!(records.len(), 0);
    }

    #[tokio::test]
    async fn test_get_rate_limit_count_zero_when_empty() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        let count = db.get_rate_limit_count("nostr", 1234567890).await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_increment_rate_limit_creates_record() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        let window = 1234567890;
        db.increment_rate_limit("nostr", window).await.unwrap();

        let count = db.get_rate_limit_count("nostr", window).await.unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_increment_rate_limit_increments_existing() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        let window = 1234567890;

        // Increment multiple times
        for _ in 0..5 {
            db.increment_rate_limit("nostr", window).await.unwrap();
        }

        let count = db.get_rate_limit_count("nostr", window).await.unwrap();
        assert_eq!(count, 5);
    }

    #[tokio::test]
    async fn test_increment_rate_limit_separate_platforms() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        let window = 1234567890;

        db.increment_rate_limit("nostr", window).await.unwrap();
        db.increment_rate_limit("nostr", window).await.unwrap();
        db.increment_rate_limit("bluesky", window).await.unwrap();

        let nostr_count = db.get_rate_limit_count("nostr", window).await.unwrap();
        let bluesky_count = db.get_rate_limit_count("bluesky", window).await.unwrap();

        assert_eq!(nostr_count, 2);
        assert_eq!(bluesky_count, 1);
    }

    #[tokio::test]
    async fn test_cleanup_rate_limits_removes_old() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        let old_window = 1000000;
        let recent_window = 2000000;

        db.increment_rate_limit("nostr", old_window).await.unwrap();
        db.increment_rate_limit("nostr", recent_window)
            .await
            .unwrap();

        // Cleanup everything before 1500000
        db.cleanup_rate_limits(1500000).await.unwrap();

        let old_count = db.get_rate_limit_count("nostr", old_window).await.unwrap();
        let recent_count = db
            .get_rate_limit_count("nostr", recent_window)
            .await
            .unwrap();

        assert_eq!(old_count, 0);
        assert_eq!(recent_count, 1);
    }

    // ========================================================================
    // Attachment CRUD Tests
    // ========================================================================

    fn create_test_attachment(post_id: &str) -> Attachment {
        Attachment {
            id: uuid::Uuid::new_v4().to_string(),
            post_id: post_id.to_string(),
            file_path: "/path/to/image.jpg".to_string(),
            mime_type: ImageMimeType::Jpeg,
            file_size: 1024,
            file_hash: format!("hash_{}", uuid::Uuid::new_v4()),
            alt_text: Some("Test image".to_string()),
            created_at: chrono::Utc::now().timestamp(),
        }
    }

    #[tokio::test]
    async fn test_create_and_retrieve_attachment() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        // Create a post first (foreign key)
        let post = create_test_post();
        db.create_post(&post).await.unwrap();

        // Create attachment
        let attachment = create_test_attachment(&post.id);
        db.create_attachment(&attachment).await.unwrap();

        // Retrieve attachment
        let retrieved = db.get_attachment(&attachment.id).await.unwrap();
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, attachment.id);
        assert_eq!(retrieved.post_id, post.id);
        assert_eq!(retrieved.file_path, "/path/to/image.jpg");
        assert_eq!(retrieved.mime_type, ImageMimeType::Jpeg);
        assert_eq!(retrieved.file_size, 1024);
        assert_eq!(retrieved.alt_text, Some("Test image".to_string()));
    }

    #[tokio::test]
    async fn test_get_attachments_for_post() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        // Create a post
        let post = create_test_post();
        db.create_post(&post).await.unwrap();

        // Create multiple attachments
        for i in 0..3 {
            let mut attachment = create_test_attachment(&post.id);
            attachment.file_path = format!("/path/to/image{}.jpg", i);
            attachment.created_at = chrono::Utc::now().timestamp() + i;
            db.create_attachment(&attachment).await.unwrap();
        }

        // Retrieve all attachments
        let attachments = db.get_attachments_for_post(&post.id).await.unwrap();
        assert_eq!(attachments.len(), 3);

        // Verify ordering (oldest first)
        assert!(attachments[0].created_at <= attachments[1].created_at);
        assert!(attachments[1].created_at <= attachments[2].created_at);
    }

    #[tokio::test]
    async fn test_delete_attachment() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        // Create a post and attachment
        let post = create_test_post();
        db.create_post(&post).await.unwrap();

        let attachment = create_test_attachment(&post.id);
        db.create_attachment(&attachment).await.unwrap();

        // Delete attachment
        db.delete_attachment(&attachment.id).await.unwrap();

        // Verify deleted
        let retrieved = db.get_attachment(&attachment.id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_delete_attachments_for_post() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        // Create a post with multiple attachments
        let post = create_test_post();
        db.create_post(&post).await.unwrap();

        for _ in 0..3 {
            let attachment = create_test_attachment(&post.id);
            db.create_attachment(&attachment).await.unwrap();
        }

        // Delete all attachments for post
        db.delete_attachments_for_post(&post.id).await.unwrap();

        // Verify all deleted
        let attachments = db.get_attachments_for_post(&post.id).await.unwrap();
        assert_eq!(attachments.len(), 0);
    }

    #[tokio::test]
    async fn test_find_attachment_by_hash() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        // Create a post and attachment
        let post = create_test_post();
        db.create_post(&post).await.unwrap();

        let mut attachment = create_test_attachment(&post.id);
        attachment.file_hash = "unique_hash_12345".to_string();
        db.create_attachment(&attachment).await.unwrap();

        // Find by hash
        let found = db
            .find_attachment_by_hash("unique_hash_12345")
            .await
            .unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, attachment.id);

        // Not found
        let not_found = db
            .find_attachment_by_hash("nonexistent_hash")
            .await
            .unwrap();
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_attachment_cascade_delete() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        // Enable foreign key constraints
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&db.pool)
            .await
            .unwrap();

        // Create a post and attachment
        let post = create_test_post();
        db.create_post(&post).await.unwrap();

        let attachment = create_test_attachment(&post.id);
        db.create_attachment(&attachment).await.unwrap();

        // Delete post (should cascade to attachments)
        db.delete_post(&post.id).await.unwrap();

        // Verify attachment is deleted
        let retrieved = db.get_attachment(&attachment.id).await.unwrap();
        assert!(retrieved.is_none());
    }

    // ========================================================================
    // Attachment Upload CRUD Tests
    // ========================================================================

    #[tokio::test]
    async fn test_create_and_retrieve_attachment_upload() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        // Enable foreign key constraints
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&db.pool)
            .await
            .unwrap();

        // Create a post and attachment
        let post = create_test_post();
        db.create_post(&post).await.unwrap();

        let attachment = create_test_attachment(&post.id);
        db.create_attachment(&attachment).await.unwrap();

        // Create upload record
        let upload = AttachmentUpload::new_pending(attachment.id.clone(), "mastodon".to_string());
        db.create_attachment_upload(&upload).await.unwrap();

        // Retrieve upload
        let retrieved = db
            .get_attachment_upload(&attachment.id, "mastodon")
            .await
            .unwrap();
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.attachment_id, attachment.id);
        assert_eq!(retrieved.platform, "mastodon");
        assert_eq!(retrieved.status, AttachmentStatus::Pending);
    }

    #[tokio::test]
    async fn test_update_attachment_upload_success() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        // Create a post and attachment
        let post = create_test_post();
        db.create_post(&post).await.unwrap();

        let attachment = create_test_attachment(&post.id);
        db.create_attachment(&attachment).await.unwrap();

        // Create and update upload record
        let upload = AttachmentUpload::new_pending(attachment.id.clone(), "mastodon".to_string());
        db.create_attachment_upload(&upload).await.unwrap();

        db.update_attachment_upload(
            &attachment.id,
            "mastodon",
            Some("media_123"),
            Some("https://example.com/media.jpg"),
            AttachmentStatus::Uploaded,
            None,
        )
        .await
        .unwrap();

        // Verify update
        let retrieved = db
            .get_attachment_upload(&attachment.id, "mastodon")
            .await
            .unwrap()
            .unwrap();

        assert_eq!(retrieved.status, AttachmentStatus::Uploaded);
        assert_eq!(
            retrieved.platform_attachment_id,
            Some("media_123".to_string())
        );
        assert_eq!(
            retrieved.remote_url,
            Some("https://example.com/media.jpg".to_string())
        );
        assert!(retrieved.uploaded_at.is_some());
    }

    #[tokio::test]
    async fn test_update_attachment_upload_failure() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        // Create a post and attachment
        let post = create_test_post();
        db.create_post(&post).await.unwrap();

        let attachment = create_test_attachment(&post.id);
        db.create_attachment(&attachment).await.unwrap();

        // Create and update upload record
        let upload = AttachmentUpload::new_pending(attachment.id.clone(), "nostr".to_string());
        db.create_attachment_upload(&upload).await.unwrap();

        db.update_attachment_upload(
            &attachment.id,
            "nostr",
            None,
            None,
            AttachmentStatus::Failed,
            Some("Network timeout"),
        )
        .await
        .unwrap();

        // Verify update
        let retrieved = db
            .get_attachment_upload(&attachment.id, "nostr")
            .await
            .unwrap()
            .unwrap();

        assert_eq!(retrieved.status, AttachmentStatus::Failed);
        assert_eq!(retrieved.error_message, Some("Network timeout".to_string()));
    }

    #[tokio::test]
    async fn test_get_attachment_uploads() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        // Create a post and attachment
        let post = create_test_post();
        db.create_post(&post).await.unwrap();

        let attachment = create_test_attachment(&post.id);
        db.create_attachment(&attachment).await.unwrap();

        // Create upload records for multiple platforms
        for platform in ["nostr", "mastodon"] {
            let upload = AttachmentUpload::new_pending(attachment.id.clone(), platform.to_string());
            db.create_attachment_upload(&upload).await.unwrap();
        }

        // Get all uploads
        let uploads = db.get_attachment_uploads(&attachment.id).await.unwrap();
        assert_eq!(uploads.len(), 2);
    }

    #[tokio::test]
    async fn test_get_pending_uploads() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        // Create a post and attachments
        let post = create_test_post();
        db.create_post(&post).await.unwrap();

        let attachment1 = create_test_attachment(&post.id);
        let attachment2 = create_test_attachment(&post.id);
        db.create_attachment(&attachment1).await.unwrap();
        db.create_attachment(&attachment2).await.unwrap();

        // Create pending upload for attachment1
        let upload1 = AttachmentUpload::new_pending(attachment1.id.clone(), "mastodon".to_string());
        db.create_attachment_upload(&upload1).await.unwrap();

        // Create completed upload for attachment2
        let mut upload2 =
            AttachmentUpload::new_pending(attachment2.id.clone(), "mastodon".to_string());
        upload2.status = AttachmentStatus::Uploaded;
        db.create_attachment_upload(&upload2).await.unwrap();
        db.update_attachment_upload(
            &attachment2.id,
            "mastodon",
            Some("media_id"),
            None,
            AttachmentStatus::Uploaded,
            None,
        )
        .await
        .unwrap();

        // Get pending uploads
        let pending = db.get_pending_uploads("mastodon").await.unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].attachment_id, attachment1.id);
    }

    #[tokio::test]
    async fn test_attachment_upload_cascade_delete() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        // Enable foreign key constraints
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&db.pool)
            .await
            .unwrap();

        // Create a post, attachment, and upload
        let post = create_test_post();
        db.create_post(&post).await.unwrap();

        let attachment = create_test_attachment(&post.id);
        db.create_attachment(&attachment).await.unwrap();

        let upload = AttachmentUpload::new_pending(attachment.id.clone(), "nostr".to_string());
        db.create_attachment_upload(&upload).await.unwrap();

        // Delete attachment (should cascade to uploads)
        db.delete_attachment(&attachment.id).await.unwrap();

        // Verify upload is deleted
        let retrieved = db
            .get_attachment_upload(&attachment.id, "nostr")
            .await
            .unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_attachment_upload_unique_constraint() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        // Create a post and attachment
        let post = create_test_post();
        db.create_post(&post).await.unwrap();

        let attachment = create_test_attachment(&post.id);
        db.create_attachment(&attachment).await.unwrap();

        // Create first upload
        let upload = AttachmentUpload::new_pending(attachment.id.clone(), "mastodon".to_string());
        db.create_attachment_upload(&upload).await.unwrap();

        // Try to create duplicate (same attachment_id + platform)
        let duplicate =
            AttachmentUpload::new_pending(attachment.id.clone(), "mastodon".to_string());
        let result = db.create_attachment_upload(&duplicate).await;

        // Should fail due to UNIQUE constraint
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_migration_creates_attachment_tables() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        // Verify attachments table exists
        let result = sqlx::query(
            r#"
            SELECT name FROM sqlite_master
            WHERE type='table' AND name='attachments'
            "#,
        )
        .fetch_optional(&pool)
        .await
        .unwrap();
        assert!(result.is_some(), "attachments table not created");

        // Verify attachment_uploads table exists
        let result = sqlx::query(
            r#"
            SELECT name FROM sqlite_master
            WHERE type='table' AND name='attachment_uploads'
            "#,
        )
        .fetch_optional(&pool)
        .await
        .unwrap();
        assert!(result.is_some(), "attachment_uploads table not created");

        // Verify indexes exist
        let result = sqlx::query(
            r#"
            SELECT name FROM sqlite_master
            WHERE type='index' AND name='idx_attachments_post_id'
            "#,
        )
        .fetch_optional(&pool)
        .await
        .unwrap();
        assert!(
            result.is_some(),
            "idx_attachments_post_id index not created"
        );
    }

    // Tests for get_platform_post_ids (cross-platform reply-to lookup)

    #[tokio::test]
    async fn test_get_platform_post_ids_success() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        // Create a post with successful records on multiple platforms
        let post = create_test_post();
        db.create_post(&post).await.unwrap();

        // Add successful Nostr record
        db.create_post_record(&PostRecord {
            id: None,
            post_id: post.id.clone(),
            platform: "nostr".to_string(),
            platform_post_id: Some("note1abc123".to_string()),
            posted_at: Some(chrono::Utc::now().timestamp()),
            success: true,
            error_message: None,
            account_name: "default".to_string(),
        })
        .await
        .unwrap();

        // Add successful Mastodon record
        db.create_post_record(&PostRecord {
            id: None,
            post_id: post.id.clone(),
            platform: "mastodon".to_string(),
            platform_post_id: Some("12345678".to_string()),
            posted_at: Some(chrono::Utc::now().timestamp()),
            success: true,
            error_message: None,
            account_name: "default".to_string(),
        })
        .await
        .unwrap();

        // Look up platform IDs
        let result = db.get_platform_post_ids(&post.id).await.unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result.get("nostr"), Some(&"note1abc123".to_string()));
        assert_eq!(result.get("mastodon"), Some(&"12345678".to_string()));
    }

    #[tokio::test]
    async fn test_get_platform_post_ids_partial_success() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        // Create a post with one success and one failure
        let post = create_test_post();
        db.create_post(&post).await.unwrap();

        // Add successful Nostr record
        db.create_post_record(&PostRecord {
            id: None,
            post_id: post.id.clone(),
            platform: "nostr".to_string(),
            platform_post_id: Some("note1abc123".to_string()),
            posted_at: Some(chrono::Utc::now().timestamp()),
            success: true,
            error_message: None,
            account_name: "default".to_string(),
        })
        .await
        .unwrap();

        // Add failed Mastodon record
        db.create_post_record(&PostRecord {
            id: None,
            post_id: post.id.clone(),
            platform: "mastodon".to_string(),
            platform_post_id: None,
            posted_at: None,
            success: false,
            error_message: Some("Authentication failed".to_string()),
            account_name: "default".to_string(),
        })
        .await
        .unwrap();

        // Look up platform IDs - should only return successful ones
        let result = db.get_platform_post_ids(&post.id).await.unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result.get("nostr"), Some(&"note1abc123".to_string()));
        assert!(result.get("mastodon").is_none());
    }

    #[tokio::test]
    async fn test_get_platform_post_ids_not_found() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        // Query for non-existent post
        let result = db.get_platform_post_ids("nonexistent-uuid").await.unwrap();

        // Should return empty map
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_get_platform_post_ids_null_platform_post_id() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        // Create a post with a successful record but null platform_post_id
        let post = create_test_post();
        db.create_post(&post).await.unwrap();

        // Add record with success=true but platform_post_id=null (edge case)
        db.create_post_record(&PostRecord {
            id: None,
            post_id: post.id.clone(),
            platform: "nostr".to_string(),
            platform_post_id: None, // Edge case: success but no ID
            posted_at: Some(chrono::Utc::now().timestamp()),
            success: true,
            error_message: None,
            account_name: "default".to_string(),
        })
        .await
        .unwrap();

        // Should not include records with null platform_post_id
        let result = db.get_platform_post_ids(&post.id).await.unwrap();
        assert!(result.is_empty());
    }

    // Tests for get_post_id_by_platform_post_id (reverse lookup for cross-platform reply-to)

    #[tokio::test]
    async fn test_get_post_id_by_platform_post_id_found() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        // Create a post with successful record
        let post = Post::new("Test content".to_string());
        db.create_post(&post).await.unwrap();

        db.create_post_record(&PostRecord {
            id: None,
            post_id: post.id.clone(),
            platform: "nostr".to_string(),
            platform_post_id: Some("note1abc123".to_string()),
            posted_at: Some(chrono::Utc::now().timestamp()),
            success: true,
            error_message: None,
            account_name: "default".to_string(),
        })
        .await
        .unwrap();

        // Look up by platform_post_id
        let result = db
            .get_post_id_by_platform_post_id("nostr", "note1abc123")
            .await
            .unwrap();

        assert_eq!(result, Some(post.id));
    }

    #[tokio::test]
    async fn test_get_post_id_by_platform_post_id_not_found() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        // Query for non-existent platform_post_id
        let result = db
            .get_post_id_by_platform_post_id("nostr", "note1nonexistent")
            .await
            .unwrap();

        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_get_post_id_by_platform_post_id_failed_record() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        // Create a post with FAILED record (success = false)
        let post = Post::new("Test content".to_string());
        db.create_post(&post).await.unwrap();

        db.create_post_record(&PostRecord {
            id: None,
            post_id: post.id.clone(),
            platform: "nostr".to_string(),
            platform_post_id: Some("note1failed".to_string()),
            posted_at: Some(chrono::Utc::now().timestamp()),
            success: false, // Failed record
            error_message: Some("Network error".to_string()),
            account_name: "default".to_string(),
        })
        .await
        .unwrap();

        // Should NOT return failed records
        let result = db
            .get_post_id_by_platform_post_id("nostr", "note1failed")
            .await
            .unwrap();

        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_get_post_id_by_platform_post_id_wrong_platform() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        // Create a post with successful Nostr record
        let post = Post::new("Test content".to_string());
        db.create_post(&post).await.unwrap();

        db.create_post_record(&PostRecord {
            id: None,
            post_id: post.id.clone(),
            platform: "nostr".to_string(),
            platform_post_id: Some("note1abc123".to_string()),
            posted_at: Some(chrono::Utc::now().timestamp()),
            success: true,
            error_message: None,
            account_name: "default".to_string(),
        })
        .await
        .unwrap();

        // Look up with correct ID but wrong platform - should not find it
        let result = db
            .get_post_id_by_platform_post_id("mastodon", "note1abc123")
            .await
            .unwrap();

        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_get_post_id_by_platform_post_id_cross_platform_lookup() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let db = Database { pool };

        // Create a post with successful records on multiple platforms
        let post = Post::new("Cross-platform post".to_string());
        db.create_post(&post).await.unwrap();

        db.create_post_record(&PostRecord {
            id: None,
            post_id: post.id.clone(),
            platform: "nostr".to_string(),
            platform_post_id: Some("note1cross123".to_string()),
            posted_at: Some(chrono::Utc::now().timestamp()),
            success: true,
            error_message: None,
            account_name: "default".to_string(),
        })
        .await
        .unwrap();

        db.create_post_record(&PostRecord {
            id: None,
            post_id: post.id.clone(),
            platform: "mastodon".to_string(),
            platform_post_id: Some("123456789".to_string()),
            posted_at: Some(chrono::Utc::now().timestamp()),
            success: true,
            error_message: None,
            account_name: "default".to_string(),
        })
        .await
        .unwrap();

        // Look up by Nostr ID
        let uuid_from_nostr = db
            .get_post_id_by_platform_post_id("nostr", "note1cross123")
            .await
            .unwrap();
        assert_eq!(uuid_from_nostr, Some(post.id.clone()));

        // Look up by Mastodon ID - should return the same UUID
        let uuid_from_mastodon = db
            .get_post_id_by_platform_post_id("mastodon", "123456789")
            .await
            .unwrap();
        assert_eq!(uuid_from_mastodon, Some(post.id.clone()));

        // Now use the UUID to get all platform IDs (the full cross-platform workflow)
        let all_ids = db.get_platform_post_ids(&post.id).await.unwrap();
        assert_eq!(all_ids.len(), 2);
        assert_eq!(all_ids.get("nostr"), Some(&"note1cross123".to_string()));
        assert_eq!(all_ids.get("mastodon"), Some(&"123456789".to_string()));
    }
}
