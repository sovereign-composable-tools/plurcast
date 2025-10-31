# ADR 001: Multi-Account Credential Management

**Status**: Proposed  
**Date**: 2025-10-31  
**Deciders**: Plurcast Core Team  
**Context**: Version 0.3.0-alpha2 planning

## Context and Problem Statement

Users need to manage multiple accounts per platform (e.g., test vs prod Nostr keys, personal vs work accounts). Currently, `plur-creds set` overwrites existing credentials without confirmation, making it risky to switch between accounts.

### Current Limitations

1. **No account isolation**: Only one credential per platform can be stored
2. **Accidental overwrites**: `plur-creds set nostr` replaces test credentials with prod without warning
3. **No account switching**: Users cannot easily switch between test and production accounts
4. **Unsafe for development**: Developers risk overwriting test accounts during development

## Decision Drivers

- **Security**: Prevent accidental credential loss or mixing of test/prod credentials
- **UX**: Natural, intuitive commands that follow Unix conventions
- **Extensibility**: Design should work for all platforms (Nostr, Mastodon, Bluesky)
- **Backward compatibility**: Existing single-account setups should continue working
- **Future-proof**: Should support future features like team accounts, delegation, etc.

## Considered Options

### Option 1: Multi-Account Management with Named Accounts (RECOMMENDED)

Add `--account` flag to credential commands, with isolated keyring namespaces per account.

**Design**:
```bash
# Store credentials for different accounts
plur-creds set nostr --account test-account
plur-creds set nostr --account prod-account
plur-creds set nostr  # Uses 'default' account

# List all accounts for a platform
plur-creds list --platform nostr
# Output:
#   ✓ nostr (default): Private Key (stored in keyring)
#   ✓ nostr (test-account): Private Key (stored in keyring)
#   ✓ nostr (prod-account): Private Key (stored in keyring)

# Set active account (stored in config or state file)
plur-creds use nostr --account test-account

# Post using active account
plur-post "Hello from test account"

# Or specify account explicitly
plur-post "Hello from prod" --account prod-account
```

**Keyring namespace**: `plurcast.nostr.{account_name}`
- Default: `plurcast.nostr.default`
- Test: `plurcast.nostr.test-account`
- Prod: `plurcast.nostr.prod-account`

**Benefits**:
- ✅ Clean, intuitive UX familiar to developers (similar to `kubectl config use-context`, `git config`)
- ✅ Complete isolation: test/prod credentials never collide
- ✅ Backward compatible: omitting `--account` uses `default`
- ✅ Extensible to other platforms (Mastodon, Bluesky)
- ✅ Natural extension for future team/org accounts
- ✅ Explicit account switching reduces errors

**Trade-offs**:
- Requires additional state management (active account tracking)
- Slightly more complex implementation (namespace handling, account listing)
- Users must remember account names (mitigated by `plur-creds list`)

### Option 2: Separate Builds/Features with Hardcoded Namespaces

Build with feature flags or separate binaries for test vs prod.

```bash
# Separate builds
cargo build --features test
cargo build --features prod

# Or environment variable
PLUR_ENV=test plur-post "Test post"
PLUR_ENV=prod plur-post "Prod post"
```

**Keyring namespace**: `plurcast.{env}.nostr`
- Test: `plurcast.test.nostr`
- Prod: `plurcast.prod.nostr`

**Benefits**:
- Simple implementation
- Clear separation of test vs prod

**Trade-offs**:
- ❌ Limited to two environments (test, prod)
- ❌ Cannot handle personal vs work, or multiple test accounts
- ❌ Requires building/managing multiple binaries
- ❌ Less flexible for evolving use cases
- ❌ Still requires env var or feature flag management

### Option 3: Confirmation Prompts + `--overwrite` Flag

Add confirmation prompts and require explicit `--overwrite` to replace credentials.

```bash
plur-creds set nostr
# Output: Credentials already exist for nostr. Use --overwrite to replace.

plur-creds set nostr --overwrite
# Output: Overwrite existing nostr credentials? [y/N]:
```

**Benefits**:
- Prevents accidental overwrites
- Simple to implement

**Trade-offs**:
- ❌ Still only one account per platform
- ❌ Doesn't solve multi-account use case
- ❌ Adds friction to legitimate credential updates
- ❌ Workaround, not a solution

## Decision

**Chosen option**: Option 1 - Multi-Account Management with Named Accounts

This is the most flexible, user-friendly, and future-proof solution. It addresses the root cause (lack of account isolation) rather than just adding safety guards.

## Implementation Plan

### Phase 1: Core Multi-Account Support (0.3.0-alpha2)

#### 1. Update CredentialStore Trait

```rust
pub trait CredentialStore: Send + Sync {
    fn store(&self, service: &str, key: &str, account: &str, value: &str) -> Result<()>;
    fn retrieve(&self, service: &str, key: &str, account: &str) -> Result<String>;
    fn delete(&self, service: &str, key: &str, account: &str) -> Result<()>;
    fn exists(&self, service: &str, key: &str, account: &str) -> Result<bool>;
    fn list_accounts(&self, service: &str, key: &str) -> Result<Vec<String>>;
    // ... existing methods ...
}
```

#### 2. Keyring Namespace Format

```rust
fn keyring_key(service: &str, key: &str, account: &str) -> String {
    format!("{}.{}.{}", service, account, key)
}

// Examples:
// - Default: "plurcast.nostr.default.private_key"
// - Test: "plurcast.nostr.test-account.private_key"
// - Prod: "plurcast.nostr.prod-account.private_key"
```

#### 3. CLI Changes

Add `--account` flag to all credential commands:
```rust
#[derive(Parser)]
struct SetArgs {
    platform: String,
    #[arg(long, default_value = "default")]
    account: String,
    #[arg(long)]
    stdin: bool,
}
```

#### 4. Active Account Tracking

Store active account per platform in config or state file:
```toml
# ~/.config/plurcast/accounts.toml
[active]
nostr = "test-account"
mastodon = "default"
bluesky = "default"
```

#### 5. Add `plur-creds use` Command

```bash
plur-creds use nostr --account prod-account
# Sets prod-account as active for nostr platform
```

### Phase 2: Integration with Posting (0.3.0-alpha2)

#### 1. Add `--account` to plur-post

```bash
plur-post "Test message" --account test-account
plur-post "Prod message"  # Uses active account from config
```

#### 2. Update Platform Creation

Modify `create_platforms()` to accept account parameter and use correct namespace when retrieving credentials.

### Phase 3: Testing (0.3.0-alpha2)

1. **Unit tests**: Namespace derivation is pure and deterministic
2. **Integration tests**: Multi-account CRUD operations
3. **Process persistence tests**: Windows-specific, spawn child process to verify keyring
4. **CLI tests**: `--account` flag behavior, active account switching

## Consequences

### Positive

- Users can safely manage multiple accounts without fear of overwrites
- Natural UX that follows familiar patterns (kubectl, git config)
- Extensible to future features (team accounts, delegation)
- Backward compatible with existing single-account setups
- Security improvement: test/prod isolation reduces risk

### Negative

- Additional complexity in credential management code
- Need to track active account state
- Users must learn new `--account` flag (mitigated by good defaults and docs)
- Migration path needed for users with existing credentials

### Neutral

- Documentation needs update to explain multi-account model
- Examples and guides should demonstrate common workflows

## Migration Strategy

### For Existing Users

1. On first upgrade to 0.3.0-alpha2, existing credentials are automatically associated with `default` account
2. No action required unless user wants multiple accounts
3. If credentials exist at old namespace, auto-migrate:
   - Old: `plurcast.nostr.private_key`
   - New: `plurcast.nostr.default.private_key`

### Migration Command (Optional)

```bash
plur-creds migrate --from-single-account
# Migrates existing credentials to default account namespace
```

## References

- Issue: Keyring persistence and credential overwrites
- Related ADR: (none yet)
- Inspiration: `kubectl config use-context`, `git config`, `aws configure --profile`
