//! SSB platform implementation
//!
//! This module contains the SSBPlatform struct and Platform trait implementation.

use async_trait::async_trait;
use std::path::PathBuf;

use crate::config::SSBConfig;
use crate::credentials::CredentialManager;
use crate::error::{PlatformError, Result};
use crate::platforms::Platform;

use super::keypair::SSBKeypair;
use super::message::SSBMessage;
use super::replication::PubConnection;

/// SSB platform implementation
pub struct SSBPlatform {
    config: SSBConfig,
    feed_path: PathBuf,
    keypair: Option<SSBKeypair>,
    initialized: bool,
    pub_connections: Vec<PubConnection>,
}

impl SSBPlatform {
    /// Create a new SSB platform instance
    pub fn new(config: &SSBConfig) -> Self {
        let expanded_path = shellexpand::tilde(&config.feed_path).to_string();
        let feed_path = PathBuf::from(expanded_path);

        tracing::info!(
            "Creating SSB platform with feed database at: {}",
            feed_path.display()
        );
        tracing::debug!(
            "SSB configuration: enabled={}, pubs={}",
            config.enabled,
            config.pubs.len()
        );

        let pub_connections: Vec<PubConnection> = config
            .pubs
            .iter()
            .filter_map(
                |addr_str| match super::replication::PubAddress::parse(addr_str) {
                    Ok(addr) => {
                        tracing::debug!("Parsed pub address: {}", addr_str);
                        Some(PubConnection::new(addr))
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse pub address '{}': {}", addr_str, e);
                        None
                    }
                },
            )
            .collect();

        if config.pubs.is_empty() {
            tracing::info!("No pub servers configured - SSB will operate in local-only mode");
        } else {
            tracing::info!(
                "Configured {} pub server(s) for replication",
                pub_connections.len()
            );
        }

        Self {
            config: config.clone(),
            feed_path,
            keypair: None,
            initialized: false,
            pub_connections,
        }
    }

    /// Initialize SSB platform with credentials
    pub async fn initialize_with_credentials(
        &mut self,
        credentials: &CredentialManager,
        account: &str,
    ) -> Result<()> {
        if self.initialized {
            tracing::debug!("SSB platform already initialized");
            return Ok(());
        }

        tracing::info!("Initializing SSB platform for account '{}'", account);
        tracing::debug!("Feed database path: {}", self.feed_path.display());

        self.create_feed_directory()?;

        tracing::debug!("Retrieving SSB keypair from credential manager");
        let keypair = Self::retrieve_keypair(credentials, account).map_err(|e| {
            if let crate::error::PlurcastError::Credential(
                crate::error::CredentialError::NotFound(_),
            ) = &e
            {
                tracing::error!("SSB credentials not found for account '{}'", account);
                PlatformError::Authentication(
                    "SSB credentials not configured - run plur-setup or plur-creds set ssb"
                        .to_string(),
                )
                .into()
            } else {
                tracing::error!("Failed to retrieve SSB credentials: {}", e);
                e
            }
        })?;

        tracing::debug!("Validating SSB keypair");
        keypair
            .validate()
            .map_err(|e| -> crate::error::PlurcastError {
                tracing::error!("SSB keypair validation failed: {}", e);
                PlatformError::Authentication(format!(
                    "Invalid SSB keypair - check credential format: {}",
                    e
                ))
                .into()
            })?;

        tracing::info!(
            "Loaded SSB keypair for account '{}' with feed ID: {}",
            account,
            keypair.id
        );

        self.keypair = Some(keypair);
        self.initialized = true;

        tracing::info!(
            "SSB platform initialized successfully with feed database at {}",
            self.feed_path.display()
        );

        if !self.pub_connections.is_empty() {
            tracing::info!(
                "Ready to replicate to {} pub server(s)",
                self.pub_connections.len()
            );
        }

        Ok(())
    }

    /// Create the feed database directory
    pub fn create_feed_directory(&self) -> Result<()> {
        if self.feed_path.exists() {
            if !self.feed_path.is_dir() {
                return Err(PlatformError::Authentication(format!(
                    "Feed path exists but is not a directory: {}",
                    self.feed_path.display()
                ))
                .into());
            }

            tracing::debug!(
                "Feed database directory already exists: {}",
                self.feed_path.display()
            );
            return Ok(());
        }

        std::fs::create_dir_all(&self.feed_path).map_err(|e| {
            PlatformError::Authentication(format!(
                "Failed to create SSB feed database at {}: {}",
                self.feed_path.display(),
                e
            ))
        })?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o700);
            std::fs::set_permissions(&self.feed_path, perms).map_err(|e| {
                PlatformError::Authentication(format!(
                    "Failed to set permissions on feed database directory {}: {}",
                    self.feed_path.display(),
                    e
                ))
            })?;

            tracing::debug!(
                "Created feed database directory with permissions 700: {}",
                self.feed_path.display()
            );
        }

        #[cfg(not(unix))]
        {
            tracing::debug!(
                "Created feed database directory: {}",
                self.feed_path.display()
            );
        }

        Ok(())
    }

    /// Store SSB keypair in credential manager
    ///
    /// # Arguments
    /// * `credentials` - Credential manager instance
    /// * `keypair` - The SSB keypair to store
    /// * `account` - Account name to store under
    /// * `force` - If true, overwrite existing credentials without checking. If false, return error if credentials exist.
    ///
    /// # Errors
    /// Returns error if credentials already exist and `force` is false
    pub fn store_keypair(
        credentials: &CredentialManager,
        keypair: &SSBKeypair,
        account: &str,
        force: bool,
    ) -> Result<()> {
        use crate::error::CredentialError;

        // Check if credentials already exist
        if !force && credentials.exists_account("plurcast.ssb", "keypair", account)? {
            return Err(CredentialError::AlreadyExists(format!(
                "SSB keypair already exists for account '{}'. This would overwrite your identity. \
                     Use force=true only if you're certain you want to overwrite.",
                account
            ))
            .into());
        }

        let json = keypair.to_json()?;
        credentials.store_account("plurcast.ssb", "keypair", account, &json)?;
        tracing::debug!(
            "Stored SSB keypair for account '{}' in credential manager",
            account
        );
        Ok(())
    }

    /// Retrieve SSB keypair from credential manager
    pub fn retrieve_keypair(credentials: &CredentialManager, account: &str) -> Result<SSBKeypair> {
        let json = credentials.retrieve_account("plurcast.ssb", "keypair", account)?;
        let keypair = SSBKeypair::from_json(&json)?;
        tracing::debug!(
            "Retrieved SSB keypair for account '{}' from credential manager",
            account
        );
        Ok(keypair)
    }

    /// Check if SSB keypair exists in credential manager
    pub fn has_keypair(credentials: &CredentialManager, account: &str) -> Result<bool> {
        credentials.exists_account("plurcast.ssb", "keypair", account)
    }

    /// Query the current feed state
    async fn query_feed_state(&self) -> Result<(u64, Option<String>)> {
        if !self.feed_path.exists() {
            tracing::debug!("Feed database does not exist, starting with sequence 1");
            return Ok((1, None));
        }

        let feed_state_file = self.feed_path.join("feed.json");

        if !feed_state_file.exists() {
            tracing::debug!("Feed state file does not exist, starting with sequence 1");
            return Ok((1, None));
        }

        let state_json = std::fs::read_to_string(&feed_state_file).map_err(|e| {
            PlatformError::Authentication(format!("Failed to read feed state file: {}", e))
        })?;

        let state: serde_json::Value = serde_json::from_str(&state_json).map_err(|e| {
            PlatformError::Authentication(format!("Failed to parse feed state file: {}", e))
        })?;

        let sequence = state
            .get("sequence")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| {
                PlatformError::Authentication("Invalid feed state: missing sequence".to_string())
            })?;

        let previous = state
            .get("previous")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let next_sequence = sequence + 1;

        tracing::debug!(
            "Feed state: current sequence {}, next sequence {}, previous: {:?}",
            sequence,
            next_sequence,
            previous.as_ref().map(|s| &s[..20.min(s.len())])
        );

        Ok((next_sequence, previous))
    }

    /// Append a signed message to the feed database
    async fn append_to_feed(&self, message: &SSBMessage) -> Result<String> {
        if message.signature.is_none() {
            return Err(PlatformError::Validation(
                "Cannot append unsigned message to feed".to_string(),
            )
            .into());
        }

        let hash = message.calculate_hash().map_err(|e| {
            PlatformError::Authentication(format!("Failed to calculate message hash: {}", e))
        })?;

        self.create_feed_directory()?;

        let messages_dir = self.feed_path.join("messages");
        if !messages_dir.exists() {
            std::fs::create_dir_all(&messages_dir).map_err(|e| {
                PlatformError::Authentication(format!("Failed to create messages directory: {}", e))
            })?;
        }

        let message_file = messages_dir.join(format!("{:010}.json", message.sequence));
        let message_json = serde_json::to_string_pretty(message).map_err(|e| {
            PlatformError::Authentication(format!("Failed to serialize message: {}", e))
        })?;

        std::fs::write(&message_file, message_json).map_err(|e| {
            PlatformError::Authentication(format!("Failed to write message to feed: {}", e))
        })?;

        let feed_state_file = self.feed_path.join("feed.json");
        let state = serde_json::json!({
            "sequence": message.sequence,
            "previous": hash,
            "author": message.author,
            "updated_at": chrono::Utc::now().to_rfc3339(),
        });

        let state_json = serde_json::to_string_pretty(&state).map_err(|e| {
            PlatformError::Authentication(format!("Failed to serialize feed state: {}", e))
        })?;

        std::fs::write(&feed_state_file, state_json).map_err(|e| {
            PlatformError::Authentication(format!("Failed to write feed state: {}", e))
        })?;

        tracing::debug!(
            "Appended message to feed: sequence {}, hash {}",
            message.sequence,
            &hash[..20.min(hash.len())]
        );

        let message_id = if hash.ends_with(".sha256") {
            format!("ssb:{}", &hash[..hash.len() - 7])
        } else {
            format!("ssb:{}", hash)
        };

        Ok(message_id)
    }

    /// Check if the platform is initialized with a keypair
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Get the feed ID (public key) if initialized
    pub fn feed_id(&self) -> Option<String> {
        self.keypair.as_ref().map(|kp| kp.id.clone())
    }

    /// Load keypair from credential manager (alias for retrieve_keypair for backward compatibility)
    pub fn load_keypair(credentials: &CredentialManager, account: &str) -> Result<SSBKeypair> {
        Self::retrieve_keypair(credentials, account)
    }

    /// Post to local feed without replication (for testing)
    pub async fn post_local(&self, content: &str) -> Result<String> {
        // This is the same as the post() implementation but without network replication
        // Create a minimal Post object for the platform
        let post = crate::Post {
            id: uuid::Uuid::new_v4().to_string(),
            content: content.to_string(),
            created_at: chrono::Utc::now().timestamp(),
            scheduled_at: None,
            status: crate::PostStatus::Pending,
            metadata: None,
        };
        self.post(&post).await
    }
}

#[async_trait]
impl Platform for SSBPlatform {
    async fn authenticate(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        Err(PlatformError::Authentication(
            "SSB platform not initialized - call initialize_with_credentials first".to_string(),
        )
        .into())
    }

    async fn post(&self, post: &crate::Post) -> Result<String> {
        if !self.initialized {
            return Err(
                PlatformError::Authentication("SSB platform not initialized".to_string()).into(),
            );
        }

        let keypair = self
            .keypair
            .as_ref()
            .ok_or_else(|| PlatformError::Authentication("SSB keypair not loaded".to_string()))?;

        tracing::debug!(
            "Validating content for SSB (length: {} bytes)",
            post.content.len()
        );
        self.validate_content(&post.content)?;

        tracing::debug!("Querying feed state from: {}", self.feed_path.display());
        let (sequence, previous) = self.query_feed_state().await?;

        tracing::debug!(
            "Creating SSB message: sequence={}, previous={:?}, content_length={}",
            sequence,
            previous.as_ref().map(|s| &s[..20.min(s.len())]),
            post.content.len()
        );

        let mut message = SSBMessage::new_post(&keypair.id, sequence, previous, &post.content);

        tracing::debug!("Signing message with keypair for feed: {}", keypair.id);
        message.sign(keypair)?;

        let message_size = message.calculate_size();
        tracing::debug!(
            "Message signed successfully, total size: {} bytes",
            message_size
        );

        tracing::debug!("Appending message to feed database");
        let message_id = self.append_to_feed(&message).await?;

        tracing::info!(
            "Posted to SSB: sequence {}, message ID: {}, size: {} bytes",
            sequence,
            message_id,
            message_size
        );

        // Log replication status
        if self.pub_connections.is_empty() {
            tracing::info!("No pub servers configured - message stored locally only");
        } else {
            tracing::info!(
                "Message ready for replication to {} pub server(s)",
                self.pub_connections.len()
            );
            // Note: Actual replication happens in background (task 8)
            tracing::debug!("Replication will occur in background process");
        }

        Ok(message_id)
    }

    fn validate_content(&self, content: &str) -> Result<()> {
        const MAX_MESSAGE_SIZE: usize = 8192;

        let test_message = SSBMessage::new_post("@test.ed25519", 1, None, content);

        let size = test_message.calculate_size();

        if size > MAX_MESSAGE_SIZE {
            return Err(PlatformError::Validation(format!(
                "Message size ({} bytes) exceeds SSB's practical limit of {} bytes",
                size, MAX_MESSAGE_SIZE
            ))
            .into());
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "ssb"
    }

    fn character_limit(&self) -> Option<usize> {
        None
    }

    fn is_configured(&self) -> bool {
        self.config.enabled
    }
}
