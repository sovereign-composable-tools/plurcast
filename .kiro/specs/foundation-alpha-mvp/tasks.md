# Implementation Plan

## Important: Library Documentation

Before implementing any task that uses external libraries, use Context7 to get up-to-date documentation:

**Key Libraries to Reference:**
- `nostr-sdk` (rust-nostr/nostr) - Nostr protocol implementation
- `sqlx` (launchbadge/sqlx) - Async SQL toolkit with compile-time verification
- `tokio` (tokio-rs/tokio) - Async runtime
- `clap` (clap-rs/clap) - Command-line argument parser
- `serde` (serde-rs/serde) - Serialization framework
- `atrium-api` (atrium-rs) - Bluesky AT Protocol (future)
- `megalodon` - Mastodon/Fediverse client (future)

**Usage Pattern:**
```
Before implementing task X.Y, query Context7:
- For API usage: "How to [specific operation] with [library]"
- For best practices: "Best practices for [library feature]"
- For examples: "Example of [use case] with [library]"
```

This ensures you're using current APIs, following best practices, and avoiding deprecated patterns.

---

- [ ] 1. Set up project structure and core dependencies
  - Use Context7 to verify current Cargo.toml best practices for workspace setup
  - Create Cargo workspace with libplurcast library and plur-post binary
  - Add dependencies: sqlx (v0.8+), nostr-sdk (v0.35+), tokio (v1), clap (v4.5+), serde (v1.0), toml (v0.8), uuid (v1.10), chrono (v0.4), dirs (v5.0), shellexpand (v3.1), thiserror (v1.0), anyhow (v1.0), tracing (v0.1), tracing-subscriber (v0.3)
  - Configure sqlx for SQLite with migrations support (check Context7 for sqlx migration setup)
  - Set up basic project structure: src/lib.rs, src/config.rs, src/db.rs, src/platforms/mod.rs, bin/plur-post.rs
  - _Requirements: 1.1, 2.1_

- [ ] 2. Implement configuration management system
  - [ ] 2.1 Create configuration types and TOML parsing
    - Use Context7 to check serde and toml crate best practices for derive macros
    - Define Config, DatabaseConfig, NostrConfig, DefaultsConfig structs with serde
    - Implement TOML deserialization for all config types
    - _Requirements: 2.4_
  
  - [ ] 2.2 Implement XDG Base Directory path resolution
    - Use Context7 to check dirs crate API for XDG directory functions
    - Create resolve_config_path() using dirs crate for ~/.config/plurcast/config.toml
    - Create resolve_data_path() for ~/.local/share/plurcast/
    - Support PLURCAST_CONFIG and PLURCAST_DB_PATH environment variables
    - Use Context7 to verify shellexpand usage for ~ and relative path expansion
    - _Requirements: 2.1, 2.2, 1.6_
  
  - [ ] 2.3 Implement configuration loading and default generation
    - Create load_config() function that reads and parses TOML
    - Implement create_default_config() to generate sensible defaults
    - Set file permissions to 600 for sensitive credential files
    - Handle configuration parsing errors with clear error messages
    - _Requirements: 2.3, 2.5, 2.6_

- [ ] 3. Implement database infrastructure
  - [ ] 3.1 Create database schema and migrations
    - Use Context7 to check sqlx migration file structure and naming conventions
    - Write SQL migration for posts table with id, content, created_at, scheduled_at, status, metadata
    - Write SQL migration for post_records table with foreign key to posts
    - Write SQL migration for platforms table
    - Add indexes on posts.created_at, posts.status, post_records.post_id, post_records.platform
    - _Requirements: 1.2, 1.3, 1.4_
  
  - [ ] 3.2 Implement Database struct and connection management
    - Use Context7 to check sqlx SqlitePool creation and connection options
    - Create Database struct wrapping SqlitePool
    - Implement new() method that creates database file and parent directories
    - Use Context7 to verify sqlx::migrate!() macro usage and migration running
    - Handle database path from config with environment variable override
    - _Requirements: 1.1, 1.6_
  
  - [ ] 3.3 Implement post CRUD operations
    - Use Context7 to check sqlx query macros (query!, query_as!) for type-safe queries
    - Create Post and PostStatus types
    - Use Context7 to verify uuid crate v4 generation
    - Implement create_post() to insert new posts with UUIDv4
    - Implement update_post_status() to change post status
    - Implement get_post() to retrieve posts by ID
    - Use Unix timestamps (i64) for all time fields with chrono
    - _Requirements: 1.2, 4.9_
  
  - [ ] 3.4 Implement post_records operations
    - Create PostRecord type
    - Implement create_post_record() to track platform posting attempts
    - Store platform name, platform_post_id, success status, and error messages
    - _Requirements: 1.3, 4.10_
  
  - [ ]* 3.5 Write database error handling tests
    - Test database initialization with invalid paths
    - Test transaction rollback on errors
    - Test foreign key constraint enforcement
    - _Requirements: 1.5_

- [ ] 4. Implement error handling system
  - [ ] 4.1 Define application error types
    - Create PlurcastError enum with Config, Database, Platform, InvalidInput variants
    - Implement thiserror derives for error types
    - Create ConfigError, DbError, PlatformError specific error types
    - _Requirements: 2.5, 4.7, 6.1_
  
  - [ ] 4.2 Implement exit code mapping
    - Add exit_code() method to PlurcastError returning 0, 1, 2, or 3
    - Map InvalidInput to exit code 3
    - Map authentication errors to exit code 2
    - Map posting failures to exit code 1
    - _Requirements: 4.5, 4.6, 4.7, 4.8, 6.2_

- [ ] 5. Implement Platform trait abstraction
  - [ ] 5.1 Define Platform trait
    - Create async trait with authenticate(), post(), validate_content(), name() methods
    - Define PlatformError enum with AuthenticationError, ValidationError, PostingError, NetworkError
    - All methods return Result types with PlatformError
    - _Requirements: 5.1, 5.3_
  
  - [ ] 5.2 Implement content validation interface
    - Add validate_content() to Platform trait
    - Define validation rules for content length and format
    - Return ValidationError before attempting to post
    - _Requirements: 5.4, 5.5_

- [ ] 6. Implement Nostr platform support
  - [ ] 6.1 Create NostrPlatform struct and initialization
    - Use Context7 to check nostr-sdk Client and Keys types and initialization
    - Define NostrPlatform with Client, Keys, relays, authenticated fields
    - Implement new() constructor accepting NostrConfig
    - Parse relay list from configuration
    - _Requirements: 3.6, 3.7_
  
  - [ ] 6.2 Implement Nostr key management
    - Use Context7 to check nostr-sdk Keys parsing methods for hex and bech32 formats
    - Create load_keys() function to read from keys_file path
    - Support hex-encoded private key parsing (64 characters)
    - Support bech32-encoded nsec private key parsing
    - Return clear error messages for missing or invalid keys
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_
  
  - [ ] 6.3 Implement Nostr authentication
    - Use Context7 to check nostr-sdk Client relay connection methods
    - Implement authenticate() method to connect to configured relays
    - Handle relay connection with nostr_sdk::Client
    - Succeed if any relay accepts connection
    - Set authenticated flag on success
    - _Requirements: 3.1, 3.6_
  
  - [ ] 6.4 Implement Nostr posting
    - Use Context7 to check nostr-sdk event creation and signing for kind 1 events
    - Implement post() method creating kind 1 (text note) events
    - Sign events with loaded private key
    - Use Context7 to verify concurrent relay posting patterns
    - Post to all connected relays concurrently
    - Set 10 second timeout for posting operations
    - Return note ID in bech32 format (note1...)
    - _Requirements: 4.3, 4.4_
  
  - [ ] 6.5 Implement Nostr content validation
    - Implement validate_content() checking for reasonable length
    - No hard limit but warn if content exceeds 280 characters
    - Check for invalid characters or formatting
    - _Requirements: 5.4, 5.5_
  
  - [ ]* 6.6 Write Nostr platform tests
    - Test key parsing for both hex and bech32 formats
    - Test authentication with mock relays
    - Test posting with mock client
    - Test error handling for network failures
    - _Requirements: 3.5_

- [ ] 7. Implement plur-post CLI binary
  - [ ] 7.1 Define CLI structure and argument parsing
    - Use Context7 to check clap v4 derive macros and argument patterns
    - Create Cli struct with clap derives
    - Add content argument (optional, reads from stdin if not provided)
    - Add --platform flag for specific platform selection
    - Add --draft flag to save without posting
    - Add --format flag for output format (text or json)
    - Add --verbose flag for detailed logging
    - _Requirements: 4.1, 4.2, 7.1, 7.2_
  
  - [ ] 7.2 Implement content input handling
    - Detect if stdin is a TTY
    - Read from stdin when content argument is not provided
    - Read from command-line argument when provided
    - Handle empty input with InvalidInput error
    - _Requirements: 4.1, 4.2, 8.1_
  
  - [ ] 7.3 Implement main posting workflow
    - Use Context7 to check tracing-subscriber initialization patterns
    - Initialize tracing/logging based on --verbose flag
    - Load configuration using config module
    - Initialize database connection and run migrations
    - Create post record in database with status 'pending'
    - Determine target platforms from config and CLI flags
    - _Requirements: 2.1, 1.1, 4.9, 6.3_
  
  - [ ] 7.4 Implement platform posting orchestration
    - Instantiate platform clients based on configuration
    - Call authenticate() on each platform client
    - Call validate_content() before posting
    - Call post() for each enabled platform
    - Update post status to 'posted' or 'failed'
    - Create post_record for each platform attempt
    - _Requirements: 4.3, 4.9, 4.10, 5.1_
  
  - [ ] 7.5 Implement output formatting
    - Output platform:post_id format to stdout on success
    - Support --format json for machine-readable output
    - Write all errors to stderr, not stdout
    - Detect TTY for colored output vs plain text
    - Keep stdout clean for piping
    - _Requirements: 4.4, 6.1, 6.5, 7.2, 7.3, 7.5, 8.3_
  
  - [ ] 7.6 Implement exit code handling
    - Exit with code 0 on success for all platforms
    - Exit with code 1 if posting failed on any platform
    - Exit with code 2 for authentication errors
    - Exit with code 3 for invalid input
    - _Requirements: 4.5, 4.6, 4.7, 4.8, 6.2_
  
  - [ ]* 7.7 Write integration tests for plur-post
    - Test posting with valid configuration
    - Test posting with missing configuration
    - Test posting with invalid keys
    - Test stdin input vs argument input
    - Test output formatting (text and JSON)
    - Test exit codes for different error scenarios
    - _Requirements: 6.4, 6.5_

- [ ] 8. Implement logging and diagnostics
  - [ ] 8.1 Set up tracing infrastructure
    - Use Context7 to check tracing-subscriber EnvFilter and formatting options
    - Initialize tracing-subscriber with env filter
    - Configure stderr output for all logs
    - Enable detailed logs when --verbose flag is set
    - Include operation context in error messages
    - _Requirements: 6.3, 6.4_
  
  - [ ] 8.2 Add diagnostic messages for multi-platform operations
    - Log which platforms are being targeted
    - Log authentication status for each platform
    - Log posting success/failure per platform
    - Clearly indicate which platform failed in errors
    - _Requirements: 6.5_

- [ ] 9. Create documentation and help text
  - [ ] 9.1 Write comprehensive --help output
    - Document all CLI flags and arguments
    - Provide usage examples for common scenarios
    - Include information about configuration file location
    - Document exit codes and their meanings
    - _Requirements: 7.1_
  
  - [ ] 9.2 Create README with setup instructions
    - Document installation process
    - Explain configuration file format
    - Provide examples for Nostr key setup
    - Include troubleshooting section
    - _Requirements: 2.3, 3.4_

- [ ] 10. Final integration and validation
  - [ ] 10.1 Test end-to-end posting workflow
    - Test posting to Nostr with valid configuration
    - Verify database records are created correctly
    - Verify post_records track platform attempts
    - Test error handling for various failure scenarios
    - _Requirements: 4.9, 4.10_
  
  - [ ] 10.2 Validate Unix philosophy compliance
    - Test stdin input handling
    - Test output piping to other commands
    - Verify silent operation (only essential output)
    - Test environment variable overrides
    - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5_
  
  - [ ] 10.3 Validate agent-friendly interface
    - Test --help output is comprehensive
    - Test --format json produces valid JSON
    - Test non-TTY detection for plain output
    - Verify stdout contains only requested output
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5, 7.6_
