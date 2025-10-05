# Design Document: Foundation Alpha MVP

## Overview

The Foundation Alpha MVP establishes the core infrastructure for Plurcast: a SQLite-backed data layer, TOML-based configuration system, and basic posting capability to Nostr. This design prioritizes Unix philosophy principles, agent-friendliness, and extensibility for future platforms.

The implementation uses mature Rust libraries: `sqlx` for database operations, `nostr-sdk` for Nostr protocol support, `clap` for CLI parsing, and `serde`/`toml` for configuration management.

## Architecture

### High-Level Component Structure

```
┌─────────────────────────────────────────────────────────────┐
│                      plur-post (binary)                      │
│  ┌────────────┐  ┌──────────────┐  ┌──────────────────┐    │
│  │ CLI Parser │→ │ Post Handler │→ │ Output Formatter │    │
│  └────────────┘  └──────────────┘  └──────────────────┘    │
└────────────┬────────────────────────────────────────────────┘
             │
             ↓
┌─────────────────────────────────────────────────────────────┐
│                    libplurcast (library)                     │
│  ┌──────────────┐  ┌──────────────┐  ┌─────────────────┐   │
│  │   Config     │  │   Database   │  │   Platforms     │   │
│  │   Manager    │  │   Manager    │  │   (Trait)       │   │
│  └──────────────┘  └──────────────┘  └─────────────────┘   │
│                                        ┌─────────────────┐   │
│                                        │ NostrPlatform   │   │
│                                        └─────────────────┘   │
└─────────────────────────────────────────────────────────────┘
             │                              │
             ↓                              ↓
┌──────────────────────┐      ┌──────────────────────────────┐
│  ~/.config/plurcast/ │      │ ~/.local/share/plurcast/     │
│  - config.toml       │      │ - posts.db (SQLite)          │
│  - nostr.keys        │      └──────────────────────────────┘
└──────────────────────┘
```

### Data Flow: Posting a Message

```
User Input (stdin/args)
    ↓
plur-post CLI Parser
    ↓
Load Configuration (config.toml)
    ↓
Initialize Database Connection
    ↓
Authenticate with Platform (Nostr)
    ↓
Create Post Record in DB (status: pending)
    ↓
Post to Nostr Relays
    ↓
Update Post Record (status: posted/failed)
    ↓
Output Result (platform:post_id)
    ↓
Exit with appropriate code
```

## Components and Interfaces

### 1. Configuration Manager (`src/config.rs`)

**Responsibility**: Load, parse, and validate configuration from TOML files and environment variables.

**Key Types**:
```rust
#[derive(Debug, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub nostr: Option<NostrConfig>,
    pub mastodon: Option<MastodonConfig>,
    pub bluesky: Option<BlueskyConfig>,
    pub defaults: DefaultsConfig,
}

#[derive(Debug, Deserialize)]
pub struct NostrConfig {
    pub enabled: bool,
    pub keys_file: PathBuf,
    pub relays: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub path: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct DefaultsConfig {
    pub platforms: Vec<String>,
}
```

**Key Functions**:
```rust
pub fn load_config() -> Result<Config, ConfigError>
pub fn resolve_config_path() -> PathBuf
pub fn resolve_data_path() -> PathBuf
pub fn create_default_config(path: &Path) -> Result<(), ConfigError>
```

**Implementation Details**:
- Use `dirs` crate for XDG Base Directory support
- Check `PLURCAST_CONFIG` environment variable first
- Fall back to `~/.config/plurcast/config.toml`
- Use `PLURCAST_DB_PATH` for database location override
- Expand `~` in paths using `shellexpand` crate
- Validate that required fields are present for enabled platforms

### 2. Database Manager (`src/db.rs`)

**Responsibility**: Initialize database, run migrations, and provide CRUD operations for posts and records.

**Key Types**:
```rust
pub struct Post {
    pub id: String,           // UUIDv4
    pub content: String,
    pub created_at: i64,      // Unix timestamp
    pub scheduled_at: Option<i64>,
    pub status: PostStatus,
    pub metadata: Option<String>, // JSON
}

pub enum PostStatus {
    Pending,
    Posted,
    Failed,
}

pub struct PostRecord {
    pub id: i64,
    pub post_id: String,
    pub platform: String,
    pub platform_post_id: Option<String>,
    pub posted_at: Option<i64>,
    pub success: bool,
    pub error_message: Option<String>,
}

pub struct Database {
    pool: SqlitePool,
}
```

**Key Functions**:
```rust
pub async fn new(db_path: &Path) -> Result<Database, DbError>
pub async fn initialize(&self) -> Result<(), DbError>
pub async fn create_post(&self, content: &str) -> Result<Post, DbError>
pub async fn update_post_status(&self, post_id: &str, status: PostStatus) -> Result<(), DbError>
pub async fn create_post_record(&self, record: &PostRecord) -> Result<(), DbError>
pub async fn get_post(&self, post_id: &str) -> Result<Option<Post>, DbError>
```

**Implementation Details**:
- Use `sqlx` with compile-time checked queries
- Store migrations in `migrations/` directory
- Use `sqlx::migrate!()` macro for automatic migration application
- Generate UUIDs with `uuid` crate (v4)
- Store timestamps as Unix epoch (i64)
- Use transactions for multi-step operations
- Create database file with 644 permissions, parent directories as needed

### 3. Platform Trait (`src/platforms/mod.rs`)

**Responsibility**: Define common interface for all social media platforms.

**Trait Definition**:
```rust
#[async_trait]
pub trait Platform: Send + Sync {
    /// Authenticate with the platform
    async fn authenticate(&mut self) -> Result<(), PlatformError>;
    
    /// Post content to the platform
    async fn post(&self, content: &str) -> Result<String, PlatformError>;
    
    /// Validate content before posting
    fn validate_content(&self, content: &str) -> Result<(), PlatformError>;
    
    /// Get platform name
    fn name(&self) -> &str;
}

#[derive(Debug, thiserror::Error)]
pub enum PlatformError {
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),
    
    #[error("Content validation failed: {0}")]
    ValidationError(String),
    
    #[error("Posting failed: {0}")]
    PostingError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
}
```

### 4. Nostr Platform Implementation (`src/platforms/nostr.rs`)

**Responsibility**: Implement Platform trait for Nostr protocol using `nostr-sdk`.

**Key Types**:
```rust
pub struct NostrPlatform {
    client: Client,
    keys: Keys,
    relays: Vec<String>,
    authenticated: bool,
}
```

**Key Functions**:
```rust
pub fn new(config: &NostrConfig) -> Result<Self, PlatformError>
async fn load_keys(keys_file: &Path) -> Result<Keys, PlatformError>
```

**Implementation Details**:
- Use `nostr_sdk::Client` for relay management
- Support both hex and bech32 (nsec) key formats
- Read keys from file specified in config
- Connect to all configured relays on authenticate()
- Create kind 1 (text note) events for posts
- Return note ID in bech32 format (note1...)
- Handle relay connection failures gracefully (succeed if any relay accepts)
- Set reasonable timeout (10 seconds) for posting
- Validate content length (no hard limit for Nostr, but warn if > 280 chars)

### 5. Error Types (`src/lib.rs`)

**Responsibility**: Define application-level error types with appropriate exit codes.

**Error Hierarchy**:
```rust
#[derive(Debug, thiserror::Error)]
pub enum PlurcastError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
    
    #[error("Database error: {0}")]
    Database(#[from] DbError),
    
    #[error("Platform error: {0}")]
    Platform(#[from] PlatformError),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

impl PlurcastError {
    pub fn exit_code(&self) -> i32 {
        match self {
            PlurcastError::InvalidInput(_) => 3,
            PlurcastError::Config(_) | PlurcastError::Platform(PlatformError::AuthenticationError(_)) => 2,
            PlurcastError::Platform(_) => 1,
            PlurcastError::Database(_) => 1,
        }
    }
}
```

### 6. CLI Binary (`bin/plur-post.rs`)

**Responsibility**: Parse command-line arguments, orchestrate posting workflow, format output.

**CLI Structure**:
```rust
#[derive(Parser)]
#[command(name = "plur-post")]
#[command(about = "Post content to decentralized social platforms")]
struct Cli {
    /// Content to post (reads from stdin if not provided)
    content: Option<String>,
    
    /// Specific platforms to post to (comma-separated)
    #[arg(short, long)]
    platform: Option<String>,
    
    /// Save as draft without posting
    #[arg(short, long)]
    draft: bool,
    
    /// Output format (text or json)
    #[arg(short, long, default_value = "text")]
    format: OutputFormat,
    
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

enum OutputFormat {
    Text,
    Json,
}
```

**Main Flow**:
```rust
#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    
    // Setup logging
    init_logging(cli.verbose);
    
    // Get content from args or stdin
    let content = get_content(&cli)?;
    
    // Load configuration
    let config = load_config()?;
    
    // Initialize database
    let db = Database::new(&config.database.path).await?;
    db.initialize().await?;
    
    // Create post record
    let post = db.create_post(&content).await?;
    
    // Determine platforms
    let platforms = determine_platforms(&cli, &config)?;
    
    // Post to each platform
    let results = post_to_platforms(&platforms, &content, &post.id, &db).await;
    
    // Output results
    output_results(&results, &cli.format);
    
    // Exit with appropriate code
    std::process::exit(determine_exit_code(&results));
}
```

## Data Models

### Database Schema

```sql
-- Posts table
CREATE TABLE IF NOT EXISTS posts (
    id TEXT PRIMARY KEY,
    content TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    scheduled_at INTEGER,
    status TEXT NOT NULL DEFAULT 'pending',
    metadata TEXT,
    CHECK (status IN ('pending', 'posted', 'failed'))
);

CREATE INDEX idx_posts_created_at ON posts(created_at);
CREATE INDEX idx_posts_status ON posts(status);

-- Post records table
CREATE TABLE IF NOT EXISTS post_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    post_id TEXT NOT NULL,
    platform TEXT NOT NULL,
    platform_post_id TEXT,
    posted_at INTEGER,
    success INTEGER NOT NULL DEFAULT 0,
    error_message TEXT,
    FOREIGN KEY (post_id) REFERENCES posts(id) ON DELETE CASCADE,
    CHECK (success IN (0, 1))
);

CREATE INDEX idx_post_records_post_id ON post_records(post_id);
CREATE INDEX idx_post_records_platform ON post_records(platform);

-- Platforms table
CREATE TABLE IF NOT EXISTS platforms (
    name TEXT PRIMARY KEY,
    enabled INTEGER NOT NULL DEFAULT 1,
    config TEXT,
    CHECK (enabled IN (0, 1))
);
```

### Configuration File Format

```toml
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

[defaults]
platforms = ["nostr"]
```

### Nostr Keys File Format

```
# Hex format (64 characters)
a1b2c3d4e5f6...

# OR bech32 format
nsec1...
```

## Error Handling

### Error Propagation Strategy

1. **Library Level**: Use `Result<T, SpecificError>` for all fallible operations
2. **Binary Level**: Convert to `PlurcastError` and extract exit code
3. **User-Facing**: Write clear error messages to stderr with context

### Error Message Format

```
Error: Authentication failed: Could not read Nostr keys file
  Caused by: No such file or directory: ~/.config/plurcast/nostr.keys
  
  Hint: Create a keys file with your Nostr private key (hex or nsec format)
```

### Exit Code Mapping

- **0**: Success on all platforms
- **1**: Posting failed on at least one platform
- **2**: Authentication error (missing/invalid credentials)
- **3**: Invalid input (empty content, malformed arguments)

## Testing Strategy

### Unit Tests

**Configuration Module**:
- Test XDG path resolution
- Test environment variable overrides
- Test TOML parsing with valid/invalid configs
- Test default config generation

**Database Module**:
- Test database initialization and migrations
- Test CRUD operations for posts and records
- Test transaction rollback on errors
- Test concurrent access patterns

**Platform Module**:
- Test Nostr key parsing (hex and bech32)
- Test content validation
- Mock relay connections for posting tests
- Test error handling for network failures

### Integration Tests

**End-to-End Posting**:
- Test posting with valid configuration
- Test posting with missing configuration
- Test posting with invalid keys
- Test stdin input vs argument input
- Test output formatting (text and JSON)

**Database Persistence**:
- Test that posts are recorded correctly
- Test that post_records track platform attempts
- Test status updates after posting

### Test Utilities

```rust
// Test helpers
pub fn create_test_config() -> Config { ... }
pub fn create_test_db() -> Database { ... }
pub fn mock_nostr_client() -> NostrPlatform { ... }
```

## Dependencies

```toml
[dependencies]
# Platform clients
nostr-sdk = "0.35"

# Database
sqlx = { version = "0.8", features = ["sqlite", "runtime-tokio", "migrate"] }

# Async runtime
tokio = { version = "1", features = ["full"] }

# CLI
clap = { version = "4.5", features = ["derive"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Utilities
uuid = { version = "1.10", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
dirs = "5.0"
shellexpand = "3.1"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Async trait support
async-trait = "0.1"

[dev-dependencies]
tempfile = "3.10"
mockall = "0.12"
```

## Security Considerations

### Credential Storage

- Keys stored in separate files, not in config.toml
- Files created with 600 permissions (owner read/write only)
- No credentials logged or included in error messages
- Database does not store private keys

### Input Validation

- Sanitize content before posting (no control characters)
- Validate platform names against whitelist
- Limit content length to prevent memory issues

### Network Security

- Use WSS (WebSocket Secure) for Nostr relays
- Validate relay URLs before connecting
- Set connection timeouts to prevent hanging

## Performance Considerations

### Database

- Use connection pooling (sqlx default)
- Create indexes on frequently queried columns
- Use prepared statements (sqlx default)

### Nostr Posting

- Post to relays concurrently (nostr-sdk handles this)
- Don't wait for all relays to respond (succeed if any accepts)
- Set reasonable timeout (10 seconds)

### Startup Time

- Lazy initialization where possible
- Don't connect to relays until posting
- Cache configuration in memory

## Future Extensibility

### Adding New Platforms

1. Implement `Platform` trait in `src/platforms/<platform>.rs`
2. Add configuration struct to `Config`
3. Register platform in platform factory function
4. Add integration tests

### Adding New Commands

1. Create new binary in `bin/`
2. Reuse `libplurcast` components
3. Follow same CLI patterns (clap, exit codes)
4. Share database and configuration

### Adding Features

- Media attachments: Add `attachments` field to Post metadata
- Threading: Add `reply_to` field to Post
- Scheduling: Already supported via `scheduled_at` field
- Analytics: Query post_records table for statistics

## Open Questions

None - design is complete and ready for implementation.
