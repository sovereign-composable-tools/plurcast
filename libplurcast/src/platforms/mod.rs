//! Platform abstraction and implementations
//!
//! This module provides a unified trait for interacting with different social media platforms.
//! Each platform implementation handles authentication, posting, and content validation
//! according to platform-specific requirements.
//!
//! # Examples
//!
//! ```no_run
//! use libplurcast::platforms::{Platform, nostr::NostrPlatform};
//! use libplurcast::config::NostrConfig;
//!
//! # async fn example() -> libplurcast::error::Result<()> {
//! let config = NostrConfig {
//!     enabled: true,
//!     keys_file: "~/.config/plurcast/nostr.keys".to_string(),
//!     relays: vec!["wss://relay.damus.io".to_string()],
//! };
//!
//! let mut platform = NostrPlatform::new(&config);
//!
//! // Check if platform is configured
//! if platform.is_configured() {
//!     // Authenticate with the platform
//!     platform.authenticate().await?;
//!     
//!     // Check character limit
//!     if let Some(limit) = platform.character_limit() {
//!         println!("Platform has a {} character limit", limit);
//!     }
//!     
//!     // Post content
//!     let post_id = platform.post("Hello, decentralized world!").await?;
//!     println!("Posted: {}", post_id);
//! }
//! # Ok(())
//! # }
//! ```

use async_trait::async_trait;

use crate::error::Result;

pub mod mastodon;
pub mod nostr;
pub mod ssb;

// Mock platform is available for all builds (not just tests) to support integration tests
pub mod mock;

/// Platform trait for unified social media platform interactions
///
/// This trait defines the common interface that all platform implementations must provide.
/// It supports async operations for network-based activities and provides methods for
/// authentication, posting, validation, and platform introspection.
#[async_trait]
pub trait Platform: Send + Sync {
    /// Authenticate with the platform
    ///
    /// This method establishes a connection and authenticates the user with the platform.
    /// It should be called before attempting to post content.
    ///
    /// # Errors
    ///
    /// Returns `PlatformError::Authentication` if authentication fails due to invalid
    /// credentials, network issues, or other authentication-related problems.
    async fn authenticate(&mut self) -> Result<()>;

    /// Post content to the platform
    ///
    /// Posts the given content to the platform and returns a platform-specific post ID.
    ///
    /// # Arguments
    ///
    /// * `content` - The text content to post
    ///
    /// # Returns
    ///
    /// Returns the platform-specific post ID (e.g., "note1abc..." for Nostr,
    /// "12345" for Mastodon, "at://..." for Bluesky)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The platform is not authenticated (`PlatformError::Authentication`)
    /// - The post fails to publish (`PlatformError::Posting`)
    /// - Network issues occur (`PlatformError::Network`)
    async fn post(&self, content: &str) -> Result<String>;

    /// Validate content before posting
    ///
    /// Checks if the content meets platform-specific requirements such as character limits,
    /// format restrictions, or other validation rules.
    ///
    /// # Arguments
    ///
    /// * `content` - The content to validate
    ///
    /// # Errors
    ///
    /// Returns `PlatformError::Validation` if the content fails validation
    fn validate_content(&self, content: &str) -> Result<()>;

    /// Get the platform name
    ///
    /// Returns a lowercase identifier for the platform (e.g., "nostr", "mastodon", "bluesky")
    fn name(&self) -> &str;

    /// Get the platform's character limit
    ///
    /// Returns the maximum number of characters allowed in a post, or `None` if there
    /// is no hard limit.
    ///
    /// # Returns
    ///
    /// - `Some(limit)` - The platform has a character limit
    /// - `None` - The platform has no hard character limit
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use libplurcast::platforms::Platform;
    /// # fn example(platform: &dyn Platform) {
    /// match platform.character_limit() {
    ///     Some(limit) => println!("{} has a {} character limit", platform.name(), limit),
    ///     None => println!("{} has no character limit", platform.name()),
    /// }
    /// # }
    /// ```
    fn character_limit(&self) -> Option<usize>;

    /// Check if the platform is properly configured
    ///
    /// Returns `true` if the platform has all necessary configuration (credentials, keys, etc.)
    /// to authenticate and post. This can be used to check configuration before attempting
    /// authentication.
    ///
    /// # Returns
    ///
    /// - `true` - Platform is configured and ready to authenticate
    /// - `false` - Platform is missing required configuration
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use libplurcast::platforms::Platform;
    /// # async fn example(platform: &mut dyn Platform) -> libplurcast::error::Result<()> {
    /// if !platform.is_configured() {
    ///     eprintln!("Platform {} is not configured", platform.name());
    ///     return Ok(());
    /// }
    /// platform.authenticate().await?;
    /// # Ok(())
    /// # }
    /// ```
    fn is_configured(&self) -> bool;
}
