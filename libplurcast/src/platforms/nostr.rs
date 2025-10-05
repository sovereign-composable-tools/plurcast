//! Nostr platform implementation

use async_trait::async_trait;
use nostr_sdk::{Client, Keys, ToBech32};

use crate::config::NostrConfig;
use crate::error::{PlatformError, Result};
use crate::platforms::Platform;

pub struct NostrPlatform {
    client: Client,
    keys: Option<Keys>,
    relays: Vec<String>,
    authenticated: bool,
}

impl NostrPlatform {
    pub fn new(config: &NostrConfig) -> Self {
        let client = Client::new(Keys::generate());

        Self {
            client,
            keys: None,
            relays: config.relays.clone(),
            authenticated: false,
        }
    }

    /// Load keys from file
    pub fn load_keys(&mut self, keys_file: &str) -> Result<()> {
        let expanded_path = shellexpand::tilde(keys_file).to_string();
        let content = std::fs::read_to_string(&expanded_path)
            .map_err(|e| PlatformError::Authentication(format!("Failed to read keys file: {}", e)))?;

        let key_str = content.trim();

        // Try parsing as hex or bech32
        let keys = if key_str.len() == 64 {
            // Hex format
            Keys::parse(key_str)
                .map_err(|e| PlatformError::Authentication(format!("Invalid hex key: {}", e)))?
        } else if key_str.starts_with("nsec") {
            // Bech32 format
            Keys::parse(key_str)
                .map_err(|e| PlatformError::Authentication(format!("Invalid bech32 key: {}", e)))?
        } else {
            return Err(PlatformError::Authentication(
                "Key must be 64-character hex or bech32 nsec format".to_string(),
            )
            .into());
        };

        self.keys = Some(keys);
        Ok(())
    }
}

#[async_trait]
impl Platform for NostrPlatform {
    async fn authenticate(&mut self) -> Result<()> {
        if self.keys.is_none() {
            return Err(PlatformError::Authentication("Keys not loaded".to_string()).into());
        }

        // Add relays
        for relay in &self.relays {
            self.client.add_relay(relay).await
                .map_err(|e| PlatformError::Network(format!("Failed to add relay {}: {}", relay, e)))?;
        }

        // Connect to relays
        self.client.connect().await;

        self.authenticated = true;
        Ok(())
    }

    async fn post(&self, content: &str) -> Result<String> {
        if !self.authenticated {
            return Err(PlatformError::Authentication("Not authenticated".to_string()).into());
        }

        // Create and sign event
        let event_id = self.client
            .publish_text_note(content, [])
            .await
            .map_err(|e| PlatformError::Posting(format!("Failed to publish: {}", e)))?;

        // Return note ID in bech32 format
        Ok(event_id.id().to_bech32().unwrap_or_else(|_| event_id.id().to_hex()))
    }

    fn validate_content(&self, content: &str) -> Result<()> {
        if content.is_empty() {
            return Err(PlatformError::Validation("Content cannot be empty".to_string()).into());
        }

        // Warn if content is very long (no hard limit for Nostr)
        if content.len() > 280 {
            tracing::warn!("Content exceeds 280 characters, may be truncated by some clients");
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "nostr"
    }
}
