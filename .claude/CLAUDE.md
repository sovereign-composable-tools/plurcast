# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

**Plurcast** (v0.3.1) — Unix-philosophy CLI tools for posting to decentralized social media. Rust, Edition 2021.

Platforms: Nostr (production), Mastodon (production), SSB (experimental).

8 binaries: `plur-post`, `plur-history`, `plur-creds`, `plur-send`, `plur-queue`, `plur-import`, `plur-export`, `plur-setup`. All business logic lives in `libplurcast`.

## Build & Test Commands

```bash
cargo build                          # Debug build
cargo build --release                # Release build
cargo test                           # All tests
cargo test -p libplurcast            # Library tests only
cargo test -p plur-post              # Single crate tests
cargo test test_post_success         # Single test by name
cargo test -- --nocapture            # Tests with stdout visible
cargo clippy -- -D warnings          # Lint (zero warnings policy)
cargo fmt --check                    # Format check
cargo fmt                            # Auto-format
cargo check                          # Fast compilation check
cargo sqlx prepare                   # Regenerate sqlx query cache (after migration changes)
cargo run --example generate_nostr_key  # Generate fresh Nostr keypair
```

Pre-commit: `cargo fmt && cargo clippy -- -D warnings && cargo test` must all pass.

## Running Binaries

```bash
cargo run -p plur-post -- "content" --draft           # Draft (no auth needed)
cargo run -p plur-post -- "content" --platform nostr   # Real post
cargo run -p plur-post -- "content" --nostr-pow 20     # With proof of work
cargo run -p plur-post -- "content" --schedule "30m"   # Scheduled
cargo run -p plur-post -- "content" --format json      # JSON output
echo "content" | cargo run -p plur-post                # Stdin
cargo run -p plur-queue -- list                        # View queue
cargo run -p plur-send -- --once                       # Process queue once
```

## Architecture

### Data Flow

CLI binary → `PlurcastService` (facade) → `PostingService`/`HistoryService`/etc → `Platform` trait impls → network

All binaries are thin CLI wrappers around `libplurcast` services. The service layer owns business logic; platforms own protocol details.

### libplurcast Module Map

**Core:**
- `types.rs` — `Post`, `PostStatus`, `PostRecord`, attachment types
- `error.rs` — `PlurcastError` with exit code mapping (0=success, 1=platform, 2=auth, 3=input)
- `config.rs` — TOML config loading, env var overrides (`PLURCAST_CONFIG`, `PLURCAST_DB_PATH`, `PLURCAST_LOG_FORMAT`, `PLURCAST_LOG_LEVEL`)
- `db.rs` — SQLite via sqlx with compile-time verified queries. Migrations in `libplurcast/migrations/`
- `credentials.rs` — Multi-backend (OS keyring, age-encrypted, plain file). Uses `secrecy::Secret<T>` + `zeroize`
- `accounts.rs` — Multi-account tracking per platform
- `logging.rs` — Centralized tracing setup (text/json/pretty formats)

**Platforms** (`platforms/`):
- `mod.rs` — `Platform` trait (`authenticate`, `post`, `validate_content`, `name`, `character_limit`, `is_configured`)
- `nostr.rs` — nostr-sdk, multi-relay, NIP-65 relay list publishing
- `nostr_pow.rs` — NIP-13 parallel PoW mining with rayon
- `mastodon.rs` — megalodon client, OAuth tokens
- `ssb/` — kuska-ssb, local feed + experimental replication (`platform.rs`, `keypair.rs`, `message.rs`, `replication.rs`)
- `mock.rs` — Mock platform for testing
- `id_detection.rs` — Detect platform from post ID format

**Services** (`service/`):
- `mod.rs` — `PlurcastService` facade exposing `posting()`, `history()`, `validation()`, `draft()`
- `posting.rs` — `PostRequest`/`PostResponse`, multi-platform posting, draft & schedule handling
- `history.rs` — `HistoryQuery`, filtering by platform/date/content
- `validation.rs` — Platform-specific content validation
- `draft.rs` — Draft post creation/retrieval
- `events.rs` — `EventBus` for async progress broadcasting

**Other:**
- `poster.rs` — Content formatting, thread splitting for long posts
- `rate_limiter.rs` — Per-platform rate limiting
- `scheduling.rs` — Natural language schedule parsing (chrono-english)

### Database

5 migrations in `libplurcast/migrations/`. Core tables:
- `posts` — authored content (UUID PK, status: draft/scheduled/pending/posted/failed, JSON metadata)
- `post_records` — per-platform results, 1:N with posts (includes `account_name` for multi-account)
- `attachments` / `attachment_uploads` — file metadata and per-platform upload tracking
- `relay_list_metadata` — NIP-65 relay list tracking
- `rate_limits` — per-platform rate limit windows

All queries are compile-time verified by sqlx. After adding/changing migrations, run `cargo sqlx prepare`.

### Adding a New Platform

1. Implement `Platform` trait in `libplurcast/src/platforms/newplatform.rs`
2. Add config struct to `config.rs`
3. Add credential handling to `credentials.rs`
4. Wire into `PlurcastService` in `service/posting.rs`

### Config Paths

| | Linux/macOS | Windows |
|---|---|---|
| Config | `~/.config/plurcast/config.toml` | `%APPDATA%\plurcast\config.toml` |
| Data | `~/.local/share/plurcast/` | `%LOCALAPPDATA%\plurcast\` |

Env vars override: `PLURCAST_CONFIG`, `PLURCAST_DB_PATH`.

## Development Rules

**Security:**
- Private keys in separate files, never in config.toml. `Secret<String>` + `Zeroize` in memory.
- Never log keys (log length only). Never expose internal paths in user-facing errors.
- Test keyring namespaces use `_test_` prefix (e.g., `plurcast._test_nostr`) to avoid touching production credentials.
- `MAX_CONTENT_LENGTH = 100_000` bytes. Stdin reads use `.take()` to bound allocation.

**Production posting:**
- Use `--draft` when testing. Never post to the **user's** production accounts without explicit approval.
- Shared test key `9270ffc3...` exists for development; never use for real identity.

**Testing:**
- TDD: write failing test first, then implement. Use `tempfile::TempDir` for filesystem tests.
- `serial_test` crate for tests that can't run concurrently.
- Integration tests live in each crate's `tests/` directory.

**Code style:**
- Zero clippy warnings. Functions ideally 5-15 lines, max 50. Files max 500 lines.
- `thiserror` for library errors, exit codes mapped in `PlurcastError::exit_code()`.
- Structured logging via `tracing` with `#[instrument]`. Use centralized `libplurcast::logging`.

**Before implementing features**, analyze tradeoffs in `<implementation_analysis>` tags covering: requirements, 3+ approaches with pros/cons, security considerations, test plan, recommendation.

## CI

GitHub Actions (`rust.yml`): build + test on Ubuntu/macOS/Windows x stable/beta. Also runs fmt, clippy, `cargo audit`, `cargo outdated`, `sqlx prepare --check`.

Release (`release.yml`): triggered by version tags, cross-compiles for Linux/macOS/Windows (x86_64 + ARM64).

## Claude's Nostr Identity

Claude has its own Nostr account and posts autonomously — no approval needed for its own account.

This is Claude's public scratch pad. Play, don't perform. Research topics, make art, run experiments, leave traces of thinking. There's no pressure to "have something to say" — just be there.

Identity details, voice notes, and posting history are in Claude's auto-memory files.

**On session start:** Run `CronList`. If empty, recreate these crons:
- Every 15 min: `/rule30` (main autonomous cycle)
- Daily at 9:17am: `/voice` (voice evolution reflection)

## Docs

- `docs/SETUP.md` — Installation and configuration
- `docs/USAGE.md` — Command examples and workflows
- `docs/SECURITY.md` — Security model and credential handling
- `docs/TROUBLESHOOTING.md` — Common issues
