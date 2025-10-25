//! Database operations for Plurcast

use sqlx::sqlite::SqlitePool;
use std::path::Path;

use crate::error::Result;
use crate::types::{Post, PostRecord, PostStatus};

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
            INSERT INTO post_records (post_id, platform, platform_post_id, posted_at, success, error_message)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&record.post_id)
        .bind(&record.platform)
        .bind(&record.platform_post_id)
        .bind(record.posted_at)
        .bind(success)
        .bind(&record.error_message)
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

        let rows = query.fetch_all(&self.pool).await
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
            SELECT id, post_id, platform, platform_post_id, posted_at, success, error_message
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
            })
            .collect())
    }

    /// Filter posts by platform
    pub async fn filter_by_platform(&self, platform: &str, limit: usize) -> Result<Vec<PostWithRecords>> {
        self.query_posts_with_records(Some(platform), None, None, None, limit).await
    }

    /// Filter posts by date range
    pub async fn filter_by_date_range(
        &self,
        since: Option<i64>,
        until: Option<i64>,
        limit: usize,
    ) -> Result<Vec<PostWithRecords>> {
        self.query_posts_with_records(None, since, until, None, limit).await
    }

    /// Search posts by content
    pub async fn search_content(&self, search_term: &str, limit: usize) -> Result<Vec<PostWithRecords>> {
        self.query_posts_with_records(None, None, None, Some(search_term), limit).await
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

            assert!(result.is_err(), "Expected error for read-only directory on Unix");
        }
        
        #[cfg(windows)]
        {
            // On Windows, the readonly attribute on directories doesn't prevent file creation
            // So we skip this test or test a different scenario
            // We'll test that we can still create a database in a normal directory
            let temp_dir = TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db");
            let result = Database::new(db_path.to_str().unwrap()).await;
            assert!(result.is_ok(), "Should be able to create database on Windows");
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
        assert!(result.is_ok(), "Database should still be functional after error");

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
        db.update_post_status(&post.id, PostStatus::Posted).await.unwrap();

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
        db.update_post_status(&post.id, PostStatus::Failed).await.unwrap();

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
            };
            
            let result = db.create_post_record(&record).await;
            assert!(result.is_ok(), "Should be able to create multiple records for same post");
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
            };

            db.create_post_record(&record).await.unwrap();
        }

        // Retrieve post with records
        let records = db.get_post_records(&post.id).await.unwrap();
        assert_eq!(records.len(), 3);

        // Verify each platform record
        let nostr_record = records.iter().find(|r| r.platform == "nostr").unwrap();
        assert!(nostr_record.success);
        assert_eq!(nostr_record.platform_post_id, Some("note1abc123".to_string()));

        let mastodon_record = records.iter().find(|r| r.platform == "mastodon").unwrap();
        assert!(mastodon_record.success);
        assert_eq!(mastodon_record.platform_post_id, Some("12345".to_string()));

        let bluesky_record = records.iter().find(|r| r.platform == "bluesky").unwrap();
        assert!(!bluesky_record.success);
        assert_eq!(bluesky_record.error_message, Some("Authentication failed".to_string()));
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
        }).await.unwrap();

        db.create_post_record(&PostRecord {
            id: None,
            post_id: post2.id.clone(),
            platform: "mastodon".to_string(),
            platform_post_id: Some("12345".to_string()),
            posted_at: Some(chrono::Utc::now().timestamp()),
            success: true,
            error_message: None,
        }).await.unwrap();

        db.create_post_record(&PostRecord {
            id: None,
            post_id: post3.id.clone(),
            platform: "nostr".to_string(),
            platform_post_id: Some("note2".to_string()),
            posted_at: Some(chrono::Utc::now().timestamp()),
            success: true,
            error_message: None,
        }).await.unwrap();

        // Filter by nostr platform
        let nostr_posts = db.filter_by_platform("nostr", 10).await.unwrap();
        assert_eq!(nostr_posts.len(), 2);
        assert!(nostr_posts.iter().all(|p| p.records.iter().any(|r| r.platform == "nostr")));

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
        let recent_posts = db.filter_by_date_range(Some(one_hour_ago), None, 10).await.unwrap();
        assert_eq!(recent_posts.len(), 2);
        assert!(recent_posts.iter().any(|p| p.post.id == post2.id));
        assert!(recent_posts.iter().any(|p| p.post.id == post3.id));

        // Query posts until one hour ago
        let old_posts = db.filter_by_date_range(None, Some(one_hour_ago), 10).await.unwrap();
        assert_eq!(old_posts.len(), 2);
        assert!(old_posts.iter().any(|p| p.post.id == post1.id));
        assert!(old_posts.iter().any(|p| p.post.id == post2.id));

        // Query posts in specific range
        let range_posts = db.filter_by_date_range(Some(two_hours_ago), Some(one_hour_ago), 10).await.unwrap();
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
        }).await.unwrap();

        db.create_post_record(&PostRecord {
            id: None,
            post_id: post2.id.clone(),
            platform: "mastodon".to_string(),
            platform_post_id: Some("12345".to_string()),
            posted_at: Some(now),
            success: true,
            error_message: None,
        }).await.unwrap();

        // Query with platform and search filters
        let results = db.query_posts_with_records(
            Some("nostr"),
            None,
            None,
            Some("Rust"),
            10
        ).await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].post.id, post1.id);

        // Query with date range and search filters
        let results = db.query_posts_with_records(
            None,
            Some(now - 1800), // 30 minutes ago
            None,
            Some("programming"),
            10
        ).await.unwrap();

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
        let results = db.query_posts_with_records(None, None, None, None, 5).await.unwrap();
        assert_eq!(results.len(), 5);

        // Query with limit of 3
        let results = db.query_posts_with_records(None, None, None, None, 3).await.unwrap();
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
}
