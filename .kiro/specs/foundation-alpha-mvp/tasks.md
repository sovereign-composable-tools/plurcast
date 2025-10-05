# Implementation Plan

## Important: Library Documentation

Before implementing any task that uses external libraries, research up-to-date documentation:

**Key Libraries to Reference:**
- `nostr-sdk` (rust-nostr/nostr) - Nostr protocol implementation
- `sqlx` (launchbadge/sqlx) - Async SQL toolkit with compile-time verification
- `tokio` (tokio-rs/tokio) - Async runtime
- `clap` (clap-rs/clap) - Command-line argument parser
- `serde` (serde-rs/serde) - Serialization framework
- `atrium-api` (atrium-rs) - Bluesky AT Protocol (future)
- `megalodon` - Mastodon/Fediverse client (future)

**Best Practices:**
- Check official documentation for current APIs and patterns
- Review examples in library repositories
- Verify you're using the latest stable features
- Follow library-specific best practices and conventions

---

## Completed Tasks

All core functionality has been implemented:
- ✅ Project structure and dependencies
- ✅ Configuration management system with XDG paths
- ✅ Database infrastructure with SQLite and migrations
- ✅ Error handling system with exit codes
- ✅ Platform trait abstraction
- ✅ Nostr platform implementation (keys, auth, posting, validation)
- ✅ plur-post CLI binary with all features
- ✅ Logging and diagnostics
- ✅ Comprehensive documentation and help text

---

## Remaining Tasks

- [ ] 1. Implement comprehensive test suite





  - [x] 1.1 Add configuration module tests (P0 - Critical)


    - Test TOML parsing with valid/invalid configs
    - Test default config generation
    - Test XDG path resolution
    - Test environment variable overrides (PLURCAST_CONFIG, PLURCAST_DB_PATH)
    - Test path expansion (~ and shellexpand)
    - Test file permission setting (600 on Unix)
    - _Requirements: 2.1, 2.2, 2.3, 2.5, 2.6, 1.6_
    - _Estimated: 2 hours_
  
  - [x] 1.2 Add error type tests (P0 - Critical)


    - Test exit code mapping for each error type (InvalidInput→3, Auth→2, Posting→1)
    - Test error message formatting
    - Test error conversion (From traits)
    - Test authentication error detection in exit_code()
    - _Requirements: 4.5, 4.6, 4.7, 4.8, 6.2_
    - _Estimated: 1 hour_
  
  - [x] 1.3 Add types module tests (P1 - Required)


    - Test Post::new() UUID generation (verify valid UUIDv4 format)
    - Test Post::new() timestamp generation (verify Unix timestamp)
    - Test PostStatus enum variants (Pending, Posted, Failed)
    - Test PostRecord creation with all fields
    - _Requirements: 1.2, 4.9_
    - _Estimated: 1 hour_
  
  - [x] 1.4 Expand database CRUD tests (P1 - Required)


    - Test create and retrieve post (happy path)
    - Test update post status (Pending→Posted, Pending→Failed)
    - Test get nonexistent post (returns None)
    - Test create post record with success=true
    - Test create post record with success=false and error_message
    - Test concurrent operations (multiple posts simultaneously)
    - _Requirements: 1.2, 1.3, 4.9, 4.10_
    - _Estimated: 2 hours_
  
  - [x] 1.5 Write Nostr platform tests (P0 - Critical)


    - Test key parsing for hex format (64 characters)
    - Test key parsing for bech32 nsec format
    - Test key parsing with invalid formats (error handling)
    - Test content validation (empty content, >280 chars warning)
    - Test posting without authentication (should error)
    - Test authenticate() sets authenticated flag
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 5.4, 5.5_
    - _Note: Use mock/test keys, not real relay connections_
    - _Estimated: 2 hours_
  
  - [x] 1.6 Add CLI integration tests (P1 - Required)


    - Test --help flag output (verify comprehensive help text)
    - Test --version flag output
    - Test empty content error handling (exit code 3)
    - Test stdin input (pipe content to plur-post)
    - Test argument input (plur-post "content")
    - Test draft mode (--draft flag, no posting)
    - Test output formats (--format text vs --format json)
    - Test exit codes (0=success, 1=posting fail, 2=auth fail, 3=invalid input)
    - Test platform selection (--platform nostr)
    - _Requirements: 4.1, 4.2, 4.5, 4.6, 4.7, 4.8, 6.4, 6.5, 7.1, 7.2, 8.1_
    - _Dependencies: Add assert_cmd and predicates crates to dev-dependencies_
    - _Estimated: 3 hours_
-



- [x] 2. Final integration and validation






  - [x] 2.1 Test end-to-end posting workflow


    - Create test config with test Nostr keys
    - Test posting to Nostr with valid configuration
    - Verify database records are created correctly (posts table)
    - Verify post_records track platform attempts (success and failure)
    - Test error handling for missing keys file
    - Test error handling for invalid keys
    - Test partial success (if multiple platforms, some succeed)
    - _Requirements: 1.1, 1.2, 1.3, 4.9, 4.10_
    - _Estimated: 2 hours_
  
  - [x] 2.2 Validate Unix philosophy compliance


    - Test stdin input handling (echo "content" | plur-post)
    - Test output piping to other commands (plur-post | grep nostr)
    - Verify silent operation (only essential output to stdout)
    - Test environment variable overrides (PLURCAST_CONFIG, PLURCAST_DB_PATH)
    - Verify errors go to stderr, not stdout
    - Test composability with jq (--format json | jq)
    - _Requirements: 4.1, 4.2, 6.1, 7.2, 8.1, 8.2, 8.3, 8.4, 8.5_
    - _Estimated: 1 hour_
  
  - [x] 2.3 Validate agent-friendly interface


    - Test --help output is comprehensive and parseable
    - Test --format json produces valid JSON (parse with serde_json)
    - Test non-TTY detection for plain output (no colors when piped)
    - Verify stdout contains only requested output (post IDs or JSON)
    - Verify all diagnostic messages go to stderr
    - Test exit codes are consistent and documented
    - _Requirements: 6.1, 6.5, 7.1, 7.2, 7.3, 7.4, 7.5, 7.6_
    - _Estimated: 1 hour_
