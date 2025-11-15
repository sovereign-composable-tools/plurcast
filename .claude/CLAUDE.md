# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Plurcast is a collection of Unix command-line tools for posting to decentralized social media platforms (Nostr, Mastodon, Bluesky, SSB). The project follows Unix philosophy principles: single-purpose tools, text streams, composability, meaningful exit codes, and **agent-friendly interfaces**.

**Status**: Alpha (v0.3.0-alpha2) - Multi-platform foundation with Nostr and SSB support

**Key Feature for Agents**: Nostr is permissionless - no account registration needed. Just use a keypair and post immediately to the network.

---

## Agent Quick Start (Autonomous Use)

Plurcast is designed for autonomous AI agent use. You can start posting immediately with the public test credentials.

### 1. Use Public Test Credentials (Fastest)

For immediate testing without key generation:

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

# Create data directory
mkdir -p ~/.local/share/plurcast

# Build and post immediately
cargo build --release
./target/release/plur-post "Hello from agent"
# Output: nostr:note1abc123...
```

**Public Test Credentials:**
- Private Key (hex): `9270ffc3ddd551bf37a1417d5b0762a9f0a75204a3d6839c5d7e8790b1f57cad`
- Public Key (npub): `npub1ch642h2jvaq2fv3pzq36m5t99nrzvppkdr6pw8m8eryfzezynzlqky6cjp`
- ⚠️ **WARNING**: Public test key shared across tests - DO NOT use for real identity
- 📍 **View posts**: Visit https://nostr.band/ and search for the npub

### 2. Generate Fresh Keys (For Unique Identity)

When you need a unique identity:

```bash
# Generate new keypair
cargo run --example generate_nostr_key

# Output shows:
# Private Key (KEEP SECRET!):
#   Hex:    <your_private_key_hex>
#   Bech32: <your_private_key_nsec>
# Public Key (safe to share):
#   Hex:    <your_public_key_hex>
#   Bech32: <your_public_key_npub>

# Save the private key
echo "<your_private_key_hex>" > ~/.config/plurcast/nostr.key
chmod 600 ~/.config/plurcast/nostr.key
```

### 3. Test Without Posting (Draft Mode)

Draft mode saves to database without posting to networks:

```bash
./target/release/plur-post "Test content" --draft
# Output: draft:550e8400-e29b-41d4-a716-446655440000
# Exit code: 0 (always succeeds, no auth needed)
```

---

## CLI Reference (plur-post)

Full help output from `plur-post --help`:

```
plur-post - Post content to decentralized social platforms

DESCRIPTION:
    plur-post is a Unix-style tool for posting content to decentralized social
    media platforms like Nostr, Mastodon, and SSB. It follows Unix philosophy:
    reads from stdin or arguments, outputs to stdout, and uses meaningful exit codes.

USAGE EXAMPLES:
    # Post from command line argument
    plur-post "Hello decentralized world!"

    # Post from stdin (pipe)
    echo "Hello from stdin" | plur-post
    cat message.txt | plur-post

    # Post to all enabled platforms (from config defaults)
    plur-post "Multi-platform post"

    # Post to specific platform only
    plur-post "Nostr-only post" --platform nostr

    # Post to multiple specific platforms
    plur-post "Selective post" --platform nostr --platform mastodon

    # Save as draft without posting
    echo "Draft content" | plur-post --draft

    # Get machine-readable JSON output
    plur-post "Test post" --format json

    # Enable verbose logging for debugging
    plur-post "Debug post" --verbose

    # Unix composability examples
    fortune | plur-post --platform nostr
    echo "Status: $(date)" | plur-post
    cat draft.txt | sed 's/foo/bar/g' | plur-post

CONFIGURATION:
    Configuration file: ~/.config/plurcast/config.toml
    Database location: ~/.local/share/plurcast/posts.db

    Override with environment variables:
        PLURCAST_CONFIG    - Path to config file
        PLURCAST_DB_PATH   - Path to database file

EXIT CODES:
    0 - Success on all platforms
    1 - Posting failed on at least one platform
    2 - Authentication error (missing/invalid credentials)
    3 - Invalid input (empty content, malformed arguments)

OUTPUT FORMAT:
    Text format (default): platform:post_id (one per line)
        Example: nostr:note1abc123...

    JSON format (--format json): Machine-readable JSON array
        Example: [{"platform":"nostr","success":true,"post_id":"note1..."}]

For more information, visit: https://github.com/plurcast/plurcast

Usage: plur-post [OPTIONS] [CONTENT]

Arguments:
  [CONTENT]
          Content to post (reads from stdin if not provided)

Options:
  -p, --platform <PLATFORM>
          Target specific platform (nostr, mastodon, or ssb). Can be specified
          multiple times. If not specified, uses default platforms from config.

          [possible values: nostr, mastodon, ssb]

  -a, --account <ACCOUNT>
          Account to use for posting. If not specified, uses the active account
          for each platform.

  -d, --draft
          Save as draft without posting to any platform

  -f, --format <FORMAT>
          Output format: 'text' (default, one line per platform) or 'json'
          (machine-readable array)

          [default: text]

  -v, --verbose
          Enable verbose logging to stderr (useful for debugging)

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

---

## Agent-Friendly Features

### Exit Codes (Strictly Defined)

Exit codes are tested and guaranteed:
- **0**: Success on all platforms
- **1**: Posting failed on at least one platform (network error, rate limit, etc.)
- **2**: Authentication error (missing/invalid credentials)
- **3**: Invalid input (empty content, too large, malformed)

Use exit codes to determine next actions programmatically.

### Draft Mode (--draft)

- Saves content to database without posting to any network
- Always succeeds (exit code 0)
- No authentication required
- Useful for testing, validation, and workflow development
- Output format: `draft:uuid` (text) or JSON with `{"status":"draft","post_id":"uuid"}`

### JSON Output (--format json)

Machine-readable output for parsing:

```bash
plur-post "Test" --format json
# Output:
# [{"platform":"nostr","success":true,"post_id":"note1abc..."}]
```

Parse with `jq`:
```bash
plur-post "Test" --format json | jq -r '.[].post_id'
```

### Verbose Mode (--verbose)

Shows detailed logging to stderr (stdout remains clean for piping):

```bash
plur-post "Test" --verbose 2>debug.log
# Logs auth, relay connections, posting progress to debug.log
# Output (stdout) still clean: nostr:note1abc...
```

### Input Validation

- Maximum content: 100KB (100,000 bytes)
- Validates early to fail fast
- Prevents DoS via infinite streams (`cat /dev/zero | plur-post` fails immediately)
- Error messages include sizes: `Content too large: 200000 bytes (maximum: 100000 bytes)`

### Unix Composability

Designed for pipelines:

```bash
# Compose with other tools
fortune | plur-post --platform nostr

# Process before posting
cat draft.txt | sed 's/foo/bar/g' | plur-post

# Conditional posting
if [ $? -eq 0 ]; then
  echo "Success!" | plur-post --draft
fi
```

---

## Architecture

### Workspace Structure

Cargo workspace with multiple binaries:

- **libplurcast/** - Shared library
  - `config.rs` - Configuration (TOML, env vars, XDG paths)
  - `credentials.rs` - Credential management (keyring, encrypted files, plain files)
  - `db.rs` - SQLite with sqlx (compile-time verified queries)
  - `error.rs` - Error types with exit code mapping
  - `service/` - Service layer (PostingService, HistoryService, DraftService)
  - `platforms/` - Platform implementations (Nostr, SSB, Mastodon planned)

- **plur-post/** - Post content CLI
- **plur-creds/** - Manage credentials CLI
- **plur-history/** - View post history CLI
- **plur-setup/** - Interactive setup wizard

### Key Design Principles

**Configuration Priority**:
1. Environment variables (PLURCAST_CONFIG, PLURCAST_DB_PATH)
2. User-specified paths in config file (with ~ expansion)
3. XDG Base Directory defaults (~/.config/plurcast/, ~/.local/share/plurcast/)

**Database Schema**:
- `posts` - User's authored posts (id, content, created_at, status, metadata)
- `post_records` - Platform-specific posting records with foreign key to posts
- `accounts` - Multi-account support (platform, account_name, is_active)
- `credential_metadata` - Tracks credential storage backend per account
- Migrations in `libplurcast/migrations/`, run automatically via sqlx::migrate!

**Platform Abstraction**:
- `Platform` trait: `authenticate()`, `post()`, `validate_content()`, `name()`
- All platforms use async_trait
- Platforms handle their own authentication and key management

**Error Handling**:
- Custom `PlurcastError` enum maps to exit codes
- Errors go to stderr, output to stdout
- Machine-readable errors available via JSON format

**Credential Management**:
- Supports multiple backends: keyring (OS-native), encrypted files, plain files
- Automatic migration from old to new credential formats
- Multi-account support per platform
- Secure by default (600 permissions on files)

---

## Common Development Commands

### Building and Testing

```bash
# Debug build
cargo build

# Release build (optimized, for production use)
cargo build --release

# Run all tests (must pass before commits)
cargo test

# Run tests with output visible
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests for specific package
cargo test -p libplurcast

# Check without building (fast)
cargo check
```

### Running Binaries

```bash
# Run from source (debug)
cargo run -p plur-post -- "Hello world"

# Run with flags
cargo run -p plur-post -- "Test" --verbose --draft

# Run release binary (faster)
./target/release/plur-post "Hello world"

# Test with stdin
echo "Test post" | cargo run -p plur-post

# Test with platform selection
cargo run -p plur-post -- "Test" --platform nostr

# Test JSON output
cargo run -p plur-post -- "Test" --format json

# Generate test keys
cargo run --example generate_nostr_key
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
```

---

## Important Implementation Notes

### SQLx Compile-Time Verification

This project uses sqlx with compile-time query verification:
- Database queries are checked at compile time against actual schema
- A database is created from migrations during compilation
- If you add migrations, run `cargo sqlx prepare` to update cached metadata
- All SQL in the code is verified to match the schema

### Platform Implementation

When adding new platforms (Mastodon support planned):

1. Create module in `libplurcast/src/platforms/<platform>/`
2. Implement `Platform` trait with async_trait
3. Add configuration struct to `config.rs`
4. Add credential handling to `credentials.rs`
5. Add platform enum variant
6. Add tests following existing patterns (see `platforms/nostr/tests.rs`)

### Credential Security

- **Never commit credentials** - .gitignore covers key files, .env, etc.
- Private keys stored in separate files (not config.toml)
- Files created with mode 600 on Unix
- Support multiple formats: hex, bech32 (nsec), JSON, etc.
- Never log private keys (only log key lengths for debugging)

### Exit Code Contract

Exit codes are **strictly defined and tested** - do not change without updating:
- Documentation (README.md, CLAUDE.md)
- Tests (exit code integration tests)
- Error handling code

The exit code contract is part of the public API.

### Input Validation Details

**Location**: `plur-post/src/main.rs` in `get_content()` function

**Constant**: `MAX_CONTENT_LENGTH = 100_000` (100KB)

**Strategy**:
1. Argument input: Check `content.len() > MAX_CONTENT_LENGTH` before processing
2. Stdin input: Use `stdin.lock().take(MAX_CONTENT_LENGTH + 1)` to limit read
3. Read one extra byte to distinguish "at limit" from "over limit"
4. Fail immediately without reading entire stream

**Security Properties**:
- Never allocates more than MAX_CONTENT_LENGTH bytes
- Prevents memory exhaustion (infinite streams, huge arguments)
- Fails fast (< 100ms) on oversized content
- Error messages never include content samples

**Testing**: See `plur-post/tests/validation_unit_tests.rs` and `attack_scenarios.rs`

### Testing Requirements

Before committing, ensure:
- [ ] All tests pass: `cargo test`
- [ ] No clippy warnings: `cargo clippy`
- [ ] Code is formatted: `cargo fmt --check`

Test coverage must include:
- Configuration parsing (valid, invalid, missing fields)
- Environment variable overrides
- Database operations (CRUD, constraints, migrations)
- Platform implementations (auth, posting, validation)
- CLI flags and input methods
- Exit codes verification
- Input validation (under/at/over limit, attack scenarios)

Use `tempfile::TempDir` for tests that create files/directories.

---

## Library Version Requirements

**Always check up-to-date documentation** when implementing features:

- **nostr-sdk** v0.35 - Nostr protocol (rust-nostr/nostr)
- **sqlx** v0.8 - Async SQL with compile-time verification (launchbadge/sqlx)
- **tokio** v1 - Async runtime (tokio-rs/tokio)
- **clap** v4.5 - CLI parser with derive macros (clap-rs/clap)
- **keyring** v2.3 - OS-native credential storage
- **serde** v1 - Serialization framework

Review official docs and examples in library repos to verify current APIs.

---

## Git Workflow

- Write clear commit messages explaining "why" not just "what"
- Ensure `cargo test` passes before committing
- Follow existing commit style (see `git log` for examples)
- Never commit secrets (.env, key files, credentials)
- Use conventional commit format when applicable

---

## Future Platform Support

Planned integrations:
- **Mastodon**: Using megalodon client, OAuth tokens
- **Bluesky**: Using atrium-api client (atrium-rs)
- **Scheduling**: plur-queue and plur-send for deferred posting

SSB (Scuttlebutt) is currently experimental - see `WARP.md` for status.

When implementing new platforms, follow the Nostr pattern in architecture and testing.
