//! Posting service for multi-platform content posting
//!
//! This module handles posting content to multiple platforms with retry logic,
//! progress tracking, and result recording.

use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};
use futures::future::join_all;

use crate::{Config, Database, Result, Post, PostRecord, PostStatus};
use crate::error::PlatformError;
use crate::platforms::Platform;
use crate::poster::create_platforms;
use super::events::{Event, EventBus, PlatformResult};

/// Posting service
///
/// Handles all posting operations including validation, multi-platform posting,
/// retry logic, and progress tracking.
#[derive(Clone)]
pub struct PostingService {
    db: Arc<Database>,
    config: Arc<Config>,
    event_bus: EventBus,
}

/// Request to post content
#[derive(Debug, Clone)]
pub struct PostRequest {
    pub content: String,
    pub platforms: Vec<String>,
    pub draft: bool,
}

/// Response from posting operation
#[derive(Debug, Clone)]
pub struct PostResponse {
    pub post_id: String,
    pub results: Vec<PlatformResult>,
    pub overall_success: bool,
}

impl PostingService {
    /// Create a new posting service
    pub fn new(db: Arc<Database>, config: Arc<Config>, event_bus: EventBus) -> Self {
        Self {
            db,
            config,
            event_bus,
        }
    }

    /// Post content to specified platforms
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails critically. Individual platform
    /// failures are captured in the response.
    pub async fn post(&self, request: PostRequest) -> Result<PostResponse> {
        // Create Post object
        let post = Post {
            id: uuid::Uuid::new_v4().to_string(),
            content: request.content.clone(),
            created_at: chrono::Utc::now().timestamp(),
            scheduled_at: None,
            status: PostStatus::Pending,
            metadata: None,
        };

        let post_id = post.id.clone();

        // Handle draft mode
        if request.draft {
            self.db.create_post(&post).await?;
            return Ok(PostResponse {
                post_id,
                results: vec![],
                overall_success: true,
            });
        }

        // Emit posting started event
        self.event_bus.emit(Event::PostingStarted {
            post_id: post_id.clone(),
            platforms: request.platforms.clone(),
        });

        // Create platform clients
        let platforms = create_platforms(&self.config).await?;

        // Filter to requested platforms
        let selected_platforms: Vec<&Box<dyn Platform>> = platforms
            .iter()
            .filter(|p| request.platforms.contains(&p.name().to_string()))
            .collect();

        // Save post to database
        self.db.create_post(&post).await?;

        // Post to platforms concurrently
        let results = self.post_to_platforms(&post, &selected_platforms).await;

        // Record results
        self.record_results(&post, &results).await;

        // Determine overall success
        let overall_success = !results.is_empty() && results.iter().any(|r| r.success);

        // Emit completion event
        if overall_success {
            self.event_bus.emit(Event::PostingCompleted {
                post_id: post_id.clone(),
                results: results.clone(),
            });
        } else {
            self.event_bus.emit(Event::PostingFailed {
                post_id: post_id.clone(),
                error: "All platforms failed".to_string(),
            });
        }

        Ok(PostResponse {
            post_id,
            results,
            overall_success,
        })
    }

    /// Create a draft without posting
    ///
    /// # Errors
    ///
    /// Returns an error if the draft cannot be saved to the database.
    pub async fn create_draft(&self, content: String) -> Result<String> {
        let post = Post {
            id: uuid::Uuid::new_v4().to_string(),
            content,
            created_at: chrono::Utc::now().timestamp(),
            scheduled_at: None,
            status: PostStatus::Pending,
            metadata: None,
        };

        let post_id = post.id.clone();
        self.db.create_post(&post).await?;
        Ok(post_id)
    }

    /// Retry a failed post
    ///
    /// # Errors
    ///
    /// Returns an error if the post doesn't exist or retry fails.
    pub async fn retry_post(&self, post_id: &str, platforms: Vec<String>) -> Result<PostResponse> {
        // Get existing post
        let post = self.db.get_post(post_id).await?
            .ok_or_else(|| crate::error::PlurcastError::InvalidInput(
                format!("Post not found: {}", post_id)
            ))?;

        // Create platform clients
        let all_platforms = create_platforms(&self.config).await?;

        // Filter to requested platforms
        let selected_platforms: Vec<&Box<dyn Platform>> = all_platforms
            .iter()
            .filter(|p| platforms.contains(&p.name().to_string()))
            .collect();

        // Emit retry event
        self.event_bus.emit(Event::PostingStarted {
            post_id: post_id.to_string(),
            platforms: platforms.clone(),
        });

        // Post to platforms
        let results = self.post_to_platforms(&post, &selected_platforms).await;

        // Record results
        self.record_results(&post, &results).await;

        let overall_success = !results.is_empty() && results.iter().any(|r| r.success);

        // Emit completion event
        if overall_success {
            self.event_bus.emit(Event::PostingCompleted {
                post_id: post_id.to_string(),
                results: results.clone(),
            });
        } else {
            self.event_bus.emit(Event::PostingFailed {
                post_id: post_id.to_string(),
                error: "All platforms failed".to_string(),
            });
        }

        Ok(PostResponse {
            post_id: post_id.to_string(),
            results,
            overall_success,
        })
    }

    /// Post to platforms concurrently with retry logic
    async fn post_to_platforms(
        &self,
        post: &Post,
        platforms: &[&Box<dyn Platform>],
    ) -> Vec<PlatformResult> {
        // Create futures for each platform
        let futures: Vec<_> = platforms
            .iter()
            .map(|platform| {
                let content = post.content.clone();
                let post_id = post.id.clone();
                let event_bus = self.event_bus.clone();
                let platform_name = platform.name().to_string();
                let platform_ref: &dyn Platform = platform.as_ref();

                async move {
                    info!("Posting to platform: {}", platform_name);

                    // Emit progress event
                    event_bus.emit(Event::PostingProgress {
                        post_id: post_id.clone(),
                        platform: platform_name.clone(),
                        status: "starting".to_string(),
                    });

                    match post_with_retry(platform_ref, &content).await {
                        Ok((name, platform_post_id)) => {
                            info!("Successfully posted to {}: {}", name, platform_post_id);
                            PlatformResult {
                                platform: name,
                                success: true,
                                post_id: Some(platform_post_id),
                                error: None,
                            }
                        }
                        Err(e) => {
                            warn!("Failed to post to {}: {}", platform_name, e);
                            PlatformResult {
                                platform: platform_name,
                                success: false,
                                post_id: None,
                                error: Some(e.to_string()),
                            }
                        }
                    }
                }
            })
            .collect();

        // Execute all futures concurrently
        join_all(futures).await
    }

    /// Record posting results in the database
    async fn record_results(&self, post: &Post, results: &[PlatformResult]) {
        let now = chrono::Utc::now().timestamp();

        // Record each platform result
        for result in results {
            let record = PostRecord {
                id: None,
                post_id: post.id.clone(),
                platform: result.platform.clone(),
                platform_post_id: result.post_id.clone(),
                posted_at: if result.success { Some(now) } else { None },
                success: result.success,
                error_message: result.error.clone(),
            };

            if let Err(e) = self.db.create_post_record(&record).await {
                warn!(
                    "Failed to record result for platform {}: {}",
                    result.platform, e
                );
            }
        }

        // Update post status based on overall results
        let new_status = if results.is_empty() {
            PostStatus::Failed
        } else if results.iter().all(|r| r.success) {
            PostStatus::Posted
        } else if results.iter().any(|r| r.success) {
            // Partial success - still mark as posted
            PostStatus::Posted
        } else {
            PostStatus::Failed
        };

        if let Err(e) = self.db.update_post_status(&post.id, new_status).await {
            warn!("Failed to update post status: {}", e);
        }
    }
}

/// Post to a platform with retry logic and exponential backoff
async fn post_with_retry(platform: &dyn Platform, content: &str) -> Result<(String, String)> {
    let max_attempts = 3;
    let platform_name = platform.name().to_string();

    for attempt in 1..=max_attempts {
        match platform.post(content).await {
            Ok(post_id) => {
                if attempt > 1 {
                    info!(
                        "Successfully posted to {} on attempt {}",
                        platform_name, attempt
                    );
                }
                return Ok((platform_name, post_id));
            }
            Err(e) => {
                if is_transient_error(&e) && attempt < max_attempts {
                    let delay_secs = 2_u64.pow(attempt - 1);
                    warn!(
                        "Transient error posting to {} (attempt {}/{}): {}. Retrying in {}s...",
                        platform_name, attempt, max_attempts, e, delay_secs
                    );
                    sleep(Duration::from_secs(delay_secs)).await;
                } else {
                    return Err(e);
                }
            }
        }
    }

    Err(PlatformError::Posting(format!(
        "Failed to post to {} after {} attempts",
        platform_name, max_attempts
    ))
    .into())
}

/// Check if an error is transient and should be retried
fn is_transient_error(error: &crate::error::PlurcastError) -> bool {
    match error {
        crate::error::PlurcastError::Platform(platform_error) => match platform_error {
            PlatformError::Network(_) | PlatformError::RateLimit(_) => true,
            PlatformError::Authentication(_)
            | PlatformError::Validation(_)
            | PlatformError::Posting(_) => false,
        },
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn setup_test_service() -> (PostingService, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = Database::new(db_path.to_str().unwrap()).await.unwrap();

        // Create minimal config
        let config = Config {
            database: crate::config::DatabaseConfig {
                path: db_path.to_str().unwrap().to_string(),
            },
            nostr: None,
            mastodon: None,
            bluesky: None,
            defaults: crate::config::DefaultsConfig {
                platforms: vec![],
            },
            credentials: None,
        };

        let event_bus = EventBus::new(100);
        let service = PostingService::new(Arc::new(db), Arc::new(config), event_bus);

        (service, temp_dir)
    }

    #[tokio::test]
    async fn test_create_draft() {
        let (service, _temp_dir) = setup_test_service().await;

        let post_id = service.create_draft("Draft content".to_string()).await.unwrap();

        // Verify post was created in database
        let post = service.db.get_post(&post_id).await.unwrap();
        assert!(post.is_some());
        let post = post.unwrap();
        assert_eq!(post.content, "Draft content");
        assert!(matches!(post.status, PostStatus::Pending));
    }

    #[tokio::test]
    async fn test_post_draft_mode() {
        let (service, _temp_dir) = setup_test_service().await;

        let request = PostRequest {
            content: "Draft post".to_string(),
            platforms: vec!["nostr".to_string()],
            draft: true,
        };

        let response = service.post(request).await.unwrap();

        assert!(response.overall_success);
        assert_eq!(response.results.len(), 0); // No posting in draft mode

        // Verify post was created
        let post = service.db.get_post(&response.post_id).await.unwrap();
        assert!(post.is_some());
    }

    #[tokio::test]
    async fn test_retry_post_nonexistent() {
        let (service, _temp_dir) = setup_test_service().await;

        let result = service
            .retry_post("nonexistent-id", vec!["nostr".to_string()])
            .await;

        assert!(result.is_err());
    }
}
