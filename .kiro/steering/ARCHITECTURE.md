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

## Security Considerations

- Credentials stored in separate files (not in main config)
- File permissions: 600 for sensitive files
- No credentials in database
- Clear documentation about key management
- Support for system keyring (future)

---

**Version**: 0.1.0-alpha
**Last Updated**: 2025-10-05
**Status**: Active Development - Phase 1 (Foundation) ~85% Complete
