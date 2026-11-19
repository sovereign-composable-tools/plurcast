//! Nostr platform implementation

use async_trait::async_trait;
use nostr_sdk::prelude::{Client, EventBuilder, EventId, FinalizeEvent, Keys, Kind, Tag, ToBech32};
use secrecy::{ExposeSecret, SecretBox, SecretString};

use crate::config::NostrConfig;
use crate::error::{PlatformError, Result};
use crate::platforms::Platform;

mod keys;
mod relay;
#[cfg(test)]
mod tests;

use keys::NostrKeys;
use relay::{send_to_relays, RELAY_CONNECT_TIMEOUT};

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
pub const SHARED_TEST_KEY: &str =
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

pub struct NostrPlatform {
    client: Option<Client>,
    keys: Option<SecretBox<NostrKeys>>, // SecretBox redacts Debug and zeroizes on drop
    relays: Vec<String>,
    authenticated: bool,
}

// Custom Debug implementation that doesn't expose keys
impl std::fmt::Debug for NostrPlatform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NostrPlatform")
            .field("client", &self.client.is_some())
            .field("keys", &self.keys) // SecretBox has a Debug impl that redacts content
            .field("relays", &self.relays)
            .field("authenticated", &self.authenticated)
            .finish()
    }
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
    /// ```ignore
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
    ///
    /// # Security
    ///
    /// The input key string is wrapped in a SecretString and automatically zeroed
    /// after parsing. The parsed Keys are stored in a Secret wrapper for automatic
    /// memory zeroing when dropped.
    pub fn load_keys_from_string(&mut self, key_str: &str) -> Result<()> {
        // Wrap input in SecretString for automatic zeroing
        let secret_input = SecretString::from(key_str.trim().to_string());

        // Parse keys from the secret input
        let keys = {
            let key_str_ref = secret_input.expose_secret();

            // Try parsing as hex or bech32
            if key_str_ref.len() == 64 {
                // Hex format
                Keys::parse(key_str_ref).map_err(|e| {
                    PlatformError::Authentication(format!(
                        "Nostr authentication failed (load keys): Invalid hex key format: {}. \
                        Suggestion: Ensure the key is a valid 64-character hexadecimal string.",
                        e
                    ))
                })?
            } else if key_str_ref.starts_with("nsec") {
                // Bech32 format
                Keys::parse(key_str_ref).map_err(|e| {
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
            }
        };

        // The client only manages relay connections; events are signed locally
        // with the stored keys before being handed to it.
        self.client = Some(Client::new());

        // Store keys wrapped in NostrKeys and Secret for automatic zeroing on drop
        self.keys = Some(SecretBox::new(Box::new(NostrKeys::new(keys))));

        // secret_input automatically zeroed on drop (SecretString behavior)

        tracing::debug!("Loaded Nostr keys (memory protected with automatic zeroing)");
        Ok(())
    }

    /// Get the public key in hex format (for database lookups)
    pub fn public_key_hex(&self) -> Option<String> {
        self.keys
            .as_ref()
            .map(|k| k.expose_secret().as_keys().public_key().to_hex())
    }

    /// Get the configured relays
    pub fn relays(&self) -> &[String] {
        &self.relays
    }

    /// Publish a NIP-65 relay list (kind 10002) event
    ///
    /// This publishes the configured relays as a relay list metadata event,
    /// enabling NIP-65 outbox model discovery.
    ///
    /// # Returns
    ///
    /// Returns the event ID of the published relay list event.
    pub async fn publish_relay_list(&self) -> Result<EventId> {
        if !self.authenticated {
            return Err(PlatformError::Authentication(
                "Nostr relay list publishing failed: Not authenticated. \
                Suggestion: Call authenticate() before publishing relay list."
                    .to_string(),
            )
            .into());
        }

        let client = self.client.as_ref().ok_or_else(|| {
            PlatformError::Authentication(
                "Nostr relay list publishing failed: Client not initialized.".to_string(),
            )
        })?;

        let keys = self.keys.as_ref().ok_or_else(|| {
            PlatformError::Authentication(
                "Nostr relay list publishing failed: Keys not loaded.".to_string(),
            )
        })?;

        // Build relay list tags
        // NIP-65 format: ["r", "wss://relay.example.com"] for read+write
        // or ["r", "wss://relay.example.com", "read"] or ["r", "...", "write"] for specific
        // We use read+write (no marker) for all configured relays
        let relay_tags: Vec<Tag> = self
            .relays
            .iter()
            .map(|relay| Tag::custom("r", vec![relay.clone()]))
            .collect();

        tracing::info!(
            "Publishing NIP-65 relay list with {} relays",
            relay_tags.len()
        );

        // Build kind 10002 event
        let event = EventBuilder::new(Kind::RelayList, "")
            .tags(relay_tags)
            .finalize(keys.expose_secret().as_keys())
            .map_err(|e| {
                PlatformError::Posting(format!("Failed to build relay list event: {}", e))
            })?;

        // Send to all relays
        let output = send_to_relays(client, &event)
            .await
            .map_err(|e| PlatformError::Posting(format!("Failed to publish relay list: {}", e)))?;

        let event_id = *output.id();
        tracing::info!(
            "Published NIP-65 relay list: {} (to {} relays)",
            event_id.to_bech32().unwrap_or_else(|_| event_id.to_hex()),
            self.relays.len()
        );

        Ok(event_id)
    }

    /// Check if the relay list needs to be updated
    ///
    /// This is a simplified check that always returns true if called.
    /// The actual staleness check is done via the database cache in the posting service.
    /// This method exists for the Platform trait implementation.
    ///
    /// # Arguments
    ///
    /// * `_staleness_days` - Number of days after which a relay list is considered stale (unused)
    pub async fn relay_list_needs_update(&self, _staleness_days: i64) -> Result<bool> {
        // The posting service checks the database cache first.
        // If this method is called, it means the cache didn't have fresh data,
        // so we should publish.
        tracing::debug!("Relay list staleness check - will publish");
        Ok(true)
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
            PlatformError::Authentication(format!("Nostr authentication failed (load keys): {}", e))
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

// Drop implementation for secure memory zeroing
impl Drop for NostrPlatform {
    fn drop(&mut self) {
        if self.keys.is_some() {
            tracing::debug!("Zeroing Nostr private key from memory");
            // Keys automatically zeroed by SecretBox<T> wrapper
            self.keys = None;
        }
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
        client.connect().and_wait(RELAY_CONNECT_TIMEOUT).await;

        self.authenticated = true;
        tracing::debug!("Nostr authentication complete");
        Ok(())
    }

    async fn post(&self, post: &crate::Post) -> Result<String> {
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

        let keys = self.keys.as_ref().ok_or_else(|| {
            PlatformError::Authentication(
                "Nostr posting failed (post): Keys not loaded. \
                Suggestion: Load keys using load_keys() before attempting to post."
                    .to_string(),
            )
        })?;

        // Extract POW difficulty from metadata if present
        let pow_difficulty: Option<u8> = post
            .metadata
            .as_ref()
            .and_then(|metadata_str| serde_json::from_str::<serde_json::Value>(metadata_str).ok())
            .and_then(|metadata| {
                metadata
                    .get("nostr")
                    .and_then(|nostr| nostr.get("pow_difficulty"))
                    .and_then(|diff| diff.as_u64())
                    .map(|d| d as u8)
            });

        // Extract 21e8 flag from metadata
        let require_21e8: bool = post
            .metadata
            .as_ref()
            .and_then(|metadata_str| serde_json::from_str::<serde_json::Value>(metadata_str).ok())
            .and_then(|metadata| {
                metadata
                    .get("nostr")
                    .and_then(|nostr| nostr.get("21e8"))
                    .and_then(|flag| flag.as_bool())
            })
            .unwrap_or(false);

        // Extract reply_to from metadata for NIP-10 threading
        // reply_to is now a per-platform map: { "nostr": "note1...", "mastodon": "12345" }
        let reply_to: Option<EventId> = post
            .metadata
            .as_ref()
            .and_then(|metadata_str| serde_json::from_str::<serde_json::Value>(metadata_str).ok())
            .and_then(|metadata| {
                metadata
                    .get("reply_to")
                    .and_then(|r| r.get("nostr")) // Get Nostr-specific ID
                    .and_then(|r| r.as_str())
                    .map(String::from)
            })
            .and_then(|id_str| {
                // Try parsing as bech32 note1... format first, then as hex
                EventId::parse(&id_str).ok()
            });

        // Build tags for NIP-10 threading
        // NIP-10 format: ["e", "<event-id>", "<relay>", "reply"]
        // The "reply" marker tells clients this is a direct reply for threading
        let tags: Vec<Tag> = if let Some(parent_id) = reply_to {
            tracing::debug!("Adding NIP-10 reply tag for parent event: {}", parent_id);
            vec![Tag::custom(
                "e",
                vec![parent_id.to_hex(), String::new(), "reply".to_string()],
            )]
        } else {
            vec![]
        };

        // Create and publish event (with or without POW)
        let event_id = if let Some(difficulty) = pow_difficulty {
            // Use parallel POW mining (multi-threaded)
            if require_21e8 {
                tracing::info!(
                    "Mining Nostr event with 21e8 pattern (difficulty {})...",
                    difficulty
                );
            } else {
                tracing::info!(
                    "Mining Nostr event with POW difficulty {} (parallel)...",
                    difficulty
                );
            }

            let event = crate::platforms::nostr_pow::mine_event_parallel(
                &post.content,
                keys.expose_secret().as_keys(),
                difficulty,
                require_21e8,
                tags.clone(), // Include NIP-10 reply tags in POW event
            )
            .await
            .map_err(|e| {
                PlatformError::Posting(format!(
                    "Nostr POW mining failed: {}. \
                    Suggestion: Try a lower difficulty value.",
                    e
                ))
            })?;

            tracing::info!("POW mining complete, publishing event...");
            send_to_relays(client, &event).await.map_err(|e| {
                PlatformError::Posting(format!(
                    "Nostr posting failed (publish POW event): Failed to publish note: {}. \
                    Suggestion: Check relay connectivity and ensure your keys are valid. \
                    The system will automatically retry transient failures.",
                    e
                ))
            })?
        } else {
            // Standard posting without POW (includes NIP-10 tags if replying)
            let event = EventBuilder::new(Kind::TextNote, post.content.as_str())
                .tags(tags)
                .finalize(keys.expose_secret().as_keys())
                .map_err(|e| {
                    PlatformError::Posting(format!(
                        "Nostr posting failed (sign): Failed to sign note: {}",
                        e
                    ))
                })?;
            send_to_relays(client, &event).await.map_err(|e| {
                PlatformError::Posting(format!(
                    "Nostr posting failed (publish): Failed to publish note: {}. \
                    Suggestion: Check relay connectivity and ensure your keys are valid. \
                    The system will automatically retry transient failures.",
                    e
                ))
            })?
        };

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

    // ========================================================================
    // NIP-65 Relay List Methods
    // ========================================================================

    fn supports_relay_list(&self) -> bool {
        true
    }

    fn relay_list_pubkey(&self) -> Option<String> {
        self.public_key_hex()
    }

    fn relay_count(&self) -> usize {
        self.relays.len()
    }

    async fn publish_relay_list(&self) -> Result<String> {
        let event_id = NostrPlatform::publish_relay_list(self).await?;
        Ok(event_id.to_bech32().unwrap_or_else(|_| event_id.to_hex()))
    }

    async fn check_relay_list_stale(&self, staleness_days: i64) -> Result<bool> {
        self.relay_list_needs_update(staleness_days).await
    }
}
