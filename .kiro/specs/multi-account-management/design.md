# Design Document: Multi-Account Credential Management

## Overview

This design implements named account support for Plurcast, allowing users to manage multiple credential sets per platform (e.g., test vs prod Nostr keys, personal vs work accounts). The system provides isolated credential storage with seamless account switching while maintaining backward compatibility with existing single-account setups.

## Architecture

### High-Level Design

```
┌─────────────────────────────────────────────────────────────┐
│                     CLI Layer                               │
│  plur-creds set/list/use/delete --account <name>           │
│  plur-post "content" --account <name>                       │
└────────────────────┬────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────┐
│              Account Manager                                │
│  • Account validation and normalization                     │
│  • Active account tracking (accounts.toml)                  │
│  • Account listing and discovery                            │
└────────────────────┬────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────┐
│           Credential Manager (Enhanced)                     │
│  • Multi-account namespace support                          │
│  • Backward compatibility with "default" account            │
│  • Migration from old namespace format                      │
└────────────────────┬────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────┐
│              Storage Backends                               │
│  KeyringStore:       plurcast.{platform}.{account}.{key}    │
│  EncryptedFileStore: plurcast.{platform}.{account}.{key}.age│
│  PlainFileStore:     {platform}.{account}.{key}             │
└─────────────────────────────────────────────────────────────┘
```

### Key Components

#### 1. Account Manager

**Purpose**: Manages account metadata, validation, and active account tracking

**Location**: `libplurcast/src/accounts.rs` (new file)

**Responsibilities**:
- Validate account names (alphanumeric, hyphens, underscores, max 64 chars)
- Track active account per platform
- List all accounts for a platform
- Persist active account state to `accounts.toml`

**Data Structure**:
```rust
pub struct AccountManager {
    state_file: PathBuf,  // ~/.config/plurcast/accounts.toml
    state: Arc<RwLock<AccountState>>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct AccountState {
    active: HashMap<String, String>,  // platform -> account_name
}
```

#### 2. Enhanced Credential Store Trait

**Changes to existing trait**:
```rust
pub trait CredentialStore: Send + Sync {
    // NEW: Multi-account methods
    fn store_account(&self, service: &str, key: &str, account: &str, value: &str) -> Result<()>;
    fn retrieve_account(&self, service: &str, key: &str, account: &str) -> Result<String>;
    fn delete_account(&self, service: &str, key: &str, account: &str) -> Result<()>;
    fn exists_account(&self, service: &str, key: &str, account: &str) -> Result<bool>;
    fn list_accounts(&self, service: &str, key: &str) -> Result<Vec<String>>;
    
    // EXISTING: Single-account methods (delegate to "default" account)
    fn store(&self, service: &str, key: &str, value: &str) -> Result<()> {
        self.store_account(service, key, "default", value)
    }
    fn retrieve(&self, service: &str, key: &str) -> Result<String> {
        self.retrieve_account(service, key, "default")
    }
    fn delete(&self, service: &str, key: &str) -> Result<()> {
        self.delete_account(service, key, "default")
    }
    fn exists(&self, service: &str, key: &str) -> Result<bool> {
        self.exists_account(service, key, "default")
    }
    
    fn backend_name(&self) -> &str;
}
```

**Rationale**: Existing methods delegate to "default" account for backward compatibility. New methods add explicit account parameter.

#### 3. Keyring Namespace Format

**Current Format** (single account):
```
plurcast.nostr.private_key
plurcast.mastodon.access_token
plurcast.bluesky.app_password
```

**New Format** (multi-account):
```
plurcast.nostr.default.private_key
plurcast.nostr.test-account.private_key
plurcast.nostr.prod-account.private_key
plurcast.mastodon.default.access_token
plurcast.mastodon.work.access_token
```

**Implementation**:
```rust
fn keyring_key(service: &str, account: &str, key: &str) -> (String, String) {
    // service: "plurcast.nostr"
    // account: "test-account"
    // key: "private_key"
    
    // Extract platform from service
    let platform = service.strip_prefix("plurcast.").unwrap_or(service);
    
    // Build new service and key for keyring
    let keyring_service = format!("plurcast.{}.{}", platform, account);
    let keyring_key = key.to_string();
    
    (keyring_service, keyring_key)
}

// Example:
// keyring_key("plurcast.nostr", "test-account", "private_key")
// => ("plurcast.nostr.test-account", "private_key")
```

#### 4. Account State File

**Location**: `~/.config/plurcast/accounts.toml`

**Format**:
```toml
# Active account per platform
[active]
nostr = "test-account"
mastodon = "default"
bluesky = "prod-account"
```

**Permissions**: 644 (readable by owner, not sensitive data)

**Behavior**:
- Created automatically on first `plur-creds use` command
- Read on every credential operation to determine active account
- Falls back to "default" if file doesn't exist or platform not listed

#### 5. Account Listing

**Implementation Strategy**:

For each storage backend, scan for credentials matching pattern:
- KeyringStore: Query keyring for all entries starting with `plurcast.{platform}.`
- EncryptedFileStore: List files matching `plurcast.{platform}.*.{key}.age`
- PlainFileStore: List files matching `{platform}.*.{key}`

**Challenges**:
- Keyring API doesn't support listing all entries
- Need to track accounts separately or scan known patterns

**Solution**: Maintain account registry in `accounts.toml`:
```toml
[active]
nostr = "test-account"
mastodon = "default"

[accounts.nostr]
names = ["default", "test-account", "prod-account"]

[accounts.mastodon]
names = ["default", "work"]
```

**Update on**:
- `plur-creds set --account <name>`: Add to registry
- `plur-creds delete --account <name>`: Remove from registry
- `plur-creds list`: Read from registry

## Components and Interfaces

### 1. AccountManager API

```rust
pub struct AccountManager {
    state_file: PathBuf,
    state: Arc<RwLock<AccountState>>,
}

impl AccountManager {
    /// Create new AccountManager with default state file location
    pub fn new() -> Result<Self>;
    
    /// Create with custom state file path
    pub fn with_path(path: PathBuf) -> Result<Self>;
    
    /// Validate account name format
    pub fn validate_account_name(name: &str) -> Result<()>;
    
    /// Get active account for platform (returns "default" if not set)
    pub fn get_active_account(&self, platform: &str) -> String;
    
    /// Set active account for platform
    pub fn set_active_account(&self, platform: &str, account: &str) -> Result<()>;
    
    /// List all accounts for a platform
    pub fn list_accounts(&self, platform: &str) -> Vec<String>;
    
    /// Register an account (called when credentials are stored)
    pub fn register_account(&self, platform: &str, account: &str) -> Result<()>;
    
    /// Unregister an account (called when credentials are deleted)
    pub fn unregister_account(&self, platform: &str, account: &str) -> Result<()>;
    
    /// Check if account exists for platform
    pub fn account_exists(&self, platform: &str, account: &str) -> bool;
    
    /// Save state to disk
    fn save(&self) -> Result<()>;
    
    /// Load state from disk
    fn load(&mut self) -> Result<()>;
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct AccountState {
    /// Active account per platform
    #[serde(default)]
    pub active: HashMap<String, String>,
    
    /// Registered accounts per platform
    #[serde(default)]
    pub accounts: HashMap<String, PlatformAccounts>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct PlatformAccounts {
    pub names: Vec<String>,
}
```

### 2. Enhanced CredentialManager

```rust
impl CredentialManager {
    /// Store credential for specific account
    pub fn store_account(&self, service: &str, key: &str, account: &str, value: &str) -> Result<()>;
    
    /// Retrieve credential for specific account
    pub fn retrieve_account(&self, service: &str, key: &str, account: &str) -> Result<String>;
    
    /// Delete credential for specific account
    pub fn delete_account(&self, service: &str, key: &str, account: &str) -> Result<()>;
    
    /// Check if credential exists for specific account
    pub fn exists_account(&self, service: &str, key: &str, account: &str) -> Result<bool>;
    
    /// List all accounts for a service/key combination
    pub fn list_accounts(&self, service: &str, key: &str) -> Result<Vec<String>>;
    
    /// Migrate old single-account credentials to default account
    pub fn migrate_to_multi_account(&self) -> Result<MigrationReport>;
}
```

### 3. CLI Changes

#### plur-creds

**New Commands**:
```bash
# Set active account
plur-creds use <platform> --account <name>

# List with account information
plur-creds list [--platform <platform>]
```

**Modified Commands**:
```bash
# All commands accept --account flag
plur-creds set <platform> --account <name> [--stdin]
plur-creds delete <platform> --account <name> [--force]
plur-creds test <platform> --account <name>
```

**Implementation**:
```rust
#[derive(Subcommand)]
enum Commands {
    Set {
        platform: String,
        #[arg(long, default_value = "default")]
        account: String,
        #[arg(long)]
        stdin: bool,
    },
    
    List {
        #[arg(long)]
        platform: Option<String>,
    },
    
    Delete {
        platform: String,
        #[arg(long, default_value = "default")]
        account: String,
        #[arg(short, long)]
        force: bool,
    },
    
    Use {
        platform: String,
        #[arg(long)]
        account: String,
    },
    
    Test {
        platform: Option<String>,
        #[arg(long, default_value = "default")]
        account: String,
        #[arg(short, long)]
        all: bool,
    },
    
    Migrate,
    Audit,
}
```

#### plur-post

**Modified Command**:
```bash
plur-post "content" --account <name>
```

**Implementation**:
```rust
#[derive(Parser)]
struct Cli {
    // ... existing fields ...
    
    /// Account to use for posting (uses active account if not specified)
    #[arg(long)]
    account: Option<String>,
}
```

**Integration**:
```rust
async fn run(cli: Cli) -> Result<()> {
    let config = Config::load()?;
    let account_manager = AccountManager::new()?;
    
    // Determine account to use
    let account = cli.account.unwrap_or_else(|| {
        // Use active account for each platform
        // This is handled per-platform in create_platforms()
        "default".to_string()
    });
    
    // Create platforms with account parameter
    let platforms = create_platforms(&config, Some(&account)).await?;
    
    // ... rest of posting logic ...
}
```

## Data Models

### Account State File Schema

```toml
# ~/.config/plurcast/accounts.toml

# Active account per platform
[active]
nostr = "test-account"
mastodon = "default"
bluesky = "prod-account"

# Registered accounts per platform
[accounts.nostr]
names = ["default", "test-account", "prod-account"]

[accounts.mastodon]
names = ["default", "work"]

[accounts.bluesky]
names = ["default", "prod-account"]
```

### Credential Namespace Mapping

| Storage Backend | Old Format | New Format (Multi-Account) |
|----------------|------------|----------------------------|
| KeyringStore | `plurcast.nostr` / `private_key` | `plurcast.nostr.test-account` / `private_key` |
| EncryptedFileStore | `plurcast.nostr.private_key.age` | `plurcast.nostr.test-account.private_key.age` |
| PlainFileStore | `nostr.keys` | `nostr.test-account.keys` |

## Error Handling

### New Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum AccountError {
    #[error("Invalid account name: {0}. Must be alphanumeric with hyphens/underscores, max 64 chars")]
    InvalidName(String),
    
    #[error("Account '{0}' not found for platform '{1}'")]
    NotFound(String, String),
    
    #[error("Account '{0}' already exists for platform '{1}'")]
    AlreadyExists(String, String),
    
    #[error("Cannot delete active account '{0}' for platform '{1}'. Set a different active account first.")]
    CannotDeleteActive(String, String),
    
    #[error("Account state file error: {0}")]
    StateFile(String),
    
    #[error("Reserved account name: {0}")]
    ReservedName(String),
}
```

### Error Handling Strategy

1. **Invalid account name**: Validate on input, reject with clear message
2. **Account not found**: Check registry before operations, suggest `plur-creds list`
3. **Credential not found**: Distinguish between "account exists but no credentials" vs "account doesn't exist"
4. **State file corruption**: Log warning, use defaults, attempt to recreate
5. **Migration failures**: Report per-credential, don't fail entire migration

## Testing Strategy

### Unit Tests

1. **Account name validation**
   - Valid names: `default`, `test-account`, `prod_123`, `work`
   - Invalid names: `test account` (space), `test@account` (special char), `a` * 65 (too long)

2. **Keyring namespace derivation**
   - Verify format: `plurcast.{platform}.{account}.{key}`
   - Test with various account names
   - Ensure deterministic output

3. **Account state persistence**
   - Save and load state file
   - Handle missing file (use defaults)
   - Handle corrupted file (log warning, use defaults)

4. **Backward compatibility**
   - Old namespace format still works
   - Automatic migration to "default" account
   - Existing credentials accessible

### Integration Tests

1. **Multi-account CRUD**
   - Store credentials for multiple accounts
   - Retrieve correct credentials per account
   - Delete specific account without affecting others
   - List all accounts for a platform

2. **Active account switching**
   - Set active account
   - Verify active account used when no explicit account specified
   - Change active account and verify switch

3. **Cross-backend consistency**
   - Store in keyring, verify namespace
   - Store in encrypted files, verify filename
   - Fallback logic works with multi-account

4. **CLI integration**
   - `plur-creds set --account test`
   - `plur-creds list --platform nostr`
   - `plur-creds use nostr --account prod`
   - `plur-post "test" --account test`

### Process Persistence Tests (Windows-specific)

```rust
#[test]
#[cfg(target_os = "windows")]
fn test_multi_account_keyring_persistence() {
    // Store credentials for multiple accounts
    // Spawn child process
    // Verify all accounts still accessible
    // Verify correct account retrieved
}
```

## Migration Strategy

### Automatic Migration

**Trigger**: First time a user runs any credential command after upgrade

**Process**:
1. Detect old namespace format: `plurcast.{platform}.{key}`
2. Check if already migrated (exists in new format)
3. If not migrated:
   - Read credential from old namespace
   - Store in new namespace: `plurcast.{platform}.default.{key}`
   - Verify by retrieving
   - Log success
4. Keep old namespace for backward compatibility (don't delete)

**Implementation**:
```rust
impl CredentialManager {
    pub fn auto_migrate_if_needed(&self) -> Result<()> {
        let platforms = vec![
            ("plurcast.nostr", "private_key"),
            ("plurcast.mastodon", "access_token"),
            ("plurcast.bluesky", "app_password"),
        ];
        
        for (service, key) in platforms {
            // Check if old format exists
            if self.exists_old_format(service, key)? {
                // Check if already migrated
                if !self.exists_account(service, key, "default")? {
                    // Migrate to default account
                    let value = self.retrieve_old_format(service, key)?;
                    self.store_account(service, key, "default", &value)?;
                    tracing::info!("Auto-migrated {}.{} to default account", service, key);
                }
            }
        }
        
        Ok(())
    }
}
```

### Manual Migration Command

**Command**: `plur-creds migrate --to-multi-account`

**Purpose**: Explicit migration for users who want control

**Process**:
1. Scan for old format credentials
2. Display what will be migrated
3. Ask for confirmation
4. Migrate to "default" account
5. Display report
6. Optionally delete old format (with confirmation)

## Backward Compatibility

### Guarantees

1. **Existing credentials work**: Old namespace format still readable
2. **No breaking changes**: Omitting `--account` uses "default"
3. **Automatic migration**: Transparent to users
4. **Config compatibility**: No changes to config.toml required

### Compatibility Matrix

| Scenario | Behavior |
|----------|----------|
| Existing user, no upgrade | Works as before (old namespace) |
| Existing user, after upgrade | Auto-migrates to "default" account |
| New user | Uses "default" account by default |
| User with `--account` flag | Uses specified account |
| User with `plur-creds use` | Uses active account |

## Security Considerations

### Account Isolation

- Each account has completely separate credentials
- No cross-account credential access
- Account names not sensitive (stored in plain text state file)
- Credentials still protected by storage backend (keyring/encrypted/plain)

### State File Security

- `accounts.toml` contains no sensitive data (only account names and active selections)
- File permissions: 644 (readable by owner and group)
- Corruption handled gracefully (log warning, use defaults)

### Attack Vectors

1. **Account name collision**: Mitigated by validation (alphanumeric + hyphens/underscores only)
2. **State file tampering**: Non-critical (only affects active account selection)
3. **Credential namespace confusion**: Prevented by deterministic namespace derivation

## Performance Considerations

### Account Operations

- **List accounts**: O(1) - read from state file registry
- **Get active account**: O(1) - hash map lookup
- **Set active account**: O(1) - hash map update + file write
- **Store credential**: O(1) - same as before, just different namespace

### State File I/O

- **Read on startup**: Once per command execution
- **Write on change**: Only when active account changes or account registered/unregistered
- **File size**: Negligible (<1KB for typical usage)

### Optimization

- Cache state in memory (Arc<RwLock<AccountState>>)
- Only write to disk on changes
- Lazy load on first access

## Future Enhancements

### Phase 1 (0.3.0-alpha2) - Core Multi-Account

- Named accounts with `--account` flag
- Active account tracking
- Account listing
- Backward compatibility

### Phase 2 (Future) - Advanced Features

- **Account templates**: Pre-configured account settings
- **Account groups**: Organize accounts hierarchically
- **Account sync**: Sync accounts across machines (encrypted)
- **Account permissions**: Fine-grained access control
- **Team accounts**: Shared accounts with role-based access

### Phase 3 (Future) - UI Integration

- TUI account switcher
- GUI account management
- Visual account indicators
- Account-specific themes/colors

## References

- ADR 001: Multi-Account Credential Management
- Existing credential system: `libplurcast/src/credentials.rs`
- Inspiration: `kubectl config use-context`, `git config`, `aws configure --profile`
- Keyring library: `keyring-rs` v2.3
- Age encryption: `age` crate

## Appendix: Example Workflows

### Workflow 1: Developer with Test and Prod Accounts

```bash
# Initial setup - store test credentials
plur-creds set nostr --account test
# Enter test private key...

# Store prod credentials
plur-creds set nostr --account prod
# Enter prod private key...

# Set test as active for development
plur-creds use nostr --account test

# Post to test account (uses active account)
plur-post "Testing new feature"

# Post to prod account explicitly
plur-post "Production announcement" --account prod

# List all accounts
plur-creds list --platform nostr
# Output:
#   ✓ nostr (test): Private Key (stored in keyring) [active]
#   ✓ nostr (prod): Private Key (stored in keyring)
```

### Workflow 2: User with Personal and Work Accounts

```bash
# Store personal Mastodon account
plur-creds set mastodon --account personal
# Enter personal access token...

# Store work Mastodon account
plur-creds set mastodon --account work
# Enter work access token...

# Set work as active during work hours
plur-creds use mastodon --account work

# Post to work account
plur-post "Team update: Sprint completed!"

# Switch to personal for evening
plur-creds use mastodon --account personal

# Post to personal account
plur-post "Weekend plans!"
```

### Workflow 3: Migrating from Single Account

```bash
# User has existing credentials (old format)
plur-post "Hello world"  # Works with old format

# After upgrade, automatic migration happens
plur-creds list
# Output:
#   ✓ nostr (default): Private Key (stored in keyring)

# Add new test account
plur-creds set nostr --account test
# Enter test private key...

# Now have both accounts
plur-creds list --platform nostr
# Output:
#   ✓ nostr (default): Private Key (stored in keyring) [active]
#   ✓ nostr (test): Private Key (stored in keyring)
```
