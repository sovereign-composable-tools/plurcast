//! Mock platform implementation for testing
//!
//! This module provides a configurable mock platform that can simulate various
//! behaviors including successes, failures, and delays. It's designed for use
//! in integration tests to verify multi-platform posting logic without requiring
//! actual platform credentials or network access.

use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;

use crate::error::{PlatformError, Result};
use crate::platforms::Platform;

/// Configuration for mock platform behavior
#[derive(Debug, Clone)]
pub struct MockConfig {
    /// Platform name (e.g., "mock-nostr", "mock-mastodon")
    pub name: String,

    /// Whether authentication should succeed
    pub auth_succeeds: bool,

    /// Whether posting should succeed
    pub post_succeeds: bool,

    /// Error to return on authentication failure
    pub auth_error: Option<String>,

    /// Error to return on posting failure
    pub post_error: Option<String>,

    /// Delay before completing operations (simulates network latency)
    pub delay: Duration,

    /// Character limit for validation
    pub character_limit: Option<usize>,

    /// Whether the platform is configured
    pub is_configured: bool,

    /// Number of times authenticate has been called
    pub auth_call_count: Arc<Mutex<usize>>,

    /// Number of times post has been called
    pub post_call_count: Arc<Mutex<usize>>,

    /// Posts that have been made (for verification)
    pub posted_content: Arc<Mutex<Vec<String>>>,
}

impl Default for MockConfig {
    fn default() -> Self {
        Self {
            name: "mock".to_string(),
            auth_succeeds: true,
            post_succeeds: true,
            auth_error: None,
            post_error: None,
            delay: Duration::from_millis(0),
            character_limit: None,
            is_configured: true,
            auth_call_count: Arc::new(Mutex::new(0)),
            post_call_count: Arc::new(Mutex::new(0)),
            posted_content: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

/// Mock platform for testing
pub struct MockPlatform {
    config: MockConfig,
    authenticated: bool,
}

impl MockPlatform {
    /// Create a new mock platform with the given configuration
    pub fn new(config: MockConfig) -> Self {
        Self {
            config,
            authenticated: false,
        }
    }

    /// Create a mock platform that always succeeds
    pub fn success(name: &str) -> Self {
        Self::new(MockConfig {
            name: name.to_string(),
            ..Default::default()
        })
    }

    /// Create a mock platform that fails authentication
    pub fn auth_failure(name: &str, error: &str) -> Self {
        Self::new(MockConfig {
            name: name.to_string(),
            auth_succeeds: false,
            auth_error: Some(error.to_string()),
            ..Default::default()
        })
    }

    /// Create a mock platform that fails posting
    pub fn post_failure(name: &str, error: &str) -> Self {
        Self::new(MockConfig {
            name: name.to_string(),
            post_succeeds: false,
            post_error: Some(error.to_string()),
            ..Default::default()
        })
    }

    /// Create a mock platform with a delay
    pub fn with_delay(name: &str, delay: Duration) -> Self {
        Self::new(MockConfig {
            name: name.to_string(),
            delay,
            ..Default::default()
        })
    }

    /// Create a mock platform with a character limit
    pub fn with_limit(name: &str, limit: usize) -> Self {
        Self::new(MockConfig {
            name: name.to_string(),
            character_limit: Some(limit),
            ..Default::default()
        })
    }

    /// Create a mock platform that is not configured
    pub fn not_configured(name: &str) -> Self {
        Self::new(MockConfig {
            name: name.to_string(),
            is_configured: false,
            ..Default::default()
        })
    }

    /// Create a simple mock platform with just a name (for compatibility)
    pub fn new_simple(name: &str) -> Self {
        let mut platform = Self::success(name);
        // Auto-authenticate for convenience in tests
        platform.authenticated = true;
        platform
    }

    /// Create a mock platform with a specific delay in milliseconds
    pub fn new_with_delay(name: &str, delay_ms: u64) -> Self {
        let mut platform = Self::with_delay(name, Duration::from_millis(delay_ms));
        // Auto-authenticate for convenience in tests
        platform.authenticated = true;
        platform
    }

    /// Get the number of times authenticate was called
    pub fn auth_call_count(&self) -> usize {
        *self.config.auth_call_count.lock().unwrap()
    }

    /// Get the number of times post was called
    pub fn post_call_count(&self) -> usize {
        *self.config.post_call_count.lock().unwrap()
    }

    /// Get all content that was posted
    pub fn posted_content(&self) -> Vec<String> {
        self.config.posted_content.lock().unwrap().clone()
    }
}

#[async_trait]
impl Platform for MockPlatform {
    async fn authenticate(&mut self) -> Result<()> {
        // Increment call count
        *self.config.auth_call_count.lock().unwrap() += 1;

        // Simulate delay
        if !self.config.delay.is_zero() {
            sleep(self.config.delay).await;
        }

        if self.config.auth_succeeds {
            self.authenticated = true;
            Ok(())
        } else {
            let error_msg = self
                .config
                .auth_error
                .clone()
                .unwrap_or_else(|| "Mock authentication failed".to_string());
            Err(PlatformError::Authentication(error_msg).into())
        }
    }

    async fn post(&self, content: &str) -> Result<String> {
        // Increment call count
        *self.config.post_call_count.lock().unwrap() += 1;

        // Check if authenticated
        if !self.authenticated {
            return Err(PlatformError::Authentication("Not authenticated".to_string()).into());
        }

        // Simulate delay
        if !self.config.delay.is_zero() {
            sleep(self.config.delay).await;
        }

        if self.config.post_succeeds {
            // Store posted content
            self.config
                .posted_content
                .lock()
                .unwrap()
                .push(content.to_string());

            // Generate mock post ID
            let post_id = format!("{}:mock-{}", self.config.name, uuid::Uuid::new_v4());
            Ok(post_id)
        } else {
            let error_msg = self
                .config
                .post_error
                .clone()
                .unwrap_or_else(|| "Mock posting failed".to_string());
            Err(PlatformError::Posting(error_msg).into())
        }
    }

    fn validate_content(&self, content: &str) -> Result<()> {
        if content.is_empty() {
            return Err(PlatformError::Validation("Content cannot be empty".to_string()).into());
        }

        if let Some(limit) = self.config.character_limit {
            if content.len() > limit {
                return Err(PlatformError::Validation(format!(
                    "Content exceeds {} character limit (got {} characters)",
                    limit,
                    content.len()
                ))
                .into());
            }
        }

        Ok(())
    }

    fn name(&self) -> &str {
        &self.config.name
    }

    fn character_limit(&self) -> Option<usize> {
        self.config.character_limit
    }

    fn is_configured(&self) -> bool {
        self.config.is_configured
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_success() {
        let mut platform = MockPlatform::success("test");

        assert!(platform.is_configured());
        assert_eq!(platform.name(), "test");
        assert_eq!(platform.character_limit(), None);

        // Authenticate
        platform.authenticate().await.unwrap();
        assert_eq!(platform.auth_call_count(), 1);

        // Post
        let post_id = platform.post("Test content").await.unwrap();
        assert!(post_id.starts_with("test:mock-"));
        assert_eq!(platform.post_call_count(), 1);

        // Verify content was stored
        let posted = platform.posted_content();
        assert_eq!(posted.len(), 1);
        assert_eq!(posted[0], "Test content");
    }

    #[tokio::test]
    async fn test_mock_auth_failure() {
        let mut platform = MockPlatform::auth_failure("test", "Invalid credentials");

        let result = platform.authenticate().await;
        assert!(result.is_err());
        assert_eq!(platform.auth_call_count(), 1);

        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid credentials"));
    }

    #[tokio::test]
    async fn test_mock_post_failure() {
        let mut platform = MockPlatform::post_failure("test", "Network error");

        platform.authenticate().await.unwrap();

        let result = platform.post("Test content").await;
        assert!(result.is_err());
        assert_eq!(platform.post_call_count(), 1);

        let err = result.unwrap_err();
        assert!(err.to_string().contains("Network error"));
    }

    #[tokio::test]
    async fn test_mock_with_delay() {
        let mut platform = MockPlatform::with_delay("test", Duration::from_millis(50));

        let start = std::time::Instant::now();
        platform.authenticate().await.unwrap();
        let auth_duration = start.elapsed();

        assert!(auth_duration >= Duration::from_millis(50));

        let start = std::time::Instant::now();
        platform.post("Test").await.unwrap();
        let post_duration = start.elapsed();

        assert!(post_duration >= Duration::from_millis(50));
    }

    #[tokio::test]
    async fn test_mock_with_character_limit() {
        let platform = MockPlatform::with_limit("test", 10);

        assert_eq!(platform.character_limit(), Some(10));

        // Valid content
        assert!(platform.validate_content("Short").is_ok());

        // Too long
        let result = platform.validate_content("This is way too long");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("character limit"));
    }

    #[tokio::test]
    async fn test_mock_not_configured() {
        let platform = MockPlatform::not_configured("test");

        assert!(!platform.is_configured());
    }

    #[tokio::test]
    async fn test_mock_requires_authentication() {
        let platform = MockPlatform::success("test");

        // Try to post without authenticating
        let result = platform.post("Test").await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Not authenticated"));
    }

    #[tokio::test]
    async fn test_mock_empty_content_validation() {
        let platform = MockPlatform::success("test");

        let result = platform.validate_content("");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }
}
