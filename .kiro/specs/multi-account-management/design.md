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

**Account Name Validation Rules**:
- **Allowed characters**: a-z, A-Z, 0-9, hyphen (-), underscore (_)
- **Maximum length**: 64 characters
- **Minimum length**: 1 character
- **Case-sensitive**: "Test" and "test" are different accounts
- **Reserved names**: None currently, but validation allows for future reserved keywords
- **Examples**:
  - Valid: `default`, `test-account`, `prod_123`, `work`, `my-account-2024`
  - Invalid: `test account` (space), `test@account` (special char), `a` * 65 (too long), `` (empty)

**Validation Implementation**:
```rust
impl AccountManager {
    pub fn validate_account_name(name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(AccountError::InvalidName("Account name cannot be empty".to_string()).into());
        }
        
        if name.len() > 64 {
            return Err(AccountError::InvalidName(format!(
                "Account name too long: {} characters (max 64)", 
                name.len()
            )).into());
        }
        
        // Check for valid characters
        if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(AccountError::InvalidName(format!(
                "Account name '{}' contains invalid characters. Only alphanumeric, hyphens, and underscores allowed",
                name
            )).into());
        }
        
        // Check for reserved names (none currently, but allows for future expansion)
        // if RESERVED_NAMES.contains(&name) {
        //     return Err(AccountError::ReservedName(name.to_string()).into());
        // }
        
        Ok(())
    }
}
```

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
    
    /// Validate account name format (alphanumeric, hyphens, underscores, max 64 chars)
    /// Returns error for invalid names or reserved keywords
    pub fn validate_account_name(name: &str) -> Result<()>;
    
    /// Get active account for platform (returns "default" if not set)
    pub fn get_active_account(&self, platform: &str) -> String;
    
    /// Set active account for platform
    /// Returns error if account doesn't exist
    pub fn set_active_account(&self, platform: &str, account: &str) -> Result<()>;
    
    /// List all accounts for a platform
    pub fn list_accounts(&self, platform: &str) -> Vec<String>;
    
    /// Register an account (called when credentials are stored)
    pub fn register_account(&self, platform: &str, account: &str) -> Result<()>;
    
    /// Unregister an account (called when credentials are deleted)
    /// Resets active account to "default" if deleting the active account
    pub fn unregister_account(&self, platform: &str, account: &str) -> Result<()>;
    
    /// Check if account exists for platform
    pub fn account_exists(&self, platform: &str, account: &str) -> bool;
    
    /// Save state to disk with appropriate permissions (600 on Unix)
    fn save(&self) -> Result<()>;
    
    /// Load state from disk, handle corruption gracefully
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

**Design Decisions**:

1. **Account Name Validation**: Enforced at the AccountManager level to ensure consistency across all operations. Alphanumeric characters, hyphens, and underscores only, with a maximum length of 64 characters. This prevents namespace collisions and ensures compatibility with all storage backends.

2. **Active Account Fallback**: Always returns "default" when no active account is set, ensuring backward compatibility and eliminating the need for null checks throughout the codebase.

3. **State File Permissions**: Set to 600 on Unix systems (owner read/write only) to prevent unauthorized access, even though the file contains no sensitive data. This follows security best practices.

4. **Graceful Corruption Handling**: If the state file is corrupted, the system logs a warning and uses default values rather than failing. This ensures the system remains operational even with a corrupted state file.

5. **Active Account Reset on Deletion**: When deleting the active account, automatically reset to "default" to prevent invalid state. This is more user-friendly than requiring manual reset.

### 2. Enhanced CredentialManager

```rust
impl CredentialManager {
    /// Store credential for specific account
    /// Automatically registers account with AccountManager
    pub fn store_account(&self, service: &str, key: &str, account: &str, value: &str) -> Result<()>;
    
    /// Retrieve credential for specific account
    /// Uses same fallback logic (keyring → encrypted → plain) for all accounts
    pub fn retrieve_account(&self, service: &str, key: &str, account: &str) -> Result<String>;
    
    /// Delete credential for specific account from all storage backends
    /// Automatically unregisters account with AccountManager
    pub fn delete_account(&self, service: &str, key: &str, account: &str) -> Result<()>;
    
    /// Check if credential exists for specific account
    pub fn exists_account(&self, service: &str, key: &str, account: &str) -> Result<bool>;
    
    /// List all accounts for a service/key combination
    /// Queries all configured storage backends and aggregates results
    pub fn list_accounts(&self, service: &str, key: &str) -> Result<Vec<String>>;
    
    /// Migrate old single-account credentials to default account
    /// Returns detailed report of successful and failed migrations
    pub fn migrate_to_multi_account(&self) -> Result<MigrationReport>;
}

#[derive(Debug)]
pub struct MigrationReport {
    pub migrated: Vec<String>,
    pub failed: Vec<(String, String)>,  // (credential_id, error_message)
    pub skipped: Vec<String>,
}
```

**Design Decisions**:

1. **Automatic Account Registration**: When storing credentials, automatically register the account with AccountManager. This eliminates the need for separate registration steps and ensures the account registry stays in sync with actual credentials.

2. **Consistent Fallback Logic**: All accounts use the same storage backend fallback logic (keyring → encrypted → plain). This ensures consistent behavior regardless of which account is being accessed.

3. **Multi-Backend Deletion**: When deleting an account, remove credentials from all storage backends to ensure complete cleanup. This prevents orphaned credentials in fallback stores.

4. **Aggregated Account Listing**: Query all configured storage backends and aggregate results to provide a complete view of available accounts. This handles cases where accounts may exist in different backends.

5. **Detailed Migration Reporting**: Provide detailed feedback on migration success/failure to help users understand what happened and take corrective action if needed.

### 3. CLI Changes

#### plur-creds

**New Commands**:
```bash
# Set active account
plur-creds use <platform> --account <name>

# List with account information
plur-creds list [--platform <platform>]

# Migrate from single-account to multi-account
plur-creds migrate --from-single-account
```

**Modified Commands**:
```bash
# All commands accept --account flag (defaults to "default")
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
    
    Migrate {
        #[arg(long)]
        from_single_account: bool,
    },
    
    Audit,
}
```

**Command Behaviors**:

1. **`plur-creds set`**: 
   - Validates account name before storing
   - Automatically registers account with AccountManager
   - Prompts for credentials interactively or reads from stdin
   - Displays success message with storage backend used

2. **`plur-creds list`**:
   - Shows all platforms with at least one stored account
   - Displays platform, account name, credential type, and storage backend
   - Indicates active account with `[active]` marker
   - Filters by platform if `--platform` specified
   - Shows "No credentials configured" if no accounts exist
   
   **Output Format**:
   ```
   Configured Accounts:
   
   nostr:
     ✓ default: Private Key (keyring) [active]
     ✓ test: Private Key (keyring)
     ✓ prod: Private Key (encrypted file)
   
   mastodon:
     ✓ default: Access Token (keyring)
     ✓ work: Access Token (keyring) [active]
   
   bluesky:
     ✓ default: App Password (keyring) [active]
   ```
   
   **With `--platform nostr`**:
   ```
   nostr accounts:
     ✓ default: Private Key (keyring) [active]
     ✓ test: Private Key (keyring)
     ✓ prod: Private Key (encrypted file)
   ```
   
   **When no accounts exist**:
   ```
   No credentials configured.
   
   To add credentials, run:
     plur-creds set <platform> --account <name>
   ```

3. **`plur-creds delete`**:
   - Prompts for confirmation unless `--force` specified
   - Removes credentials from all storage backends (keyring, encrypted files, plain text)
   - Automatically unregisters account with AccountManager
   - Resets active account to "default" if deleting active account
   - Displays error if account doesn't exist
   
   **Deletion Process**:
   1. Validate account exists
   2. Check if account is active
   3. Prompt for confirmation (unless `--force`)
   4. Remove credentials from all storage backends
   5. Unregister account from AccountManager
   6. If deleting active account, reset to "default"
   7. Display success message
   
   **Example Output**:
   ```bash
   $ plur-creds delete nostr --account test
   Warning: This will permanently delete credentials for account 'test' on platform 'nostr'
   Continue? [y/N]: y
   ✓ Removed credentials from keyring
   ✓ Removed credentials from encrypted files
   ✓ Unregistered account 'test'
   ✓ Account 'test' deleted successfully
   
   $ plur-creds delete nostr --account test
   # (test is active account)
   Warning: Account 'test' is currently active for platform 'nostr'
   Warning: This will permanently delete credentials and reset active account to 'default'
   Continue? [y/N]: y
   ✓ Removed credentials from keyring
   ✓ Unregistered account 'test'
   ✓ Reset active account to 'default'
   ✓ Account 'test' deleted successfully
   
   $ plur-creds delete nostr --account nonexistent
   Error: Account 'nonexistent' not found for platform 'nostr'
   Hint: Run 'plur-creds list --platform nostr' to see available accounts
   ```

4. **`plur-creds use`**:
   - Validates that account exists before setting as active
   - Persists selection to accounts.toml
   - Displays confirmation message
   - Displays error if account doesn't exist

5. **`plur-creds test`**:
   - Tests authentication using specified account's credentials
   - Uses active account if `--account` not specified
   - Tests all platforms if `--all` specified
   - Displays success/failure with account details
   - Displays error if account doesn't exist
   
   **Testing Process**:
   1. Validate account exists
   2. Retrieve credentials for account
   3. Initialize platform client with credentials
   4. Attempt authentication (platform-specific)
   5. Display result with timing information
   
   **Example Output**:
   ```bash
   $ plur-creds test nostr --account test
   Testing nostr with account 'test'...
   ✓ Authentication successful (342ms)
   Account: test
   Platform: nostr
   Credentials: Private Key (keyring)
   
   $ plur-creds test nostr --account test
   # (with invalid credentials)
   Testing nostr with account 'test'...
   ✗ Authentication failed (156ms)
   Error: Invalid private key format
   Hint: Run 'plur-creds set nostr --account test' to update credentials
   
   $ plur-creds test --all
   Testing all platforms with active accounts...
   
   nostr (account: test):
     ✓ Authentication successful (342ms)
   
   mastodon (account: work):
     ✓ Authentication successful (521ms)
   
   bluesky (account: default):
     ✗ Authentication failed (234ms)
     Error: Invalid app password
   
   Results: 2/3 platforms authenticated successfully
   
   $ plur-creds test nostr --account nonexistent
   Error: Account 'nonexistent' not found for platform 'nostr'
   Hint: Run 'plur-creds list --platform nostr' to see available accounts
   ```
   
   **Platform-Specific Authentication Tests**:
   - **Nostr**: Validate key format, attempt to sign a test event
   - **Mastodon**: Verify access token with `/api/v1/accounts/verify_credentials`
   - **Bluesky**: Authenticate with `com.atproto.server.createSession`

6. **`plur-creds migrate`**:
   - Displays summary of credentials to be migrated
   - Prompts for confirmation before proceeding
   - Migrates all old-format credentials to "default" account
   - Displays detailed report of successful and failed migrations
   - Preserves original credentials (doesn't delete)
   - Displays "Nothing to migrate" if no old-format credentials exist

**Design Decisions**:

1. **Default Account Value**: All commands default to "default" account when `--account` is omitted, ensuring backward compatibility and consistent behavior.

2. **Confirmation Prompts**: Delete operations require confirmation to prevent accidental credential loss. Use `--force` to skip confirmation for scripting.

3. **Automatic Registration**: Account registration happens automatically during `set` operations, eliminating manual registration steps.

4. **Active Account Indicators**: List command clearly shows which account is active for each platform, making it easy to see current configuration.

5. **Explicit Migration Command**: Provide explicit `migrate` command for users who want control over when migration happens, in addition to automatic migration.

#### plur-post

**Modified Command**:
```bash
# Post using specific account
plur-post "content" --account <name>

# Post using active account (default behavior)
plur-post "content"
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
    let credential_manager = CredentialManager::new()?;
    
    // Determine account to use per platform
    let platforms = if let Some(account) = &cli.account {
        // Use specified account for all enabled platforms
        create_platforms_with_account(&config, &credential_manager, account).await?
    } else {
        // Use active account for each platform
        create_platforms_with_active_accounts(&config, &credential_manager, &account_manager).await?
    };
    
    // ... rest of posting logic ...
}

async fn create_platforms_with_active_accounts(
    config: &Config,
    credentials: &CredentialManager,
    accounts: &AccountManager,
) -> Result<Vec<Box<dyn Platform>>> {
    let mut platforms = Vec::new();
    
    for platform_name in &config.enabled_platforms {
        // Get active account for this platform
        let account = accounts.get_active_account(platform_name);
        
        // Create platform with account-specific credentials
        match create_platform(platform_name, config, credentials, &account).await {
            Ok(platform) => platforms.push(platform),
            Err(e) => {
                eprintln!("Warning: Failed to initialize {} with account '{}': {}", 
                         platform_name, account, e);
                // Continue with other platforms
            }
        }
    }
    
    if platforms.is_empty() {
        return Err(anyhow!("No platforms could be initialized"));
    }
    
    Ok(platforms)
}
```

**Example Scenarios**:

1. **Posting with explicit account**:
   ```bash
   $ plur-post "Test message" --account test
   # Uses "test" account for all enabled platforms (nostr, mastodon, bluesky)
   ```

2. **Posting with active accounts** (different per platform):
   ```bash
   # Active accounts: nostr=test, mastodon=work, bluesky=default
   $ plur-post "Production announcement"
   # Uses: nostr/test, mastodon/work, bluesky/default
   ```

3. **Posting when account doesn't exist for a platform**:
   ```bash
   $ plur-post "Test" --account nonexistent
   Warning: No credentials found for account 'nonexistent' on platform 'nostr'
   Warning: No credentials found for account 'nonexistent' on platform 'mastodon'
   Warning: No credentials found for account 'nonexistent' on platform 'bluesky'
   Error: No platforms could be initialized
   ```

4. **Posting with partial account coverage**:
   ```bash
   # "test" account exists for nostr but not mastodon
   $ plur-post "Test" --account test
   Warning: No credentials found for account 'test' on platform 'mastodon'
   ✓ Posted to nostr (note1abc...)
   ```

**Design Decisions**:

1. **Per-Platform Active Accounts**: When no `--account` is specified, use the active account for each platform independently. This allows users to have different active accounts for different platforms (e.g., "test" for Nostr, "work" for Mastodon).

2. **Explicit Account Override**: When `--account` is specified, use that account for all enabled platforms. This provides a way to override active accounts for a single post.

3. **Graceful Degradation**: If a platform fails to initialize with the specified account (e.g., credentials don't exist), log a warning and continue with other platforms. This prevents one misconfigured platform from blocking posts to other platforms.

4. **Clear Error Messages**: When an account doesn't exist or has no credentials for an enabled platform, provide clear error messages indicating which account and platform failed.

5. **Backward Compatibility**: When no `--account` is specified and no active account is set, fall back to "default" account, ensuring existing workflows continue to work.

6. **Fail Fast on Total Failure**: If no platforms can be initialized (all accounts missing credentials), fail with an error rather than silently succeeding with no posts.

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

**File Properties**:
- **Location**: `~/.config/plurcast/accounts.toml`
- **Permissions**: 600 on Unix systems (owner read/write only)
- **Format**: TOML
- **Size**: Typically <1KB
- **Sensitivity**: Non-sensitive (contains only account names and active selections)

**Behavior**:
- Created automatically on first account operation
- Read on every credential operation to determine active account
- Falls back to "default" if file doesn't exist or platform not listed
- Corruption handled gracefully (log warning, use defaults, attempt to recreate)

**Design Decisions**:

1. **Separate Active and Registry Sections**: Keep active account selection separate from account registry for clarity and easier parsing.

2. **Platform-Specific Sections**: Use separate sections per platform for account registry to allow easy addition of platform-specific metadata in the future.

3. **Explicit Account Lists**: Store explicit list of account names rather than relying on credential store queries, which may not support listing operations (e.g., OS keyring).

4. **Graceful Degradation**: If file is corrupted or missing, use sensible defaults ("default" account) rather than failing, ensuring system remains operational.

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
    
    #[error("No credentials found for account '{0}' on platform '{1}'")]
    NoCredentials(String, String),
    
    #[error("Multiple storage backends failed: {0}")]
    MultipleBackendFailures(String),
}
```

### Error Handling Strategy

1. **Invalid account name**: Validate on input, reject with clear message describing valid format
2. **Account not found**: Check registry before operations, suggest `plur-creds list` to see available accounts
3. **Credential not found**: Distinguish between "account exists but no credentials" vs "account doesn't exist"
4. **State file corruption**: Log warning, use defaults, attempt to recreate file
5. **Migration failures**: Report per-credential, don't fail entire migration, preserve original credentials
6. **Multiple backend failures**: Aggregate all failure messages and report together for debugging
7. **Reserved names**: Reject operations using reserved keywords with clear explanation

**Error Message Examples**:

```bash
# Invalid account name
$ plur-creds set nostr --account "test account"
Error: Invalid account name: 'test account'. Must be alphanumeric with hyphens/underscores, max 64 chars

# Account not found
$ plur-creds use nostr --account nonexistent
Error: Account 'nonexistent' not found for platform 'nostr'
Hint: Run 'plur-creds list --platform nostr' to see available accounts

# No credentials for account
$ plur-post "test" --account test
Error: No credentials found for account 'test' on platform 'nostr'
Hint: Run 'plur-creds set nostr --account test' to configure credentials

# State file corruption
Warning: Account state file corrupted, using defaults
Hint: Run 'plur-creds list' to verify your accounts

# Migration failure
Warning: Failed to migrate plurcast.nostr.private_key: Keyring unavailable
Original credentials preserved, you can retry migration later
```

**Design Decisions**:

1. **Descriptive Error Messages**: All errors include context about what failed and why, making it easier for users to understand and fix issues.

2. **Actionable Hints**: Error messages include hints about how to resolve the issue (e.g., which command to run).

3. **Graceful Degradation**: Non-critical errors (like state file corruption) are logged as warnings rather than failures, allowing the system to continue operating.

4. **Preserve Data on Failure**: Migration failures preserve original credentials, ensuring users don't lose data even if migration fails.

5. **Aggregate Multiple Failures**: When multiple storage backends fail, aggregate all error messages to provide complete debugging information.

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
2. Check if already migrated (exists in new format: `plurcast.{platform}.default.{key}`)
3. If not migrated:
   - Read credential from old namespace
   - Store in new namespace: `plurcast.{platform}.default.{key}`
   - Verify by retrieving and comparing
   - Log success
4. Keep old namespace for backward compatibility (don't delete)
5. If migration fails, log error and continue using old format

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
                    match self.retrieve_old_format(service, key) {
                        Ok(value) => {
                            match self.store_account(service, key, "default", &value) {
                                Ok(_) => {
                                    // Verify migration
                                    match self.retrieve_account(service, key, "default") {
                                        Ok(retrieved) if retrieved == value => {
                                            tracing::info!("Auto-migrated {}.{} to default account", service, key);
                                        }
                                        _ => {
                                            tracing::error!("Migration verification failed for {}.{}", service, key);
                                        }
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("Failed to migrate {}.{}: {}", service, key, e);
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to read old format {}.{}: {}", service, key, e);
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
}
```

**Design Decisions**:

1. **Non-Blocking Migration**: Migration failures don't prevent the system from working. Old format credentials remain accessible if migration fails.

2. **Verification Step**: After migration, verify that the new credential can be retrieved and matches the original value.

3. **Preserve Original**: Keep old format credentials to ensure backward compatibility and provide a fallback if migration fails.

4. **Detailed Logging**: Log all migration attempts, successes, and failures for debugging and user awareness.

### Manual Migration Command

**Command**: `plur-creds migrate --from-single-account`

**Purpose**: Explicit migration for users who want control over when and how migration happens

**Process**:
1. Scan for old format credentials across all storage backends
2. Display summary of credentials to be migrated
3. Ask for confirmation before proceeding
4. Migrate each credential to "default" account
5. Verify each migration by retrieving and comparing
6. Display detailed report of successful and failed migrations
7. Preserve original credentials (don't delete)
8. If no old-format credentials exist, display "Nothing to migrate"

**Implementation**:
```rust
pub struct MigrationReport {
    pub migrated: Vec<String>,
    pub failed: Vec<(String, String)>,  // (credential_id, error_message)
    pub skipped: Vec<String>,
}

impl CredentialManager {
    pub fn migrate_to_multi_account(&self) -> Result<MigrationReport> {
        let mut report = MigrationReport {
            migrated: Vec::new(),
            failed: Vec::new(),
            skipped: Vec::new(),
        };
        
        let platforms = vec![
            ("plurcast.nostr", "private_key"),
            ("plurcast.mastodon", "access_token"),
            ("plurcast.bluesky", "app_password"),
        ];
        
        // Scan for old format credentials
        let mut to_migrate = Vec::new();
        for (service, key) in &platforms {
            if self.exists_old_format(service, key)? {
                if self.exists_account(service, key, "default")? {
                    report.skipped.push(format!("{}.{}", service, key));
                } else {
                    to_migrate.push((service, key));
                }
            }
        }
        
        if to_migrate.is_empty() {
            return Ok(report);
        }
        
        // Display summary and ask for confirmation
        println!("The following credentials will be migrated to 'default' account:");
        for (service, key) in &to_migrate {
            println!("  - {}.{}", service, key);
        }
        println!("\nOriginal credentials will be preserved.");
        print!("Continue? [y/N]: ");
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            return Err(anyhow!("Migration cancelled by user"));
        }
        
        // Migrate each credential
        for (service, key) in to_migrate {
            let credential_id = format!("{}.{}", service, key);
            match self.retrieve_old_format(service, key) {
                Ok(value) => {
                    match self.store_account(service, key, "default", &value) {
                        Ok(_) => {
                            // Verify migration
                            match self.retrieve_account(service, key, "default") {
                                Ok(retrieved) if retrieved == value => {
                                    report.migrated.push(credential_id);
                                }
                                _ => {
                                    report.failed.push((credential_id, "Verification failed".to_string()));
                                }
                            }
                        }
                        Err(e) => {
                            report.failed.push((credential_id, e.to_string()));
                        }
                    }
                }
                Err(e) => {
                    report.failed.push((credential_id, e.to_string()));
                }
            }
        }
        
        Ok(report)
    }
}
```

**Design Decisions**:

1. **User Confirmation**: Require explicit confirmation before migrating to give users control over the process.

2. **Detailed Summary**: Show exactly what will be migrated before asking for confirmation.

3. **Comprehensive Reporting**: Provide detailed report of what succeeded, what failed, and what was skipped.

4. **Preserve Originals**: Never delete original credentials, ensuring users can roll back if needed.

5. **Graceful Failure Handling**: Continue migrating remaining credentials even if some fail, providing complete report at the end.

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
