# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Common Development Commands

### Building
- Debug: `cargo build`
- Optimized: `cargo build --release`
- Check without building: `cargo check`

### Testing
- All tests: `cargo test`
- With output: `cargo test -- --nocapture`
- Specific package: `cargo test -p <package>`
- Specific test: `cargo test <test_name>`
- Package-specific tests: `cargo test -p <package> <test_name>`

### Running Binaries
- Post content: `cargo run -p plur-post -- "content"`
- Query history: `cargo run -p plur-history`
- Manage credentials: `cargo run -p plur-creds`
- Interactive setup: `cargo run -p plur-setup`

### Database Operations
- Inspect database: `sqlite3 ~/.local/share/plurcast/posts.db "SELECT * FROM posts;"`
- SQLx maintenance: `cargo sqlx prepare` (update cached query metadata after migrations)

## High-Level Architecture

### Workspace Structure
Cargo workspace with 5 packages:
- `libplurcast` - Shared library containing core functionality
- `plur-post` - CLI for posting content
- `plur-history` - CLI for querying post history
- `plur-creds` - CLI for credential management
- `plur-setup` - Interactive setup wizard

### Core Library Modules
- **`config.rs`** - Configuration management with XDG Base Directory support and TOML parsing
- **`db.rs`** - SQLite database operations using sqlx with compile-time query verification
- **`error.rs`** - Error types with exit code mapping (0=success, 1=posting failure, 2=auth error, 3=invalid input)
- **`types.rs`** - Shared data structures (Post, PostRecord, PostStatus)
- **`platforms/`** - Platform trait and implementations for Nostr, Mastodon, Bluesky
- **`credentials.rs`** - Multi-backend credential storage (OS Keyring, Encrypted Files, Plain Text)
- **`service/`** - Service layer facade with specialized services (PostingService, HistoryService, DraftService, ValidationService)
- **`poster.rs`** - Posting orchestration logic

### Platform Abstraction
All platforms implement the `Platform` trait with async methods:
- `authenticate()` - Establish connection and authenticate
- `post(content)` - Post content and return platform-specific post ID
- `validate_content(content)` - Check platform-specific requirements
- `name()` - Return platform identifier (e.g., "nostr", "mastodon")
- `character_limit()` - Return character limit (or None)
- `is_configured()` - Check if platform has required configuration

### Service Layer
Uses facade pattern with `PlurcastService` as the main entry point:
- Coordinates specialized sub-services (posting, history, drafts, validation)
- Services share `Arc<Database>` and `Arc<Config>` for efficient concurrent access
- Event bus for progress tracking and UI updates
- All services are async and support concurrent operations

### Credential Storage
Three backends with automatic fallback:
1. **OS Keyring** (recommended) - Native secure storage (Keychain/Credential Manager/Secret Service)
2. **Encrypted Files** - Password-protected files using age encryption
3. **Plain Text** - Legacy compatibility only (not recommended)

`CredentialManager` controls fallback logic and provides unified interface.

### Error Handling
`PlurcastError` enum with exit code mapping:
- **Exit Code 0** - Success on all platforms
- **Exit Code 1** - Posting failed on at least one platform (includes Config, Database, Platform errors)
- **Exit Code 2** - Authentication error (missing/invalid credentials)
- **Exit Code 3** - Invalid input (empty content, malformed arguments, content too large)

### Database Schema
SQLite database with automatic migrations via `sqlx::migrate!`:
- **`posts`** - User's authored posts (id, content, created_at, scheduled_at, status, metadata)
- **`post_records`** - Platform-specific posting records (FK to posts)
- **`platforms`** - Platform configurations

Migrations located in `libplurcast/migrations/` and run automatically on database initialization.

## Important Implementation Details

### SQLx Compile-Time Verification
- Database queries are checked at compile time against actual schema
- Database is created from migrations during build
- After adding new migrations, run `cargo sqlx prepare` to update cached query metadata
- Requires SQLite to be available during compilation

### Input Validation
- `MAX_CONTENT_LENGTH = 100KB` enforced before processing to prevent memory exhaustion
- Argument input: Length check via `.len()` before processing
- Stdin input: Uses `.take(MAX_CONTENT_LENGTH + 1)` to limit bytes read and detect overflow
- Prevents DoS attacks via infinite streams (e.g., `cat /dev/zero | plur-post`)
- Error messages include actual and maximum size for clarity

### Test-Driven Development
- **Always write failing tests first, then implement features until tests pass**
- All tests must pass before commits (enforced project rule)
- Use `tempfile::TempDir` for tests that create files/directories
- Test coverage requirements:
  - All configuration parsing (valid, invalid, missing fields)
  - All environment variable overrides
  - All database operations (CRUD, concurrency, constraints)
  - All platform implementations (key parsing, validation, posting)
  - All CLI flags and input methods
  - All exit codes
  - All input validation scenarios (under/at/over limit, attack vectors)

### Exit Code Contract
- Exit codes are strictly defined and tested
- **Do not change exit codes without updating documentation and tests**
- Integration tests verify exit codes for all error scenarios
- Exit code mapping is centralized in `error.rs`

### Key Management Security
- Private keys are stored in **separate files** (not in `config.toml`)
- Credential files are created with 600 permissions (Unix)
- **Never log or output private keys** (only log lengths for debugging)
- Support for both hex and bech32 (nsec) formats for Nostr keys

### Configuration Priority
Configuration is loaded in this order:
1. Environment variables (`PLURCAST_CONFIG`, `PLURCAST_DB_PATH`)
2. User-specified paths in config file (with `~` expansion via `shellexpand`)
3. XDG Base Directory defaults (`~/.config/plurcast/`, `~/.local/share/plurcast/`)

### Adding New Platforms
When implementing new platforms:
1. Create new module in `libplurcast/src/platforms/`
2. Implement the `Platform` trait with `#[async_trait]`
3. Add configuration struct to `config.rs`
4. Add platform-specific authentication in the `authenticate()` method
5. Handle platform-specific key/token storage (prefer separate files with mode 600)
6. Follow the existing Nostr implementation pattern for consistency
7. Add comprehensive tests for all Platform trait methods

## Notes from .claude/CLAUDE.md

### Library Documentation
**Always consult official documentation** before implementing features with:
- `nostr-sdk` (rust-nostr/nostr) - Currently v0.35
- `sqlx` (launchbadge/sqlx) - Currently v0.8
- `tokio` (tokio-rs/tokio) - Currently v1
- `clap` (clap-rs/clap) - Currently v4.5

Review official documentation and examples to verify current APIs and patterns.

### Testing Expectations
- Maintain comprehensive test coverage across all modules
- Follow TDD: write failing tests first, then implement until green
- All tests must pass before committing
- Use meaningful test names that describe the scenario being tested
- Test both success and failure paths

### Input Validation
- Preserve strict input limits and boundary checks
- If validation constraints need to change, update tests first
- Never skip validation for performance reasons
- Security properties must be maintained (DoS prevention, memory safety)
