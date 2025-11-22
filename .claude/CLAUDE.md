# CLAUDE.md - Development Guide for Plurcast

You are an expert Rust engineer working on **Plurcast**, a collection of Unix command-line tools for posting to decentralized social media platforms. You will build production-ready, secure, and well-tested code following strict development standards.

**Language**: Rust (Edition 2021)

---

## 🎯 Core Development Principles

### 1. Test-Driven Development (TDD) - MANDATORY

Follow the Red-Green-Refactor cycle:
- **RED**: Write a failing test first
- **GREEN**: Write minimal code to pass the test
- **REFACTOR**: Improve code while keeping tests passing

**Requirements**:
- Every function must have corresponding tests
- Achieve comprehensive test coverage including edge cases and error scenarios
- Before committing: `cargo test`, `cargo clippy`, `cargo fmt --check` must all pass
- Use `tempfile::TempDir` for tests that create files/directories
- Tests must cover: success cases, edge cases, error scenarios, boundary conditions

**Test Organization**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_case() { /* ... */ }

    #[test]
    fn test_edge_case_empty_input() { /* ... */ }

    #[test]
    fn test_error_invalid_format() { /* ... */ }

    #[tokio::test]
    async fn test_async_operation() { /* ... */ }
}
```

### 2. Security-First Mindset

**Critical Security Rules**:
- ❌ **NEVER commit credentials** - .gitignore covers key files, .env, etc.
- ✅ **Use environment variables** for secrets (PLURCAST_*, config paths)
- ✅ **Validate all inputs** - See `MAX_CONTENT_LENGTH = 100_000` in plur-post
- ✅ **Sanitize outputs** - Never expose secrets in error messages or logs
- ✅ **Secure defaults** - File permissions 600 for credentials, 644 for config
- ✅ **Memory protection** - Use `secrecy::Secret<T>` and `zeroize::Zeroize` for keys

**Input Validation Pattern**:
```rust
const MAX_CONTENT_LENGTH: usize = 100_000; // 100KB

fn validate_input(content: &str) -> Result<()> {
    if content.is_empty() {
        return Err(PlurcastError::InvalidInput("Content cannot be empty".into()));
    }
    if content.len() > MAX_CONTENT_LENGTH {
        return Err(PlurcastError::InvalidInput(format!(
            "Content too large: {} bytes (maximum: {} bytes)",
            content.len(), MAX_CONTENT_LENGTH
        )));
    }
    Ok(())
}
```

**Credential Security**:
- Store private keys in separate files (NOT in config.toml)
- Use `Secret<String>` from `secrecy` crate for in-memory keys
- Implement `Zeroize` for sensitive data structs
- Log only key lengths, NEVER log actual keys: `tracing::debug!("Key length: {}", key.len())`

### 3. Functional Programming Patterns

**Prefer**:
- Pure functions (no side effects, deterministic output)
- Immutable data structures
- Function composition over large, complex functions
- Explicit error handling with `Result<T, E>`

**Pattern Examples**:
```rust
// Pure function - same input always produces same output
fn calculate_pow_nonce(content: &str, difficulty: u8) -> u64 {
    // No side effects, deterministic
}

// Compose small functions
fn process_post(content: &str) -> Result<PostResponse> {
    let validated = validate_content(content)?;
    let formatted = format_for_platform(validated)?;
    publish_to_platforms(formatted)
}

// Avoid mutable global state - use Arc<RwLock<T>> if needed
```

### 4. Code Quality Standards

**Function Size**:
- Ideal: 5-15 lines
- Maximum: 50 lines
- If longer, refactor into smaller composable functions

**File Size**:
- Maximum: 500 lines
- Split larger modules into submodules (e.g., `platforms/nostr/mod.rs`, `platforms/nostr/pow.rs`)

**Naming Conventions**:
- Use clear, descriptive names
- Functions: `verb_noun` (e.g., `create_platform`, `validate_content`)
- Types: `PascalCase` (e.g., `PostingService`, `PlatformError`)
- Constants: `SCREAMING_SNAKE_CASE` (e.g., `MAX_CONTENT_LENGTH`)

**Documentation**:
- Document "why", not "what" (code shows "what")
- Add examples for public APIs
- Use `//!` for module docs, `///` for item docs

**Zero Warnings Policy**:
- Treat all warnings as errors
- Fix or explicitly allow with `#[allow(clippy::lint_name)]` and comment why

### 5. Observability and Debugging

**Structured Logging** (using centralized `libplurcast::logging`):
```rust
use tracing::{info, debug, error, instrument};

#[instrument(skip(service))]
async fn post_to_platform(service: &PostingService, content: &str) -> Result<String> {
    debug!(content_len = content.len(), "Starting post operation");

    let result = service.post(content).await?;

    info!(
        post_id = %result.post_id,
        platform = %result.platform,
        "Post published successfully"
    );

    Ok(result.post_id)
}
```

**Error Context**:
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PlurcastError {
    #[error("Failed to post to {platform}: {source}")]
    PlatformError {
        platform: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}
```

**Logging Formats**:
- `--log-format text` - Default, human-readable
- `--log-format json` - Machine-parseable for production monitoring
- `--log-format pretty` - Colored, detailed for development

### 6. Multi-Option Analysis

Before implementing any feature, analyze in `<implementation_analysis>` tags:

1. **Requirements**: Write out core requirements and constraints verbatim
2. **Approaches**: List 3+ different architectural approaches with pros/cons
3. **Security**: Enumerate security considerations (input validation, secrets, errors, privileges)
4. **Test Plan**: Detail test cases, edge cases, error scenarios
5. **Observability**: Plan logging, error context, performance metrics
6. **Recommendation**: Choose best approach with clear rationale

**Example**:
```
<implementation_analysis>
Requirements:
- Add support for Bluesky platform
- Must follow existing Platform trait pattern
- Needs OAuth authentication
- Character limit: 300 chars

Approaches:
1. Use official AT Protocol SDK
   Pros: Well-maintained, full feature support
   Cons: Heavy dependency, may have API changes

2. Direct HTTP API calls
   Pros: Lightweight, full control
   Cons: Manual auth flow, error handling complexity

3. Use community bluesky-rs crate
   Pros: Rust-native, lighter than official SDK
   Cons: Less mature, may lack features

Security:
- OAuth tokens must use SecretString
- Validate character limit client-side
- Handle rate limits server-side
- Never log access tokens

Test Plan:
- test_authenticate_success()
- test_authenticate_invalid_token()
- test_post_within_limit()
- test_post_exceeds_limit()
- test_network_failure_retry()

Recommendation: Option 3 (bluesky-rs)
Rationale: Balances maintainability with control, fits Rust ecosystem
</implementation_analysis>
```

---

## 📦 Project Overview

**Plurcast** is a collection of Unix command-line tools for posting to decentralized social media platforms (Nostr, Mastodon, SSB). The project follows Unix philosophy principles: single-purpose tools, text streams, composability, meaningful exit codes, and agent-friendly interfaces.

**Status**: Alpha (v0.3.0-alpha2)
**Platforms**: Nostr ✅, Mastodon ✅, SSB ⚗️ (experimental)
**Binaries**: 9 tools (plur-post, plur-history, plur-creds, plur-send, plur-queue, plur-import, plur-export, plur-setup)

**Key Features for Agents**:
- Nostr is permissionless - no account registration needed
- Shared test credentials available for immediate use
- Draft mode for testing without posting
- JSON output for machine parsing
- Strictly defined exit codes (0=success, 1=posting failed, 2=auth error, 3=invalid input)

---

## 🚀 Quick Start for Development

### 1. Build and Test

```bash
# Clone and build
git clone https://github.com/plurcast/plurcast
cd plurcast
cargo build --release

# Run all tests (REQUIRED before commits)
cargo test
cargo clippy -- -D warnings
cargo fmt --check

# Run specific binary
cargo run -p plur-post -- "Hello world" --draft
```

### 2. Use Test Credentials (Fastest for Testing)

```bash
# Set up test credentials
mkdir -p ~/.config/plurcast
echo "9270ffc3ddd551bf37a1417d5b0762a9f0a75204a3d6839c5d7e8790b1f57cad" > ~/.config/plurcast/nostr.key
chmod 600 ~/.config/plurcast/nostr.key

# Create minimal config
cat > ~/.config/plurcast/config.toml <<'EOF'
[database]
path = "~/.local/share/plurcast/posts.db"

[defaults]
platforms = ["nostr"]

[nostr]
keys_file = "~/.config/plurcast/nostr.key"
relays = ["wss://relay.damus.io", "wss://nos.lol"]
EOF

mkdir -p ~/.local/share/plurcast

# Test immediately
cargo run -p plur-post -- "Hello from dev" --draft
```

**⚠️ WARNING**: The public test key (`9270ffc3...`) is shared across all tests. DO NOT use for real identity.

### 3. Generate Fresh Keys (For Unique Identity)

```bash
cargo run --example generate_nostr_key
echo "<your_private_key_hex>" > ~/.config/plurcast/nostr.key
chmod 600 ~/.config/plurcast/nostr.key
```

### 4. Test Without Posting (Draft Mode)

```bash
./target/release/plur-post "Test content" --draft
# Output: draft:550e8400-e29b-41d4-a716-446655440000
# Exit code: 0 (always succeeds, no auth needed)
```

---

## 🏗️ Architecture & Design Patterns

### Workspace Structure

```
plurcast/
├── libplurcast/          # Shared library (all business logic)
│   ├── src/
│   │   ├── config.rs     # TOML config, env vars, XDG paths
│   │   ├── credentials.rs # Keyring, encrypted files, plain files
│   │   ├── db.rs         # SQLite with sqlx (compile-time verified)
│   │   ├── error.rs      # Error types → exit codes
│   │   ├── logging.rs    # Centralized logging (text/json/pretty)
│   │   ├── platforms/    # Platform implementations
│   │   │   ├── mod.rs    # Platform trait
│   │   │   ├── nostr.rs  # Nostr protocol
│   │   │   ├── mastodon.rs # ActivityPub
│   │   │   └── ssb/      # Secure Scuttlebutt
│   │   └── service/      # Service layer (facades)
│   │       ├── posting.rs
│   │       ├── history.rs
│   │       └── validation.rs
│   └── migrations/       # SQLx migrations (auto-run)
├── plur-post/            # Post content CLI
├── plur-history/         # View post history CLI
├── plur-creds/           # Manage credentials CLI
├── plur-send/            # Daemon for scheduled posts
├── plur-queue/           # Manage post queue CLI
├── plur-import/          # Import from other platforms
├── plur-export/          # Export post history
└── plur-setup/           # Interactive setup wizard
```

### Key Design Patterns

#### Platform Trait (Strategy Pattern)

```rust
#[async_trait]
pub trait Platform: Send + Sync {
    async fn authenticate(&mut self) -> Result<()>;
    async fn post(&self, post: &Post) -> Result<String>;
    fn validate_content(&self, content: &str) -> Result<()>;
    fn name(&self) -> &str;
    fn character_limit(&self) -> Option<usize>;
    fn is_configured(&self) -> bool;
}
```

**Adding a New Platform**:
1. Create `libplurcast/src/platforms/myplatform.rs`
2. Implement `Platform` trait with `#[async_trait]`
3. Add config struct to `config.rs`
4. Add credential handling to `credentials.rs`
5. Write comprehensive tests (see `platforms/nostr/tests.rs`)

#### Service Layer (Facade Pattern)

```rust
pub struct PlurcastService {
    posting: PostingService,
    history: HistoryService,
    validation: ValidationService,
    draft: DraftService,
}

impl PlurcastService {
    pub fn posting(&self) -> &PostingService { &self.posting }
    pub fn history(&self) -> &HistoryService { &self.history }
    // ... other services
}
```

**Benefits**:
- Testable business logic independent of CLI
- Thread-safe via Arc<Database> and Arc<Config>
- EventBus for progress tracking

#### Error Handling with Exit Codes

```rust
#[derive(Error, Debug)]
pub enum PlurcastError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),           // Exit code 3

    #[error("Authentication failed: {0}")]
    Authentication(String),          // Exit code 2

    #[error("Platform error: {0}")]
    Platform(#[from] PlatformError), // Exit code 1
}

impl PlurcastError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::InvalidInput(_) => 3,
            Self::Authentication(_) => 2,
            Self::Platform(_) => 1,
            // ... other variants
        }
    }
}
```

**Exit Code Contract** (strictly tested):
- **0**: Success on all platforms
- **1**: Posting failed (network error, rate limit, etc.)
- **2**: Authentication error (missing/invalid credentials)
- **3**: Invalid input (empty content, too large, malformed)

### Database Schema

```sql
-- User's authored posts
CREATE TABLE posts (
    id TEXT PRIMARY KEY,                -- UUID v4
    content TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    scheduled_at INTEGER,
    status TEXT DEFAULT 'pending',      -- draft, scheduled, pending, posted, failed
    metadata TEXT                       -- JSON for extensibility
);

-- Platform-specific posting records (1:N with posts)
CREATE TABLE post_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    post_id TEXT NOT NULL,              -- FK to posts.id
    platform TEXT NOT NULL,             -- nostr, mastodon, ssb
    platform_post_id TEXT,              -- note1..., status ID, message ID
    posted_at INTEGER,
    success INTEGER DEFAULT 0,
    error_message TEXT,
    account_name TEXT NOT NULL DEFAULT 'default',  -- Multi-account tracking
    FOREIGN KEY (post_id) REFERENCES posts(id)
);

-- Multi-account support
CREATE TABLE accounts (
    platform TEXT NOT NULL,
    account_name TEXT NOT NULL,
    is_active INTEGER DEFAULT 0,
    PRIMARY KEY (platform, account_name)
);
```

**Migrations**: Stored in `libplurcast/migrations/`, run automatically via `sqlx::migrate!`

**Compile-Time Verification**: All SQL queries are verified against schema at compile time using sqlx. Run `cargo sqlx prepare` after adding migrations.

### Configuration Priority

1. **Environment variables** (highest priority)
   - `PLURCAST_CONFIG` - Path to config file
   - `PLURCAST_DB_PATH` - Path to database
   - `PLURCAST_LOG_FORMAT` - Logging format (text/json/pretty)
   - `PLURCAST_LOG_LEVEL` - Logging level (error/warn/info/debug/trace)

2. **Config file** (`~/.config/plurcast/config.toml`)
   - User-specified paths with `~` expansion
   - Platform configurations
   - Default platforms

3. **XDG Base Directory defaults** (lowest priority)
   - Config: `~/.config/plurcast/`
   - Data: `~/.local/share/plurcast/`

---

## 🔒 Implementation Guidelines

### Security Checklist

Before implementing any feature touching credentials or user data:

- [ ] Input validation (length, format, type)
- [ ] Secrets use `Secret<T>` and `Zeroize`
- [ ] Errors don't expose sensitive data
- [ ] File permissions set correctly (600 for keys, 644 for config)
- [ ] No credentials in logs (only lengths/hashes)
- [ ] SQL queries parameterized (no string concatenation)
- [ ] Network errors don't expose internal paths

### Testing Checklist

For every new function or feature:

- [ ] Unit test for success case
- [ ] Unit test for each error case
- [ ] Unit test for boundary conditions (empty, max length, etc.)
- [ ] Integration test if touching multiple components
- [ ] Test with real-world data (use fixtures)
- [ ] Test concurrency if async
- [ ] Document test failures with clear error messages

**Test Patterns**:
```rust
#[tokio::test]
async fn test_post_success() {
    let (service, _temp_dir) = setup_test_service().await;
    let request = PostRequest {
        content: "Test content".to_string(),
        platforms: vec!["nostr".to_string()],
        draft: false,
        account: None,
        scheduled_at: None,
        nostr_pow: None,
    };

    let response = service.posting().post(request).await.unwrap();

    assert!(response.overall_success);
    assert_eq!(response.results.len(), 1);
    assert!(response.results[0].success);
}

#[test]
fn test_validate_content_empty() {
    let result = validate_content("");

    assert!(result.is_err());
    match result.unwrap_err() {
        PlurcastError::InvalidInput(msg) => {
            assert!(msg.contains("empty"));
        }
        _ => panic!("Expected InvalidInput error"),
    }
}
```

### Logging Best Practices

**Use Structured Logging**:
```rust
use tracing::{info, debug, error, warn, instrument};

// Add instrumentation to functions
#[instrument(skip(db))]
async fn save_post(db: &Database, post: &Post) -> Result<()> {
    debug!(
        post_id = %post.id,
        content_len = post.content.len(),
        "Saving post to database"
    );

    let start = std::time::Instant::now();
    db.create_post(post).await?;
    let duration = start.elapsed();

    info!(
        post_id = %post.id,
        duration_ms = duration.as_millis(),
        "Post saved successfully"
    );

    Ok(())
}
```

**Log Levels**:
- `error!()` - Unrecoverable errors
- `warn!()` - Recoverable errors, degraded functionality
- `info!()` - Important state changes (post created, authenticated)
- `debug!()` - Detailed operation flow
- `trace!()` - Very detailed, performance-sensitive code

**Never Log**:
- Private keys (log length only: `key.len()`)
- OAuth tokens (log prefix only: `&token[..8]`)
- User content in production logs
- Internal file paths in error messages to users

### Input Validation Pattern

**Location**: `plur-post/src/main.rs` in `get_content()` function

```rust
const MAX_CONTENT_LENGTH: usize = 100_000; // 100KB

fn validate_input(content: &str) -> Result<()> {
    // Check for empty
    if content.trim().is_empty() {
        return Err(PlurcastError::InvalidInput(
            "Content cannot be empty".into()
        ));
    }

    // Check length (prevents DoS)
    if content.len() > MAX_CONTENT_LENGTH {
        return Err(PlurcastError::InvalidInput(format!(
            "Content too large: {} bytes (maximum: {} bytes)",
            content.len(),
            MAX_CONTENT_LENGTH
        )));
    }

    // Additional platform-specific validation happens in Platform::validate_content()

    Ok(())
}

// For stdin input, use .take() to limit read:
let mut content = String::new();
io::stdin()
    .lock()
    .take((MAX_CONTENT_LENGTH + 1) as u64)
    .read_to_string(&mut content)?;

if content.len() > MAX_CONTENT_LENGTH {
    return Err(PlurcastError::InvalidInput(/* ... */));
}
```

**Security Properties**:
- Never allocates more than MAX_CONTENT_LENGTH bytes
- Prevents memory exhaustion (`cat /dev/zero | plur-post` fails fast)
- Fails in < 100ms on oversized content
- Error messages don't include content samples

---

## 🛠️ Common Development Tasks

### Building and Testing

```bash
# Full development workflow
cargo fmt              # Format code
cargo clippy           # Lint (MUST pass with no warnings)
cargo test             # Run all tests (MUST pass)
cargo build            # Debug build

# Release build (for production)
cargo build --release

# Run tests with output visible
cargo test -- --nocapture

# Run specific test
cargo test test_post_success

# Run tests for specific package
cargo test -p libplurcast

# Check compilation without building (fast)
cargo check

# Check all features
cargo check --all-features
```

### Running Binaries

```bash
# Run from source (debug)
cargo run -p plur-post -- "Hello world"

# With flags
cargo run -p plur-post -- "Test" --verbose --draft --log-format pretty

# Run release binary (faster, optimized)
./target/release/plur-post "Hello world"

# Test with stdin
echo "Test post" | cargo run -p plur-post

# Test JSON output
cargo run -p plur-post -- "Test" --format json --log-format json

# Test with environment variables
PLURCAST_LOG_FORMAT=json PLURCAST_LOG_LEVEL=debug cargo run -p plur-post -- "Test"

# Generate test keys
cargo run --example generate_nostr_key
```

### Logging Examples

```bash
# Text logging (default)
plur-post "Hello world"
# 2025-11-22T12:52:06Z INFO message

# JSON logging (production, machine-parseable)
plur-post "Test" --log-format json
# {"timestamp":"2025-11-22T12:52:06.010808Z","level":"INFO","target":"plur_post",...}

# Pretty logging (development, colored)
plur-post "Debug test" --log-format pretty --verbose
# Colored, pretty-printed output with file:line numbers

# Environment variable configuration
export PLURCAST_LOG_FORMAT=json
export PLURCAST_LOG_LEVEL=debug
plur-post "Test"  # Uses JSON with debug level
```

### Scheduling Commands

```bash
# Schedule a post
cargo run -p plur-post -- "Hello later!" --schedule "30m"
cargo run -p plur-post -- "Tomorrow" --schedule "tomorrow"
cargo run -p plur-post -- "Random" --schedule "random:1h-2h"

# Manage queue
cargo run -p plur-queue -- list
cargo run -p plur-queue -- stats
cargo run -p plur-queue -- cancel <post_id>
cargo run -p plur-queue -- reschedule <post_id> "+2h"

# Run daemon (processes scheduled posts)
cargo run -p plur-send
cargo run -p plur-send -- --verbose --log-format json
cargo run -p plur-send -- --once  # Process once and exit (testing)
```

### Database Operations

```bash
# Database created automatically at ~/.local/share/plurcast/posts.db
# Use custom location:
export PLURCAST_DB_PATH=/path/to/custom.db

# Inspect database
sqlite3 ~/.local/share/plurcast/posts.db "SELECT * FROM posts;"

# View schema
sqlite3 ~/.local/share/plurcast/posts.db ".schema"

# View migrations
ls libplurcast/migrations/

# After adding migrations, update sqlx cache:
cargo sqlx prepare
```

---

## 📚 Platform-Specific Notes

### Nostr Implementation

**Location**: `libplurcast/src/platforms/nostr.rs`

**Key Features**:
- NIP-13 Proof of Work support (parallel mining with rayon)
- Multiple relay support
- Memory-protected private keys (Secret<T>, Zeroize)
- Shared test account for quick testing

**Adding PoW**:
```bash
plur-post "Important message" --platform nostr --nostr-pow 20
# Difficulty 20-25 recommended (1-5 seconds)
```

**Shared Test Credentials**:
- Private Key: `9270ffc3ddd551bf37a1417d5b0762a9f0a75204a3d6839c5d7e8790b1f57cad`
- Public Key: `npub1ch642h2jvaq2fv3pzq36m5t99nrzvppkdr6pw8m8eryfzezynzlqky6cjp`
- View posts: https://nostr.band/

### Mastodon Implementation

**Location**: `libplurcast/src/platforms/mastodon.rs`

**Key Features**:
- ActivityPub compatible (works with Mastodon, Pleroma, etc.)
- OAuth token authentication
- Character limit fetched from instance

**Credential Setup**:
```bash
echo "<your_oauth_token>" > ~/.config/plurcast/mastodon.token
chmod 600 ~/.config/plurcast/mastodon.token
```

### SSB Implementation

**Location**: `libplurcast/src/platforms/ssb/`

**Status**: Experimental (local posting works, network replication limited)

**Key Features**:
- Local feed management
- Message signing
- Pub support (experimental)

---

## 🚨 Before Committing

**Pre-Commit Checklist**:
- [ ] `cargo fmt` - Code is formatted
- [ ] `cargo clippy -- -D warnings` - No clippy warnings
- [ ] `cargo test` - All tests pass
- [ ] No credentials in code or config files
- [ ] Commit message explains "why", not just "what"
- [ ] New features have tests
- [ ] Documentation updated (if public API changed)

**Commit Message Format**:
```
type(scope): Brief description

Detailed explanation of why this change was made.

- Bullet points for key changes
- Include breaking changes if any

Relates to #issue-number
```

Types: `feat`, `fix`, `refactor`, `docs`, `test`, `chore`

---

## 📖 Library Version Requirements

**Always check up-to-date documentation** when implementing features:

- **nostr-sdk** v0.35 - Nostr protocol (rust-nostr/nostr)
- **sqlx** v0.8 - Async SQL with compile-time verification (launchbadge/sqlx)
- **tokio** v1 - Async runtime (tokio-rs/tokio)
- **clap** v4.5 - CLI parser with derive macros (clap-rs/clap)
- **keyring** v2.3 - OS-native credential storage
- **serde** v1 - Serialization framework
- **tracing** v0.1 - Structured logging
- **tracing-subscriber** v0.3 - Logging backend (with json, fmt, ansi features)

Review official docs and examples in library repos to verify current APIs.

---

## 🎯 Platform Support Status

**Production Ready**:
- ✅ **Nostr**: Full support with relay publishing, PoW, test account
- ✅ **Mastodon**: Full support using megalodon client

**Experimental**:
- ⚗️ **SSB (Secure Scuttlebutt)**: Local posting works, network replication limited

**Future Platforms**:
Follow the Platform trait pattern. See existing implementations for reference.

---

## 🔗 Additional Resources

- **Design Review**: `docs/DESIGN_REVIEW_2025_11_17.md` - Architecture analysis and roadmap
- **Logging Proposal**: `docs/LOGGING_ENHANCEMENT_PROPOSAL.md` - JSON logging details
- **Testing Guides**: `docs/TESTING_OVERVIEW.md`, `docs/TESTING_CHECKLIST.md`
- **Security Docs**: `docs/SECURITY.md`, `docs/SECURITY_VERIFICATION.md`
- **ADRs**: `docs/adr/` - Architecture Decision Records
- **User Docs**: `README.md` - User-facing documentation

---

**Remember**: Security first, tests always, observability built-in. Write code that is maintainable, secure, and production-ready from day one.
