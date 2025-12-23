//! Posting service for multi-platform content posting
//!
//! This module handles posting content to multiple platforms with retry logic,
//! progress tracking, and result recording.
//!
//! # Examples
//!
//! ## Basic posting
//!
//! ```no_run
//! use libplurcast::service::{PlurcastService, posting::PostRequest};
//! use std::collections::HashMap;
//!
//! # async fn example() -> libplurcast::Result<()> {
//! let service = PlurcastService::new().await?;
//!
//! let request = PostRequest {
//!     content: "Hello from Plurcast!".to_string(),
//!     platforms: vec!["nostr".to_string(), "mastodon".to_string()],
//!     draft: false,
//!     account: None,
//!     scheduled_at: None,
//!     nostr_pow: None,
//!     nostr_21e8: false,
//!     reply_to: HashMap::new(),
//! };
//!
//! let response = service.posting().post(request).await?;
//!
//! if response.overall_success {
//!     println!("Posted successfully!");
//!     for result in response.results {
//!         if result.success {
//!             println!("  {}: {}", result.platform, result.post_id.unwrap());
//!         }
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Retrying failed posts
//!
//! ```no_run
//! use libplurcast::service::PlurcastService;
//!
//! # async fn example() -> libplurcast::Result<()> {
//! let service = PlurcastService::new().await?;
//!
//! // Retry a failed post on specific platforms
//! let response = service.posting()
//!     .retry_post("post-id-123", vec!["nostr".to_string()])
//!     .await?;
//! # Ok(())
//! # }
//! ```

use futures::future::join_all;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

use super::events::{Event, EventBus, PlatformResult};
use crate::error::PlatformError;
use crate::platforms::Platform;
use crate::poster::create_platforms;
use crate::{Config, Database, Post, PostRecord, PostStatus, Result};

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
///
/// # Fields
///
/// * `content` - The text content to post (max 100KB)
/// * `platforms` - List of platform names (e.g., "nostr", "mastodon", "ssb")
/// * `draft` - If true, saves as draft without posting
/// * `account` - Optional account name to use for posting. If None, uses active account.
/// * `scheduled_at` - Optional Unix timestamp to schedule the post for later
/// * `nostr_pow` - Optional Proof of Work difficulty for Nostr events (NIP-13)
/// * `nostr_21e8` - If true, mine for 21e8 pattern in Nostr event ID
/// * `reply_to` - Per-platform parent post IDs for threading
///
/// # Example
///
/// ```
/// use libplurcast::service::posting::PostRequest;
/// use std::collections::HashMap;
///
/// let request = PostRequest {
///     content: "My post content".to_string(),
///     platforms: vec!["nostr".to_string()],
///     draft: false,
///     account: None,
///     scheduled_at: None,
///     nostr_pow: Some(20), // POW difficulty for Nostr
///     nostr_21e8: false,
///     reply_to: HashMap::new(), // Empty for new post, or per-platform IDs for replies
/// };
/// ```
#[derive(Debug, Clone)]
pub struct PostRequest {
    pub content: String,
    pub platforms: Vec<String>,
    pub draft: bool,
    pub account: Option<String>,
    pub scheduled_at: Option<i64>,
    pub nostr_pow: Option<u8>,
    pub nostr_21e8: bool,
    /// Per-platform parent post IDs for threading.
    /// Key: platform name (e.g., "nostr", "mastodon")
    /// Value: platform-specific post ID (e.g., "note1abc..." for Nostr, "12345678" for Mastodon)
    /// Empty HashMap for new posts (not replies).
    pub reply_to: HashMap<String, String>,
}

/// Response from posting operation
///
/// # Fields
///
/// * `post_id` - Unique identifier for the post
/// * `results` - Per-platform results (success/failure, post IDs, errors)
/// * `overall_success` - True if at least one platform succeeded
///
/// # Example
///
/// ```no_run
/// # use libplurcast::service::{PlurcastService, posting::PostRequest};
/// # async fn example() -> libplurcast::Result<()> {
/// # use std::collections::HashMap;
/// # let service = PlurcastService::new().await?;
/// # let request = PostRequest {
/// #     content: "test".to_string(),
/// #     platforms: vec!["nostr".to_string()],
/// #     draft: false,
/// #     account: None,
/// #     scheduled_at: None,
/// #     nostr_pow: None,
/// #     nostr_21e8: false,
/// #     reply_to: HashMap::new(),
/// # };
/// let response = service.posting().post(request).await?;
///
/// println!("Post ID: {}", response.post_id);
/// println!("Overall success: {}", response.overall_success);
///
/// for result in response.results {
///     if result.success {
///         println!("✓ {}: {}", result.platform, result.post_id.unwrap());
///     } else {
///         println!("✗ {}: {}", result.platform, result.error.unwrap());
///     }
/// }
/// # Ok(())
/// # }
/// ```
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
        // Determine status based on request
        let (status, scheduled_at) = if request.draft {
            (PostStatus::Pending, None)
        } else if let Some(ts) = request.scheduled_at {
            (PostStatus::Scheduled, Some(ts))
        } else {
            (PostStatus::Pending, None)
        };

        // Build metadata for platform-specific options
        let metadata = {
            // Start with base metadata containing platforms
            let mut meta = serde_json::json!({
                "platforms": request.platforms.clone()
            });

            // Add reply_to for threading support (per-platform IDs)
            if !request.reply_to.is_empty() {
                meta["reply_to"] = serde_json::json!(request.reply_to);
            }

            // Add Nostr-specific options
            let has_nostr_options = request.nostr_pow.is_some()
                || request.nostr_21e8
                || (self.config.nostr.is_some()
                    && self
                        .config
                        .nostr
                        .as_ref()
                        .unwrap()
                        .default_pow_difficulty
                        .is_some());

            if has_nostr_options {
                // Determine effective POW difficulty (CLI flag overrides config)
                let pow_difficulty = request.nostr_pow.or_else(|| {
                    self.config
                        .nostr
                        .as_ref()
                        .and_then(|c| c.default_pow_difficulty)
                });

                if let Some(difficulty) = pow_difficulty {
                    let mut nostr_metadata = serde_json::json!({
                        "pow_difficulty": difficulty
                    });

                    // Add 21e8 flag if requested
                    if request.nostr_21e8 {
                        nostr_metadata["21e8"] = serde_json::json!(true);
                    }

                    meta["nostr"] = nostr_metadata;
                }
            }

            Some(meta.to_string())
        };

        // Create Post object
        let post = Post {
            id: uuid::Uuid::new_v4().to_string(),
            content: request.content.clone(),
            created_at: chrono::Utc::now().timestamp(),
            scheduled_at,
            status,
            metadata,
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

        // Handle scheduled mode
        if request.scheduled_at.is_some() {
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

        // Create platform clients only for requested platforms
        let account_ref = request.account.as_deref();
        let platforms =
            create_platforms(&self.config, Some(&request.platforms), account_ref).await?;

        // Save post to database
        self.db.create_post(&post).await?;

        // Post to platforms concurrently
        let platform_refs: Vec<&dyn Platform> = platforms.iter().map(|p| p.as_ref()).collect();
        let results = self.post_to_platforms(&post, &platform_refs).await;

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
    pub async fn retry_post(
        &self,
        post_id: &str,
        platforms: Vec<String>,
        account: Option<String>,
    ) -> Result<PostResponse> {
        // Get existing post
        let post = self.db.get_post(post_id).await?.ok_or_else(|| {
            crate::error::PlurcastError::InvalidInput(format!("Post not found: {}", post_id))
        })?;

        // Create platform clients only for requested platforms
        let account_ref = account.as_deref();
        let all_platforms = create_platforms(&self.config, Some(&platforms), account_ref).await?;

        // Emit retry event
        self.event_bus.emit(Event::PostingStarted {
            post_id: post_id.to_string(),
            platforms: platforms.clone(),
        });

        // Post to platforms
        let platform_refs: Vec<&dyn Platform> = all_platforms.iter().map(|p| p.as_ref()).collect();
        let results = self.post_to_platforms(&post, &platform_refs).await;

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

    /// Post a scheduled post that already exists in the database
    ///
    /// This method is used by the plur-send daemon to process scheduled posts.
    /// Unlike `post()` which creates a new post, this method updates an existing
    /// scheduled post's status and posts it to the specified platforms.
    ///
    /// # Arguments
    ///
    /// * `post` - The existing scheduled Post object
    /// * `platforms` - List of platform names to post to
    /// * `account` - Optional account name to use for posting
    ///
    /// # Errors
    ///
    /// Returns an error if posting fails critically. Individual platform
    /// failures are captured in the response.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use libplurcast::service::PlurcastService;
    ///
    /// # async fn example() -> libplurcast::Result<()> {
    /// let service = PlurcastService::new().await?;
    /// let db = service.db();
    ///
    /// // Get a scheduled post from the database
    /// let post = db.get_post("post-id").await?.unwrap();
    ///
    /// // Post it to platforms
    /// let response = service.posting()
    ///     .post_scheduled(post, vec!["nostr".to_string()], None)
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn post_scheduled(
        &self,
        post: Post,
        platforms: Vec<String>,
        account: Option<String>,
    ) -> Result<PostResponse> {
        let post_id = post.id.clone();

        // Update post status from Scheduled to Pending before posting
        self.db
            .update_post_status(&post_id, PostStatus::Pending)
            .await?;

        // Create platform clients only for requested platforms
        let account_ref = account.as_deref();
        let all_platforms = create_platforms(&self.config, Some(&platforms), account_ref).await?;

        // Emit posting started event
        self.event_bus.emit(Event::PostingStarted {
            post_id: post_id.clone(),
            platforms: platforms.clone(),
        });

        // Post to platforms concurrently
        let platform_refs: Vec<&dyn Platform> = all_platforms.iter().map(|p| p.as_ref()).collect();
        let results = self.post_to_platforms(&post, &platform_refs).await;

        // Record results (this will update status to Posted or Failed)
        self.record_results(&post, &results).await;

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

    /// Post to platforms concurrently with retry logic
    async fn post_to_platforms(
        &self,
        post: &Post,
        platforms: &[&dyn Platform],
    ) -> Vec<PlatformResult> {
        // Create futures for each platform
        let futures: Vec<_> = platforms
            .iter()
            .map(|platform| {
                let post = post.clone();
                let event_bus = self.event_bus.clone();
                let platform_name = platform.name().to_string();

                async move {
                    info!("Posting to platform: {}", platform_name);

                    // Emit progress event
                    event_bus.emit(Event::PostingProgress {
                        post_id: post.id.clone(),
                        platform: platform_name.clone(),
                        status: "starting".to_string(),
                    });

                    match post_with_retry(*platform, &post).await {
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
                account_name: "default".to_string(),
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
async fn post_with_retry(platform: &dyn Platform, post: &crate::Post) -> Result<(String, String)> {
    let max_attempts = 3;
    let platform_name = platform.name().to_string();

    for attempt in 1..=max_attempts {
        match platform.post(post).await {
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
            | PlatformError::Posting(_)
            | PlatformError::NotImplemented(_) => false,
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
            ssb: None,
            defaults: crate::config::DefaultsConfig { platforms: vec![] },
            credentials: None,
            scheduling: None,
        };

        let event_bus = EventBus::new(100);
        let service = PostingService::new(Arc::new(db), Arc::new(config), event_bus);

        (service, temp_dir)
    }

    #[tokio::test]
    async fn test_create_draft() {
        let (service, _temp_dir) = setup_test_service().await;

        let post_id = service
            .create_draft("Draft content".to_string())
            .await
            .unwrap();

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
            account: None,
            scheduled_at: None,
            nostr_pow: None,
            nostr_21e8: false,
            reply_to: HashMap::new(),
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
            .retry_post("nonexistent-id", vec!["nostr".to_string()], None)
            .await;

        assert!(result.is_err());
    }
}
