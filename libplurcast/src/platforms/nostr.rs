//! Nostr platform implementation

use async_trait::async_trait;
use nostr_sdk::{Client, Keys, ToBech32};

use crate::config::NostrConfig;
use crate::error::{PlatformError, Result};
use crate::platforms::Platform;

/// Shared test account private key (publicly known, for testing/demos only)
/// 
/// This is a well-known test key that anyone can use. It's intentionally public
/// and serves as:
/// - A quick way to test Plurcast without setting up credentials
/// - A community bulletin board for Plurcast users
/// - A demo account for documentation and tutorials
/// 
/// Public key (npub): npub1qyv34w2prnz66zxrgqsmy2emrg0uqtrnvarhrrfaktxk9vp2dgllsajv05m
/// Handle: satoshi@nakamoto.btc
/// 
/// ⚠️ WARNING: Never use this for real posts! Anyone can post to this account.
pub const SHARED_TEST_KEY: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

pub struct NostrPlatform {
    client: Option<Client>,
    keys: Option<Keys>,
    relays: Vec<String>,
    authenticated: bool,
}

impl NostrPlatform {
    pub fn new(config: &NostrConfig) -> Self {
        Self {
            client: None,
            keys: None,
            relays: config.relays.clone(),
            authenticated: false,
        }
    }

    /// Load the shared test account keys
    /// 
    /// This is a publicly known test key that anyone can use for testing.
    /// It's an easter egg feature that allows users to try Plurcast without
    /// setting up credentials.
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use libplurcast::platforms::nostr::NostrPlatform;
    /// # use libplurcast::config::NostrConfig;
    /// let mut platform = NostrPlatform::new(&NostrConfig::default());
    /// platform.load_shared_test_keys().unwrap();
    /// // Now you can post to the shared test account!
    /// ```
    pub fn load_shared_test_keys(&mut self) -> Result<()> {
        tracing::info!("🎉 Using shared test account (publicly accessible)");
        self.load_keys_from_string(SHARED_TEST_KEY)
    }

    /// Load keys from a credential string
    ///
    /// Accepts keys in hex (64 characters) or bech32 (nsec) format.
    ///
    /// # Arguments
    ///
    /// * `key_str` - The private key as a string (hex or nsec format)
    ///
    /// # Errors
    ///
    /// Returns an error if the key format is invalid.
    pub fn load_keys_from_string(&mut self, key_str: &str) -> Result<()> {
        let key_str = key_str.trim();

        // Try parsing as hex or bech32
        let keys = if key_str.len() == 64 {
            // Hex format
            Keys::parse(key_str).map_err(|e| {
                PlatformError::Authentication(format!(
                    "Nostr authentication failed (load keys): Invalid hex key format: {}. \
                    Suggestion: Ensure the key is a valid 64-character hexadecimal string.",
                    e
                ))
            })?
        } else if key_str.starts_with("nsec") {
            // Bech32 format
            Keys::parse(key_str).map_err(|e| {
                PlatformError::Authentication(format!(
                    "Nostr authentication failed (load keys): Invalid bech32 key format: {}. \
                    Suggestion: Ensure the key is a valid nsec-prefixed bech32 string.",
                    e
                ))
            })?
        } else {
            return Err(PlatformError::Authentication(
                "Nostr authentication failed (load keys): Key must be 64-character hex or bech32 nsec format. \
                Suggestion: Generate a new key or ensure your existing key is in the correct format.".to_string(),
            )
            .into());
        };

        // Create client with the loaded keys
        self.client = Some(Client::new(keys.clone()));
        self.keys = Some(keys);
        Ok(())
    }

    /// Load keys from file (deprecated - use CredentialManager instead)
    ///
    /// This method is kept for backward compatibility but should be replaced
    /// with credential manager usage.
    #[deprecated(
        since = "0.2.0",
        note = "Use CredentialManager to retrieve credentials instead"
    )]
    pub fn load_keys(&mut self, keys_file: &str) -> Result<()> {
        let expanded_path = shellexpand::tilde(keys_file).to_string();
        let path = std::path::Path::new(&expanded_path);

        // Security: Validate that the keys file is not a symlink
        crate::credentials::validate_not_symlink(path).map_err(|e| {
            PlatformError::Authentication(format!(
                "Nostr authentication failed (load keys): {}",
                e
            ))
        })?;

        let content = std::fs::read_to_string(&expanded_path)
            .map_err(|e| PlatformError::Authentication(format!(
                "Nostr authentication failed (load keys): Failed to read keys file at '{}': {}. \
                Suggestion: Ensure the keys file exists and has proper read permissions (chmod 600).",
                expanded_path, e
            )))?;

        self.load_keys_from_string(&content)
    }
}

#[async_trait]
impl Platform for NostrPlatform {
    async fn authenticate(&mut self) -> Result<()> {
        if self.keys.is_none() {
            return Err(PlatformError::Authentication(
                "Nostr authentication failed (authenticate): Keys not loaded. \
                Suggestion: Load keys using load_keys() before calling authenticate()."
                    .to_string(),
            )
            .into());
        }

        let client = self.client.as_ref().ok_or_else(|| {
            PlatformError::Authentication(
                "Nostr authentication failed (authenticate): Client not initialized. \
                Suggestion: Load keys using load_keys() before calling authenticate()."
                    .to_string(),
            )
        })?;

        // Add relays
        tracing::debug!("Adding {} Nostr relays", self.relays.len());
        for relay in &self.relays {
            tracing::debug!("  Adding relay: {}", relay);
            client.add_relay(relay).await.map_err(|e| {
                PlatformError::Network(format!(
                    "Nostr network error (add relay): Failed to add relay '{}': {}. \
                    Suggestion: Check that the relay URL is valid and accessible.",
                    relay, e
                ))
            })?;
        }

        // Connect to relays
        tracing::debug!("Connecting to Nostr relays...");
        client.connect().await;

        self.authenticated = true;
        tracing::debug!("Nostr authentication complete");
        Ok(())
    }

    async fn post(&self, content: &str) -> Result<String> {
        if !self.authenticated {
            return Err(PlatformError::Authentication(
                "Nostr posting failed (post): Not authenticated. \
                Suggestion: Call authenticate() before attempting to post."
                    .to_string(),
            )
            .into());
        }

        let client = self.client.as_ref().ok_or_else(|| {
            PlatformError::Authentication(
                "Nostr posting failed (post): Client not initialized. \
                Suggestion: Load keys using load_keys() before attempting to post."
                    .to_string(),
            )
        })?;

        // Create and sign event
        let event_id = client.publish_text_note(content, []).await.map_err(|e| {
            PlatformError::Posting(format!(
                "Nostr posting failed (publish): Failed to publish note: {}. \
                Suggestion: Check relay connectivity and ensure your keys are valid. \
                The system will automatically retry transient failures.",
                e
            ))
        })?;

        // Return note ID in bech32 format
        Ok(event_id
            .id()
            .to_bech32()
            .unwrap_or_else(|_| event_id.id().to_hex()))
    }

    fn validate_content(&self, content: &str) -> Result<()> {
        if content.is_empty() {
            return Err(PlatformError::Validation(
                "Nostr validation failed (validate content): Content cannot be empty. \
                Suggestion: Provide non-empty content to post."
                    .to_string(),
            )
            .into());
        }

        // Warn if content is very long (no hard limit for Nostr)
        if content.len() > 280 {
            tracing::warn!(
                "Nostr: Content exceeds 280 characters ({} chars), may be truncated by some clients",
                content.len()
            );
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "nostr"
    }

    fn character_limit(&self) -> Option<usize> {
        // Nostr has no hard character limit enforced by the protocol
        // Some clients may have their own limits, but the protocol itself doesn't
        None
    }

    fn is_configured(&self) -> bool {
        // Platform is configured if keys have been loaded
        self.keys.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::NostrConfig;
    use tempfile::TempDir;

    fn create_test_config() -> NostrConfig {
        NostrConfig {
            enabled: true,
            keys_file: "/tmp/test_keys".to_string(),
            relays: vec![
                "wss://relay.damus.io".to_string(),
                "wss://nos.lol".to_string(),
            ],
        }
    }

    #[test]
    fn test_key_parsing_hex_format() {
        let temp_dir = TempDir::new().unwrap();
        let keys_file = temp_dir.path().join("hex_keys");

        // Generate a test key and get its hex representation
        let test_keys = Keys::generate();
        let hex_key = test_keys.secret_key().to_secret_hex();

        // Write hex key to file (should be 64 characters)
        assert_eq!(hex_key.len(), 64, "Hex key should be 64 characters");
        std::fs::write(&keys_file, &hex_key).unwrap();

        // Test parsing
        let config = NostrConfig {
            enabled: true,
            keys_file: keys_file.to_str().unwrap().to_string(),
            relays: vec![],
        };

        let mut platform = NostrPlatform::new(&config);
        let keys_content = std::fs::read_to_string(&keys_file).unwrap();
        let result = platform.load_keys_from_string(&keys_content);

        assert!(result.is_ok(), "Should parse valid hex key");
        assert!(platform.keys.is_some());
    }

    #[test]
    fn test_key_parsing_bech32_nsec_format() {
        let temp_dir = TempDir::new().unwrap();
        let keys_file = temp_dir.path().join("bech32_keys");

        // Generate a test key and get its bech32 representation
        let test_keys = Keys::generate();
        let bech32_key = test_keys.secret_key().to_bech32().unwrap();

        // Verify it starts with nsec
        assert!(
            bech32_key.starts_with("nsec"),
            "Bech32 key should start with nsec"
        );
        std::fs::write(&keys_file, &bech32_key).unwrap();

        // Test parsing
        let config = NostrConfig {
            enabled: true,
            keys_file: keys_file.to_str().unwrap().to_string(),
            relays: vec![],
        };

        let mut platform = NostrPlatform::new(&config);
        let keys_content = std::fs::read_to_string(&keys_file).unwrap();
        let result = platform.load_keys_from_string(&keys_content);

        assert!(result.is_ok(), "Should parse valid bech32 nsec key");
        assert!(platform.keys.is_some());
    }

    #[test]
    fn test_key_parsing_invalid_hex_format() {
        let temp_dir = TempDir::new().unwrap();
        let keys_file = temp_dir.path().join("invalid_hex_keys");

        // Write invalid hex key (wrong length)
        std::fs::write(&keys_file, "invalid_hex_key_too_short").unwrap();

        let config = NostrConfig {
            enabled: true,
            keys_file: keys_file.to_str().unwrap().to_string(),
            relays: vec![],
        };

        let mut platform = NostrPlatform::new(&config);
        let keys_content = std::fs::read_to_string(&keys_file).unwrap();
        let result = platform.load_keys_from_string(&keys_content);

        assert!(result.is_err(), "Should fail on invalid hex key");

        match result {
            Err(crate::PlurcastError::Platform(PlatformError::Authentication(msg))) => {
                assert!(msg.contains("must be 64-character hex or bech32 nsec format"));
            }
            _ => panic!("Expected authentication error"),
        }
    }

    #[test]
    fn test_key_parsing_invalid_bech32_format() {
        let temp_dir = TempDir::new().unwrap();
        let keys_file = temp_dir.path().join("invalid_bech32_keys");

        // Write invalid bech32 key
        std::fs::write(&keys_file, "nsec_invalid_checksum_12345").unwrap();

        let config = NostrConfig {
            enabled: true,
            keys_file: keys_file.to_str().unwrap().to_string(),
            relays: vec![],
        };

        let mut platform = NostrPlatform::new(&config);
        let keys_content = std::fs::read_to_string(&keys_file).unwrap();
        let result = platform.load_keys_from_string(&keys_content);

        assert!(result.is_err(), "Should fail on invalid bech32 key");
    }

    #[test]
    fn test_key_parsing_missing_file() {
        // Test that file I/O fails appropriately
        let missing_file = std::path::PathBuf::from("/nonexistent/path/keys");
        let file_result = std::fs::read_to_string(&missing_file);
        assert!(
            file_result.is_err(),
            "Should fail when keys file doesn't exist"
        );

        // Test that the parser also rejects invalid content
        let config = NostrConfig {
            enabled: true,
            keys_file: "/nonexistent/path/keys".to_string(),
            relays: vec![],
        };

        let mut platform = NostrPlatform::new(&config);
        let result = platform.load_keys_from_string("not_a_valid_key");

        assert!(result.is_err(), "Should fail on invalid key format");

        match result {
            Err(crate::PlurcastError::Platform(PlatformError::Authentication(msg))) => {
                assert!(msg.contains("must be 64-character hex or bech32 nsec format"));
            }
            _ => panic!("Expected authentication error"),
        }
    }

    #[test]
    fn test_content_validation_empty_content() {
        let config = create_test_config();
        let platform = NostrPlatform::new(&config);

        let result = platform.validate_content("");

        assert!(result.is_err(), "Should fail on empty content");

        match result {
            Err(crate::PlurcastError::Platform(PlatformError::Validation(msg))) => {
                assert!(msg.contains("Content cannot be empty"));
                assert!(msg.contains("Nostr"));
            }
            _ => panic!("Expected validation error"),
        }
    }

    #[test]
    fn test_content_validation_normal_content() {
        let config = create_test_config();
        let platform = NostrPlatform::new(&config);

        let result = platform.validate_content("This is a normal post");

        assert!(result.is_ok(), "Should accept normal content");
    }

    #[test]
    fn test_content_validation_long_content() {
        let config = create_test_config();
        let platform = NostrPlatform::new(&config);

        // Create content longer than 280 characters
        let long_content = "a".repeat(300);

        // Should still succeed (Nostr has no hard limit), but may log a warning
        let result = platform.validate_content(&long_content);

        assert!(
            result.is_ok(),
            "Should accept long content (Nostr has no hard limit)"
        );
    }

    #[test]
    fn test_content_validation_exactly_280_chars() {
        let config = create_test_config();
        let platform = NostrPlatform::new(&config);

        let content = "a".repeat(280);
        let result = platform.validate_content(&content);

        assert!(
            result.is_ok(),
            "Should accept content at 280 character boundary"
        );
    }

    #[tokio::test]
    async fn test_posting_without_authentication() {
        let config = create_test_config();
        let platform = NostrPlatform::new(&config);

        // Try to post without authenticating
        let result = platform.post("Test content").await;

        assert!(result.is_err(), "Should fail when not authenticated");

        match result {
            Err(crate::PlurcastError::Platform(PlatformError::Authentication(msg))) => {
                assert!(msg.contains("Not authenticated"));
                assert!(msg.contains("Nostr"));
            }
            _ => panic!("Expected authentication error"),
        }
    }

    #[tokio::test]
    async fn test_authenticate_without_keys() {
        let config = create_test_config();
        let mut platform = NostrPlatform::new(&config);

        // Try to authenticate without loading keys
        let result = platform.authenticate().await;

        assert!(result.is_err(), "Should fail when keys not loaded");

        match result {
            Err(crate::PlurcastError::Platform(PlatformError::Authentication(msg))) => {
                assert!(msg.contains("Keys not loaded"));
                assert!(msg.contains("Nostr"));
            }
            _ => panic!("Expected authentication error"),
        }
    }

    #[tokio::test]
    async fn test_authenticate_sets_authenticated_flag() {
        let temp_dir = TempDir::new().unwrap();
        let keys_file = temp_dir.path().join("test_keys");

        // Generate and save test keys
        let test_keys = Keys::generate();
        let hex_key = test_keys.secret_key().to_secret_hex();
        std::fs::write(&keys_file, &hex_key).unwrap();

        let config = NostrConfig {
            enabled: true,
            keys_file: keys_file.to_str().unwrap().to_string(),
            relays: vec![], // Empty relays to avoid actual network connections
        };

        let mut platform = NostrPlatform::new(&config);

        // Load keys
        let keys_content = std::fs::read_to_string(&keys_file).unwrap();
        platform.load_keys_from_string(&keys_content).unwrap();

        // Verify not authenticated initially
        assert!(!platform.authenticated);

        // Authenticate (with empty relays, this should succeed without network calls)
        let result = platform.authenticate().await;

        // Should succeed even with no relays
        assert!(
            result.is_ok(),
            "Authentication should succeed with loaded keys"
        );

        // Verify authenticated flag is set
        assert!(platform.authenticated);
    }

    #[test]
    fn test_platform_name() {
        let config = create_test_config();
        let platform = NostrPlatform::new(&config);

        assert_eq!(platform.name(), "nostr");
    }

    #[test]
    fn test_key_parsing_with_whitespace() {
        let temp_dir = TempDir::new().unwrap();
        let keys_file = temp_dir.path().join("keys_with_whitespace");

        // Generate a test key with surrounding whitespace
        let test_keys = Keys::generate();
        let hex_key = test_keys.secret_key().to_secret_hex();
        let key_with_whitespace = format!("\n  {}  \n", hex_key);

        std::fs::write(&keys_file, key_with_whitespace).unwrap();

        let config = NostrConfig {
            enabled: true,
            keys_file: keys_file.to_str().unwrap().to_string(),
            relays: vec![],
        };

        let mut platform = NostrPlatform::new(&config);
        let keys_content = std::fs::read_to_string(&keys_file).unwrap();
        let result = platform.load_keys_from_string(&keys_content);

        assert!(result.is_ok(), "Should handle whitespace in keys file");
    }

    #[test]
    fn test_multiple_relays_configuration() {
        let config = NostrConfig {
            enabled: true,
            keys_file: "/tmp/keys".to_string(),
            relays: vec![
                "wss://relay1.example.com".to_string(),
                "wss://relay2.example.com".to_string(),
                "wss://relay3.example.com".to_string(),
            ],
        };

        let platform = NostrPlatform::new(&config);

        assert_eq!(platform.relays.len(), 3);
        assert_eq!(platform.relays[0], "wss://relay1.example.com");
        assert_eq!(platform.relays[1], "wss://relay2.example.com");
        assert_eq!(platform.relays[2], "wss://relay3.example.com");
    }

    #[test]
    fn test_character_limit_returns_none() {
        let config = create_test_config();
        let platform = NostrPlatform::new(&config);

        // Nostr has no hard character limit
        assert_eq!(platform.character_limit(), None);
    }

    #[test]
    fn test_is_configured_without_keys() {
        let config = create_test_config();
        let platform = NostrPlatform::new(&config);

        // Platform should not be configured without keys loaded
        assert!(!platform.is_configured());
    }

    #[test]
    fn test_is_configured_with_keys() {
        let temp_dir = TempDir::new().unwrap();
        let keys_file = temp_dir.path().join("test_keys");

        // Generate and save test keys
        let test_keys = Keys::generate();
        let hex_key = test_keys.secret_key().to_secret_hex();
        std::fs::write(&keys_file, &hex_key).unwrap();

        let config = NostrConfig {
            enabled: true,
            keys_file: keys_file.to_str().unwrap().to_string(),
            relays: vec![],
        };

        let mut platform = NostrPlatform::new(&config);

        // Should not be configured before loading keys
        assert!(!platform.is_configured());

        // Load keys
        let keys_content = std::fs::read_to_string(&keys_file).unwrap();
        platform.load_keys_from_string(&keys_content).unwrap();

        // Should be configured after loading keys
        assert!(platform.is_configured());
    }

    #[test]
    fn test_is_configured_with_invalid_keys_file() {
        let config = NostrConfig {
            enabled: true,
            keys_file: "/nonexistent/path/keys".to_string(),
            relays: vec![],
        };

        let mut platform = NostrPlatform::new(&config);

        // Should not be configured initially
        assert!(!platform.is_configured());

        // Try to load invalid key content (will fail)
        let result = platform.load_keys_from_string("invalid_key_format");
        assert!(result.is_err());

        // Should still not be configured after failed load
        assert!(!platform.is_configured());
    }
}
