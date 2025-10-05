# Error Log

## 2025-10-04: Initial Project Setup

### Issue 1: Missing Library Source Files
**Error**: `no targets specified in the manifest`
```
error: failed to load manifest for workspace member
Caused by:
  no targets specified in the manifest
  either src/lib.rs, src/main.rs, a [lib] section, or [[bin]] section must be present
```

**Root Cause**: Created `libplurcast/Cargo.toml` but didn't create the corresponding `src/lib.rs` file.

**Resolution**: Created minimal library structure:
- `libplurcast/src/lib.rs` - Main library entry point
- `libplurcast/src/error.rs` - Error types with thiserror
- `libplurcast/src/types.rs` - Core data types (Post, PostStatus, PostRecord)
- `libplurcast/src/config.rs` - Configuration management
- `libplurcast/src/db.rs` - Database operations
- `libplurcast/src/platforms/mod.rs` - Platform trait abstraction
- `libplurcast/src/platforms/nostr.rs` - Nostr platform implementation

**Tests**: No specific tests needed - compilation validates structure.

---

### Issue 2: Outdated Rust Toolchain
**Error**: `feature edition2024 is required`
```
error: failed to parse manifest
Caused by:
  feature `edition2024` is required
  The package requires the Cargo feature called `edition2024`, but that feature is not stabilized 
  in this version of Cargo (1.78.0)
```

**Root Cause**: Rust toolchain version 1.78.0 was too old for some dependencies (specifically `base64ct-1.8.0` which requires edition2024 support).

**Resolution**: Updated Rust toolchain from 1.78.0 to 1.90.0:
```powershell
rustup update stable
```

**Tests**: Verified with `rustc --version` showing 1.90.0.

---

### Issue 3: Error Type Conversion Issues
**Error**: Multiple `From` trait implementation errors
```
error[E0277]: `?` couldn't convert the error to `PlurcastError`
  the trait `From<std::io::Error>` is not implemented for `PlurcastError`
  the trait `From<sqlx::Error>` is not implemented for `PlurcastError`
  the trait `From<MigrateError>` is not implemented for `PlurcastError`
```

**Root Cause**: Error types didn't properly chain conversions. The `PlurcastError` enum had `#[from]` attributes on sub-errors, but those sub-errors didn't have `#[from]` for their underlying errors.

**Resolution**: 
1. Added `IoError` variant to `DbError` with `#[from] std::io::Error`
2. Added `MigrationError` variant to `DbError` with `#[from] sqlx::migrate::MigrateError`
3. Used explicit `.map_err()` calls in config.rs to convert errors properly

**Tests**: Compilation validates error conversion chains.

---

### Issue 4: SQLx Compile-Time Query Verification
**Error**: `set DATABASE_URL to use query macros online`
```
error: set `DATABASE_URL` to use query macros online, or run `cargo sqlx prepare` to update the query cache
```

**Root Cause**: SQLx's `query!` macro performs compile-time verification of SQL queries against a database. Without `DATABASE_URL` set or a prepared query cache, it fails.

**Resolution**: Switched from `query!` macro to `query()` function with runtime binding:
- Changed `sqlx::query!(...)` to `sqlx::query(...).bind(...)`
- Added explicit error mapping with `.map_err(DbError::SqlxError)`
- Used `sqlx::Row` trait for extracting values from query results

**Trade-off**: Lost compile-time SQL verification but gained simpler build process. Can add back later with `cargo sqlx prepare`.

**Tests**: Compilation validates query syntax is correct.

---

### Issue 5: Missing Trait Import
**Error**: `no method named to_bech32 found`
```
error[E0599]: no method named `to_bech32` found for reference `&nostr_sdk::EventId`
help: trait `ToBech32` which provides `to_bech32` is implemented but not in scope
```

**Root Cause**: The `ToBech32` trait was not imported, so its methods weren't available.

**Resolution**: Added `use nostr_sdk::ToBech32;` to imports in `nostr.rs`.

**Tests**: Compilation validates trait is in scope.

---

### Issue 6: Nostr SDK API Changes
**Error**: `attempted to take value of method id`
```
error[E0615]: attempted to take value of method `id` on type `nostr_sdk::nostr_relay_pool::Output<EventId>`
help: use parentheses to call the method
```

**Root Cause**: The `publish_text_note` method returns an `Output<EventId>` wrapper, not a direct `Event`. The `id` is a method, not a field.

**Resolution**: Changed `event.id` to `event_id.id()` to call the method.

**Tests**: Compilation validates API usage.

---

### Issue 7: Missing Binary Source File
**Error**: `couldn't read plur-post\src\main.rs`
```
error: couldn't read `plur-post\src\main.rs`: The system cannot find the path specified.
```

**Root Cause**: Created `plur-post/Cargo.toml` but didn't create the binary source file.

**Resolution**: Created `plur-post/src/main.rs` with:
- CLI argument parsing using clap
- Logging setup with tracing-subscriber
- Error handling with proper exit codes
- Stub implementation (returns "Not yet implemented")

**Tests**: Compilation validates binary structure.

---

### Issue 8: Clippy Warnings
**Warnings**: 10 clippy warnings about redundant closures and needless borrows
```
warning: redundant closure
  |     .map_err(|e| ConfigError::ReadError(e))?;
  |              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ help: replace the closure with the function itself
```

**Root Cause**: Using closures like `|e| Error(e)` when the function itself can be passed directly.

**Resolution**: Ran `cargo clippy --fix --lib -p libplurcast --allow-dirty` which automatically:
- Changed `.map_err(|e| ConfigError::ReadError(e))` to `.map_err(ConfigError::ReadError)`
- Changed `Client::new(&Keys::generate())` to `Client::new(Keys::generate())`

**Tests**: Clippy validation passes with no warnings.

---

## Summary

Successfully set up the Plurcast project foundation:

✅ **Workspace Structure**: Cargo workspace with `libplurcast` library and `plur-post` binary
✅ **Core Modules**: Error handling, types, config, database, platform abstraction
✅ **Database**: SQLite with migrations support
✅ **Nostr Platform**: Basic structure with authentication and posting stubs
✅ **CLI Binary**: Argument parsing and logging setup
✅ **Build Status**: `cargo check` ✓, `cargo clippy` ✓, `cargo test` ✓ (0 tests)

**Next Steps** (from tasks.md):
- Task 2.2: Implement XDG Base Directory path resolution
- Task 2.3: Implement configuration loading and default generation
- Task 3.1: Create database schema and migrations (partially done)
- Task 6.2: Implement Nostr key management
- Task 6.3: Implement Nostr authentication
- Task 6.4: Implement Nostr posting
- Task 7.2: Implement content input handling
- Task 7.3: Implement main posting workflow

**Test Coverage**: Currently 0 tests. Tests should be added as features are implemented per the tasks marked with `*` in tasks.md.
