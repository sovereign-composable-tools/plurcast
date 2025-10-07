//! Secure credential storage abstraction for Plurcast
//!
//! This module provides a trait-based abstraction for storing platform credentials
//! securely using multiple backends: OS keyring, encrypted files, or plain text files.
//!
//! # Architecture
//!
//! The credential storage system uses a layered approach:
//! - `CredentialStore` trait: Common interface for all storage backends
//! - `KeyringStore`: OS-native secure storage (primary)
//! - `EncryptedFileStore`: Password-protected file storage (fallback)
//! - `PlainFileStore`: Plain text files (legacy/backward compatibility)
//! - `CredentialManager`: Facade that manages fallback logic
//!
//! # Example
//!
//! ```no_run
//! use libplurcast::credentials::{CredentialManager, CredentialConfig, StorageBackend};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = CredentialConfig {
//!     storage: StorageBackend::Keyring,
//!     path: "~/.config/plurcast/credentials".to_string(),
//!     master_password: None,
//! };
//!
//! let manager = CredentialManager::new(config)?;
//!
//! // Store a credential
//! manager.store("plurcast.nostr", "private_key", "nsec1...")?;
//!
//! // Retrieve a credential
//! let key = manager.retrieve("plurcast.nostr", "private_key")?;
//!
//! // Check if credential exists
//! if manager.exists("plurcast.mastodon", "access_token")? {
//!     println!("Mastodon credentials found");
//! }
//!
//! // Delete a credential
//! manager.delete("plurcast.bluesky", "app_password")?;
//! # Ok(())
//! # }
//! ```

use crate::error::Result;

/// Trait for credential storage backends
///
/// This trait defines the common interface that all credential storage
/// implementations must provide. It supports storing, retrieving, deleting,
/// and checking for the existence of credentials.
///
/// # Service and Key Naming
///
/// - **Service**: Format is `plurcast.{platform}` (e.g., "plurcast.nostr", "plurcast.mastodon")
/// - **Key**: Format is `{credential_type}` (e.g., "private_key", "access_token", "app_password")
///
/// # Example Implementation
///
/// ```no_run
/// use libplurcast::credentials::CredentialStore;
/// use libplurcast::error::Result;
///
/// struct MyStore;
///
/// impl CredentialStore for MyStore {
///     fn store(&self, service: &str, key: &str, value: &str) -> Result<()> {
///         // Store credential securely
///         Ok(())
///     }
///
///     fn retrieve(&self, service: &str, key: &str) -> Result<String> {
///         // Retrieve credential
///         Ok("credential_value".to_string())
///     }
///
///     fn delete(&self, service: &str, key: &str) -> Result<()> {
///         // Delete credential
///         Ok(())
///     }
///
///     fn exists(&self, service: &str, key: &str) -> Result<bool> {
///         // Check if credential exists
///         Ok(true)
///     }
///
///     fn backend_name(&self) -> &str {
///         "my_store"
///     }
/// }
/// ```
pub trait CredentialStore: Send + Sync {
    /// Store a credential
    ///
    /// # Arguments
    ///
    /// * `service` - Service identifier (e.g., "plurcast.nostr")
    /// * `key` - Credential key (e.g., "private_key")
    /// * `value` - Credential value to store
    ///
    /// # Errors
    ///
    /// Returns an error if the credential cannot be stored due to:
    /// - Permission issues
    /// - Storage backend unavailable
    /// - Invalid service or key format
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use libplurcast::credentials::CredentialStore;
    /// # fn example(store: &dyn CredentialStore) -> libplurcast::error::Result<()> {
    /// store.store("plurcast.nostr", "private_key", "nsec1...")?;
    /// # Ok(())
    /// # }
    /// ```
    fn store(&self, service: &str, key: &str, value: &str) -> Result<()>;

    /// Retrieve a credential
    ///
    /// # Arguments
    ///
    /// * `service` - Service identifier (e.g., "plurcast.nostr")
    /// * `key` - Credential key (e.g., "private_key")
    ///
    /// # Returns
    ///
    /// The credential value as a String
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Credential not found (`CredentialError::NotFound`)
    /// - Storage backend unavailable
    /// - Decryption failed (for encrypted storage)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use libplurcast::credentials::CredentialStore;
    /// # fn example(store: &dyn CredentialStore) -> libplurcast::error::Result<()> {
    /// let key = store.retrieve("plurcast.nostr", "private_key")?;
    /// println!("Retrieved key: {}", key);
    /// # Ok(())
    /// # }
    /// ```
    fn retrieve(&self, service: &str, key: &str) -> Result<String>;

    /// Delete a credential
    ///
    /// # Arguments
    ///
    /// * `service` - Service identifier (e.g., "plurcast.nostr")
    /// * `key` - Credential key (e.g., "private_key")
    ///
    /// # Errors
    ///
    /// Returns an error if the credential cannot be deleted due to:
    /// - Permission issues
    /// - Storage backend unavailable
    ///
    /// Note: It is not an error to delete a non-existent credential.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use libplurcast::credentials::CredentialStore;
    /// # fn example(store: &dyn CredentialStore) -> libplurcast::error::Result<()> {
    /// store.delete("plurcast.nostr", "private_key")?;
    /// # Ok(())
    /// # }
    /// ```
    fn delete(&self, service: &str, key: &str) -> Result<()>;

    /// Check if a credential exists
    ///
    /// # Arguments
    ///
    /// * `service` - Service identifier (e.g., "plurcast.nostr")
    /// * `key` - Credential key (e.g., "private_key")
    ///
    /// # Returns
    ///
    /// `true` if the credential exists, `false` otherwise
    ///
    /// # Errors
    ///
    /// Returns an error if the storage backend is unavailable or
    /// cannot be queried.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use libplurcast::credentials::CredentialStore;
    /// # fn example(store: &dyn CredentialStore) -> libplurcast::error::Result<()> {
    /// if store.exists("plurcast.nostr", "private_key")? {
    ///     println!("Nostr credentials are configured");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    fn exists(&self, service: &str, key: &str) -> Result<bool>;

    /// Get the name of this storage backend
    ///
    /// Used for logging and debugging to identify which backend
    /// is being used for credential operations.
    ///
    /// # Returns
    ///
    /// A string identifier for this backend (e.g., "keyring", "encrypted_file", "plain_file")
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use libplurcast::credentials::CredentialStore;
    /// # fn example(store: &dyn CredentialStore) {
    /// println!("Using {} backend", store.backend_name());
    /// # }
    /// ```
    fn backend_name(&self) -> &str;
}


use crate::error::CredentialError;

/// OS-native keyring storage backend
///
/// This implementation uses the operating system's secure credential storage:
/// - **macOS**: Keychain via Security framework
/// - **Windows**: Credential Manager via Windows API
/// - **Linux**: Secret Service (GNOME Keyring/KWallet) via D-Bus
///
/// # Service and Key Format
///
/// - Service: `plurcast.{platform}` (e.g., "plurcast.nostr")
/// - Key: `{credential_type}` (e.g., "private_key", "access_token")
///
/// # Availability
///
/// The keyring may not be available in all environments:
/// - Headless Linux systems without Secret Service
/// - Containers without D-Bus access
/// - Systems with disabled keyring services
///
/// When unavailable, returns `CredentialError::KeyringUnavailable`.
///
/// # Example
///
/// ```no_run
/// use libplurcast::credentials::{CredentialStore, KeyringStore};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let store = KeyringStore::new()?;
///
/// // Store a credential
/// store.store("plurcast.nostr", "private_key", "nsec1...")?;
///
/// // Retrieve it
/// let key = store.retrieve("plurcast.nostr", "private_key")?;
///
/// // Check existence
/// if store.exists("plurcast.mastodon", "access_token")? {
///     println!("Mastodon token found");
/// }
///
/// // Delete
/// store.delete("plurcast.bluesky", "app_password")?;
/// # Ok(())
/// # }
/// ```
pub struct KeyringStore;

impl KeyringStore {
    /// Create a new KeyringStore
    ///
    /// # Errors
    ///
    /// Returns `CredentialError::KeyringUnavailable` if the OS keyring
    /// cannot be accessed (e.g., headless Linux without Secret Service).
    pub fn new() -> Result<Self> {
        // Test if keyring is available by attempting to create an entry
        let test_entry = keyring::Entry::new("plurcast.test", "availability_check");
        
        match test_entry {
            Ok(_) => Ok(Self),
            Err(e) => Err(CredentialError::KeyringUnavailable(format!(
                "OS keyring not accessible: {}",
                e
            )).into()),
        }
    }
}

impl CredentialStore for KeyringStore {
    fn store(&self, service: &str, key: &str, value: &str) -> Result<()> {
        let entry = keyring::Entry::new(service, key)
            .map_err(|e| CredentialError::KeyringUnavailable(e.to_string()))?;
        
        entry.set_password(value)
            .map_err(|e| CredentialError::Keyring(e.to_string()))?;
        
        tracing::debug!("Stored credential for {}.{} in OS keyring", service, key);
        Ok(())
    }

    fn retrieve(&self, service: &str, key: &str) -> Result<String> {
        let entry = keyring::Entry::new(service, key)
            .map_err(|e| CredentialError::KeyringUnavailable(e.to_string()))?;
        
        match entry.get_password() {
            Ok(password) => {
                tracing::debug!("Retrieved credential for {}.{} from OS keyring", service, key);
                Ok(password)
            }
            Err(keyring::Error::NoEntry) => {
                Err(CredentialError::NotFound(format!("{}.{}", service, key)).into())
            }
            Err(e) => Err(CredentialError::Keyring(e.to_string()).into()),
        }
    }

    fn delete(&self, service: &str, key: &str) -> Result<()> {
        let entry = keyring::Entry::new(service, key)
            .map_err(|e| CredentialError::KeyringUnavailable(e.to_string()))?;
        
        // Attempt to delete - it's not an error if the entry doesn't exist
        match entry.delete_password() {
            Ok(_) => {
                tracing::debug!("Deleted credential for {}.{} from OS keyring", service, key);
                Ok(())
            }
            Err(keyring::Error::NoEntry) => {
                // Not an error to delete non-existent credential
                tracing::debug!("Credential {}.{} not found (already deleted)", service, key);
                Ok(())
            }
            Err(e) => Err(CredentialError::Keyring(e.to_string()).into()),
        }
    }

    fn exists(&self, service: &str, key: &str) -> Result<bool> {
        let entry = keyring::Entry::new(service, key)
            .map_err(|e| CredentialError::KeyringUnavailable(e.to_string()))?;
        
        match entry.get_password() {
            Ok(_) => Ok(true),
            Err(keyring::Error::NoEntry) => Ok(false),
            Err(e) => Err(CredentialError::Keyring(e.to_string()).into()),
        }
    }

    fn backend_name(&self) -> &str {
        "keyring"
    }
}


use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::io::{Read, Write};

/// Encrypted file storage backend
///
/// This implementation stores credentials in encrypted files using the `age` encryption format.
/// Files are encrypted with a master password and stored in `~/.config/plurcast/credentials/`.
///
/// # File Format
///
/// - Location: `~/.config/plurcast/credentials/`
/// - Naming: `{service}.{key}.age` (e.g., `plurcast.nostr.private_key.age`)
/// - Permissions: 600 (owner read/write only) on Unix systems
///
/// # Master Password
///
/// The master password must be set before storing or retrieving credentials.
/// It should be at least 8 characters long (12+ recommended for security).
///
/// # Example
///
/// ```no_run
/// use libplurcast::credentials::{CredentialStore, EncryptedFileStore};
/// use std::path::PathBuf;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let store = EncryptedFileStore::new(
///     PathBuf::from("~/.config/plurcast/credentials")
/// );
///
/// // Set master password
/// store.set_master_password("my-secure-password-123".to_string())?;
///
/// // Store a credential
/// store.store("plurcast.nostr", "private_key", "nsec1...")?;
///
/// // Retrieve it
/// let key = store.retrieve("plurcast.nostr", "private_key")?;
/// # Ok(())
/// # }
/// ```
pub struct EncryptedFileStore {
    base_path: PathBuf,
    master_password: Arc<RwLock<Option<String>>>,
}

impl EncryptedFileStore {
    /// Create a new EncryptedFileStore
    ///
    /// # Arguments
    ///
    /// * `base_path` - Directory where encrypted credential files will be stored
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            base_path,
            master_password: Arc::new(RwLock::new(None)),
        }
    }

    /// Set the master password for encryption/decryption
    ///
    /// # Arguments
    ///
    /// * `password` - Master password (minimum 8 characters)
    ///
    /// # Errors
    ///
    /// Returns `CredentialError::WeakPassword` if the password is less than 8 characters.
    pub fn set_master_password(&self, password: String) -> Result<()> {
        if password.len() < 8 {
            return Err(CredentialError::WeakPassword.into());
        }
        
        *self.master_password.write().unwrap() = Some(password);
        tracing::debug!("Master password set for encrypted file store");
        Ok(())
    }

    /// Encrypt data using the master password
    fn encrypt(&self, data: &str) -> Result<Vec<u8>> {
        let password = self.master_password.read().unwrap();
        let password = password.as_ref()
            .ok_or_else(|| CredentialError::MasterPasswordNotSet)?;
        
        let encryptor = age::Encryptor::with_user_passphrase(
            age::secrecy::Secret::new(password.clone())
        );
        
        let mut encrypted = vec![];
        let mut writer = encryptor.wrap_output(&mut encrypted)
            .map_err(|e| CredentialError::Encryption(e.to_string()))?;
        
        writer.write_all(data.as_bytes())
            .map_err(|e| CredentialError::Encryption(e.to_string()))?;
        
        writer.finish()
            .map_err(|e| CredentialError::Encryption(e.to_string()))?;
        
        Ok(encrypted)
    }

    /// Decrypt data using the master password
    fn decrypt(&self, data: &[u8]) -> Result<String> {
        let password = self.master_password.read().unwrap();
        let password = password.as_ref()
            .ok_or_else(|| CredentialError::MasterPasswordNotSet)?;
        
        let decryptor = match age::Decryptor::new(data) {
            Ok(age::Decryptor::Passphrase(d)) => d,
            Ok(_) => return Err(CredentialError::Encryption(
                "Invalid encryption format (expected passphrase)".to_string()
            ).into()),
            Err(e) => return Err(CredentialError::Encryption(e.to_string()).into()),
        };
        
        let mut decrypted = vec![];
        let mut reader = decryptor.decrypt(
            &age::secrecy::Secret::new(password.clone()),
            None
        ).map_err(|e| {
            // Check if it's a decryption failure (wrong password)
            if e.to_string().contains("decryption") || e.to_string().contains("MAC") {
                CredentialError::DecryptionFailed
            } else {
                CredentialError::Encryption(e.to_string())
            }
        })?;
        
        reader.read_to_end(&mut decrypted)
            .map_err(|e| CredentialError::Encryption(e.to_string()))?;
        
        Ok(String::from_utf8(decrypted)
            .map_err(|e| CredentialError::Encryption(format!("Invalid UTF-8: {}", e)))?)
    }

    /// Get the file path for a credential
    fn get_file_path(&self, service: &str, key: &str) -> PathBuf {
        self.base_path.join(format!("{}.{}.age", service, key))
    }
}

impl CredentialStore for EncryptedFileStore {
    fn store(&self, service: &str, key: &str, value: &str) -> Result<()> {
        let encrypted = self.encrypt(value)?;
        let file_path = self.get_file_path(service, key);
        
        // Create parent directories
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| CredentialError::Io(e))?;
        }
        
        std::fs::write(&file_path, encrypted)
            .map_err(|e| CredentialError::Io(e))?;
        
        // Set file permissions to 600 on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(&file_path, perms)
                .map_err(|e| CredentialError::Io(e))?;
        }
        
        tracing::debug!("Stored encrypted credential for {}.{} at {:?}", service, key, file_path);
        Ok(())
    }

    fn retrieve(&self, service: &str, key: &str) -> Result<String> {
        let file_path = self.get_file_path(service, key);
        
        if !file_path.exists() {
            return Err(CredentialError::NotFound(format!("{}.{}", service, key)).into());
        }
        
        let encrypted = std::fs::read(&file_path)
            .map_err(|e| CredentialError::Io(e))?;
        let decrypted = self.decrypt(&encrypted)?;
        
        tracing::debug!("Retrieved encrypted credential for {}.{} from {:?}", service, key, file_path);
        Ok(decrypted)
    }

    fn delete(&self, service: &str, key: &str) -> Result<()> {
        let file_path = self.get_file_path(service, key);
        
        if file_path.exists() {
            std::fs::remove_file(&file_path)
                .map_err(|e| CredentialError::Io(e))?;
            tracing::debug!("Deleted encrypted credential for {}.{} at {:?}", service, key, file_path);
        } else {
            tracing::debug!("Credential {}.{} not found (already deleted)", service, key);
        }
        
        Ok(())
    }

    fn exists(&self, service: &str, key: &str) -> Result<bool> {
        let file_path = self.get_file_path(service, key);
        Ok(file_path.exists())
    }

    fn backend_name(&self) -> &str {
        "encrypted_file"
    }
}


use std::collections::HashSet;
use std::sync::Mutex;

/// Plain text file storage backend (legacy/backward compatibility)
///
/// **⚠️ DEPRECATED: This storage backend is insecure and should only be used
/// for backward compatibility with existing plain text credential files.**
///
/// This implementation stores credentials in plain text files with only Unix
/// file permissions (600) for protection. It maintains compatibility with
/// existing Plurcast credential files.
///
/// # File Mapping
///
/// Legacy file paths are mapped as follows:
/// - `plurcast.nostr/private_key` → `nostr.keys`
/// - `plurcast.mastodon/access_token` → `mastodon.token`
/// - `plurcast.bluesky/app_password` → `bluesky.auth`
///
/// # Security Warnings
///
/// - Credentials are stored in plain text
/// - Only protected by file permissions (600)
/// - Vulnerable if file system is compromised
/// - Not recommended for new installations
///
/// Use `plur-creds migrate` to upgrade to secure storage.
///
/// # Example
///
/// ```no_run
/// use libplurcast::credentials::{CredentialStore, PlainFileStore};
/// use std::path::PathBuf;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let store = PlainFileStore::new(
///     PathBuf::from("~/.config/plurcast")
/// );
///
/// // This will log a deprecation warning
/// store.store("plurcast.nostr", "private_key", "nsec1...")?;
///
/// let key = store.retrieve("plurcast.nostr", "private_key")?;
/// # Ok(())
/// # }
/// ```
#[deprecated(
    since = "0.2.0",
    note = "Plain text credential storage is insecure. Use KeyringStore or EncryptedFileStore instead."
)]
pub struct PlainFileStore {
    base_path: PathBuf,
    warned: Arc<Mutex<HashSet<String>>>,
}

#[allow(deprecated)]
impl PlainFileStore {
    /// Create a new PlainFileStore
    ///
    /// # Arguments
    ///
    /// * `base_path` - Directory where plain text credential files are stored
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            base_path,
            warned: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Get the legacy file path for a credential
    ///
    /// Maps service/key pairs to legacy file paths for backward compatibility.
    fn get_legacy_path(&self, service: &str, key: &str) -> PathBuf {
        match (service, key) {
            ("plurcast.nostr", "private_key") => {
                self.base_path.join("nostr.keys")
            }
            ("plurcast.mastodon", "access_token") => {
                self.base_path.join("mastodon.token")
            }
            ("plurcast.bluesky", "app_password") => {
                self.base_path.join("bluesky.auth")
            }
            _ => {
                // For unknown combinations, use a generic format
                self.base_path.join(format!("{}.{}", service, key))
            }
        }
    }

    /// Log a deprecation warning once per credential
    fn warn_once(&self, service: &str, key: &str) {
        let service_key = format!("{}.{}", service, key);
        let mut warned = self.warned.lock().unwrap();
        
        if !warned.contains(&service_key) {
            tracing::warn!(
                "Reading credentials from plain text file for {}.{}. \
                 Consider migrating to secure storage with 'plur-creds migrate'.",
                service, key
            );
            warned.insert(service_key);
        }
    }
}

#[allow(deprecated)]
impl CredentialStore for PlainFileStore {
    fn store(&self, service: &str, key: &str, value: &str) -> Result<()> {
        // Log deprecation warning
        tracing::warn!(
            "Storing credentials in plain text is deprecated and insecure. \
             Use 'plur-creds migrate' to upgrade to secure storage."
        );
        
        let file_path = self.get_legacy_path(service, key);
        
        // Create parent directories
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| CredentialError::Io(e))?;
        }
        
        std::fs::write(&file_path, value)
            .map_err(|e| CredentialError::Io(e))?;
        
        // Set file permissions to 600 on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(&file_path, perms)
                .map_err(|e| CredentialError::Io(e))?;
        }
        
        tracing::debug!("Stored plain text credential for {}.{} at {:?}", service, key, file_path);
        Ok(())
    }

    fn retrieve(&self, service: &str, key: &str) -> Result<String> {
        // Warn once per service
        self.warn_once(service, key);
        
        let file_path = self.get_legacy_path(service, key);
        
        if !file_path.exists() {
            return Err(CredentialError::NotFound(format!("{}.{}", service, key)).into());
        }
        
        let content = std::fs::read_to_string(&file_path)
            .map_err(|e| CredentialError::Io(e))?;
        
        tracing::debug!("Retrieved plain text credential for {}.{} from {:?}", service, key, file_path);
        Ok(content.trim().to_string())
    }

    fn delete(&self, service: &str, key: &str) -> Result<()> {
        let file_path = self.get_legacy_path(service, key);
        
        if file_path.exists() {
            std::fs::remove_file(&file_path)
                .map_err(|e| CredentialError::Io(e))?;
            tracing::debug!("Deleted plain text credential for {}.{} at {:?}", service, key, file_path);
        } else {
            tracing::debug!("Credential {}.{} not found (already deleted)", service, key);
        }
        
        Ok(())
    }

    fn exists(&self, service: &str, key: &str) -> Result<bool> {
        let file_path = self.get_legacy_path(service, key);
        Ok(file_path.exists())
    }

    fn backend_name(&self) -> &str {
        "plain_file"
    }
}


use serde::{Deserialize, Serialize};

/// Storage backend type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum StorageBackend {
    /// OS-native keyring (macOS Keychain, Windows Credential Manager, Linux Secret Service)
    Keyring,
    /// Encrypted files with master password
    Encrypted,
    /// Plain text files (deprecated, insecure)
    Plain,
}

impl Default for StorageBackend {
    fn default() -> Self {
        StorageBackend::Keyring
    }
}

/// Credential storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialConfig {
    /// Storage backend to use
    #[serde(default)]
    pub storage: StorageBackend,
    
    /// Path for encrypted/plain file storage (keyring doesn't use files)
    #[serde(default = "default_credential_path")]
    pub path: String,
    
    /// Master password for encrypted storage (not serialized)
    #[serde(skip)]
    pub master_password: Option<String>,
}

fn default_credential_path() -> String {
    "~/.config/plurcast/credentials".to_string()
}

impl Default for CredentialConfig {
    fn default() -> Self {
        Self {
            storage: StorageBackend::Keyring,
            path: default_credential_path(),
            master_password: None,
        }
    }
}

impl CredentialConfig {
    /// Load master password from environment variable if available
    ///
    /// Checks for PLURCAST_MASTER_PASSWORD environment variable and sets
    /// the master_password field if found.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use libplurcast::credentials::CredentialConfig;
    ///
    /// let mut config = CredentialConfig::default();
    /// config.load_master_password_from_env();
    /// ```
    pub fn load_master_password_from_env(&mut self) {
        if let Ok(password) = std::env::var("PLURCAST_MASTER_PASSWORD") {
            if !password.is_empty() {
                self.master_password = Some(password);
                tracing::debug!("Loaded master password from PLURCAST_MASTER_PASSWORD environment variable");
            }
        }
    }

    /// Validate the credential configuration
    ///
    /// Checks that:
    /// - Storage backend is valid
    /// - Path is not empty
    /// - Path can be expanded
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails.
    pub fn validate(&self) -> Result<()> {
        // Validate path is not empty
        if self.path.is_empty() {
            return Err(CredentialError::Encryption(
                "Credential path cannot be empty".to_string()
            ).into());
        }

        // Validate path can be expanded
        let _ = shellexpand::tilde(&self.path);

        Ok(())
    }

    /// Expand shell variables in the credential path
    ///
    /// Expands ~ and environment variables in the path.
    ///
    /// # Returns
    ///
    /// The expanded path as a PathBuf
    pub fn expand_path(&self) -> PathBuf {
        let expanded = shellexpand::tilde(&self.path).to_string();
        PathBuf::from(expanded)
    }
}

/// Report of credential migration results
///
/// Contains information about which credentials were successfully migrated,
/// which failed, and which were skipped.
#[derive(Debug, Clone)]
pub struct MigrationReport {
    /// Successfully migrated credentials (service.key)
    pub migrated: Vec<String>,
    
    /// Failed migrations (service.key, error message)
    pub failed: Vec<(String, String)>,
    
    /// Skipped credentials (already in secure storage)
    pub skipped: Vec<String>,
}

impl MigrationReport {
    /// Create a new empty migration report
    pub fn new() -> Self {
        Self {
            migrated: Vec::new(),
            failed: Vec::new(),
            skipped: Vec::new(),
        }
    }

    /// Check if the migration was completely successful
    pub fn is_success(&self) -> bool {
        self.failed.is_empty()
    }

    /// Get the total number of credentials processed
    pub fn total(&self) -> usize {
        self.migrated.len() + self.failed.len() + self.skipped.len()
    }
}

impl Default for MigrationReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Credential manager facade
///
/// This is the main entry point for credential storage operations. It manages
/// multiple storage backends with automatic fallback logic:
///
/// 1. Try KeyringStore (if configured and available)
/// 2. Try EncryptedFileStore (if master password set or can prompt)
/// 3. Fall back to PlainFileStore (with warnings)
///
/// # Example
///
/// ```no_run
/// use libplurcast::credentials::{CredentialManager, CredentialConfig, StorageBackend};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = CredentialConfig {
///     storage: StorageBackend::Keyring,
///     path: "~/.config/plurcast/credentials".to_string(),
///     master_password: None,
/// };
///
/// let manager = CredentialManager::new(config)?;
///
/// // Store a credential (uses first available backend)
/// manager.store("plurcast.nostr", "private_key", "nsec1...")?;
///
/// // Retrieve a credential (tries all backends in order)
/// let key = manager.retrieve("plurcast.nostr", "private_key")?;
///
/// // Delete from all backends
/// manager.delete("plurcast.mastodon", "access_token")?;
///
/// // Check if credential exists in any backend
/// if manager.exists("plurcast.bluesky", "app_password")? {
///     println!("Bluesky credentials found");
/// }
/// # Ok(())
/// # }
/// ```
pub struct CredentialManager {
    stores: Vec<Box<dyn CredentialStore>>,
    config: CredentialConfig,
}

impl CredentialManager {
    /// Create a new CredentialManager
    ///
    /// Builds a priority list of storage backends based on configuration:
    /// 1. KeyringStore (if configured and available)
    /// 2. EncryptedFileStore (if master password available)
    /// 3. PlainFileStore (always available as fallback)
    ///
    /// # Arguments
    ///
    /// * `config` - Credential storage configuration
    ///
    /// # Errors
    ///
    /// Returns an error if no storage backend is available.
    pub fn new(config: CredentialConfig) -> Result<Self> {
        let mut stores: Vec<Box<dyn CredentialStore>> = vec![];
        
        // Expand path with shell variables
        let expanded_path = shellexpand::tilde(&config.path).to_string();
        let credential_path = PathBuf::from(expanded_path);
        
        // Try keyring first (if configured)
        if config.storage == StorageBackend::Keyring {
            match KeyringStore::new() {
                Ok(store) => {
                    tracing::info!("Using OS keyring for credential storage");
                    stores.push(Box::new(store));
                }
                Err(e) => {
                    tracing::warn!(
                        "OS keyring unavailable: {}. Falling back to encrypted files.",
                        e
                    );
                }
            }
        }
        
        // Add encrypted file store (if configured or as fallback)
        if config.storage == StorageBackend::Encrypted 
            || (config.storage == StorageBackend::Keyring && stores.is_empty()) {
            let encrypted_store = EncryptedFileStore::new(credential_path.clone());
            
            // Set master password if provided
            if let Some(password) = &config.master_password {
                encrypted_store.set_master_password(password.clone())?;
                tracing::info!("Using encrypted file storage for credentials");
                stores.push(Box::new(encrypted_store));
            } else {
                // Try to prompt for password if TTY available
                if atty::is(atty::Stream::Stdin) {
                    match rpassword::prompt_password("Enter master password for credential encryption: ") {
                        Ok(password) if !password.is_empty() => {
                            match encrypted_store.set_master_password(password) {
                                Ok(_) => {
                                    tracing::info!("Using encrypted file storage for credentials");
                                    stores.push(Box::new(encrypted_store));
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        "Failed to set master password: {}. Falling back to plain text storage.",
                                        e
                                    );
                                }
                            }
                        }
                        Ok(_) => {
                            tracing::warn!(
                                "Empty master password provided. Falling back to plain text storage."
                            );
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Failed to prompt for master password: {}. Falling back to plain text storage.",
                                e
                            );
                        }
                    }
                } else {
                    tracing::warn!(
                        "Master password not set and no TTY available. Falling back to plain text storage."
                    );
                }
            }
        }
        
        // Add plain file store as final fallback
        #[allow(deprecated)]
        let plain_store = PlainFileStore::new(credential_path);
        stores.push(Box::new(plain_store));
        
        if stores.is_empty() {
            return Err(CredentialError::NoStoreAvailable.into());
        }
        
        // Warn user if only plain text storage is available
        if stores.len() == 1 && stores[0].backend_name() == "plain_file" {
            tracing::warn!(
                "⚠️  WARNING: Credentials will be stored in PLAIN TEXT files!\n\
                 This is INSECURE and only provided for backward compatibility.\n\
                 \n\
                 To use secure storage:\n\
                 1. OS Keyring (recommended): Set storage = \"keyring\" in config.toml\n\
                 2. Encrypted files: Set storage = \"encrypted\" and provide a master password\n\
                 \n\
                 To migrate existing credentials: Run 'plur-creds migrate' (coming soon)\n\
                 To remove plain text files: Delete files in ~/.config/plurcast/"
            );
        }
        
        Ok(Self { stores, config })
    }

    /// Store a credential using the first available backend
    ///
    /// # Arguments
    ///
    /// * `service` - Service identifier (e.g., "plurcast.nostr")
    /// * `key` - Credential key (e.g., "private_key")
    /// * `value` - Credential value to store
    ///
    /// # Errors
    ///
    /// Returns an error if the credential cannot be stored in any backend.
    pub fn store(&self, service: &str, key: &str, value: &str) -> Result<()> {
        if let Some(store) = self.stores.first() {
            // Warn if storing to plain text
            if store.backend_name() == "plain_file" {
                tracing::warn!(
                    "⚠️  Storing credential for {}.{} in PLAIN TEXT. \
                     This is insecure! Use 'plur-creds migrate' to upgrade to secure storage.",
                    service, key
                );
            }
            
            store.store(service, key, value)?;
            tracing::debug!(
                "Stored credential for {}.{} using {} backend",
                service, key, store.backend_name()
            );
            Ok(())
        } else {
            Err(CredentialError::NoStoreAvailable.into())
        }
    }

    /// Retrieve a credential, trying all backends in order
    ///
    /// # Arguments
    ///
    /// * `service` - Service identifier (e.g., "plurcast.nostr")
    /// * `key` - Credential key (e.g., "private_key")
    ///
    /// # Returns
    ///
    /// The credential value as a String
    ///
    /// # Errors
    ///
    /// Returns `CredentialError::NotFound` if the credential is not found in any backend.
    pub fn retrieve(&self, service: &str, key: &str) -> Result<String> {
        let mut last_error: Option<crate::error::PlurcastError> = None;
        
        for store in &self.stores {
            match store.retrieve(service, key) {
                Ok(value) => {
                    tracing::debug!(
                        "Retrieved credential for {}.{} from {} backend",
                        service, key, store.backend_name()
                    );
                    return Ok(value);
                }
                Err(e) => {
                    // Check if it's a NotFound error
                    if let crate::error::PlurcastError::Credential(crate::error::CredentialError::NotFound(_)) = &e {
                        // Try next store
                        last_error = Some(e);
                        continue;
                    } else {
                        // Other errors are fatal
                        return Err(e);
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| CredentialError::NotFound(format!("{}.{}", service, key)).into()))
    }

    /// Delete a credential from all backends
    ///
    /// # Arguments
    ///
    /// * `service` - Service identifier (e.g., "plurcast.nostr")
    /// * `key` - Credential key (e.g., "private_key")
    ///
    /// # Errors
    ///
    /// Returns an error if deletion fails in any backend.
    pub fn delete(&self, service: &str, key: &str) -> Result<()> {
        for store in &self.stores {
            store.delete(service, key)?;
        }
        
        tracing::debug!("Deleted credential for {}.{} from all backends", service, key);
        Ok(())
    }

    /// Check if a credential exists in any backend
    ///
    /// # Arguments
    ///
    /// * `service` - Service identifier (e.g., "plurcast.nostr")
    /// * `key` - Credential key (e.g., "private_key")
    ///
    /// # Returns
    ///
    /// `true` if the credential exists in any backend, `false` otherwise
    pub fn exists(&self, service: &str, key: &str) -> Result<bool> {
        for store in &self.stores {
            if store.exists(service, key)? {
                return Ok(true);
            }
        }
        
        Ok(false)
    }

    /// Get the configuration
    pub fn config(&self) -> &CredentialConfig {
        &self.config
    }

    /// Get the list of available backends
    pub fn backends(&self) -> Vec<&str> {
        self.stores.iter().map(|s| s.backend_name()).collect()
    }
    
    /// Check if the primary storage backend is insecure (plain text)
    ///
    /// Returns `true` if credentials are being stored in plain text files,
    /// which is insecure and should be avoided.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use libplurcast::credentials::{CredentialManager, CredentialConfig};
    /// # fn example(manager: &CredentialManager) {
    /// if manager.is_insecure() {
    ///     eprintln!("WARNING: Using insecure plain text credential storage!");
    ///     eprintln!("Run 'plur-creds migrate' to upgrade to secure storage.");
    /// }
    /// # }
    /// ```
    pub fn is_insecure(&self) -> bool {
        self.stores.first()
            .map(|s| s.backend_name() == "plain_file")
            .unwrap_or(false)
    }
    
    /// Get the name of the primary storage backend being used
    ///
    /// Returns the backend name (e.g., "keyring", "encrypted_file", "plain_file")
    /// of the first (primary) storage backend.
    pub fn primary_backend(&self) -> Option<&str> {
        self.stores.first().map(|s| s.backend_name())
    }

    /// Detect plain text credential files that can be migrated
    ///
    /// Scans the configuration directory for plain text credential files
    /// and returns a list of credentials that were found.
    ///
    /// # Returns
    ///
    /// A vector of tuples containing (service, key, file_path) for each
    /// plain text credential file found.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use libplurcast::credentials::{CredentialManager, CredentialConfig};
    /// # fn example(manager: &CredentialManager) -> libplurcast::error::Result<()> {
    /// let plain_creds = manager.detect_plain_credentials()?;
    /// 
    /// if !plain_creds.is_empty() {
    ///     println!("Found {} plain text credential files:", plain_creds.len());
    ///     for (service, key, path) in &plain_creds {
    ///         println!("  - {}.{} at {}", service, key, path.display());
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn detect_plain_credentials(&self) -> Result<Vec<(String, String, PathBuf)>> {
        let mut found_credentials = Vec::new();
        
        // Get the config directory path
        let config_dir = dirs::config_dir()
            .ok_or_else(|| CredentialError::Encryption("Config directory not found".to_string()))?
            .join("plurcast");
        
        // Check for known plain text credential files
        let known_files = vec![
            ("plurcast.nostr", "private_key", "nostr.keys"),
            ("plurcast.mastodon", "access_token", "mastodon.token"),
            ("plurcast.bluesky", "app_password", "bluesky.auth"),
        ];
        
        for (service, key, filename) in known_files {
            let file_path = config_dir.join(filename);
            if file_path.exists() {
                tracing::debug!("Found plain text credential file: {}", file_path.display());
                found_credentials.push((service.to_string(), key.to_string(), file_path));
            }
        }
        
        Ok(found_credentials)
    }

    /// Migrate credentials from plain text files to secure storage
    ///
    /// This method:
    /// 1. Detects all plain text credential files
    /// 2. Reads each credential from the plain text file
    /// 3. Stores it in the secure storage backend (first available store)
    /// 4. Verifies the credential can be retrieved
    /// 5. Returns a report of what was migrated, failed, or skipped
    ///
    /// # Returns
    ///
    /// A `MigrationReport` containing details about the migration results.
    ///
    /// # Errors
    ///
    /// Returns an error if the primary storage backend is unavailable.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use libplurcast::credentials::{CredentialManager, CredentialConfig};
    /// # fn example(manager: &CredentialManager) -> libplurcast::error::Result<()> {
    /// let report = manager.migrate_from_plain()?;
    /// 
    /// println!("Migration complete:");
    /// println!("  Migrated: {}", report.migrated.len());
    /// println!("  Failed: {}", report.failed.len());
    /// println!("  Skipped: {}", report.skipped.len());
    /// 
    /// if !report.is_success() {
    ///     eprintln!("Some migrations failed:");
    ///     for (cred, error) in &report.failed {
    ///         eprintln!("  - {}: {}", cred, error);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn migrate_from_plain(&self) -> Result<MigrationReport> {
        let mut report = MigrationReport::new();
        
        // Ensure we have a non-plain-text primary store
        let primary_store = self.stores.first()
            .ok_or_else(|| CredentialError::NoStoreAvailable)?;
        
        if primary_store.backend_name() == "plain_file" {
            return Err(CredentialError::MigrationFailed(
                "Cannot migrate to plain text storage. Configure keyring or encrypted storage first.".to_string()
            ).into());
        }
        
        tracing::info!("Starting credential migration to {} backend", primary_store.backend_name());
        
        // Detect plain text credentials
        let plain_credentials = self.detect_plain_credentials()?;
        
        if plain_credentials.is_empty() {
            tracing::info!("No plain text credentials found to migrate");
            return Ok(report);
        }
        
        tracing::info!("Found {} plain text credential files to migrate", plain_credentials.len());
        
        // Migrate each credential
        for (service, key, file_path) in plain_credentials {
            let credential_name = format!("{}.{}", service, key);
            
            // Check if already in secure storage
            if let Ok(true) = primary_store.exists(&service, &key) {
                tracing::debug!("Credential {} already exists in secure storage, skipping", credential_name);
                report.skipped.push(credential_name);
                continue;
            }
            
            // Read from plain text file
            let value = match std::fs::read_to_string(&file_path) {
                Ok(content) => content.trim().to_string(),
                Err(e) => {
                    let error_msg = format!("Failed to read file: {}", e);
                    tracing::error!("Migration failed for {}: {}", credential_name, error_msg);
                    report.failed.push((credential_name, error_msg));
                    continue;
                }
            };
            
            // Store in secure storage
            if let Err(e) = primary_store.store(&service, &key, &value) {
                let error_msg = format!("Failed to store in secure storage: {}", e);
                tracing::error!("Migration failed for {}: {}", credential_name, error_msg);
                report.failed.push((credential_name, error_msg));
                continue;
            }
            
            // Verify by retrieving
            match primary_store.retrieve(&service, &key) {
                Ok(retrieved) if retrieved == value => {
                    tracing::info!("Successfully migrated credential: {}", credential_name);
                    report.migrated.push(credential_name);
                }
                Ok(_) => {
                    let error_msg = "Verification failed: retrieved value doesn't match".to_string();
                    tracing::error!("Migration failed for {}: {}", credential_name, error_msg);
                    report.failed.push((credential_name, error_msg));
                }
                Err(e) => {
                    let error_msg = format!("Verification failed: {}", e);
                    tracing::error!("Migration failed for {}: {}", credential_name, error_msg);
                    report.failed.push((credential_name, error_msg));
                }
            }
        }
        
        tracing::info!(
            "Migration complete: {} migrated, {} failed, {} skipped",
            report.migrated.len(),
            report.failed.len(),
            report.skipped.len()
        );
        
        Ok(report)
    }

    /// Delete plain text credential files after successful migration
    ///
    /// This method deletes the plain text credential files for credentials
    /// that were successfully migrated. It should only be called after
    /// verifying that the migration was successful.
    ///
    /// # Arguments
    ///
    /// * `migrated_credentials` - List of successfully migrated credentials (service.key format)
    ///
    /// # Returns
    ///
    /// A vector of file paths that were successfully deleted.
    ///
    /// # Errors
    ///
    /// Returns an error if file deletion fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use libplurcast::credentials::{CredentialManager, CredentialConfig};
    /// # fn example(manager: &CredentialManager) -> libplurcast::error::Result<()> {
    /// let report = manager.migrate_from_plain()?;
    /// 
    /// if report.is_success() && !report.migrated.is_empty() {
    ///     // Ask user for confirmation
    ///     println!("Delete plain text files? [y/N]");
    ///     // ... get user input ...
    ///     
    ///     let deleted = manager.cleanup_plain_files(&report.migrated)?;
    ///     println!("Deleted {} plain text files", deleted.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn cleanup_plain_files(&self, migrated_credentials: &[String]) -> Result<Vec<PathBuf>> {
        let mut deleted_files = Vec::new();
        
        // Get the config directory path
        let config_dir = dirs::config_dir()
            .ok_or_else(|| CredentialError::Encryption("Config directory not found".to_string()))?
            .join("plurcast");
        
        // Map credential names to file paths
        let file_mapping = vec![
            ("plurcast.nostr.private_key", "nostr.keys"),
            ("plurcast.mastodon.access_token", "mastodon.token"),
            ("plurcast.bluesky.app_password", "bluesky.auth"),
        ];
        
        for credential_name in migrated_credentials {
            // Find the corresponding file
            if let Some((_, filename)) = file_mapping.iter().find(|(name, _)| name == credential_name) {
                let file_path = config_dir.join(filename);
                
                if file_path.exists() {
                    match std::fs::remove_file(&file_path) {
                        Ok(_) => {
                            tracing::info!("Deleted plain text file: {}", file_path.display());
                            deleted_files.push(file_path);
                        }
                        Err(e) => {
                            tracing::error!("Failed to delete {}: {}", file_path.display(), e);
                            return Err(CredentialError::Io(e).into());
                        }
                    }
                }
            }
        }
        
        Ok(deleted_files)
    }
}

#[cfg(test)]
mod tests;
