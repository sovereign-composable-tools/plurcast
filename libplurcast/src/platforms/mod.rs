//! Platform abstraction and implementations

use async_trait::async_trait;

use crate::error::Result;

pub mod nostr;

#[async_trait]
pub trait Platform: Send + Sync {
    /// Authenticate with the platform
    async fn authenticate(&mut self) -> Result<()>;

    /// Post content to the platform
    async fn post(&self, content: &str) -> Result<String>;

    /// Validate content before posting
    fn validate_content(&self, content: &str) -> Result<()>;

    /// Get the platform name
    fn name(&self) -> &str;
}
