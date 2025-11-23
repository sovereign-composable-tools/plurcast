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
use crate::accounts::AccountManager;

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
    /// Store a credential for a specific account
    ///
    /// # Arguments
    ///
    /// * `service` - Service identifier (e.g., "plurcast.nostr")
    /// * `key` - Credential key (e.g., "private_key")
    /// * `account` - Account name (e.g., "default", "test-account")
    /// * `value` - Credential value to store
    ///
    /// # Errors
    ///
    /// Returns an error if the credential cannot be stored due to:
    /// - Permission issues
    /// - Storage backend unavailable
    /// - Invalid service, key, or account format
    fn store_account(&self, service: &str, key: &str, account: &str, value: &str) -> Result<()>;

    /// Retrieve a credential for a specific account
    ///
    /// # Arguments
    ///
    /// * `service` - Service identifier (e.g., "plurcast.nostr")
    /// * `key` - Credential key (e.g., "private_key")
    /// * `account` - Account name (e.g., "default", "test-account")
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
    fn retrieve_account(&self, service: &str, key: &str, account: &str) -> Result<String>;

    /// Delete a credential for a specific account
    ///
    /// # Arguments
    ///
    /// * `service` - Service identifier (e.g., "plurcast.nostr")
    /// * `key` - Credential key (e.g., "private_key")
    /// * `account` - Account name (e.g., "default", "test-account")
    ///
    /// # Errors
    ///
    /// Returns an error if the credential cannot be deleted due to:
    /// - Permission issues
    /// - Storage backend unavailable
    ///
    /// Note: It is not an error to delete a non-existent credential.
    fn delete_account(&self, service: &str, key: &str, account: &str) -> Result<()>;

    /// Check if a credential exists for a specific account
    ///
    /// # Arguments
    ///
    /// * `service` - Service identifier (e.g., "plurcast.nostr")
    /// * `key` - Credential key (e.g., "private_key")
    /// * `account` - Account name (e.g., "default", "test-account")
    ///
    /// # Returns
    ///
    /// `true` if the credential exists, `false` otherwise
    ///
    /// # Errors
    ///
    /// Returns an error if the storage backend is unavailable or
    /// cannot be queried.
    fn exists_account(&self, service: &str, key: &str, account: &str) -> Result<bool>;

    /// List all accounts for a service/key combination
    ///
    /// # Arguments
    ///
    /// * `service` - Service identifier (e.g., "plurcast.nostr")
    /// * `key` - Credential key (e.g., "private_key")
    ///
    /// # Returns
    ///
    /// A vector of account names that have credentials for this service/key
    ///
    /// # Errors
    ///
    /// Returns an error if the storage backend is unavailable or
    /// cannot be queried.
    fn list_accounts(&self, service: &str, key: &str) -> Result<Vec<String>>;

    /// Store a credential (delegates to "default" account for backward compatibility)
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
    fn store(&self, service: &str, key: &str, value: &str) -> Result<()> {
        self.store_account(service, key, "default", value)
    }

    /// Retrieve a credential (delegates to "default" account for backward compatibility)
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
    fn retrieve(&self, service: &str, key: &str) -> Result<String> {
        self.retrieve_account(service, key, "default")
    }

    /// Delete a credential (delegates to "default" account for backward compatibility)
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
    fn delete(&self, service: &str, key: &str) -> Result<()> {
        self.delete_account(service, key, "default")
    }

    /// Check if a credential exists (delegates to "default" account for backward compatibility)
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
    fn exists(&self, service: &str, key: &str) -> Result<bool> {
        self.exists_account(service, key, "default")
    }

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
            ))
            .into()),
        }
    }

    /// Build keyring namespace for multi-account support
    ///
    /// Converts service/account/key into keyring service and key:
    /// - Input: service="plurcast.nostr", account="test-account", key="private_key"
    /// - Output: (service="plurcast.nostr.test-account", key="private_key")
    ///
    /// # Arguments
    ///
    /// * `service` - Service identifier (e.g., "plurcast.nostr")
    /// * `account` - Account name (e.g., "test-account")
    /// * `key` - Credential key (e.g., "private_key")
    ///
    /// # Returns
    ///
    /// A tuple of (keyring_service, keyring_key)
    fn keyring_key(service: &str, account: &str, key: &str) -> (String, String) {
        // Extract platform from service (remove "plurcast." prefix if present)
        let platform = service.strip_prefix("plurcast.").unwrap_or(service);

        // Build new service with account: plurcast.{platform}.{account}
        let keyring_service = format!("plurcast.{}.{}", platform, account);

        // Key stays the same
        (keyring_service, key.to_string())
    }
}

impl CredentialStore for KeyringStore {
    fn store_account(&self, service: &str, key: &str, account: &str, value: &str) -> Result<()> {
        let (keyring_service, keyring_key) = Self::keyring_key(service, account, key);

        let entry = keyring::Entry::new(&keyring_service, &keyring_key)
            .map_err(|e| CredentialError::KeyringUnavailable(e.to_string()))?;

        entry
            .set_password(value)
            .map_err(|e| CredentialError::Keyring(e.to_string()))?;

        tracing::debug!(
            "Stored credential for {}.{}.{} in OS keyring",
            service,
            account,
            key
        );
        Ok(())
    }

    fn retrieve_account(&self, service: &str, key: &str, account: &str) -> Result<String> {
        let (keyring_service, keyring_key) = Self::keyring_key(service, account, key);

        let entry = keyring::Entry::new(&keyring_service, &keyring_key)
            .map_err(|e| CredentialError::KeyringUnavailable(e.to_string()))?;

        match entry.get_password() {
            Ok(password) => {
                tracing::debug!(
                    "Retrieved credential for {}.{}.{} from OS keyring",
                    service,
                    account,
                    key
                );
                Ok(password)
            }
            Err(keyring::Error::NoEntry) => Err(CredentialError::NotFound(format!(
                "{}.{}.{}",
                service, account, key
            ))
            .into()),
            Err(e) => Err(CredentialError::Keyring(e.to_string()).into()),
        }
    }

    fn delete_account(&self, service: &str, key: &str, account: &str) -> Result<()> {
        let (keyring_service, keyring_key) = Self::keyring_key(service, account, key);

        let entry = keyring::Entry::new(&keyring_service, &keyring_key)
            .map_err(|e| CredentialError::KeyringUnavailable(e.to_string()))?;

        // Attempt to delete - it's not an error if the entry doesn't exist
        match entry.delete_password() {
            Ok(_) => {
                tracing::debug!(
                    "Deleted credential for {}.{}.{} from OS keyring",
                    service,
                    account,
                    key
                );
                Ok(())
            }
            Err(keyring::Error::NoEntry) => {
                // Not an error to delete non-existent credential
                tracing::debug!(
                    "Credential {}.{}.{} not found (already deleted)",
                    service,
                    account,
                    key
                );
                Ok(())
            }
            Err(e) => Err(CredentialError::Keyring(e.to_string()).into()),
        }
    }

    fn exists_account(&self, service: &str, key: &str, account: &str) -> Result<bool> {
        let (keyring_service, keyring_key) = Self::keyring_key(service, account, key);

        let entry = keyring::Entry::new(&keyring_service, &keyring_key)
            .map_err(|e| CredentialError::KeyringUnavailable(e.to_string()))?;

        match entry.get_password() {
            Ok(_) => Ok(true),
            Err(keyring::Error::NoEntry) => Ok(false),
            Err(e) => Err(CredentialError::Keyring(e.to_string()).into()),
        }
    }

    fn list_accounts(&self, _service: &str, _key: &str) -> Result<Vec<String>> {
        // Keyring API doesn't support listing entries
        // This will be handled by AccountManager's registry
        // Return empty vec as keyring can't enumerate accounts
        Ok(Vec::new())
    }

    fn backend_name(&self) -> &str {
        "keyring"
    }
}

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

/// Validate that a path is not a symlink
///
/// This prevents symlink attacks where an attacker could trick the application
/// into reading/writing to a different file location than intended.
///
/// # Arguments
///
/// * `path` - The file path to validate
///
/// # Errors
///
/// Returns `CredentialError::Io` if:
/// - The path is a symlink
/// - The metadata cannot be read
///
/// # Security
///
/// This is a defense-in-depth measure against:
/// - Symlink attacks (reading sensitive files via symlink substitution)
/// - TOCTTOU (Time-of-check-time-of-use) attacks
/// - Privilege escalation via symlink manipulation
///
/// # Example
///
/// ```no_run
/// # use std::path::Path;
/// # use libplurcast::credentials::validate_not_symlink;
/// # fn example() -> libplurcast::error::Result<()> {
/// let path = Path::new("/path/to/credential.age");
/// validate_not_symlink(path)?;
/// // Safe to read the file now
/// # Ok(())
/// # }
/// ```
pub fn validate_not_symlink(path: &Path) -> Result<()> {
    // Get metadata without following symlinks
    let metadata = std::fs::symlink_metadata(path).map_err(|e| {
        CredentialError::Io(std::io::Error::new(
            e.kind(),
            format!(
                "Failed to read metadata for '{}': {}. \
                This may indicate a permission issue or the file doesn't exist.",
                path.display(),
                e
            ),
        ))
    })?;

    // Check if it's a symlink
    if metadata.is_symlink() {
        return Err(CredentialError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "Security: Credential file '{}' is a symbolic link. \
                For security reasons, credential files must be regular files, not symlinks. \
                Please use a regular file instead.",
                path.display()
            ),
        ))
        .into());
    }

    Ok(())
}

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
    pub(crate) fn encrypt(&self, data: &str) -> Result<Vec<u8>> {
        let password = self.master_password.read().unwrap();
        let password = password
            .as_ref()
            .ok_or(CredentialError::MasterPasswordNotSet)?;

        let encryptor =
            age::Encryptor::with_user_passphrase(age::secrecy::Secret::new(password.clone()));

        let mut encrypted = vec![];
        let mut writer = encryptor
            .wrap_output(&mut encrypted)
            .map_err(|e| CredentialError::Encryption(e.to_string()))?;

        writer
            .write_all(data.as_bytes())
            .map_err(|e| CredentialError::Encryption(e.to_string()))?;

        writer
            .finish()
            .map_err(|e| CredentialError::Encryption(e.to_string()))?;

        Ok(encrypted)
    }

    /// Decrypt data using the master password
    fn decrypt(&self, data: &[u8]) -> Result<String> {
        let password = self.master_password.read().unwrap();
        let password = password
            .as_ref()
            .ok_or(CredentialError::MasterPasswordNotSet)?;

        let decryptor = match age::Decryptor::new(data) {
            Ok(age::Decryptor::Passphrase(d)) => d,
            Ok(_) => {
                return Err(CredentialError::Encryption(
                    "Invalid encryption format (expected passphrase)".to_string(),
                )
                .into())
            }
            Err(e) => return Err(CredentialError::Encryption(e.to_string()).into()),
        };

        let mut decrypted = vec![];
        let mut reader = decryptor
            .decrypt(&age::secrecy::Secret::new(password.clone()), None)
            .map_err(|e| {
                // Check if it's a decryption failure (wrong password)
                if e.to_string().contains("decryption") || e.to_string().contains("MAC") {
                    CredentialError::DecryptionFailed
                } else {
                    CredentialError::Encryption(e.to_string())
                }
            })?;

        reader
            .read_to_end(&mut decrypted)
            .map_err(|e| CredentialError::Encryption(e.to_string()))?;

        Ok(String::from_utf8(decrypted)
            .map_err(|e| CredentialError::Encryption(format!("Invalid UTF-8: {}", e)))?)
    }

    /// Get the file path for a credential with account support
    fn get_file_path_account(&self, service: &str, key: &str, account: &str) -> PathBuf {
        self.base_path
            .join(format!("{}.{}.{}.age", service, account, key))
    }
}

impl CredentialStore for EncryptedFileStore {
    fn store_account(&self, service: &str, key: &str, account: &str, value: &str) -> Result<()> {
        let encrypted = self.encrypt(value)?;
        let file_path = self.get_file_path_account(service, key, account);

        // Create parent directories
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).map_err(CredentialError::Io)?;
        }

        std::fs::write(&file_path, encrypted).map_err(CredentialError::Io)?;

        // Set file permissions to 600 on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(&file_path, perms).map_err(CredentialError::Io)?;
        }

        tracing::debug!(
            "Stored encrypted credential for {}.{}.{} at {:?}",
            service,
            account,
            key,
            file_path
        );
        Ok(())
    }

    fn retrieve_account(&self, service: &str, key: &str, account: &str) -> Result<String> {
        let file_path = self.get_file_path_account(service, key, account);

        if !file_path.exists() {
            return Err(CredentialError::NotFound(format!(
                "{}.{}.{}",
                service, account, key
            ))
            .into());
        }

        // Security: Validate that the credential file is not a symlink
        validate_not_symlink(&file_path)?;

        let encrypted = std::fs::read(&file_path).map_err(CredentialError::Io)?;
        let decrypted = self.decrypt(&encrypted)?;

        tracing::debug!(
            "Retrieved encrypted credential for {}.{}.{} from {:?}",
            service,
            account,
            key,
            file_path
        );
        Ok(decrypted)
    }

    fn delete_account(&self, service: &str, key: &str, account: &str) -> Result<()> {
        let file_path = self.get_file_path_account(service, key, account);

        if file_path.exists() {
            std::fs::remove_file(&file_path).map_err(CredentialError::Io)?;
            tracing::debug!(
                "Deleted encrypted credential for {}.{}.{} at {:?}",
                service,
                account,
                key,
                file_path
            );
        } else {
            tracing::debug!(
                "Credential {}.{}.{} not found (already deleted)",
                service,
                account,
                key
            );
        }

        Ok(())
    }

    fn exists_account(&self, service: &str, key: &str, account: &str) -> Result<bool> {
        let file_path = self.get_file_path_account(service, key, account);
        Ok(file_path.exists())
    }

    fn list_accounts(&self, service: &str, key: &str) -> Result<Vec<String>> {
        let mut accounts = Vec::new();

        // Scan directory for matching files: {service}.*.{key}.age
        if !self.base_path.exists() {
            return Ok(accounts);
        }

        let entries = std::fs::read_dir(&self.base_path).map_err(CredentialError::Io)?;

        let prefix = format!("{}.", service);
        let suffix = format!(".{}.age", key);

        for entry in entries {
            let entry = entry.map_err(CredentialError::Io)?;
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            // Check if filename matches pattern: {service}.{account}.{key}.age
            if file_name_str.starts_with(&prefix) && file_name_str.ends_with(&suffix) {
                // Extract account name
                let account = file_name_str
                    .strip_prefix(&prefix)
                    .and_then(|s| s.strip_suffix(&suffix))
                    .map(|s| s.to_string());

                if let Some(account) = account {
                    accounts.push(account);
                }
            }
        }

        accounts.sort();
        Ok(accounts)
    }

    fn backend_name(&self) -> &str {
        "encrypted_file"
    }
}

use serde::{Deserialize, Serialize};

/// Storage backend type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum StorageBackend {
    /// OS-native keyring (macOS Keychain, Windows Credential Manager, Linux Secret Service)
    #[default]
    Keyring,
    /// Encrypted files with master password
    Encrypted,
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
                tracing::debug!(
                    "Loaded master password from PLURCAST_MASTER_PASSWORD environment variable"
                );
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
            return Err(
                CredentialError::Encryption("Credential path cannot be empty".to_string()).into(),
            );
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
            || (config.storage == StorageBackend::Keyring && stores.is_empty())
        {
            let encrypted_store = EncryptedFileStore::new(credential_path.clone());

            // Set master password if provided
            if let Some(password) = &config.master_password {
                encrypted_store.set_master_password(password.clone())?;
                tracing::info!("Using encrypted file storage for credentials");
                stores.push(Box::new(encrypted_store));
            } else {
                // Try to prompt for password if TTY available
                if atty::is(atty::Stream::Stdin) {
                    match rpassword::prompt_password(
                        "Enter master password for credential encryption: ",
                    ) {
                        Ok(password) if !password.is_empty() => {
                            match encrypted_store.set_master_password(password) {
                                Ok(_) => {
                                    tracing::info!("Using encrypted file storage for credentials");
                                    stores.push(Box::new(encrypted_store));
                                }
                                Err(e) => {
                                    tracing::error!(
                                        "Failed to set master password: {}. No secure storage available.",
                                        e
                                    );
                                }
                            }
                        }
                        Ok(_) => {
                            tracing::error!(
                                "Empty master password provided. No secure storage available."
                            );
                        }
                        Err(e) => {
                            tracing::error!(
                                "Failed to prompt for master password: {}. No secure storage available.",
                                e
                            );
                        }
                    }
                } else {
                    tracing::error!(
                        "Master password not set and no TTY available. No secure storage available."
                    );
                }
            }
        }

        if stores.is_empty() {
            return Err(CredentialError::NoStoreAvailable.into());
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
            store.store(service, key, value)?;
            tracing::debug!(
                "Stored credential for {}.{} using {} backend",
                service,
                key,
                store.backend_name()
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
                        service,
                        key,
                        store.backend_name()
                    );
                    return Ok(value);
                }
                Err(e) => {
                    // Check if it's a NotFound error
                    if let crate::error::PlurcastError::Credential(
                        crate::error::CredentialError::NotFound(_),
                    ) = &e
                    {
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

        Err(last_error
            .unwrap_or_else(|| CredentialError::NotFound(format!("{}.{}", service, key)).into()))
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

        tracing::debug!(
            "Deleted credential for {}.{} from all backends",
            service,
            key
        );
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

    /// Check if the primary storage backend is insecure
    ///
    /// Always returns `false` since we only support secure storage backends
    /// (OS keyring and encrypted files).
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use libplurcast::credentials::{CredentialManager, CredentialConfig};
    /// # fn example(manager: &CredentialManager) {
    /// if manager.is_insecure() {
    ///     eprintln!("WARNING: Using insecure credential storage!");
    /// }
    /// # }
    /// ```
    pub fn is_insecure(&self) -> bool {
        // We only support secure storage backends now
        false
    }

    /// Get the name of the primary storage backend being used
    ///
    /// Returns the backend name (e.g., "keyring", "encrypted_file", "plain_file")
    /// of the first (primary) storage backend.
    pub fn primary_backend(&self) -> Option<&str> {
        self.stores.first().map(|s| s.backend_name())
    }

    /// Store a credential for a specific account using the first available backend
    ///
    /// # Arguments
    ///
    /// * `service` - Service identifier (e.g., "plurcast.nostr")
    /// * `key` - Credential key (e.g., "private_key")
    /// * `account` - Account name (e.g., "test-account", "prod-account")
    /// * `value` - Credential value to store
    ///
    /// # Errors
    ///
    /// Returns an error if the credential cannot be stored in any backend.
    pub fn store_account(&self, service: &str, key: &str, account: &str, value: &str) -> Result<()> {
        if let Some(store) = self.stores.first() {
            store.store_account(service, key, account, value)?;
            tracing::debug!(
                "Stored credential for {}.{}.{} using {} backend",
                service,
                account,
                key,
                store.backend_name()
            );
            Ok(())
        } else {
            Err(CredentialError::NoStoreAvailable.into())
        }
    }

    /// Retrieve a credential for a specific account, trying all backends in order
    ///
    /// # Arguments
    ///
    /// * `service` - Service identifier (e.g., "plurcast.nostr")
    /// * `key` - Credential key (e.g., "private_key")
    /// * `account` - Account name (e.g., "test-account", "prod-account")
    ///
    /// # Returns
    ///
    /// The credential value as a String
    ///
    /// # Errors
    ///
    /// Returns `CredentialError::NotFound` if the credential is not found in any backend.
    pub fn retrieve_account(&self, service: &str, key: &str, account: &str) -> Result<String> {
        let mut last_error: Option<crate::error::PlurcastError> = None;

        for store in &self.stores {
            match store.retrieve_account(service, key, account) {
                Ok(value) => {
                    tracing::debug!(
                        "Retrieved credential for {}.{}.{} from {} backend",
                        service,
                        account,
                        key,
                        store.backend_name()
                    );
                    return Ok(value);
                }
                Err(e) => {
                    // Check if it's a NotFound error
                    if let crate::error::PlurcastError::Credential(
                        crate::error::CredentialError::NotFound(_),
                    ) = &e
                    {
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

        Err(last_error.unwrap_or_else(|| {
            CredentialError::NotFound(format!("{}.{}.{}", service, account, key)).into()
        }))
    }

    /// Delete a credential for a specific account from all backends
    ///
    /// # Arguments
    ///
    /// * `service` - Service identifier (e.g., "plurcast.nostr")
    /// * `key` - Credential key (e.g., "private_key")
    /// * `account` - Account name (e.g., "test-account", "prod-account")
    ///
    /// # Errors
    ///
    /// Returns an error if deletion fails in any backend.
    pub fn delete_account(&self, service: &str, key: &str, account: &str) -> Result<()> {
        for store in &self.stores {
            store.delete_account(service, key, account)?;
        }

        tracing::debug!(
            "Deleted credential for {}.{}.{} from all backends",
            service,
            account,
            key
        );
        Ok(())
    }

    /// Check if a credential exists for a specific account in any backend
    ///
    /// # Arguments
    ///
    /// * `service` - Service identifier (e.g., "plurcast.nostr")
    /// * `key` - Credential key (e.g., "private_key")
    /// * `account` - Account name (e.g., "test-account", "prod-account")
    ///
    /// # Returns
    ///
    /// `true` if the credential exists in any backend, `false` otherwise
    pub fn exists_account(&self, service: &str, key: &str, account: &str) -> Result<bool> {
        for store in &self.stores {
            if store.exists_account(service, key, account)? {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// List all accounts for a service/key combination, aggregating from all backends
    ///
    /// # Arguments
    ///
    /// * `service` - Service identifier (e.g., "plurcast.nostr")
    /// * `key` - Credential key (e.g., "private_key")
    ///
    /// # Returns
    ///
    /// A vector of unique account names that have credentials for this service/key
    ///
    /// # Errors
    ///
    /// Returns an error if any backend cannot be queried.
    pub fn list_accounts(&self, service: &str, key: &str) -> Result<Vec<String>> {
        let mut all_accounts = std::collections::HashSet::new();

        for store in &self.stores {
            let accounts = store.list_accounts(service, key)?;
            all_accounts.extend(accounts);
        }

        let mut accounts: Vec<String> = all_accounts.into_iter().collect();
        accounts.sort();
        Ok(accounts)
    }

    /// Automatically migrate credentials from old namespace format to multi-account format
    ///
    /// This method detects credentials stored in the old single-account format
    /// (`plurcast.{platform}.{key}`) and migrates them to the new multi-account format
    /// (`plurcast.{platform}.default.{key}`).
    ///
    /// The migration is automatic and transparent:
    /// - Checks if credential exists in old format
    /// - Checks if already migrated to new format
    /// - If not migrated, reads from old format and stores in new format
    /// - Verifies migration by retrieving from new format
    /// - Keeps old format for backward compatibility (doesn't delete)
    ///
    /// # Errors
    ///
    /// Returns an error if migration fails for any credential. Individual migration
    /// failures are logged but don't stop the overall process.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use libplurcast::credentials::{CredentialManager, CredentialConfig};
    /// # fn example(manager: &CredentialManager) -> libplurcast::error::Result<()> {
    /// // Automatically migrate on first use
    /// manager.auto_migrate_if_needed()?;
    ///
    /// // Now credentials are available in multi-account format
    /// let key = manager.retrieve_account("plurcast.nostr", "private_key", "default")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn auto_migrate_if_needed(&self) -> Result<()> {
        // Known platform credentials to check for migration
        let platforms = vec![
            ("plurcast.nostr", "private_key"),
            ("plurcast.mastodon", "access_token"),
            ("plurcast.bluesky", "app_password"),
        ];

        for (service, key) in platforms {
            // Check if old format exists (using single-account retrieve)
            let old_exists = self.exists(service, key).unwrap_or(false);

            if old_exists {
                // Check if already migrated to new format
                let new_exists = self.exists_account(service, key, "default").unwrap_or(false);

                if !new_exists {
                    // Need to migrate
                    match self.retrieve(service, key) {
                        Ok(value) => {
                            // Store in new format
                            match self.store_account(service, key, "default", &value) {
                                Ok(_) => {
                                    // Verify migration
                                    match self.retrieve_account(service, key, "default") {
                                        Ok(retrieved) if retrieved == value => {
                                            tracing::info!(
                                                "Auto-migrated {}.{} to default account",
                                                service,
                                                key
                                            );

                                            // Register the account in the account manager
                                            if let Ok(account_manager) = AccountManager::new() {
                                                // Extract platform from service (remove "plurcast." prefix)
                                                let platform = service.strip_prefix("plurcast.").unwrap_or(service);
                                                if let Err(e) = account_manager.register_account(platform, "default") {
                                                    tracing::warn!("Failed to register default account for {}: {}", platform, e);
                                                }
                                            }
                                        }
                                        Ok(_) => {
                                            tracing::error!(
                                                "Auto-migration verification failed for {}.{}: retrieved value doesn't match",
                                                service,
                                                key
                                            );
                                        }
                                        Err(e) => {
                                            tracing::error!(
                                                "Auto-migration verification failed for {}.{}: {}",
                                                service,
                                                key,
                                                e
                                            );
                                        }
                                    }
                                }
                                Err(e) => {
                                    tracing::error!(
                                        "Auto-migration failed for {}.{}: {}",
                                        service,
                                        key,
                                        e
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!(
                                "Failed to read old format credential for {}.{}: {}",
                                service,
                                key,
                                e
                            );
                        }
                    }
                } else {
                    tracing::debug!(
                        "Credential {}.{} already migrated to default account",
                        service,
                        key
                    );
                }
            }
        }

        Ok(())
    }

    /// Migrate credentials from old single-account format to multi-account format
    ///
    /// This method provides explicit migration control for users upgrading from
    /// single-account to multi-account credential storage. It:
    /// 1. Scans for old format credentials across all platforms
    /// 2. Migrates each credential to "default" account
    /// 3. Returns a detailed report of migration results
    ///
    /// Unlike `auto_migrate_if_needed()`, this method is intended to be called
    /// explicitly by the user via CLI command.
    ///
    /// # Returns
    ///
    /// A `MigrationReport` containing:
    /// - `migrated`: Successfully migrated credentials
    /// - `failed`: Failed migrations with error messages
    /// - `skipped`: Credentials already in multi-account format
    ///
    /// # Errors
    ///
    /// Returns an error if the storage backend is unavailable. Individual
    /// migration failures are reported in the MigrationReport, not as errors.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use libplurcast::credentials::{CredentialManager, CredentialConfig};
    /// # fn example(manager: &CredentialManager) -> libplurcast::error::Result<()> {
    /// let report = manager.migrate_to_multi_account()?;
    ///
    /// println!("Migration Summary:");
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
    pub fn migrate_to_multi_account(&self) -> Result<MigrationReport> {
        let mut report = MigrationReport::new();

        // Ensure we have a storage backend
        if self.stores.is_empty() {
            return Err(CredentialError::NoStoreAvailable.into());
        }

        tracing::info!("Starting multi-account migration");

        // Known platform credentials to migrate
        let platforms = vec![
            ("plurcast.nostr", "private_key"),
            ("plurcast.mastodon", "access_token"),
            ("plurcast.bluesky", "app_password"),
        ];

        for (service, key) in platforms {
            let credential_name = format!("{}.{}", service, key);

            // Check if new format exists (Orphan check & Pre-migration check)
            let new_exists = match self.exists_account(service, key, "default") {
                Ok(exists) => exists,
                Err(e) => {
                    tracing::error!(
                        "Failed to check new format existence of {}: {}",
                        credential_name,
                        e
                    );
                    report.failed.push((
                        credential_name.clone(),
                        format!("Failed to check new format: {}", e),
                    ));
                    false // Assume false on error
                }
            };

            // If new format exists, ensure it is registered in AccountManager
            if new_exists {
                if let Ok(account_manager) = AccountManager::new() {
                    let platform = service.strip_prefix("plurcast.").unwrap_or(service);
                    
                    // Check if already registered to avoid noise
                    if !account_manager.account_exists(platform, "default") {
                        tracing::info!("Found unregistered default account for {}, registering...", platform);
                        if let Err(e) = account_manager.register_account(platform, "default") {
                            tracing::warn!("Failed to register default account for {}: {}", platform, e);
                        } else {
                            tracing::info!("Successfully registered default account for {}", platform);
                        }
                    } else {
                        tracing::debug!("Default account for {} is already registered", platform);
                    }
                }
            }

            // Check if old format exists
            let old_exists = match self.exists(service, key) {
                Ok(exists) => exists,
                Err(e) => {
                    tracing::error!(
                        "Failed to check existence of {}: {}",
                        credential_name,
                        e
                    );
                    report.failed.push((
                        credential_name.clone(),
                        format!("Failed to check existence: {}", e),
                    ));
                    continue;
                }
            };

            if !old_exists {
                // No old format credential found, nothing to migrate
                continue;
            }

            if new_exists {
                tracing::debug!(
                    "Credential {} already migrated to default account",
                    credential_name
                );
                report.skipped.push(credential_name);
                continue;
            }

            // Read from old format
            let value = match self.retrieve(service, key) {
                Ok(v) => v,
                Err(e) => {
                    let error_msg = format!("Failed to read old format: {}", e);
                    tracing::error!("Migration failed for {}: {}", credential_name, error_msg);
                    report.failed.push((credential_name.clone(), error_msg));
                    continue;
                }
            };

            // Store in new format
            match self.store_account(service, key, "default", &value) {
                Ok(_) => {
                    // Verify migration
                    match self.retrieve_account(service, key, "default") {
                        Ok(retrieved) if retrieved == value => {
                            // Delete old format
                            if let Err(e) = self.delete(service, key) {
                                tracing::warn!("Failed to delete old credential {}: {}", credential_name, e);
                            }
                            
                            // Register account (redundant with store_account but safe)
                            if let Ok(account_manager) = AccountManager::new() {
                                let platform = service.strip_prefix("plurcast.").unwrap_or(service);
                                if let Err(e) = account_manager.register_account(platform, "default") {
                                    tracing::warn!("Failed to register default account for {}: {}", platform, e);
                                }
                            }

                            report.migrated.push(credential_name);
                        }
                        Ok(_) => {
                            let error_msg = "Verification failed: retrieved value mismatch".to_string();
                            tracing::error!("Migration failed for {}: {}", credential_name, error_msg);
                            report.failed.push((credential_name.clone(), error_msg));
                        }
                        Err(e) => {
                            let error_msg = format!("Verification failed: {}", e);
                            tracing::error!("Migration failed for {}: {}", credential_name, error_msg);
                            report.failed.push((credential_name.clone(), error_msg));
                        }
                    }
                }
                Err(e) => {
                    let error_msg = format!("Failed to store new format: {}", e);
                    tracing::error!("Migration failed for {}: {}", credential_name, error_msg);
                    report.failed.push((credential_name, error_msg));
                }
            }
        }

        tracing::info!(
            "Multi-account migration complete: {} migrated, {} failed, {} skipped",
            report.migrated.len(),
            report.failed.len(),
            report.skipped.len()
        );

        Ok(report)
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
        let primary_store = self
            .stores
            .first()
            .ok_or(CredentialError::NoStoreAvailable)?;

        if primary_store.backend_name() == "plain_file" {
            return Err(CredentialError::MigrationFailed(
                "Cannot migrate to plain text storage. Configure keyring or encrypted storage first.".to_string()
            ).into());
        }

        tracing::info!(
            "Starting credential migration to {} backend",
            primary_store.backend_name()
        );

        // Detect plain text credentials
        let plain_credentials = self.detect_plain_credentials()?;

        if plain_credentials.is_empty() {
            tracing::info!("No plain text credentials found to migrate");
            return Ok(report);
        }

        tracing::info!(
            "Found {} plain text credential files to migrate",
            plain_credentials.len()
        );

        // Migrate each credential
        for (service, key, file_path) in plain_credentials {
            let credential_name = format!("{}.{}", service, key);

            // Check if already in secure storage
            if let Ok(true) = primary_store.exists(&service, &key) {
                tracing::debug!(
                    "Credential {} already exists in secure storage, skipping",
                    credential_name
                );
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

                    // Register the account in the account manager
                    if let Ok(account_manager) = AccountManager::new() {
                        // Extract platform from service (remove "plurcast." prefix)
                        let platform = service.strip_prefix("plurcast.").unwrap_or(&service);
                        // For plain text migration, we're always migrating to "default" account if using store()
                        // But wait, store() delegates to store_account(..., "default", ...), so it is "default".
                        if let Err(e) = account_manager.register_account(platform, "default") {
                            tracing::warn!("Failed to register default account for {}: {}", platform, e);
                        }
                    }
                }
                Ok(_) => {
                    let error_msg =
                        "Verification failed: retrieved value doesn't match".to_string();
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
        let file_mapping = [
            ("plurcast.nostr.private_key", "nostr.keys"),
            ("plurcast.mastodon.access_token", "mastodon.token"),
            ("plurcast.bluesky.app_password", "bluesky.auth"),
        ];

        for credential_name in migrated_credentials {
            // Find the corresponding file
            if let Some((_, filename)) = file_mapping
                .iter()
                .find(|(name, _)| name == credential_name)
            {
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
