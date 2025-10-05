//! Database operations for Plurcast

use sqlx::sqlite::SqlitePool;
use std::path::Path;

use crate::error::Result;
use crate::types::{Post, PostRecord, PostStatus};

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
        let pool = SqlitePool::connect(&format!("sqlite:{}", expanded_path))
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
}
