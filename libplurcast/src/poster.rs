//! Multi-platform posting orchestration
//!
//! This module provides functionality for posting to multiple platforms concurrently,
//! with retry logic, error handling, and database recording.

use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

use crate::config::Config;
use crate::credentials::CredentialManager;
use crate::db::Database;
use crate::error::{PlatformError, Result};
use crate::platforms::{mastodon::MastodonClient, nostr::NostrPlatform, Platform};
use crate::types::{Post, PostRecord, PostStatus};

/// Result of posting to a single platform
#[derive(Debug, Clone)]
pub struct PostResult {
    /// Platform name (e.g., "nostr", "mastodon", "ssb")
    pub platform: String,
    /// Whether the post was successful
    pub success: bool,
    /// Platform-specific post ID (if successful)
    pub platform_post_id: Option<String>,
    /// Error message (if failed)
    pub error: Option<String>,
}

/// Check if an error is transient and should be retried
///
/// Transient errors include network issues and rate limiting.
/// Permanent errors include authentication and validation failures.
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

/// Post to a platform with retry logic and exponential backoff
///
/// This function attempts to post content to a platform with up to 3 attempts.
/// It uses exponential backoff (1s, 2s, 4s) for transient errors.
///
/// # Arguments
///
/// * `platform` - Reference to the platform to post to
/// * `post` - The Post object containing content and metadata
///
/// # Returns
///
/// Returns a tuple of (platform_name, post_id) on success, or an error on failure.
///
/// # Errors
///
/// Returns the final error if all retry attempts are exhausted or if a permanent error occurs.
async fn post_with_retry(platform: &dyn Platform, post: &Post) -> Result<(String, String)> {
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
                    // Permanent error or exhausted retries
                    if attempt == max_attempts {
                        warn!(
                            "Failed to post to {} after {} attempts: {}",
                            platform_name, max_attempts, e
                        );
                    }
                    return Err(e);
                }
            }
        }
    }

    // This should never be reached, but just in case
    Err(PlatformError::Posting(format!(
        "Failed to post to {} after {} attempts",
        platform_name, max_attempts
    ))
    .into())
}

/// Multi-platform poster for orchestrating posts across multiple platforms
///
/// This struct manages posting to multiple platforms concurrently, with retry logic,
/// error handling, and database recording.
pub struct MultiPlatformPoster {
    /// Platform clients
    platforms: Vec<Box<dyn Platform>>,
    /// Database for recording results
    db: Database,
}

impl MultiPlatformPoster {
    /// Create a new MultiPlatformPoster
    ///
    /// # Arguments
    ///
    /// * `platforms` - Vector of platform clients
    /// * `db` - Database for recording results
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use libplurcast::config::Config;
    /// use libplurcast::db::Database;
    /// use libplurcast::poster::{create_platforms, MultiPlatformPoster};
    ///
    /// # async fn example() -> libplurcast::error::Result<()> {
    /// let config = Config::load()?;
    /// let db = Database::new(&config.database.path).await?;
    /// let platforms = create_platforms(&config, None, None).await?;
    /// let poster = MultiPlatformPoster::new(platforms, db);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(platforms: Vec<Box<dyn Platform>>, db: Database) -> Self {
        Self { platforms, db }
    }

    /// Post to all enabled platforms
    ///
    /// This method posts the content to all platforms concurrently and returns
    /// the results for each platform. It also records the post and results in the database.
    ///
    /// # Arguments
    ///
    /// * `post` - The post to publish
    ///
    /// # Returns
    ///
    /// Returns a vector of PostResult, one for each platform.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use libplurcast::types::{Post, PostStatus};
    /// use libplurcast::poster::MultiPlatformPoster;
    ///
    /// # async fn example(poster: MultiPlatformPoster) -> libplurcast::error::Result<()> {
    /// let post = Post {
    ///     id: uuid::Uuid::new_v4().to_string(),
    ///     content: "Hello, decentralized world!".to_string(),
    ///     created_at: chrono::Utc::now().timestamp(),
    ///     scheduled_at: None,
    ///     status: PostStatus::Pending,
    ///     metadata: None,
    /// };
    ///
    /// let results = poster.post_to_all(&post).await;
    /// for result in results {
    ///     if result.success {
    ///         println!("Posted to {}: {}", result.platform, result.platform_post_id.unwrap());
    ///     } else {
    ///         eprintln!("Failed to post to {}: {}", result.platform, result.error.unwrap());
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn post_to_all(&self, post: &Post) -> Vec<PostResult> {
        // Create post record in database before posting
        if let Err(e) = self.db.create_post(post).await {
            warn!("Failed to create post record in database: {}", e);
        }

        // Post to all platforms
        let results = self.post_to_platforms(post, &self.platforms).await;

        // Record results in database
        self.record_results(post, &results).await;

        results
    }

    /// Post to selected platforms
    ///
    /// This method posts the content only to the specified platforms.
    /// It also records the post and results in the database.
    ///
    /// # Arguments
    ///
    /// * `post` - The post to publish
    /// * `platform_names` - Names of platforms to post to (e.g., ["nostr", "mastodon"])
    ///
    /// # Returns
    ///
    /// Returns a vector of PostResult for the selected platforms.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use libplurcast::types::{Post, PostStatus};
    /// use libplurcast::poster::MultiPlatformPoster;
    ///
    /// # async fn example(poster: MultiPlatformPoster) -> libplurcast::error::Result<()> {
    /// let post = Post {
    ///     id: uuid::Uuid::new_v4().to_string(),
    ///     content: "Hello, Nostr!".to_string(),
    ///     created_at: chrono::Utc::now().timestamp(),
    ///     scheduled_at: None,
    ///     status: PostStatus::Pending,
    ///     metadata: None,
    /// };
    ///
    /// let results = poster.post_to_selected(&post, &["nostr"]).await;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn post_to_selected(&self, post: &Post, platform_names: &[&str]) -> Vec<PostResult> {
        // Create post record in database before posting
        if let Err(e) = self.db.create_post(post).await {
            warn!("Failed to create post record in database: {}", e);
        }

        // Filter platforms by name
        let selected_platforms: Vec<&Box<dyn Platform>> = self
            .platforms
            .iter()
            .filter(|p| platform_names.contains(&p.name()))
            .collect();

        // Convert to owned references for posting
        let platforms_to_use: Vec<&dyn Platform> =
            selected_platforms.iter().map(|p| p.as_ref()).collect();

        // Post to selected platforms
        let results = self.post_to_platforms_refs(post, &platforms_to_use).await;

        // Record results in database
        self.record_results(post, &results).await;

        results
    }

    /// Internal method to post to a list of platforms
    async fn post_to_platforms(
        &self,
        post: &Post,
        platforms: &[Box<dyn Platform>],
    ) -> Vec<PostResult> {
        let platform_refs: Vec<&dyn Platform> = platforms.iter().map(|p| p.as_ref()).collect();
        self.post_to_platforms_refs(post, &platform_refs).await
    }

    /// Internal method to post to platform references
    async fn post_to_platforms_refs(
        &self,
        post: &Post,
        platforms: &[&dyn Platform],
    ) -> Vec<PostResult> {
        use futures::future::join_all;

        // Create futures for each platform
        let futures: Vec<_> = platforms
            .iter()
            .map(|platform| {
                let post = post.clone();
                async move {
                    let platform_name = platform.name().to_string();
                    info!("Posting to platform: {}", platform_name);

                    match post_with_retry(*platform, &post).await {
                        Ok((name, post_id)) => {
                            info!("Successfully posted to {}: {}", name, post_id);
                            PostResult {
                                platform: name,
                                success: true,
                                platform_post_id: Some(post_id),
                                error: None,
                            }
                        }
                        Err(e) => {
                            warn!("Failed to post to {}: {}", platform_name, e);
                            PostResult {
                                platform: platform_name,
                                success: false,
                                platform_post_id: None,
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
    ///
    /// This method creates post_records entries for each platform attempt and updates
    /// the overall post status based on the results.
    async fn record_results(&self, post: &Post, results: &[PostResult]) {
        let now = chrono::Utc::now().timestamp();

        // Record each platform result
        for result in results {
            let record = PostRecord {
                id: None,
                post_id: post.id.clone(),
                platform: result.platform.clone(),
                platform_post_id: result.platform_post_id.clone(),
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

/// Create platform instances from configuration
///
/// This function reads the configuration and creates platform clients for all enabled platforms.
/// It handles credential file reading and provides helpful error messages for configuration issues.
///
/// # Arguments
///
/// * `config` - Reference to the configuration
/// * `filter_platforms` - Optional list of platform names to create. If None, creates all enabled platforms.
/// * `account` - Optional account name to use for credentials. If None, uses active account from AccountManager.
///
/// # Returns
///
/// Returns a vector of boxed Platform trait objects for all enabled and properly configured platforms.
///
/// # Errors
///
/// Returns an error if:
/// - Required credential files are missing
/// - Credential files cannot be read
/// - Platform configuration is invalid
/// - Account does not exist or has no credentials
///
/// # Examples
///
/// ```no_run
/// use libplurcast::config::Config;
/// use libplurcast::poster::create_platforms;
///
/// # async fn example() -> libplurcast::error::Result<()> {
/// let config = Config::load()?;
/// let platforms = create_platforms(&config, None, None).await?;
/// println!("Created {} platform clients", platforms.len());
/// # Ok(())
/// # }
/// ```
pub async fn create_platforms(
    config: &Config,
    filter_platforms: Option<&[String]>,
    account: Option<&str>,
) -> Result<Vec<Box<dyn Platform>>> {
    let mut platforms: Vec<Box<dyn Platform>> = Vec::new();

    // Create CredentialManager if credentials config exists, otherwise use plain file fallback
    let credential_manager = if let Some(cred_config) = &config.credentials {
        Some(CredentialManager::new(cred_config.clone())?)
    } else {
        None
    };

    // Create AccountManager to determine which account to use
    let account_manager = crate::accounts::AccountManager::new()?;

    // Create Nostr client if enabled and requested
    if let Some(nostr_config) = &config.nostr {
        let should_create = nostr_config.enabled
            && filter_platforms.is_none_or(|platforms| platforms.contains(&"nostr".to_string()));

        if should_create {
            info!("Creating Nostr platform client");

            // Determine which account to use
            let active_account = account_manager.get_active_account("nostr");
            let account_to_use = account.unwrap_or(active_account.as_str());

            tracing::debug!("Using account '{}' for Nostr", account_to_use);

            // Check for shared test account (easter egg!)
            let keys_content = if account_to_use == "shared-test" {
                tracing::info!("🎉 Using shared test account - a publicly accessible Nostr account for testing!");
                tracing::info!("   Anyone can post to this account. Perfect for demos and testing.");
                tracing::info!("   npub: npub1qyv34w2prnz66zxrgqsmy2emrg0uqtrnvarhrrfaktxk9vp2dgllsajv05m");
                use crate::platforms::nostr::SHARED_TEST_KEY;
                SHARED_TEST_KEY.to_string()
            } else if let Some(ref cred_mgr) = credential_manager {
                // Try to retrieve from credential manager with account
                match cred_mgr.retrieve_account("plurcast.nostr", "private_key", account_to_use) {
                    Ok(key) => {
                        tracing::debug!("Retrieved Nostr credentials from secure storage for account '{}'", account_to_use);
                        key
                    }
                    Err(_) => {
                        // Fall back to file reading for backward compatibility
                        tracing::debug!(
                            "Nostr credentials not found in secure storage for account '{}', falling back to file",
                            account_to_use
                        );
                        let keys_path = nostr_config.expand_keys_file_path()?;

                        if !keys_path.exists() {
                            return Err(PlatformError::Authentication(format!(
                                "Nostr keys file not found: {}. Please create this file with your Nostr private key (hex or nsec format) or use 'plur-creds set nostr --account {}' to store credentials securely.",
                                keys_path.display(),
                                account_to_use
                            )).into());
                        }

                        std::fs::read_to_string(&keys_path).map_err(|e| {
                            PlatformError::Authentication(format!(
                                "Failed to read Nostr keys file {}: {}",
                                keys_path.display(),
                                e
                            ))
                        })?
                    }
                }
            } else {
                // No credential manager, use file reading
                let keys_path = nostr_config.expand_keys_file_path()?;

                if !keys_path.exists() {
                    return Err(PlatformError::Authentication(format!(
                        "Nostr keys file not found: {}. Please create this file with your Nostr private key (hex or nsec format).",
                        keys_path.display()
                    )).into());
                }

                std::fs::read_to_string(&keys_path).map_err(|e| {
                    PlatformError::Authentication(format!(
                        "Failed to read Nostr keys file {}: {}",
                        keys_path.display(),
                        e
                    ))
                })?
            };

            // Create NostrPlatform and load keys
            let mut nostr_platform = NostrPlatform::new(nostr_config);
            nostr_platform.load_keys_from_string(&keys_content)?;

            // Authenticate the platform
            nostr_platform.authenticate().await?;

            platforms.push(Box::new(nostr_platform));
        }
    }

    // Create Mastodon client if enabled and requested
    if let Some(mastodon_config) = &config.mastodon {
        let should_create = mastodon_config.enabled
            && filter_platforms.is_none_or(|platforms| {
                platforms.contains(&"mastodon".to_string())
            });

        if should_create {
            info!("Creating Mastodon platform client");

            // Determine which account to use
            let active_account = account_manager.get_active_account("mastodon");
            let account_to_use = account.unwrap_or(active_account.as_str());

            tracing::debug!("Using account '{}' for Mastodon", account_to_use);

            // Try to get credentials from CredentialManager first, then fall back to file
            let token = if let Some(ref cred_mgr) = credential_manager {
                // Try to retrieve from credential manager with account
                match cred_mgr.retrieve_account("plurcast.mastodon", "access_token", account_to_use) {
                    Ok(token) => {
                        tracing::debug!("Retrieved Mastodon credentials from secure storage for account '{}'", account_to_use);
                        token
                    }
                    Err(_) => {
                        // Fall back to file reading for backward compatibility
                        tracing::debug!("Mastodon credentials not found in secure storage for account '{}', falling back to file", account_to_use);
                        let token_path = mastodon_config.expand_token_file_path()?;

                        if !token_path.exists() {
                            return Err(PlatformError::Authentication(format!(
                                "Mastodon token file not found: {}. Please create this file with your OAuth access token or use 'plur-creds set mastodon --account {}' to store credentials securely.",
                                token_path.display(),
                                account_to_use
                            )).into());
                        }

                        std::fs::read_to_string(&token_path)
                            .map_err(|e| {
                                PlatformError::Authentication(format!(
                                    "Failed to read Mastodon token file {}: {}",
                                    token_path.display(),
                                    e
                                ))
                            })?
                            .trim()
                            .to_string()
                    }
                }
            } else {
                // No credential manager, use file reading
                let token_path = mastodon_config.expand_token_file_path()?;

                if !token_path.exists() {
                    return Err(PlatformError::Authentication(format!(
                        "Mastodon token file not found: {}. Please create this file with your OAuth access token.",
                        token_path.display()
                    )).into());
                }

                std::fs::read_to_string(&token_path)
                    .map_err(|e| {
                        PlatformError::Authentication(format!(
                            "Failed to read Mastodon token file {}: {}",
                            token_path.display(),
                            e
                        ))
                    })?
                    .trim()
                    .to_string()
            };

            // Ensure instance URL has https:// prefix
            let instance_url = if mastodon_config.instance.starts_with("http://")
                || mastodon_config.instance.starts_with("https://")
            {
                mastodon_config.instance.clone()
            } else {
                format!("https://{}", mastodon_config.instance)
            };

            // Create MastodonClient
            let mut mastodon_client = MastodonClient::new(instance_url, token)?;

            // Fetch instance info to get character limit
            mastodon_client.fetch_instance_info().await?;

            platforms.push(Box::new(mastodon_client));
        }
    }

    // Create SSB client if enabled and requested
    if let Some(ssb_config) = &config.ssb {
        let should_create = ssb_config.enabled
            && filter_platforms.is_none_or(|platforms| platforms.contains(&"ssb".to_string()));

        if should_create {
            info!("Creating SSB platform client");
            tracing::debug!("SSB feed path: {}", ssb_config.feed_path);
            tracing::debug!("SSB pub servers: {}", ssb_config.pubs.len());

            // Determine which account to use
            let active_account = account_manager.get_active_account("ssb");
            let account_to_use = account.unwrap_or(active_account.as_str());

            tracing::info!("Using account '{}' for SSB", account_to_use);

            // Create SSBPlatform
            let mut ssb_platform = crate::platforms::ssb::SSBPlatform::new(ssb_config);

            // Initialize with credentials
            if let Some(ref cred_mgr) = credential_manager {
                ssb_platform
                    .initialize_with_credentials(cred_mgr, account_to_use)
                    .await?;
            } else {
                return Err(PlatformError::Authentication(
                    "SSB requires credential manager for initialization".to_string(),
                )
                .into());
            }

            platforms.push(Box::new(ssb_platform));
        }
    }

    if platforms.is_empty() {
        warn!("No platforms are enabled in configuration");
    } else {
        info!("Created {} platform client(s)", platforms.len());
    }

    Ok(platforms)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, DatabaseConfig, DefaultsConfig, MastodonConfig, NostrConfig};
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_create_platforms_no_enabled_platforms() {
        let config = Config {
            database: DatabaseConfig {
                path: ":memory:".to_string(),
            },
            credentials: None,
            nostr: None,
            mastodon: None,
            ssb: None,
            defaults: DefaultsConfig::default(),
            scheduling: None,
        };

        let platforms = create_platforms(&config, None, None).await.unwrap();
        assert_eq!(platforms.len(), 0);
    }

    #[tokio::test]
    async fn test_create_platforms_nostr_missing_keys_file() {
        let config = Config {
            database: DatabaseConfig {
                path: ":memory:".to_string(),
            },
            credentials: None,
            nostr: Some(NostrConfig {
                enabled: true,
                keys_file: "/nonexistent/nostr.keys".to_string(),
                relays: vec!["wss://relay.damus.io".to_string()],
            }),
            mastodon: None,
            ssb: None,
            defaults: DefaultsConfig::default(),
            scheduling: None,
        };

        let result = create_platforms(&config, None, None).await;
        assert!(result.is_err());

        match result {
            Err(crate::error::PlurcastError::Platform(PlatformError::Authentication(msg))) => {
                assert!(msg.contains("keys file not found"));
            }
            _ => panic!("Expected authentication error for missing keys file"),
        }
    }

    #[tokio::test]
    async fn test_create_platforms_mastodon_missing_token_file() {
        let config = Config {
            database: DatabaseConfig {
                path: ":memory:".to_string(),
            },
            credentials: None,
            nostr: None,
            mastodon: Some(MastodonConfig {
                enabled: true,
                instance: "mastodon.social".to_string(),
                token_file: "/nonexistent/mastodon.token".to_string(),
            }),
            ssb: None,
            defaults: DefaultsConfig::default(),
            scheduling: None,
        };

        let result = create_platforms(&config, None, None).await;
        assert!(result.is_err());

        match result {
            Err(crate::error::PlurcastError::Platform(PlatformError::Authentication(msg))) => {
                assert!(msg.contains("token file not found"));
            }
            _ => panic!("Expected authentication error for missing token file"),
        }
    }

    #[tokio::test]
    async fn test_create_platforms_disabled_platforms_skipped() {
        let temp_dir = TempDir::new().unwrap();
        let keys_file = temp_dir.path().join("nostr.keys");
        std::fs::write(&keys_file, "test_key").unwrap();

        let config = Config {
            database: DatabaseConfig {
                path: ":memory:".to_string(),
            },
            credentials: None,
            nostr: Some(NostrConfig {
                enabled: false, // Disabled
                keys_file: keys_file.to_str().unwrap().to_string(),
                relays: vec!["wss://relay.damus.io".to_string()],
            }),
            mastodon: None,
            ssb: None,
            defaults: DefaultsConfig::default(),
            scheduling: None,
        };

        let platforms = create_platforms(&config, None, None).await.unwrap();
        assert_eq!(platforms.len(), 0);
    }

    #[tokio::test]
    async fn test_multi_platform_poster_creation() {
        let db = Database::new(":memory:").await.unwrap();
        let platforms: Vec<Box<dyn Platform>> = Vec::new();

        let poster = MultiPlatformPoster::new(platforms, db);

        // Just verify it can be created
        assert_eq!(poster.platforms.len(), 0);
    }

    #[tokio::test]
    async fn test_post_to_all_with_no_platforms() {
        let db = Database::new(":memory:").await.unwrap();
        let platforms: Vec<Box<dyn Platform>> = Vec::new();
        let poster = MultiPlatformPoster::new(platforms, db);

        let post = Post {
            id: uuid::Uuid::new_v4().to_string(),
            content: "Test post".to_string(),
            created_at: chrono::Utc::now().timestamp(),
            scheduled_at: None,
            status: PostStatus::Pending,
            metadata: None,
        };

        let results = poster.post_to_all(&post).await;
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_post_to_selected_with_no_platforms() {
        let db = Database::new(":memory:").await.unwrap();
        let platforms: Vec<Box<dyn Platform>> = Vec::new();
        let poster = MultiPlatformPoster::new(platforms, db);

        let post = Post {
            id: uuid::Uuid::new_v4().to_string(),
            content: "Test post".to_string(),
            created_at: chrono::Utc::now().timestamp(),
            scheduled_at: None,
            status: PostStatus::Pending,
            metadata: None,
        };

        let results = poster.post_to_selected(&post, &["nostr"]).await;
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_is_transient_error_network() {
        let error = crate::error::PlurcastError::Platform(PlatformError::Network(
            "Connection timeout".to_string(),
        ));
        assert!(is_transient_error(&error));
    }

    #[test]
    fn test_is_transient_error_rate_limit() {
        let error = crate::error::PlurcastError::Platform(PlatformError::RateLimit(
            "Too many requests".to_string(),
        ));
        assert!(is_transient_error(&error));
    }

    #[test]
    fn test_is_not_transient_error_authentication() {
        let error = crate::error::PlurcastError::Platform(PlatformError::Authentication(
            "Invalid token".to_string(),
        ));
        assert!(!is_transient_error(&error));
    }

    #[test]
    fn test_is_not_transient_error_validation() {
        let error = crate::error::PlurcastError::Platform(PlatformError::Validation(
            "Content too long".to_string(),
        ));
        assert!(!is_transient_error(&error));
    }

    #[test]
    fn test_is_not_transient_error_posting() {
        let error = crate::error::PlurcastError::Platform(PlatformError::Posting(
            "Failed to post".to_string(),
        ));
        assert!(!is_transient_error(&error));
    }

    // Mock platform for testing retry logic
    struct MockPlatform {
        name: String,
        attempts_before_success: usize,
        current_attempt: std::sync::Arc<std::sync::Mutex<usize>>,
        error_type: PlatformError,
    }

    impl MockPlatform {
        fn new_failing_then_success(name: &str, attempts_before_success: usize) -> Self {
            Self {
                name: name.to_string(),
                attempts_before_success,
                current_attempt: std::sync::Arc::new(std::sync::Mutex::new(0)),
                error_type: PlatformError::Network("Temporary network error".to_string()),
            }
        }

        fn new_permanent_failure(name: &str) -> Self {
            Self {
                name: name.to_string(),
                attempts_before_success: 999, // Never succeeds
                current_attempt: std::sync::Arc::new(std::sync::Mutex::new(0)),
                error_type: PlatformError::Authentication("Invalid credentials".to_string()),
            }
        }
    }

    #[async_trait::async_trait]
    impl Platform for MockPlatform {
        async fn authenticate(&mut self) -> Result<()> {
            Ok(())
        }

        async fn post(&self, _content: &str) -> Result<String> {
            let mut attempt = self.current_attempt.lock().unwrap();
            *attempt += 1;

            if *attempt >= self.attempts_before_success {
                Ok(format!("{}:mock_post_id", self.name))
            } else {
                Err(self.error_type.clone().into())
            }
        }

        fn validate_content(&self, _content: &str) -> Result<()> {
            Ok(())
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn character_limit(&self) -> Option<usize> {
            None
        }

        fn is_configured(&self) -> bool {
            true
        }
    }

    #[tokio::test]
    async fn test_post_with_retry_success_first_attempt() {
        let platform = MockPlatform::new_failing_then_success("test", 1);
        let result = post_with_retry(&platform, "Test content").await;

        assert!(result.is_ok());
        let (platform_name, post_id) = result.unwrap();
        assert_eq!(platform_name, "test");
        assert_eq!(post_id, "test:mock_post_id");
    }

    #[tokio::test]
    async fn test_post_with_retry_success_after_retries() {
        let platform = MockPlatform::new_failing_then_success("test", 2);
        let result = post_with_retry(&platform, "Test content").await;

        assert!(result.is_ok());
        let (platform_name, post_id) = result.unwrap();
        assert_eq!(platform_name, "test");
        assert_eq!(post_id, "test:mock_post_id");
    }

    #[tokio::test]
    async fn test_post_with_retry_permanent_failure() {
        let platform = MockPlatform::new_permanent_failure("test");
        let result = post_with_retry(&platform, "Test content").await;

        assert!(result.is_err());
        match result {
            Err(crate::error::PlurcastError::Platform(PlatformError::Authentication(_))) => {
                // Expected - permanent error should not retry
            }
            _ => panic!("Expected authentication error"),
        }

        // Should only attempt once for permanent errors
        let attempts = platform.current_attempt.lock().unwrap();
        assert_eq!(*attempts, 1);
    }

    #[tokio::test]
    async fn test_post_with_retry_exhausted_retries() {
        let platform = MockPlatform::new_failing_then_success("test", 10); // More than max attempts
        let result = post_with_retry(&platform, "Test content").await;

        assert!(result.is_err());

        // Should attempt exactly 3 times
        let attempts = platform.current_attempt.lock().unwrap();
        assert_eq!(*attempts, 3);
    }

    #[tokio::test]
    async fn test_concurrent_posting_all_success() {
        let db = Database::new(":memory:").await.unwrap();

        let platforms: Vec<Box<dyn Platform>> = vec![
            Box::new(MockPlatform::new_failing_then_success("platform1", 1)),
            Box::new(MockPlatform::new_failing_then_success("platform2", 1)),
            Box::new(MockPlatform::new_failing_then_success("platform3", 1)),
        ];

        let poster = MultiPlatformPoster::new(platforms, db);

        let post = Post {
            id: uuid::Uuid::new_v4().to_string(),
            content: "Test concurrent post".to_string(),
            created_at: chrono::Utc::now().timestamp(),
            scheduled_at: None,
            status: PostStatus::Pending,
            metadata: None,
        };

        let results = poster.post_to_all(&post).await;

        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.success));
        assert!(results.iter().any(|r| r.platform == "platform1"));
        assert!(results.iter().any(|r| r.platform == "platform2"));
        assert!(results.iter().any(|r| r.platform == "platform3"));
    }

    #[tokio::test]
    async fn test_concurrent_posting_partial_failure() {
        let db = Database::new(":memory:").await.unwrap();

        let platforms: Vec<Box<dyn Platform>> = vec![
            Box::new(MockPlatform::new_failing_then_success("platform1", 1)),
            Box::new(MockPlatform::new_permanent_failure("platform2")),
            Box::new(MockPlatform::new_failing_then_success("platform3", 1)),
        ];

        let poster = MultiPlatformPoster::new(platforms, db);

        let post = Post {
            id: uuid::Uuid::new_v4().to_string(),
            content: "Test partial failure".to_string(),
            created_at: chrono::Utc::now().timestamp(),
            scheduled_at: None,
            status: PostStatus::Pending,
            metadata: None,
        };

        let results = poster.post_to_all(&post).await;

        assert_eq!(results.len(), 3);

        // platform1 and platform3 should succeed
        let platform1_result = results.iter().find(|r| r.platform == "platform1").unwrap();
        assert!(platform1_result.success);

        let platform3_result = results.iter().find(|r| r.platform == "platform3").unwrap();
        assert!(platform3_result.success);

        // platform2 should fail
        let platform2_result = results.iter().find(|r| r.platform == "platform2").unwrap();
        assert!(!platform2_result.success);
        assert!(platform2_result.error.is_some());
    }

    #[tokio::test]
    async fn test_post_to_selected_platforms() {
        let db = Database::new(":memory:").await.unwrap();

        let platforms: Vec<Box<dyn Platform>> = vec![
            Box::new(MockPlatform::new_failing_then_success("nostr", 1)),
            Box::new(MockPlatform::new_failing_then_success("mastodon", 1)),
            Box::new(MockPlatform::new_failing_then_success("ssb", 1)),
        ];

        let poster = MultiPlatformPoster::new(platforms, db);

        let post = Post {
            id: uuid::Uuid::new_v4().to_string(),
            content: "Test selective posting".to_string(),
            created_at: chrono::Utc::now().timestamp(),
            scheduled_at: None,
            status: PostStatus::Pending,
            metadata: None,
        };

        // Post only to nostr and ssb
        let results = poster.post_to_selected(&post, &["nostr", "ssb"]).await;

        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|r| r.platform == "nostr" && r.success));
        assert!(results.iter().any(|r| r.platform == "ssb" && r.success));
        assert!(!results.iter().any(|r| r.platform == "mastodon"));
    }

    #[tokio::test]
    async fn test_concurrent_execution_timing() {
        use std::time::Instant;

        let db = Database::new(":memory:").await.unwrap();

        // Create platforms that would take 3 seconds total if executed sequentially
        // (1 second each), but should complete faster when concurrent
        let platforms: Vec<Box<dyn Platform>> = vec![
            Box::new(MockPlatform::new_failing_then_success("platform1", 1)),
            Box::new(MockPlatform::new_failing_then_success("platform2", 1)),
            Box::new(MockPlatform::new_failing_then_success("platform3", 1)),
        ];

        let poster = MultiPlatformPoster::new(platforms, db);

        let post = Post {
            id: uuid::Uuid::new_v4().to_string(),
            content: "Test timing".to_string(),
            created_at: chrono::Utc::now().timestamp(),
            scheduled_at: None,
            status: PostStatus::Pending,
            metadata: None,
        };

        let start = Instant::now();
        let results = poster.post_to_all(&post).await;
        let duration = start.elapsed();

        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.success));

        // Should complete much faster than sequential execution
        // Allow some overhead, but should be well under 2 seconds
        assert!(
            duration.as_secs() < 2,
            "Concurrent execution took too long: {:?}",
            duration
        );
    }

    #[tokio::test]
    async fn test_database_recording_all_success() {
        let db = Database::new(":memory:").await.unwrap();

        let platforms: Vec<Box<dyn Platform>> = vec![
            Box::new(MockPlatform::new_failing_then_success("platform1", 1)),
            Box::new(MockPlatform::new_failing_then_success("platform2", 1)),
        ];

        let poster = MultiPlatformPoster::new(platforms, db.clone());

        let post = Post {
            id: uuid::Uuid::new_v4().to_string(),
            content: "Test database recording".to_string(),
            created_at: chrono::Utc::now().timestamp(),
            scheduled_at: None,
            status: PostStatus::Pending,
            metadata: None,
        };

        let results = poster.post_to_all(&post).await;

        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.success));

        // Verify post was created in database
        let retrieved_post = db.get_post(&post.id).await.unwrap();
        assert!(retrieved_post.is_some());

        let retrieved_post = retrieved_post.unwrap();
        assert!(matches!(retrieved_post.status, PostStatus::Posted));
    }

    #[tokio::test]
    async fn test_database_recording_partial_failure() {
        let db = Database::new(":memory:").await.unwrap();

        let platforms: Vec<Box<dyn Platform>> = vec![
            Box::new(MockPlatform::new_failing_then_success("platform1", 1)),
            Box::new(MockPlatform::new_permanent_failure("platform2")),
        ];

        let poster = MultiPlatformPoster::new(platforms, db.clone());

        let post = Post {
            id: uuid::Uuid::new_v4().to_string(),
            content: "Test partial failure recording".to_string(),
            created_at: chrono::Utc::now().timestamp(),
            scheduled_at: None,
            status: PostStatus::Pending,
            metadata: None,
        };

        let results = poster.post_to_all(&post).await;

        assert_eq!(results.len(), 2);

        // Verify post status is Posted (partial success)
        let retrieved_post = db.get_post(&post.id).await.unwrap().unwrap();
        assert!(matches!(retrieved_post.status, PostStatus::Posted));
    }

    #[tokio::test]
    async fn test_database_recording_all_failure() {
        let db = Database::new(":memory:").await.unwrap();

        let platforms: Vec<Box<dyn Platform>> = vec![
            Box::new(MockPlatform::new_permanent_failure("platform1")),
            Box::new(MockPlatform::new_permanent_failure("platform2")),
        ];

        let poster = MultiPlatformPoster::new(platforms, db.clone());

        let post = Post {
            id: uuid::Uuid::new_v4().to_string(),
            content: "Test all failure recording".to_string(),
            created_at: chrono::Utc::now().timestamp(),
            scheduled_at: None,
            status: PostStatus::Pending,
            metadata: None,
        };

        let results = poster.post_to_all(&post).await;

        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| !r.success));

        // Verify post status is Failed
        let retrieved_post = db.get_post(&post.id).await.unwrap().unwrap();
        assert!(matches!(retrieved_post.status, PostStatus::Failed));
    }

    #[tokio::test]
    async fn test_database_recording_selected_platforms() {
        let db = Database::new(":memory:").await.unwrap();

        let platforms: Vec<Box<dyn Platform>> = vec![
            Box::new(MockPlatform::new_failing_then_success("nostr", 1)),
            Box::new(MockPlatform::new_failing_then_success("mastodon", 1)),
            Box::new(MockPlatform::new_failing_then_success("ssb", 1)),
        ];

        let poster = MultiPlatformPoster::new(platforms, db.clone());

        let post = Post {
            id: uuid::Uuid::new_v4().to_string(),
            content: "Test selective recording".to_string(),
            created_at: chrono::Utc::now().timestamp(),
            scheduled_at: None,
            status: PostStatus::Pending,
            metadata: None,
        };

        let results = poster.post_to_selected(&post, &["nostr", "ssb"]).await;

        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.success));

        // Verify post was created and marked as Posted
        let retrieved_post = db.get_post(&post.id).await.unwrap().unwrap();
        assert!(matches!(retrieved_post.status, PostStatus::Posted));
    }
}
