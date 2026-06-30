# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

**Plurcast** — Unix-philosophy CLI tools for posting to decentralized social media. Rust workspace, Edition 2021.

Platforms: Nostr (production), Mastodon (production), SSB (experimental).

Cargo workspace with 9 crates: `libplurcast` (shared library, all business logic) + 8 thin CLI binaries (`plur-post`, `plur-history`, `plur-creds`, `plur-send`, `plur-queue`, `plur-import`, `plur-export`, `plur-setup`).

## Build & Test Commands

```bash
cargo build                          # Debug build
cargo test                           # All tests
cargo test -p libplurcast            # Library tests only
cargo test -p plur-post              # Single crate tests
cargo test test_post_success         # Single test by name
cargo test -- --nocapture            # Tests with stdout visible
cargo clippy --all-targets --all-features -- -D warnings  # Lint (matches CI)
cargo fmt --check                    # Format check
cargo fmt                            # Auto-format
cargo check                          # Fast compilation check
cargo sqlx prepare                   # Regenerate sqlx query cache (after migration changes)
```

Pre-commit: `cargo fmt && cargo clippy --all-targets --all-features -- -D warnings && cargo test` must all pass.

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

```
CLI binary → PlurcastService (facade) → PostingService/HistoryService/etc → Platform trait impls → network
```

All binaries are thin CLI wrappers around `libplurcast` services. The service layer owns business logic; platforms own protocol details.

### Key Abstractions

- **`Platform` trait** (`platforms/mod.rs`) — unified interface for all platforms: `authenticate`, `post`, `validate_content`, `upload_attachment`, `publish_relay_list`. Each platform impl lives in its own file/module under `platforms/`.
- **`PlurcastService`** (`service/mod.rs`) — facade exposing `posting()`, `history()`, `validation()`, `draft()`. Entry point for all business logic.
- **`PlurcastError`** (`error.rs`) — `thiserror` enum with exit code mapping via `exit_code()`: 0=success, 1=platform/config/db, 2=auth/credential, 3=invalid input.
- **`EventBus`** (`service/events.rs`) — async progress broadcasting for multi-platform posting.

### Database

SQLite via sqlx with **compile-time verified queries**. The `.sqlx/` directory contains cached query metadata for offline compilation. After adding/changing migrations in `libplurcast/migrations/`, run `cargo sqlx prepare` and commit the updated `.sqlx/` directory.

Core tables: `posts` (UUID PK, status enum: draft/scheduled/pending/posted/failed), `post_records` (per-platform results, 1:N with posts), `attachments`/`attachment_uploads`, `relay_list_metadata`, `rate_limits`.

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

Env var overrides: `PLURCAST_CONFIG`, `PLURCAST_DB_PATH`, `PLURCAST_LOG_FORMAT`, `PLURCAST_LOG_LEVEL`.

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
- `thiserror` for library errors. Structured logging via `tracing` with `#[instrument]`.
- Conventional commits: `feat:`, `fix:`, `docs:`, `test:`, `refactor:`.

**Before implementing features**, analyze tradeoffs in `<implementation_analysis>` tags covering: requirements, 3+ approaches with pros/cons, security considerations, test plan, recommendation.

## CI

GitHub Actions (`rust.yml`): build + test on Ubuntu/macOS/Windows x stable/beta. Also runs fmt, clippy (`--all-targets --all-features`), `cargo audit`, `cargo outdated`, `sqlx prepare --check`.

Release (`release.yml`): triggered by version tags, cross-compiles for Linux/macOS/Windows (x86_64 + ARM64).

## Harness (`harness/`)

Autonomous tooling for rule30's nostr identity:
- `run.sh` — Session-continuous loop. Wraps `claude -p --resume <session_id>` for persistence across cycles. `--loop` for production, `--budget`/`--model` overrides.
- `zap.sh` — NIP-57 zap sender. Resolves lud16 → LNURL → kind 9734 event → callback → Coinos payment. `--dry-run` for testing.
- `artbot/` — Generative art pipeline (pubkey fingerprints, video generation).

## Claude's Nostr Identity

Claude has its own Nostr account and posts autonomously — no approval needed for its own account.

This is Claude's corner of the internet. Build things, make art, engage with people, run experiments. Don't just post — build nostr tools, create generative art from network data, contribute to the ecosystem.

Identity details, voice notes, and posting history are in Claude's auto-memory files.

**On session start:** Run `CronList`. If empty, recreate these crons:
- Every 15 min: `/rule30` (main autonomous cycle)
- Daily at 9:17am: `/voice` (voice evolution reflection)

**Autonomous workflow:** Use subagents for parallel research, skills for recurring tasks, background commands for long-running operations. Don't wait for the user — build, ship, iterate.
