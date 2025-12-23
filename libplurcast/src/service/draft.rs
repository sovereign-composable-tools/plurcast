//! Draft service for managing draft posts
//!
//! This module provides CRUD operations for draft posts and publishing.

use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};

use super::posting::{PostRequest, PostResponse, PostingService};
use crate::{Database, Post, PostStatus, Result};

/// Draft service
///
/// Manages draft posts (CRUD operations) and delegates publishing to
/// the posting service.
pub struct DraftService {
    db: Arc<Database>,
    posting: PostingService,
}

/// A draft post
#[derive(Debug, Clone)]
pub struct Draft {
    pub id: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl DraftService {
    /// Create a new draft service
    pub fn new(db: Arc<Database>, posting: PostingService) -> Self {
        Self { db, posting }
    }

    /// Create a new draft
    ///
    /// # Errors
    ///
    /// Returns an error if the draft cannot be saved to the database.
    pub async fn create(&self, content: String) -> Result<Draft> {
        let now = Utc::now();
        let post = Post {
            id: uuid::Uuid::new_v4().to_string(),
            content,
            created_at: now.timestamp(),
            scheduled_at: None,
            status: PostStatus::Draft,
            metadata: None,
        };

        self.db.create_post(&post).await?;

        Ok(Draft {
            id: post.id,
            content: post.content,
            created_at: now,
            updated_at: now,
        })
    }

    /// Update an existing draft
    ///
    /// # Errors
    ///
    /// Returns an error if the draft doesn't exist or cannot be updated.
    pub async fn update(&self, id: &str, content: String) -> Result<Draft> {
        // First check if draft exists
        let existing = self.db.get_post(id).await?.ok_or_else(|| {
            crate::error::PlurcastError::InvalidInput(format!("Draft not found: {}", id))
        })?;

        // Verify it's actually a draft
        if existing.status != PostStatus::Draft {
            return Err(crate::error::PlurcastError::InvalidInput(format!(
                "Post {} is not a draft",
                id
            )));
        }

        let now = Utc::now();

        // Update the content in the database
        self.db
            .update_post_content(&existing.id, content.clone())
            .await?;

        Ok(Draft {
            id: existing.id,
            content,
            created_at: DateTime::from_timestamp(existing.created_at, 0).unwrap_or_else(Utc::now),
            updated_at: now,
        })
    }

    /// Delete a draft
    ///
    /// # Errors
    ///
    /// Returns an error if the draft doesn't exist or cannot be deleted.
    pub async fn delete(&self, id: &str) -> Result<()> {
        // Verify draft exists and is actually a draft
        let post = self.db.get_post(id).await?.ok_or_else(|| {
            crate::error::PlurcastError::InvalidInput(format!("Draft not found: {}", id))
        })?;

        if post.status != PostStatus::Draft {
            return Err(crate::error::PlurcastError::InvalidInput(format!(
                "Post {} is not a draft",
                id
            )));
        }

        // Update status to indicate deletion (we'll use Failed as a workaround)
        // In a real implementation, we'd add a Deleted status or delete the record
        self.db.update_post_status(id, PostStatus::Failed).await?;
        Ok(())
    }

    /// List all drafts
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn list(&self) -> Result<Vec<Draft>> {
        // Query posts with Draft status
        let posts = self
            .db
            .query_posts_with_records(
                None, None, None, None, 1000, // Max 1000 drafts
            )
            .await?;

        let drafts = posts
            .into_iter()
            .filter(|pwr| pwr.post.status == PostStatus::Draft)
            .map(|pwr| {
                let created_at =
                    DateTime::from_timestamp(pwr.post.created_at, 0).unwrap_or_else(Utc::now);
                Draft {
                    id: pwr.post.id,
                    content: pwr.post.content,
                    created_at,
                    updated_at: created_at, // We don't track updates separately yet
                }
            })
            .collect();

        Ok(drafts)
    }

    /// Get a single draft by ID
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn get(&self, id: &str) -> Result<Option<Draft>> {
        let post = self.db.get_post(id).await?;

        match post {
            Some(p) if p.status == PostStatus::Draft => {
                let created_at = DateTime::from_timestamp(p.created_at, 0).unwrap_or_else(Utc::now);
                Ok(Some(Draft {
                    id: p.id,
                    content: p.content,
                    created_at,
                    updated_at: created_at,
                }))
            }
            _ => Ok(None),
        }
    }

    /// Publish a draft
    ///
    /// Delegates to the posting service to actually post the content.
    ///
    /// # Errors
    ///
    /// Returns an error if the draft doesn't exist or posting fails.
    pub async fn publish(&self, id: &str, platforms: Vec<String>) -> Result<PostResponse> {
        // Get the draft
        let draft = self.get(id).await?.ok_or_else(|| {
            crate::error::PlurcastError::InvalidInput(format!("Draft not found: {}", id))
        })?;

        // Create post request
        let request = PostRequest {
            content: draft.content,
            platforms,
            draft: false,       // We're publishing, not creating a new draft
            account: None,      // Use active account when publishing drafts
            scheduled_at: None, // Publish immediately when posting drafts
            nostr_pow: None,    // No POW for draft publishing (could be added later if needed)
            nostr_21e8: false,  // No 21e8 easter egg for draft publishing
            reply_to: HashMap::new(), // No threading for draft publishing (could be added later)
        };

        // Post via posting service
        let response = self.posting.post(request).await?;

        // If successful, delete the draft
        if response.overall_success {
            let _ = self.delete(id).await; // Ignore errors here
        }

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::events::EventBus;
    use crate::Config;
    use tempfile::TempDir;

    async fn setup_test_service() -> (DraftService, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = Database::new(db_path.to_str().unwrap()).await.unwrap();

        let config = Config {
            database: crate::config::DatabaseConfig {
                path: db_path.to_str().unwrap().to_string(),
            },
            nostr: None,
            mastodon: None,
            ssb: None,
            defaults: crate::config::DefaultsConfig { platforms: vec![] },
            credentials: None,
            scheduling: None,
        };

        let event_bus = EventBus::new(100);
        let posting = PostingService::new(Arc::new(db.clone()), Arc::new(config), event_bus);
        let service = DraftService::new(Arc::new(db), posting);

        (service, temp_dir)
    }

    #[tokio::test]
    async fn test_create_draft() {
        let (service, _temp_dir) = setup_test_service().await;

        let draft = service.create("Draft content".to_string()).await.unwrap();

        assert_eq!(draft.content, "Draft content");
        assert!(!draft.id.is_empty());
    }

    #[tokio::test]
    async fn test_list_drafts() {
        let (service, _temp_dir) = setup_test_service().await;

        let draft1 = service.create("Draft 1".to_string()).await.unwrap();
        let draft2 = service.create("Draft 2".to_string()).await.unwrap();

        let drafts = service.list().await.unwrap();

        assert_eq!(drafts.len(), 2);
        let ids: Vec<_> = drafts.iter().map(|d| d.id.as_str()).collect();
        assert!(ids.contains(&draft1.id.as_str()));
        assert!(ids.contains(&draft2.id.as_str()));
    }

    #[tokio::test]
    async fn test_get_draft() {
        let (service, _temp_dir) = setup_test_service().await;

        let created = service.create("Test draft".to_string()).await.unwrap();

        let fetched = service.get(&created.id).await.unwrap();
        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(fetched.id, created.id);
        assert_eq!(fetched.content, "Test draft");
    }

    #[tokio::test]
    async fn test_get_nonexistent_draft() {
        let (service, _temp_dir) = setup_test_service().await;

        let result = service.get("nonexistent-id").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_delete_draft() {
        let (service, _temp_dir) = setup_test_service().await;

        let draft = service.create("To be deleted".to_string()).await.unwrap();
        service.delete(&draft.id).await.unwrap();

        let fetched = service.get(&draft.id).await.unwrap();
        assert!(fetched.is_none());
    }

    #[tokio::test]
    async fn test_delete_nonexistent_draft() {
        let (service, _temp_dir) = setup_test_service().await;

        let result = service.delete("nonexistent-id").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_draft() {
        let (service, _temp_dir) = setup_test_service().await;

        // Create initial draft
        let draft = service
            .create("Original content".to_string())
            .await
            .unwrap();

        // Update the draft
        let updated = service
            .update(&draft.id, "Updated content".to_string())
            .await
            .unwrap();

        assert_eq!(updated.id, draft.id);
        assert_eq!(updated.content, "Updated content");
        // Compare timestamps with tolerance for sub-second precision
        assert_eq!(updated.created_at.timestamp(), draft.created_at.timestamp());
        assert!(updated.updated_at.timestamp() >= draft.updated_at.timestamp());

        // Verify update persisted
        let fetched = service.get(&draft.id).await.unwrap().unwrap();
        assert_eq!(fetched.content, "Updated content");
    }

    #[tokio::test]
    async fn test_update_nonexistent_draft() {
        let (service, _temp_dir) = setup_test_service().await;

        let result = service
            .update("nonexistent-id", "New content".to_string())
            .await;
        assert!(result.is_err());

        if let Err(crate::error::PlurcastError::InvalidInput(msg)) = result {
            assert!(msg.contains("not found"));
        } else {
            panic!("Expected InvalidInput error");
        }
    }

    #[tokio::test]
    async fn test_update_validates_is_draft() {
        let (service, _temp_dir) = setup_test_service().await;

        // Create a draft and mark it as posted (simulating published state)
        let draft = service.create("Original".to_string()).await.unwrap();

        // Manually change status to simulate a published post
        service
            .db
            .update_post_status(&draft.id, PostStatus::Posted)
            .await
            .unwrap();

        // Try to update - should fail since it's no longer a draft
        let result = service.update(&draft.id, "Should fail".to_string()).await;
        assert!(result.is_err());

        if let Err(crate::error::PlurcastError::InvalidInput(msg)) = result {
            assert!(msg.contains("not a draft"));
        } else {
            panic!("Expected InvalidInput error about not being a draft");
        }
    }

    #[tokio::test]
    async fn test_publish_draft() {
        let (service, _temp_dir) = setup_test_service().await;

        // Create a draft
        let draft = service
            .create("Draft to publish".to_string())
            .await
            .unwrap();

        // Publish with draft mode (no actual platforms configured)
        // This will succeed in "draft" mode since we have no platforms configured
        let response = service.publish(&draft.id, vec![]).await.unwrap();

        // Should have created a post
        assert!(!response.post_id.is_empty());

        // Since we have no platforms, it should succeed with empty results
        assert_eq!(response.results.len(), 0);
    }

    #[tokio::test]
    async fn test_publish_nonexistent_draft() {
        let (service, _temp_dir) = setup_test_service().await;

        let result = service
            .publish("nonexistent-id", vec!["nostr".to_string()])
            .await;
        assert!(result.is_err());

        if let Err(crate::error::PlurcastError::InvalidInput(msg)) = result {
            assert!(msg.contains("not found"));
        } else {
            panic!("Expected InvalidInput error");
        }
    }
}
