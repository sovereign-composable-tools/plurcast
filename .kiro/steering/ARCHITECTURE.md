# Plurcast: Architecture

**Related Documentation**:
- [Vision](./VISION.md) - Philosophy and design principles
- [Roadmap](./ROADMAP.md) - Development phases and progress
- [Tools](./TOOLS.md) - Tool specifications and usage
- [Future](./FUTURE.md) - Extensibility and future plans

---

## Architecture Overview

### Tool Suite

```
plurcast/
├── plur-post          # Post content to one or more platforms
├── plur-queue         # Schedule posts for later
├── plur-send          # Daemon that processes queue
├── plur-history       # Query posting history
├── plur-import        # Import from platform exports
├── plur-export        # Export posts to various formats
└── libplurcast        # Shared library (internal)
```

### Data Storage

**Location**: `~/.local/share/plurcast/`
- `posts.db` - SQLite database
- `embeddings.db` - Vector embeddings (optional, future)

**Configuration**: `~/.config/plurcast/`
- `config.toml` - Platform credentials and preferences
- `*.key` - Platform-specific authentication files

## Platform Support (Alpha Release)

### Nostr
**Library**: `nostr-sdk` v0.35+ (rust-nostr)
- **Status**: Alpha, actively maintained with strong ecosystem adoption
- **Features**: Complete protocol implementation, relay management, cryptographic key handling
- **Maturity**: High adoption in Nostr ecosystem, production-ready core
- **Key Capabilities**:
  - Multiple relay support
  - Event signing and verification
  - NIP (Nostr Implementation Possibilities) support
  - Key management (hex/bech32)

### Bluesky
**Library**: `atrium-api` v0.24+ (atrium-rs)
- **Status**: Active development, protocol stabilizing
- **Features**: Complete AT Protocol implementation via XRPC
- **Maturity**: Growing ecosystem, protocol reached stability in 2024-2025
- **Key Capabilities**:
  - AT Protocol (authenticated transfer protocol)
  - DID-based identity
  - Lexicon schema support
  - PDS (Personal Data Server) federation

### Mastodon
**Library**: `megalodon` v0.14+
- **Status**: Stable, well-maintained
- **Features**: ActivityPub/Mastodon API with multi-platform Fediverse support
- **Maturity**: Battle-tested across multiple Fediverse implementations
- **Key Capabilities**:
  - Supports Mastodon, Pleroma, Friendica, Firefish, GoToSocial, Akkoma
  - Unified API across platforms
  - OAuth authentication
  - Media upload support

## Core Components

### Database Schema

```sql
-- Posts authored by user
CREATE TABLE posts (
    id TEXT PRIMARY KEY,              -- UUIDv4
    content TEXT NOT NULL,
    created_at INTEGER NOT NULL,      -- Unix timestamp
    scheduled_at INTEGER,             -- NULL = post immediately
    status TEXT DEFAULT 'pending',    -- pending, posted, failed
    metadata TEXT                     -- JSON: tags, attachments, etc.
);

-- Platform-specific post records
CREATE TABLE post_records (
    id INTEGER PRIMARY KEY,
    post_id TEXT NOT NULL,
    platform TEXT NOT NULL,           -- nostr, mastodon, bluesky
    platform_post_id TEXT,            -- Platform's ID for the post
    posted_at INTEGER,
    success INTEGER DEFAULT 0,        -- 0 or 1
    error_message TEXT,
    FOREIGN KEY (post_id) REFERENCES posts(id)
);

-- Platform configurations
CREATE TABLE platforms (
    name TEXT PRIMARY KEY,
    enabled INTEGER DEFAULT 1,
    config TEXT                       -- JSON: instance URLs, relay lists, etc.
);
```

### Configuration Format

```toml
# ~/.config/plurcast/config.toml

[database]
path = "~/.local/share/plurcast/posts.db"

[nostr]
enabled = true
keys_file = "~/.config/plurcast/nostr.keys"
relays = [
    "wss://relay.damus.io",
    "wss://nos.lol",
    "wss://relay.nostr.band"
]

[mastodon]
enabled = true
instance = "mastodon.social"
token_file = "~/.config/plurcast/mastodon.token"

[bluesky]
enabled = true
handle = "user.bsky.social"
auth_file = "~/.config/plurcast/bluesky.auth"

[defaults]
# Which platforms to post to by default (can override with --platform flag)
platforms = ["nostr", "mastodon", "bluesky"]
```

## Technology Stack

### Core Dependencies

```toml
[dependencies]
# Platform clients - mature, open-source libraries
nostr-sdk = "0.35"           # Nostr protocol (rust-nostr)
megalodon = "0.14"           # Mastodon/Fediverse (ActivityPub)
atrium-api = "0.24"          # Bluesky AT Protocol

# Data & persistence
sqlx = { version = "0.8", features = ["sqlite", "runtime-tokio"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"

# Async runtime
tokio = { version = "1", features = ["full"] }

# CLI & utilities
clap = { version = "4.5", features = ["derive"] }
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.10", features = ["v4", "serde"] }

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"
```

### Project Structure

```
plurcast/
├── Cargo.toml
├── README.md
├── LICENSE
├── src/
│   ├── lib.rs              # Shared library code
│   ├── db.rs               # Database operations
│   ├── config.rs           # Configuration management
│   ├── platforms/
│   │   ├── mod.rs
│   │   ├── nostr.rs        # Nostr client wrapper
│   │   ├── mastodon.rs     # Mastodon client wrapper
│   │   └── bluesky.rs      # Bluesky client wrapper
│   └── types.rs            # Shared types
├── bin/
│   ├── plur-post.rs
│   ├── plur-queue.rs
│   ├── plur-send.rs
│   ├── plur-history.rs
│   ├── plur-import.rs
│   └── plur-export.rs
├── migrations/
│   └── 001_initial.sql     # SQLx migrations
└── tests/
    └── integration.rs
```

## Design Rationale

### Why SQLite?
- Zero-configuration
- Fast for local workloads
- ACID guarantees
- Built-in full-text search
- Easy backup (single file)

### Why Separate Binaries?
- Unix composability
- Test components independently
- Users install only what they need
- Clear separation of concerns
- Easier to understand and maintain

### Why Not a Monolithic CLI?
- Subcommands create complexity
- Harder to compose with Unix tools
- Violates "do one thing well"
- Each tool has focused documentation

### Configuration Philosophy
- XDG Base Directory compliance
- Environment variables for overrides
- Sensible defaults
- No required configuration for basic use

## Credential Storage Architecture

Plurcast implements a layered credential storage system with multiple backends and automatic fallback.

### CredentialStore Trait

All storage backends implement the `CredentialStore` trait:

```rust
pub trait CredentialStore: Send + Sync {
    fn store(&self, service: &str, key: &str, value: &str) -> Result<()>;
    fn retrieve(&self, service: &str, key: &str) -> Result<String>;
    fn delete(&self, service: &str, key: &str) -> Result<()>;
    fn exists(&self, service: &str, key: &str) -> Result<bool>;
    fn backend_name(&self) -> &str;
}
```

### Storage Backends

#### 1. KeyringStore (Primary)

**Implementation**: Uses `keyring` crate for OS-native secure storage

**Platform Support**:
- **macOS**: Keychain via Security framework
- **Windows**: Credential Manager via Windows API
- **Linux**: Secret Service (GNOME Keyring/KWallet) via D-Bus

**Service Naming**: `plurcast.{platform}` (e.g., "plurcast.nostr")
**Key Naming**: `{credential_type}` (e.g., "private_key", "access_token")

**Error Handling**: Returns `CredentialError::KeyringUnavailable` when OS keyring not accessible

#### 2. EncryptedFileStore (Fallback)

**Implementation**: Uses `age` crate for file encryption

**Technical Details**:
- **Encryption**: ChaCha20-Poly1305 (authenticated encryption)
- **Key Derivation**: scrypt (work factor: N=2^18, r=8, p=1)
- **File Format**: age v1 format with armor encoding (.age files)
- **Location**: `~/.config/plurcast/credentials/*.age`
- **Permissions**: 600 (owner read/write only)

**Master Password**:
- Minimum 8 characters (enforced)
- Stored only in memory during session
- Can be provided via environment variable or interactive prompt

**File Naming**: `{service}.{key}.age` (e.g., `plurcast.nostr.private_key.age`)

#### 3. PlainFileStore (Legacy)

**Implementation**: Plain text files with file permissions only

**Purpose**: Backward compatibility with Phase 1 credential files

**File Mapping**:
- `plurcast.nostr/private_key` → `nostr.keys`
- `plurcast.mastodon/access_token` → `mastodon.token`
- `plurcast.bluesky/app_password` → `bluesky.auth`

**Deprecation**: Logs warning on first use, marked as deprecated

### CredentialManager Facade

The `CredentialManager` provides a unified interface with automatic fallback:

**Fallback Logic**:
1. Try KeyringStore (if configured and available)
2. Try EncryptedFileStore (if master password set or can prompt)
3. Fall back to PlainFileStore (with warnings)

**Operations**:
- `store()`: Uses first available store
- `retrieve()`: Tries stores in order until success
- `delete()`: Removes from all stores
- `exists()`: Checks all stores

**Configuration**:
```toml
[credentials]
storage = "keyring"  # or "encrypted" or "plain"
path = "~/.config/plurcast/credentials"
```

### Migration Strategy

The `CredentialManager` supports migrating from plain text to secure storage:

```rust
pub struct MigrationReport {
    pub migrated: Vec<String>,
    pub failed: Vec<(String, String)>,
    pub skipped: Vec<String>,
}
```

**Migration Process**:
1. Detect plain text credential files
2. Read credentials from plain text
3. Store in secure storage (keyring or encrypted)
4. Verify by retrieving and comparing
5. Optionally delete plain text files after confirmation

### Security Properties

**What's Protected**:
- Nostr private keys (hex or nsec format)
- Mastodon access tokens
- Bluesky app passwords

**Protection Mechanisms**:
- OS keyring: System-level encryption
- Encrypted files: age encryption with user password
- Plain text: File permissions (600) only

**What's Not Sensitive** (stored in config.toml):
- Mastodon instance URLs
- Bluesky handles
- Nostr relay URLs
- Database paths

**Security Best Practices**:
- Credentials never appear in logs
- Error messages don't include credential values
- Memory cleared on exit (best effort)
- File permissions enforced (600 on Unix)

### Error Handling

**CredentialError Types**:
- `NotFound`: Credential doesn't exist
- `KeyringUnavailable`: OS keyring not accessible
- `MasterPasswordNotSet`: Encrypted storage requires password
- `WeakPassword`: Password doesn't meet requirements
- `DecryptionFailed`: Incorrect password or corrupted file
- `NoStoreAvailable`: No storage backend available
- `MigrationFailed`: Migration encountered errors

**Integration**: All credential errors are wrapped in `PlurcastError::Credential`

### Platform Client Integration

Platform clients receive a reference to `CredentialManager`:

```rust
impl NostrClient {
    pub fn new(credentials: &CredentialManager) -> Result<Self> {
        let private_key = credentials.retrieve("plurcast.nostr", "private_key")?;
        // ... initialize client
    }
}
```

**Benefits**:
- Centralized credential management
- Automatic fallback handling
- Consistent error handling
- Easy testing with mock stores

### Command-Line Tools

**plur-creds**: Credential management CLI
- `set <platform>`: Store credentials
- `list`: Show configured platforms
- `delete <platform>`: Remove credentials
- `test <platform>`: Verify authentication
- `migrate`: Migrate from plain text
- `audit`: Security audit

**plur-setup**: Interactive setup wizard
- Choose storage backend
- Configure platform credentials
- Test authentication
- Save configuration

## Security Considerations

### Threat Model

**Protected Against**:
- Casual file system access
- Credential theft via file system
- Accidental credential exposure

**Not Protected Against**:
- Root/administrator access
- Memory dumps
- Malware/keyloggers
- Physical access to unlocked system

### Best Practices

1. **Use OS Keyring** - Most secure option
2. **Set File Permissions** - Ensure 600 on all credential files
3. **Use Strong Master Password** - If using encrypted storage
4. **Audit Regularly** - Run `plur-creds audit`
5. **Migrate from Plain Text** - Use `plur-creds migrate`

### Compliance

- **Encryption**: age (modern, secure), OS keyrings (platform-specific)
- **File Permissions**: Unix 600 (owner read/write only)
- **Password Standards**: Minimum 8 characters (NIST SP 800-63B guidelines)
- **Audit**: Credential access logged (service/key, not values)

For detailed security information, see [SECURITY.md](../../SECURITY.md).

---

**Version**: 0.2.0-alpha
**Last Updated**: 2025-10-07
**Status**: Active Development - Phase 2 (Multi-Platform) with Secure Credentials
