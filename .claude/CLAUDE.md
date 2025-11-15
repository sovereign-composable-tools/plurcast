# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Plurcast is a collection of Unix command-line tools for posting to decentralized social media platforms (Nostr, Mastodon, SSB). The project follows Unix philosophy principles: single-purpose tools, text streams, composability, meaningful exit codes, and agent-friendly interfaces.

**Status**: Alpha MVP (v0.1.0) - Foundation phase with basic Nostr support

## Architecture

### Workspace Structure

This is a Cargo workspace with two main components:

- **libplurcast/** - Shared library containing core functionality
  - `config.rs` - Configuration management with XDG Base Directory support, TOML parsing
  - `db.rs` - SQLite database operations using sqlx with compile-time verification
  - `error.rs` - Error types with exit code mapping
  - `types.rs` - Shared data structures (Post, PostRecord, PostStatus)
  - `platforms/` - Platform trait and implementations (currently Nostr)

- **plur-post/** - CLI binary for posting content
  - Handles stdin/argument input, platform selection, output formatting
  - Implements Unix-style interface with exit codes: 0=success, 1=posting failure, 2=auth error, 3=invalid input

### Key Design Principles

**Configuration Priority**:
1. Environment variables (PLURCAST_CONFIG, PLURCAST_DB_PATH)
2. User-specified paths in config file (with ~ expansion via shellexpand)
3. XDG Base Directory defaults (~/.config/plurcast/, ~/.local/share/plurcast/)

**Database Schema**:
- `posts` - User's authored posts (id, content, created_at, scheduled_at, status, metadata)
- `post_records` - Platform-specific posting records with foreign key to posts
- `platforms` - Platform configurations
- Migrations are in `libplurcast/migrations/` and run automatically via sqlx::migrate!

**Platform Abstraction**:
- `Platform` trait defines: authenticate(), post(), validate_content(), name()
- All platforms must implement async_trait
- Platforms are responsible for their own authentication and key management

**Error Handling**:
- All errors use custom PlurcastError enum (Config, Database, Platform, Auth, Posting, InvalidInput)
- Each error type maps to a specific exit code via exit_code() method
- Errors are displayed to stderr, output goes to stdout

**Input Validation**:
- Maximum content length: 100KB (100,000 bytes) enforced via MAX_CONTENT_LENGTH constant
- Validation occurs before processing to fail fast and prevent memory exhaustion
- Argument path: Length check via .len() before any processing
- Stdin path: Uses .take(MAX_CONTENT_LENGTH + 1) to limit bytes read and detect overflow
- Security: Prevents DoS attacks via infinite streams (e.g., cat /dev/zero | plur-post)
- Error messages include actual size and maximum size for user clarity

## Common Development Commands

### Building and Testing

```bash
# Debug build (default)
cargo build

# Release build (optimized)
cargo build --release

# Run all tests (must pass before commits)
cargo test

# Run tests with output visible
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests in specific module
cargo test -p libplurcast config::tests

# Check code without building (fast)
cargo check
```

### Running the Binary

```bash
# Run from source (debug)
cargo run -p plur-post -- "Hello world"

# Run from source with verbose logging
cargo run -p plur-post -- "Test" --verbose

# Run release binary
./target/release/plur-post "Hello world"

# Test with stdin
echo "Test post" | cargo run -p plur-post

# Test with platform selection
cargo run -p plur-post -- "Test" --platform nostr

# Test JSON output
cargo run -p plur-post -- "Test" --format json
```

### Database Operations

```bash
# Database is created automatically at ~/.local/share/plurcast/posts.db
# To use custom location:
export PLURCAST_DB_PATH=/path/to/custom.db

# Inspect database directly
sqlite3 ~/.local/share/plurcast/posts.db "SELECT * FROM posts;"

# View migrations
ls libplurcast/migrations/
```

## Important Implementation Notes

### SQLx Compile-Time Verification

This project uses sqlx with compile-time query verification. This means:
- Database queries are checked at compile time against actual database schema
- You need a database available during compilation (sqlx creates one from migrations)
- If you add new migrations, run `cargo sqlx prepare` to update cached query metadata

### Platform Implementation

**Currently Implemented Platforms**:
- **Nostr**: `platforms/nostr.rs` - Nostr protocol with relay support
- **Mastodon**: `platforms/mastodon.rs` - ActivityPub (Mastodon, Pleroma, etc.)
- **SSB**: `platforms/ssb/` - Secure Scuttlebutt (experimental, local posting works)

When adding new platforms:
1. Create new module in `libplurcast/src/platforms/`
2. Implement the `Platform` trait with async_trait
3. Add configuration struct to `config.rs`
4. Add platform-specific authentication in the authenticate() method
5. Handle platform-specific key/token storage (prefer separate files with mode 600)
6. Add platform case to `post_to_platform()` in plur-post/src/main.rs

### Key Management Security

- Private keys are stored in separate files (not in config.toml)
- Config files and key files are created with 600 permissions on Unix
- Never log or output private keys except for debugging length
- Support for both hex and bech32 (nsec) formats for Nostr keys

### Exit Code Contract

Exit codes are strictly defined and tested:
- **0**: Success on all platforms
- **1**: Posting failed on at least one platform
- **2**: Authentication error (missing/invalid credentials)
- **3**: Invalid input (empty content, malformed arguments, content too large)

Do not change these without updating documentation and tests.

### Input Validation Implementation

**Location**: `plur-post/src/main.rs` in `get_content()` function

**Constant**: `MAX_CONTENT_LENGTH = 100_000` (100KB)

**Validation Strategy**:
1. **Argument Input**: Check `content.len() > MAX_CONTENT_LENGTH` before processing
2. **Stdin Input**: Use `stdin.lock().take((MAX_CONTENT_LENGTH + 1) as u64)` to limit read
3. **Overflow Detection**: Read one extra byte to distinguish "at limit" from "over limit"
4. **Early Termination**: Validation fails immediately without reading entire stream

**Error Message Format**:
```rust
format!(
    "Content too large: {} bytes (maximum: {} bytes)",
    actual_size,
    MAX_CONTENT_LENGTH
)
```

**Security Properties**:
- Never allocates more than MAX_CONTENT_LENGTH bytes for content
- Prevents memory exhaustion attacks (infinite streams, huge arguments)
- Fails fast (< 100ms) on oversized content
- No content samples in error messages (security requirement)

**Testing**:
- Unit tests in `plur-post/tests/validation_unit_tests.rs`
- Integration tests in `plur-post/tests/cli_integration.rs`
- Coverage includes: under limit, at limit, over limit, attack scenarios

### Testing Requirements

- All configuration parsing must be tested (valid, invalid, missing fields)
- All environment variable overrides must be tested
- All database operations must be tested (CRUD, concurrency, constraints)
- All platform implementations must be tested (key parsing, validation, posting)
- All CLI flags and input methods must have integration tests
- All exit codes must be verified
- All input validation scenarios must be tested (under/at/over limit, attack vectors)

Use `tempfile::TempDir` for tests that create files/directories.

**Input Validation Test Coverage**:
- Content under limit (should pass)
- Content exactly at limit (should pass)
- Content at limit + 1 byte (should fail with exit code 3)
- Significantly oversized content (should fail fast)
- Empty content after trim (should fail)
- Simulated infinite stream (should terminate immediately)
- Error message format verification (includes sizes, no content samples)

## Library Version Requirements

**Important**: Always check up-to-date documentation before implementing features with these libraries:

- **nostr-sdk** (rust-nostr/nostr) - Nostr protocol implementation (currently v0.35)
- **sqlx** (launchbadge/sqlx) - Async SQL with compile-time verification (v0.8)
- **tokio** (tokio-rs/tokio) - Async runtime (v1)
- **clap** (clap-rs/clap) - Command-line parser with derive macros (v4.5)

Review official documentation and examples in library repositories to verify current APIs and patterns.

## Git Workflow

- Write clear, conventional commit messages that explain "why" not just "what"
- Ensure all tests pass before committing (cargo test)
- Follow existing commit style (see recent commits with git log)
- Never commit files with secrets (.env, keys files, credentials)
- Maintain a clean commit history for version control

## Platform Support Status

**Implemented Platforms**:
- ✅ **Nostr**: Full support with relay publishing and shared test account
- ✅ **Mastodon**: Full support using megalodon client, OAuth token in ~/.config/plurcast/mastodon.token
- ⚗️ **SSB (Secure Scuttlebutt)**: Experimental support - local posting works, network replication limited

**Platform Decision**: Removed Bluesky (centralized, banned test accounts). Replaced with SSB for true decentralization.

**Future Features**:
- ✅ **Scheduling**: plur-queue (completed) and plur-send (in progress) for deferred posting
- 🚧 Additional platforms may be added following the Platform trait pattern

When implementing new platforms, follow the existing Nostr/Mastodon/SSB patterns in architecture and testing.
