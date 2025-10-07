# Design Document

## Overview

This design implements a layered credential storage system for Plurcast that prioritizes security while maintaining backward compatibility. The architecture uses a trait-based abstraction that allows multiple storage backends (OS keyring, encrypted files, plain text) with automatic fallback and a clear migration path.

The design follows the Unix philosophy by providing separate command-line tools for credential management (`plur-creds`) and setup (`plur-setup`), while the core library (`libplurcast`) provides the storage abstraction used by all platform clients.

## Architecture

### Component Diagram

```
┌─────────────────────────────────────────────────────────┐
│                   CLI Tools                             │
├──────────────┬──────────────────┬──────────────────────┤
│  plur-creds  │    plur-setup    │  plur-post/history   │
│  (manage)    │    (wizard)      │  (use credentials)   │
└──────┬───────┴────────┬─────────┴────────┬─────────────┘
       │                │                  │
       └────────────────┴──────────────────┘
                        │
       ┌────────────────▼────────────────┐
       │      CredentialManager          │
       │   (facade with fallback logic)  │
       └────────────────┬────────────────┘
                        │
       ┌────────────────┴────────────────┐
       │      CredentialStore trait      │
       └────────────────┬────────────────┘
                        │
       ┌────────────────┼────────────────┐
       │                │                │
┌──────▼──────┐  ┌─────▼──────┐  ┌─────▼──────┐
│  KeyringStore│  │ Encrypted  │  │  PlainFile │
│  (primary)   │  │ FileStore  │  │  Store     │
│              │  │ (fallback) │  │ (legacy)   │
└──────────────┘  └────────────┘  └────────────┘
       │                │                │
┌──────▼──────┐  ┌─────▼──────┐  ┌─────▼──────┐
│ OS Keyring  │  │ age crypto │  │ Plain text │
│ (native)    │  │ + files    │  │ files      │
└─────────────┘  └────────────┘  └────────────┘
```

### Storage Priority

The CredentialManager tries storage backends in this order:

1. **KeyringStore** (if available and configured)
2. **EncryptedFileStore** (if master password is set)
3. **PlainFileStore** (with security warnings)

## Components and Interfaces

### 1. CredentialStore Trait

The core abstraction for credential storage:

```rust
pub trait CredentialStore: Send + Sync {
    /// Store a credential
    fn store(&self, service: &str, key: &str, value: &str) -> Result<()>;
    
    /// Retrieve a credential
    fn retrieve(&self, service: &str, key: &str) -> Result<String>;
    
    /// Delete a credential
    fn delete(&self, service: &str, key: &str) -> Result<()>;
    
    /// Check if a credential exists
    fn exists(&self, service: &str, key: &str) -> Result<bool>;
    
    /// Get the storage backend name (for logging/debugging)
    fn backend_name(&self) -> &str;
}
```

**Service naming convention:** `plurcast.{platform}` (e.g., "plurcast.nostr", "plurcast.mastodon")

**Key naming convention:** `{credential_type}` (e.g., "private_key", "access_token", "app_password")

### 2. KeyringStore Implementation

Uses the `keyring` crate for OS-native secure storage:

```rust
pub struct KeyringStore;

impl CredentialStore for KeyringStore {
    fn store(&self, service: &str, key: &str, value: &str) -> Result<()> {
        let entry = keyring::Entry::new(service, key)?;
        entry.set_password(value)?;
        Ok(())
    }
    
    fn retrieve(&self, service: &str, key: &str) -> Result<String> {
        let entry = keyring::Entry::new(service, key)?;
        Ok(entry.get_password()?)
    }
    
    // ... other methods
}
```

**Platform mapping:**
- **macOS:** Keychain (via Security framework)
- **Windows:** Credential Manager (via Windows Credential API)
- **Linux:** Secret Service (GNOME Keyring, KWallet via D-Bus)

**Error handling:** If keyring is unavailable (e.g., headless Linux), return specific error that triggers fallback.

### 3. EncryptedFileStore Implementation

Uses the `age` crate for file encryption:

```rust
pub struct EncryptedFileStore {
    base_path: PathBuf,
    master_password: Arc<RwLock<Option<String>>>,
}

impl EncryptedFileStore {
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            base_path,
            master_password: Arc::new(RwLock::new(None)),
        }
    }
    
    pub fn set_master_password(&self, password: String) -> Result<()> {
        // Validate password strength (min 12 chars recommended)
        if password.len() < 8 {
            return Err(CredentialError::WeakPassword);
        }
        *self.master_password.write().unwrap() = Some(password);
        Ok(())
    }
    
    fn encrypt(&self, data: &str) -> Result<Vec<u8>> {
        let password = self.master_password.read().unwrap();
        let password = password.as_ref()
            .ok_or(CredentialError::MasterPasswordNotSet)?;
        
        // Use age encryption with passphrase
        let encryptor = age::Encryptor::with_user_passphrase(
            age::secrecy::Secret::new(password.clone())
        );
        
        let mut encrypted = vec![];
        let mut writer = encryptor.wrap_output(&mut encrypted)?;
        writer.write_all(data.as_bytes())?;
        writer.finish()?;
        
        Ok(encrypted)
    }
    
    fn decrypt(&self, data: &[u8]) -> Result<String> {
        let password = self.master_password.read().unwrap();
        let password = password.as_ref()
            .ok_or(CredentialError::MasterPasswordNotSet)?;
        
        let decryptor = age::Decryptor::new(data)?;
        let mut decrypted = vec![];
        
        match decryptor {
            age::Decryptor::Passphrase(d) => {
                let mut reader = d.decrypt(
                    &age::secrecy::Secret::new(password.clone()),
                    None
                )?;
                reader.read_to_end(&mut decrypted)?;
            }
            _ => return Err(CredentialError::InvalidEncryption),
        }
        
        Ok(String::from_utf8(decrypted)?)
    }
}

impl CredentialStore for EncryptedFileStore {
    fn store(&self, service: &str, key: &str, value: &str) -> Result<()> {
        let encrypted = self.encrypt(value)?;
        let file_path = self.base_path
            .join(format!("{}.{}.age", service, key));
        
        // Create parent directories
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        std::fs::write(&file_path, encrypted)?;
        
        // Set file permissions to 600 on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(&file_path, perms)?;
        }
        
        Ok(())
    }
    
    fn retrieve(&self, service: &str, key: &str) -> Result<String> {
        let file_path = self.base_path
            .join(format!("{}.{}.age", service, key));
        
        if !file_path.exists() {
            return Err(CredentialError::NotFound);
        }
        
        let encrypted = std::fs::read(&file_path)?;
        self.decrypt(&encrypted)
    }
    
    // ... other methods
}
```

**File location:** `~/.config/plurcast/credentials/`

**File naming:** `{service}.{key}.age` (e.g., `plurcast.nostr.private_key.age`)

### 4. PlainFileStore Implementation

Maintains backward compatibility with existing plain text files:

```rust
pub struct PlainFileStore {
    base_path: PathBuf,
    warned: Arc<Mutex<HashSet<String>>>,
}

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
            std::fs::create_dir_all(parent)?;
        }
        
        std::fs::write(&file_path, value)?;
        
        // Set file permissions to 600 on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(&file_path, perms)?;
        }
        
        Ok(())
    }
    
    fn retrieve(&self, service: &str, key: &str) -> Result<String> {
        // Warn once per service
        let service_key = format!("{}.{}", service, key);
        let mut warned = self.warned.lock().unwrap();
        if !warned.contains(&service_key) {
            tracing::warn!(
                "Reading credentials from plain text file. \
                 Consider migrating to secure storage with 'plur-creds migrate'."
            );
            warned.insert(service_key);
        }
        
        let file_path = self.get_legacy_path(service, key);
        
        if !file_path.exists() {
            return Err(CredentialError::NotFound);
        }
        
        Ok(std::fs::read_to_string(&file_path)?.trim().to_string())
    }
    
    fn get_legacy_path(&self, service: &str, key: &str) -> PathBuf {
        // Map to existing file paths for backward compatibility
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
            _ => self.base_path.join(format!("{}.{}", service, key)),
        }
    }
    
    // ... other methods
}
```

### 5. CredentialManager Facade

Manages fallback logic and provides a unified API:

```rust
pub struct CredentialManager {
    stores: Vec<Box<dyn CredentialStore>>,
    config: CredentialConfig,
}

impl CredentialManager {
    pub fn new(config: CredentialConfig) -> Result<Self> {
        let mut stores: Vec<Box<dyn CredentialStore>> = vec![];
        
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
            let encrypted_store = EncryptedFileStore::new(
                config.credential_path.clone()
            );
            
            // Prompt for master password if not set
            if let Some(password) = &config.master_password {
                encrypted_store.set_master_password(password.clone())?;
            } else {
                // Prompt interactively if TTY
                if atty::is(atty::Stream::Stdin) {
                    let password = rpassword::prompt_password(
                        "Enter master password for credential encryption: "
                    )?;
                    encrypted_store.set_master_password(password)?;
                } else {
                    tracing::warn!(
                        "Master password not set and no TTY available. \
                         Falling back to plain text storage."
                    );
                }
            }
            
            stores.push(Box::new(encrypted_store));
        }
        
        // Add plain file store as final fallback
        let plain_store = PlainFileStore::new(config.credential_path.clone());
        stores.push(Box::new(plain_store));
        
        Ok(Self { stores, config })
    }
    
    pub fn store(&self, service: &str, key: &str, value: &str) -> Result<()> {
        // Use the first available store
        if let Some(store) = self.stores.first() {
            store.store(service, key, value)?;
            tracing::debug!(
                "Stored credential for {}.{} using {}",
                service, key, store.backend_name()
            );
            Ok(())
        } else {
            Err(CredentialError::NoStoreAvailable)
        }
    }
    
    pub fn retrieve(&self, service: &str, key: &str) -> Result<String> {
        // Try each store in order until one succeeds
        let mut last_error = None;
        
        for store in &self.stores {
            match store.retrieve(service, key) {
                Ok(value) => {
                    tracing::debug!(
                        "Retrieved credential for {}.{} from {}",
                        service, key, store.backend_name()
                    );
                    return Ok(value);
                }
                Err(e) if matches!(e, CredentialError::NotFound) => {
                    // Try next store
                    last_error = Some(e);
                    continue;
                }
                Err(e) => {
                    // Other errors are fatal
                    return Err(e);
                }
            }
        }
        
        Err(last_error.unwrap_or(CredentialError::NotFound))
    }
    
    pub fn migrate_from_plain(&self) -> Result<MigrationReport> {
        // Find all plain text credentials
        // Copy to secure storage
        // Verify they work
        // Optionally delete plain text files
        // Return report of what was migrated
        todo!()
    }
    
    // ... other methods
}
```

## Data Models

### Configuration

Add to `config.toml`:

```toml
[credentials]
# Storage backend: "keyring", "encrypted", "plain"
storage = "keyring"  # default

# Path for encrypted/plain file storage
# (keyring doesn't use files)
path = "~/.config/plurcast/credentials"

# Optional: master password for encrypted storage
# If not set, will prompt interactively
# master_password_env = "PLURCAST_MASTER_PASSWORD"
```

### Rust Configuration Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialConfig {
    #[serde(default = "default_storage_backend")]
    pub storage: StorageBackend,
    
    #[serde(default = "default_credential_path")]
    pub path: String,
    
    #[serde(skip)]
    pub master_password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum StorageBackend {
    Keyring,
    Encrypted,
    Plain,
}

fn default_storage_backend() -> StorageBackend {
    StorageBackend::Keyring
}

fn default_credential_path() -> String {
    "~/.config/plurcast/credentials".to_string()
}
```

## Error Handling

### New Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum CredentialError {
    #[error("Credential not found: {0}")]
    NotFound(String),
    
    #[error("OS keyring unavailable: {0}")]
    KeyringUnavailable(String),
    
    #[error("Master password not set")]
    MasterPasswordNotSet,
    
    #[error("Master password is too weak (minimum 8 characters)")]
    WeakPassword,
    
    #[error("Decryption failed: incorrect password or corrupted file")]
    DecryptionFailed,
    
    #[error("No credential store available")]
    NoStoreAvailable,
    
    #[error("Migration failed: {0}")]
    MigrationFailed(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Keyring error: {0}")]
    Keyring(#[from] keyring::Error),
    
    #[error("Encryption error: {0}")]
    Encryption(String),
}
```

## Testing Strategy

### Unit Tests

1. **CredentialStore implementations:**
   - Test store/retrieve/delete/exists for each backend
   - Test error conditions (not found, permission denied, etc.)
   - Test file permissions on Unix systems
   - Mock keyring for CI/CD environments

2. **CredentialManager:**
   - Test fallback logic (keyring → encrypted → plain)
   - Test migration from plain to secure storage
   - Test configuration parsing

3. **Encryption:**
   - Test age encryption/decryption
   - Test password validation
   - Test corrupted file handling

### Integration Tests

1. **End-to-end credential flow:**
   - Store credential → retrieve → use in platform client
   - Test with each storage backend
   - Test migration scenarios

2. **Platform client integration:**
   - Update NostrClient to use CredentialManager
   - Update MastodonClient to use CredentialManager
   - Update BlueskyClient to use CredentialManager
   - Verify backward compatibility with plain text files

### Security Tests

1. **File permissions:**
   - Verify 600 permissions on credential files
   - Test permission enforcement on Unix

2. **Memory safety:**
   - Verify credentials are cleared from memory on exit
   - Test that credentials don't leak in logs or error messages

3. **Encryption strength:**
   - Verify age encryption is used correctly
   - Test password strength validation

## CLI Tools Design

### plur-creds Binary

```bash
# Set credentials (prompts for value)
plur-creds set nostr
plur-creds set mastodon
plur-creds set bluesky

# List stored credentials (doesn't show values)
plur-creds list

# Delete credentials
plur-creds delete nostr

# Test credentials (authenticate with platform)
plur-creds test nostr
plur-creds test --all

# Migrate from plain text
plur-creds migrate

# Security audit
plur-creds audit
```

### plur-setup Binary

Interactive wizard for first-time setup:

```bash
plur-setup

# Output:
🌟 Welcome to Plurcast!

Let's set up your platform credentials securely.

Storage backend:
  1. OS Keyring (recommended) - macOS Keychain, Windows Credential Manager, Linux Secret Service
  2. Encrypted files - Password-protected files
  3. Plain text (not recommended) - Unencrypted files

Choose [1-3]: 1

✓ Using OS keyring

Configure Nostr? [Y/n]: y
Enter Nostr private key (hex or nsec format): nsec1...
✓ Testing authentication...
✓ Nostr configured successfully

Configure Mastodon? [Y/n]: y
Mastodon instance (e.g., mastodon.social): mastodon.social
Enter OAuth access token: ...
✓ Testing authentication...
✓ Mastodon configured successfully

Configure Bluesky? [Y/n]: y
Bluesky handle (e.g., user.bsky.social): user.bsky.social
Enter app password: ...
✓ Testing authentication...
✓ Bluesky configured successfully

✓ Setup complete! You can now use:
  - plur-post "Hello world"
  - plur-history
  - plur-creds list
```

## Migration Strategy

### Backward Compatibility

1. **Existing plain text files continue to work**
   - No breaking changes to file locations
   - Security warnings logged but not blocking

2. **Configuration is optional**
   - If no `[credentials]` section exists, use plain text (legacy behavior)
   - If `[credentials]` section exists, use specified backend

3. **Gradual migration**
   - Users can migrate one platform at a time
   - Both old and new storage can coexist during transition

### Migration Process

```bash
# 1. User runs migration command
plur-creds migrate

# 2. System detects plain text files
Found plain text credentials:
  - nostr: ~/.config/plurcast/nostr.keys
  - mastodon: ~/.config/plurcast/mastodon.token

# 3. System copies to secure storage
Migrating to OS keyring...
✓ Migrated nostr credentials
✓ Migrated mastodon credentials

# 4. System verifies credentials work
Testing migrated credentials...
✓ nostr authentication successful
✓ mastodon authentication successful

# 5. System offers to delete plain text files
Delete plain text files? [y/N]: y
✓ Deleted ~/.config/plurcast/nostr.keys
✓ Deleted ~/.config/plurcast/mastodon.token

Migration complete!
```

## Security Considerations

### Threat Model

**Protected against:**
- File system access by other users (Unix permissions)
- Backup exposure (encrypted at rest)
- Memory dumps (credentials cleared on exit)
- Accidental logging (credentials never logged)

**Not protected against:**
- Root/admin access to the system
- Malware running as the user
- Physical access to unlocked system
- Keylogger capturing master password

### Best Practices

1. **Use OS keyring when available** - Leverages OS security features
2. **Strong master passwords** - Minimum 12 characters recommended
3. **File permissions** - Always 600 for credential files
4. **No credentials in logs** - Only log access events, not values
5. **Clear documentation** - Users understand security model

## Dependencies

### New Crates

```toml
[dependencies]
# OS keyring integration
keyring = "2.3"

# Password prompts
rpassword = "7.3"

# File encryption
age = "0.10"

# TTY detection
atty = "0.2"
```

## Performance Considerations

1. **Keyring access** - May require OS authentication (acceptable for security)
2. **Encryption overhead** - Minimal for small credential files
3. **Memory caching** - Credentials cached in memory during session
4. **Migration** - One-time operation, performance not critical

## Future Enhancements

1. **Hardware security keys** - Support for YubiKey, etc.
2. **Credential rotation** - Automated credential refresh
3. **Multi-device sync** - Secure credential sync across devices
4. **Audit logging** - Detailed credential access logs
5. **Biometric authentication** - Touch ID, Windows Hello integration
