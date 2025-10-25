//! History service for querying post history
//!
//! This module provides flexible querying and analysis of post history.

use std::sync::Arc;
use crate::{Database, Result, PostStatus};
use crate::db::PostWithRecords;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// History service
///
/// Provides querying and analysis of post history with flexible filtering
/// and pagination.
pub struct HistoryService {
    db: Arc<Database>,
}

/// Query parameters for filtering posts
#[derive(Debug, Clone, Default)]
pub struct HistoryQuery {
    pub platform: Option<String>,
    pub status: Option<PostStatus>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub search: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// Statistics about post history
#[derive(Debug, Clone)]
pub struct HistoryStats {
    pub total_posts: usize,
    pub platform_stats: HashMap<String, PlatformStats>,
}

/// Statistics for a single platform
#[derive(Debug, Clone)]
pub struct PlatformStats {
    pub total: usize,
    pub successful: usize,
    pub failed: usize,
    pub success_rate: f64,
}

impl HistoryService {
    /// Create a new history service
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// List posts with filtering and pagination
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn list_posts(&self, query: HistoryQuery) -> Result<Vec<PostWithRecords>> {
        let platform = query.platform.as_deref();
        let since = query.since.map(|dt| dt.timestamp());
        let until = query.until.map(|dt| dt.timestamp());
        let search = query.search.as_deref();
        let limit = query.limit.unwrap_or(20);

        let mut results = self.db.query_posts_with_records(
            platform,
            since,
            until,
            search,
            limit,
        ).await?;

        // Apply offset if specified
        if let Some(offset) = query.offset {
            if offset < results.len() {
                results = results.into_iter().skip(offset).collect();
            } else {
                results = Vec::new();
            }
        }

        // Filter by status if specified
        if let Some(status) = query.status {
            results.retain(|pwr| matches_status(&pwr.post.status, &status));
        }

        Ok(results)
    }

    /// Get a single post by ID
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn get_post(&self, post_id: &str) -> Result<Option<PostWithRecords>> {
        let post = self.db.get_post(post_id).await?;
        
        match post {
            Some(p) => {
                let records = self.db.get_post_records(post_id).await?;
                Ok(Some(PostWithRecords {
                    post: p,
                    records,
                }))
            }
            None => Ok(None),
        }
    }

    /// Get statistics for posts matching the query
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn get_stats(&self, query: HistoryQuery) -> Result<HistoryStats> {
        let posts = self.list_posts(query).await?;
        
        let mut platform_stats: HashMap<String, PlatformStats> = HashMap::new();
        
        for post_with_records in &posts {
            for record in &post_with_records.records {
                let stats = platform_stats.entry(record.platform.clone()).or_insert(PlatformStats {
                    total: 0,
                    successful: 0,
                    failed: 0,
                    success_rate: 0.0,
                });
                
                stats.total += 1;
                if record.success {
                    stats.successful += 1;
                } else {
                    stats.failed += 1;
                }
            }
        }
        
        // Calculate success rates
        for stats in platform_stats.values_mut() {
            if stats.total > 0 {
                stats.success_rate = (stats.successful as f64 / stats.total as f64) * 100.0;
            }
        }
        
        Ok(HistoryStats {
            total_posts: posts.len(),
            platform_stats,
        })
    }

    /// Count posts matching the query
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn count_posts(&self, query: HistoryQuery) -> Result<usize> {
        let posts = self.list_posts(query).await?;
        Ok(posts.len())
    }
}

/// Helper function to match post status
fn matches_status(post_status: &PostStatus, filter_status: &PostStatus) -> bool {
    matches!(
        (post_status, filter_status),
        (PostStatus::Draft, PostStatus::Draft)
            | (PostStatus::Pending, PostStatus::Pending)
            | (PostStatus::Posted, PostStatus::Posted)
            | (PostStatus::Failed, PostStatus::Failed)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Post, PostRecord};
    use tempfile::TempDir;

    async fn setup_test_db() -> (Database, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = Database::new(db_path.to_str().unwrap()).await.unwrap();
        (db, temp_dir)
    }

    async fn create_test_post(db: &Database, content: &str, status: PostStatus) -> String {
        let post = Post {
            id: uuid::Uuid::new_v4().to_string(),
            content: content.to_string(),
            created_at: chrono::Utc::now().timestamp(),
            scheduled_at: None,
            status,
            metadata: None,
        };
        let post_id = post.id.clone();
        db.create_post(&post).await.unwrap();
        post_id
    }

    async fn create_test_record(
        db: &Database,
        post_id: &str,
        platform: &str,
        success: bool,
    ) {
        let record = PostRecord {
            id: None,
            post_id: post_id.to_string(),
            platform: platform.to_string(),
            platform_post_id: if success {
                Some(format!("{}:note123", platform))
            } else {
                None
            },
            posted_at: Some(chrono::Utc::now().timestamp()),
            success,
            error_message: if !success {
                Some("Test error".to_string())
            } else {
                None
            },
        };
        db.create_post_record(&record).await.unwrap();
    }

    #[tokio::test]
    async fn test_list_posts_empty() {
        let (db, _temp_dir) = setup_test_db().await;
        let service = HistoryService::new(Arc::new(db));

        let query = HistoryQuery::default();
        let results = service.list_posts(query).await.unwrap();

        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_list_posts_with_records() {
        let (db, _temp_dir) = setup_test_db().await;
        let service = HistoryService::new(Arc::new(db.clone()));

        let post_id = create_test_post(&db, "Test post", PostStatus::Posted).await;
        create_test_record(&db, &post_id, "nostr", true).await;
        create_test_record(&db, &post_id, "mastodon", true).await;

        let query = HistoryQuery::default();
        let results = service.list_posts(query).await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].post.id, post_id);
        assert_eq!(results[0].records.len(), 2);
    }

    #[tokio::test]
    async fn test_list_posts_with_platform_filter() {
        let (db, _temp_dir) = setup_test_db().await;
        let service = HistoryService::new(Arc::new(db.clone()));

        let post_id = create_test_post(&db, "Test post", PostStatus::Posted).await;
        create_test_record(&db, &post_id, "nostr", true).await;
        create_test_record(&db, &post_id, "mastodon", true).await;

        let query = HistoryQuery {
            platform: Some("nostr".to_string()),
            ..Default::default()
        };
        let results = service.list_posts(query).await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].records.len(), 2); // Still gets all records for the post
    }

    #[tokio::test]
    async fn test_list_posts_with_limit() {
        let (db, _temp_dir) = setup_test_db().await;
        let service = HistoryService::new(Arc::new(db.clone()));

        // Create 3 posts
        for i in 0..3 {
            let post_id = create_test_post(&db, &format!("Post {}", i), PostStatus::Posted).await;
            create_test_record(&db, &post_id, "nostr", true).await;
        }

        let query = HistoryQuery {
            limit: Some(2),
            ..Default::default()
        };
        let results = service.list_posts(query).await.unwrap();

        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_get_post_existing() {
        let (db, _temp_dir) = setup_test_db().await;
        let service = HistoryService::new(Arc::new(db.clone()));

        let post_id = create_test_post(&db, "Test post", PostStatus::Posted).await;
        create_test_record(&db, &post_id, "nostr", true).await;

        let result = service.get_post(&post_id).await.unwrap();

        assert!(result.is_some());
        let post_with_records = result.unwrap();
        assert_eq!(post_with_records.post.id, post_id);
        assert_eq!(post_with_records.records.len(), 1);
    }

    #[tokio::test]
    async fn test_get_post_nonexistent() {
        let (db, _temp_dir) = setup_test_db().await;
        let service = HistoryService::new(Arc::new(db));

        let result = service.get_post("nonexistent-id").await.unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_get_stats() {
        let (db, _temp_dir) = setup_test_db().await;
        let service = HistoryService::new(Arc::new(db.clone()));

        // Create posts with different success/failure rates
        let post_id1 = create_test_post(&db, "Post 1", PostStatus::Posted).await;
        create_test_record(&db, &post_id1, "nostr", true).await;
        create_test_record(&db, &post_id1, "mastodon", true).await;

        let post_id2 = create_test_post(&db, "Post 2", PostStatus::Failed).await;
        create_test_record(&db, &post_id2, "nostr", true).await;
        create_test_record(&db, &post_id2, "mastodon", false).await;

        let query = HistoryQuery::default();
        let stats = service.get_stats(query).await.unwrap();

        assert_eq!(stats.total_posts, 2);
        assert_eq!(stats.platform_stats.len(), 2);
        
        let nostr_stats = stats.platform_stats.get("nostr").unwrap();
        assert_eq!(nostr_stats.total, 2);
        assert_eq!(nostr_stats.successful, 2);
        assert_eq!(nostr_stats.failed, 0);
        assert_eq!(nostr_stats.success_rate, 100.0);
        
        let mastodon_stats = stats.platform_stats.get("mastodon").unwrap();
        assert_eq!(mastodon_stats.total, 2);
        assert_eq!(mastodon_stats.successful, 1);
        assert_eq!(mastodon_stats.failed, 1);
        assert_eq!(mastodon_stats.success_rate, 50.0);
    }

    #[tokio::test]
    async fn test_count_posts() {
        let (db, _temp_dir) = setup_test_db().await;
        let service = HistoryService::new(Arc::new(db.clone()));

        // Create 3 posts
        for i in 0..3 {
            let post_id = create_test_post(&db, &format!("Post {}", i), PostStatus::Posted).await;
            create_test_record(&db, &post_id, "nostr", true).await;
        }

        let query = HistoryQuery::default();
        let count = service.count_posts(query).await.unwrap();

        assert_eq!(count, 3);
    }
}
