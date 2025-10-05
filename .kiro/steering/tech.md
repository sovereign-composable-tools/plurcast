## Tech Stack

**Language**: Rust (stable)

**Platform Libraries**:
- `nostr-sdk` v0.35+ - Nostr protocol implementation
- `megalodon` v0.14+ - Mastodon/Fediverse (ActivityPub)
- `atrium-api` v0.24+ - Bluesky AT Protocol

**Core Dependencies**:
- `sqlx` v0.8 - SQLite database with async support
- `tokio` v1 - Async runtime
- `clap` v4.5 - CLI argument parsing
- `serde` / `serde_json` - Serialization
- `chrono` v0.4 - Date/time handling
- `uuid` v1.10 - UUID generation
- `anyhow` / `thiserror` - Error handling
- `tracing` / `tracing-subscriber` - Logging

## Build System

Standard Cargo workspace with multiple binaries.

**Common Commands**:

```bash
# Build all binaries
cargo build --release

# Build specific binary
cargo build --bin plur-post

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run --bin plur-post

# Check without building
cargo check

# Format code
cargo fmt

# Lint
cargo clippy
```

## Data Storage

- **Database**: `~/.local/share/plurcast/posts.db` (SQLite)
- **Config**: `~/.config/plurcast/config.toml`
- **Credentials**: `~/.config/plurcast/*.key`, `*.token`, `*.auth`

## Platform Conventions

- XDG Base Directory compliance
- Exit codes: 0 (success), 1 (failure), 2 (auth error), 3 (invalid input)
- Errors to stderr, output to stdout
- JSON output via `--format json` for scripting
- File permissions: 600 for sensitive files
