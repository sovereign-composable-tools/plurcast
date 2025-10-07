# Implementation Plan

## Phase 1: Secure Credential Storage (NEW - PRIORITY)

- [ ] 1. Add secure credential storage dependencies
  - Add `keyring = "2.3"` to workspace dependencies for OS keyring integration
  - Add `rpassword = "7.3"` for secure password prompts
  - Add `age = "0.10"` for optional file encryption (fallback)
  - Update libplurcast Cargo.toml to include new dependencies
  - _Security: OS-native credential storage_

- [ ] 2. Implement credential storage abstraction
  - [ ] 2.1 Create CredentialStore trait
    - Create `libplurcast/src/credentials.rs`
    - Define CredentialStore trait with methods:
      - `store(service: &str, key: &str, value: &str) -> Result<()>`
      - `retrieve(service: &str, key: &str) -> Result<String>`
      - `delete(service: &str, key: &str) -> Result<()>`
      - `exists(service: &str, key: &str) -> Result<bool>`
    - _Security: Platform-agnostic credential API_
  
  - [ ] 2.2 Implement KeyringStore (primary)
    - Implement CredentialStore using `keyring` crate
    - Store credentials in OS keyring:
      - macOS: Keychain
      - Windows: Credential Manager
      - Linux: Secret Service (GNOME Keyring/KWallet)
    - Service name format: `plurcast.{platform}` (e.g., "plurcast.nostr")
    - Key format: `{credential_type}` (e.g., "private_key", "access_token")
    - Handle keyring unavailable gracefully (fall back to encrypted files)
    - _Security: OS-native secure storage_
  
  - [ ] 2.3 Implement EncryptedFileStore (fallback)
    - Implement CredentialStore using `age` encryption
    - Store encrypted credentials in `~/.config/plurcast/credentials/`
    - File format: `{platform}.{credential_type}.age`
    - Prompt for master password on first use
    - Cache decrypted credentials in memory during session
    - Set file permissions to 600
    - _Security: Encrypted at-rest storage when keyring unavailable_
  
  - [ ] 2.4 Implement PlainFileStore (legacy/testing only)
    - Implement CredentialStore for plain text files (current behavior)
    - Mark as deprecated with security warnings
    - Only use when explicitly configured or in tests
    - Log warning when used
    - _Security: Backward compatibility with clear warnings_
  
  - [ ] 2.5 Create CredentialManager
    - Create manager that tries stores in order:
      1. KeyringStore (if available)
      2. EncryptedFileStore (if master password set)
      3. PlainFileStore (with warning)
    - Implement migration from plain files to secure storage
    - Add `migrate_credentials()` method to upgrade existing setups
    - _Security: Automatic upgrade path_

- [ ] 3. Update configuration for secure credentials
  - [ ] 3.1 Add credential storage configuration
    - Add `[credentials]` section to config.toml:
      ```toml
      [credentials]
      # Storage backend: "keyring", "encrypted", "plain" (not recommended)
      storage = "keyring"  # default
      # For encrypted storage, prompt for password
      # For plain storage, show security warning
      ```
    - Add validation for storage backend
    - _Security: User control over storage method_
  
  - [ ] 3.2 Update platform configs to reference credentials
    - Change from file paths to credential references:
      ```toml
      [nostr]
      enabled = true
      # OLD: keys_file = "~/.config/plurcast/nostr.keys"
      # NEW: credentials stored in keyring/encrypted storage
      relays = [...]
      ```
    - Maintain backward compatibility with file paths (with warnings)
    - _Security: Remove plain text credential paths_
  
  - [ ] 3.3 Add credential setup wizard
    - Create `plur-setup` binary for interactive credential setup
    - Prompt for each platform's credentials
    - Store using CredentialManager
    - Verify credentials work before saving
    - _UX: Easy secure setup_

- [ ] 4. Update platform clients for secure credentials
  - [ ] 4.1 Update NostrClient to use CredentialManager
    - Replace file reading with `credentials.retrieve("plurcast.nostr", "private_key")`
    - Support both hex and bech32 formats
    - Cache keys in memory during session
    - _Security: No plain text key files_
  
  - [ ] 4.2 Update MastodonClient to use CredentialManager
    - Replace token file with `credentials.retrieve("plurcast.mastodon", "access_token")`
    - Store instance URL in config (not sensitive)
    - _Security: OAuth tokens in secure storage_
  
  - [ ] 4.3 Update BlueskyClient to use CredentialManager
    - Replace auth file with `credentials.retrieve("plurcast.bluesky", "app_password")`
    - Store handle in config (not sensitive)
    - _Security: App passwords in secure storage_

- [ ] 5. Add credential management commands
  - [ ] 5.1 Create plur-creds binary
    - Add `plur-creds set <platform> <credential-type>` command
    - Add `plur-creds get <platform> <credential-type>` command (for testing)
    - Add `plur-creds delete <platform> <credential-type>` command
    - Add `plur-creds list` command (show what's stored, not values)
    - Add `plur-creds migrate` command (upgrade from plain files)
    - Add `plur-creds test <platform>` command (verify credentials work)
    - _UX: Easy credential management_
  
  - [ ] 5.2 Add security audit command
    - Add `plur-creds audit` command that:
      - Checks for plain text credential files
      - Verifies file permissions
      - Reports security issues
      - Suggests improvements
    - _Security: Help users identify vulnerabilities_

- [ ] 6. Testing and documentation
  - [ ] 6.1 Add credential storage tests
    - Test KeyringStore on each platform
    - Test EncryptedFileStore encryption/decryption
    - Test migration from plain files
    - Test fallback behavior
    - Mock keyring for CI/CD
    - _Quality: Comprehensive security testing_
  
  - [ ] 6.2 Update security documentation
    - Document credential storage options
    - Document migration process
    - Add security best practices guide
    - Update ARCHITECTURE.md with security model
    - _Documentation: Clear security guidance_
  
  - [ ] 6.3 Add security warnings
    - Warn on first run if using plain text storage
    - Warn when migrating from plain files
    - Log credential access (without values)
    - _Security: User awareness_

## Phase 2: Multi-Platform Integration (AFTER SECURE CREDENTIALS)

- [x] 7. Add new dependencies to workspace





  - Add `megalodon = "0.14"` to workspace dependencies
  - Add `atrium-api = "0.24"` to workspace dependencies
  - Add `futures = "0.3"` to workspace dependencies
  - Update libplurcast Cargo.toml to include new dependencies
  - _Requirements: 2.1, 3.1_

- [x] 8. Enhance platform abstraction trait




  - [x] 2.1 Add new methods to Platform trait


    - Add `character_limit()` method returning `Option<usize>`
    - Add `is_configured()` method returning `bool`
    - Update trait documentation with examples
    - _Requirements: 1.1, 1.2, 1.3_
  - [x] 2.2 Update Nostr implementation for enhanced trait


    - Implement `character_limit()` returning None (no hard limit)
    - Implement `is_configured()` checking for keys file
    - Ensure error mapping is consistent with PlatformError types
    - _Requirements: 1.2, 1.4_
  - [x] 2.3 Add unit tests for enhanced trait methods


    - Test character_limit returns correct values
    - Test is_configured with valid and invalid configurations
    - _Requirements: 10.1_


- [x] 3. Implement Mastodon platform client




  - [x] 3.1 Create MastodonClient struct and basic implementation


    - Create `libplurcast/src/platforms/mastodon.rs`
    - Define MastodonClient struct with `Box<dyn megalodon::Megalodon + Send + Sync>` field
    - Store instance_url and character_limit fields
    - Implement `new()` constructor that:
      - Takes instance_url and access_token as parameters
      - Uses `megalodon::generator()` to create client with SNS::Mastodon
      - Example: `megalodon::generator(megalodon::SNS::Mastodon, instance_url, Some(token), None)`
    - Add `fetch_instance_info()` async method that:
      - Calls `client.get_instance().await` to fetch instance metadata
      - Extracts `max_toot_chars` or `configuration.statuses.max_characters` for character limit
      - Defaults to 500 if not available
    - _Requirements: 2.1, 2.2, 2.4_


  - [x] 3.2 Implement Platform trait for MastodonClient




    - Implement `authenticate()` method that:
      - Calls `client.verify_account_credentials().await` to validate token
      - Returns PlatformError::Authentication on failure
    - Implement `post()` method that:
      - Creates `megalodon::megalodon::PostStatusInputOptions` with status text
      - Calls `client.post_status(content, None).await`
      - Extracts status ID from response
      - Returns numeric ID as String
    - Implement `validate_content()` that:
      - Checks content length against character_limit
      - Returns PlatformError::Validation if exceeded
    - Implement `name()` returning "mastodon"
    - Implement `character_limit()` returning Some(character_limit)


    - Implement `is_configured()` checking if client is initialized
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6_


  - [x] 3.3 Add error handling for Mastodon-specific errors



    - Map `megalodon::error::Error` variants to PlatformError:
      - `Error::HTTPError` with 401/403 → PlatformError::Authentication
      - `Error::HTTPError` with 422 → PlatformError::Validation
      - `Error::HTTPError` with 429 → PlatformError::RateLimit
      - `Error::HTTPError` other → PlatformError::Network
      - `Error::ParseError` → PlatformError::Posting


    - Handle OAuth token expiration (exit code 2)
    - Handle invalid instance URLs (configuration error)
    - Handle network errors with retry logic

    --_Requirements: 2.6, 2.7, 2.8, 8.3_

  - [x] 3.4 Add unit tests for MastodonClient



    - Test authentication with valid and invalid tokens
    - Test posting with mock megalodon client
    - Test character limit validation
    - Test error mapping
    - _Requirements: 10.1, 10.2_


- [x] 4. Implement Bluesky platform client




  - [x] 4.1 Create BlueskyClient struct and basic implementation


    - Create `libplurcast/src/platforms/bluesky.rs`
    - Define BlueskyClient struct with:
      - `agent: atrium_api::agent::AtpAgent` field
      - `did: String` field to store authenticated DID
    - Implement `new()` constructor that:
      - Takes handle (e.g., "user.bsky.social") and app_password as parameters
      - Creates AtpAgent with `AtpAgent::new(AtpServiceClient::new("https://bsky.social"))`
      - Stores handle for later authentication
    - Add `create_session()` async method that:
      - Calls `agent.login(handle, app_password).await`
      - Extracts and stores DID from session response
      - Returns Result indicating success/failure
    - _Requirements: 3.1, 3.2, 3.7_
  - [x] 4.2 Implement Platform trait for BlueskyClient


    - Implement `authenticate()` method that:
      - Calls `create_session()` to establish authenticated session
      - Returns PlatformError::Authentication on failure
    - Implement `post()` method that:
      - Creates `app::bsky::feed::post::Record` with text content
      - Uses `agent.api.app.bsky.feed.post.create()` to post
      - Constructs AT URI from response: `at://{did}/app.bsky.feed.post/{rkey}`
      - Returns AT URI as String
    - Implement `validate_content()` that:
      - Checks content length is ≤ 300 characters
      - Returns PlatformError::Validation if exceeded
    - Implement `name()` returning "bluesky"
    - Implement `character_limit()` returning Some(300)
    - Implement `is_configured()` checking if DID is set (authenticated)
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6_


  - [x] 4.3 Add error handling for Bluesky-specific errors





    - Map `atrium_api::error::Error` variants to PlatformError:
      - Authentication errors (invalid credentials) → PlatformError::Authentication
      - XRPC errors with status 400 → PlatformError::Validation
      - XRPC errors with status 429 → PlatformError::RateLimit
      - Network/connection errors → PlatformError::Network
      - Other XRPC errors → PlatformError::Posting
    - Handle invalid handle/authentication (exit code 2)
    - Handle PDS unreachable errors as Network errors


    - Include AT Protocol error codes in error messages

    --_Requirements: 3.6, 3.7, 3.8, 8.3_

  - [x] 4.4 Add unit tests for BlueskyClient



    - Test authentication with valid and invalid credentials
    - Test posting with mock atrium agent
    - Test 300 character limit validation
    - Test error mapping
    - _Requirements: 10.1, 10.2_

- [x] 5. Update configuration system for multi-platform




  - [x] 5.1 Extend Config struct with new platform configs


    - Add MastodonConfig struct with enabled, instance, token_file fields
    - Add BlueskyConfig struct with enabled, handle, auth_file fields
    - Add DefaultsConfig struct with platforms vector
    - Update Config struct to include new fields
    - _Requirements: 5.1, 5.2_
  - [x] 5.2 Implement configuration parsing and validation


    - Parse mastodon and bluesky sections from TOML
    - Validate required fields per platform
    - Expand shell variables in credential file paths
    - Provide clear error messages for missing configuration
    - _Requirements: 5.3, 5.4, 5.6_
  - [x] 5.3 Add default configuration generation


    - Create default config with all platforms disabled
    - Include helpful comments in generated config
    - Set sensible defaults for each platform
    - _Requirements: 5.5_
  - [x] 5.4 Add configuration tests


    - Test parsing valid multi-platform configs
    - Test missing required fields
    - Test path expansion
    - Test platform enable/disable logic
    - _Requirements: 10.4_


- [x] 6. Implement multi-platform posting orchestration




  - [x] 6.1 Create platform factory for instantiation


    - Create `create_platforms()` function in libplurcast that:
      - Takes Config reference as parameter
      - Returns `Result<Vec<Box<dyn Platform>>>`
      - For each enabled platform in config:
        - Nostr: Read keys from keys_file, create NostrClient with relays
        - Mastodon: Read token from token_file, create MastodonClient with instance URL
        - Bluesky: Read credentials from auth_file, create BlueskyClient with handle
      - Filter to only enabled platforms (where config.enabled == true)
      - Return error if required credential files are missing
      - Provide helpful error messages for configuration issues
    - _Requirements: 4.2, 5.2_
  - [x] 6.2 Implement MultiPlatformPoster struct


    - Create `libplurcast/src/poster.rs`
    - Define MultiPlatformPoster with platforms vector and database
    - Implement `post_to_all()` method for posting to all enabled platforms
    - Implement `post_to_selected()` method for specific platforms
    - _Requirements: 4.1, 4.2_
  - [x] 6.3 Add concurrent posting logic




    - Implement concurrent posting pattern:
      - Create vector of futures using `platforms.iter().map(|p| post_with_retry(p, content))`
      - Use `futures::future::join_all(futures).await` to execute concurrently
      - Each future returns `Result<(String, String)>` with (platform_name, post_id)
    - Collect results into Vec<PostResult> with success/failure status
    - Continue on individual failures (don't short-circuit with `?` operator)
    - Log progress for each platform completion
    - _Requirements: 4.3, 4.5_
  - [x] 6.4 Implement retry logic with exponential backoff


    - Create `post_with_retry()` async function that:
      - Takes platform reference and content as parameters
      - Implements retry loop with max 3 attempts
      - Uses `tokio::time::sleep(Duration::from_secs(2_u64.pow(attempt - 1)))` for backoff
      - Checks if error is transient using helper function:
        - Transient: PlatformError::Network, PlatformError::RateLimit
        - Permanent: PlatformError::Authentication, PlatformError::Validation
      - Logs each retry attempt with `tracing::warn!`
      - Returns final error if all retries exhausted
    - _Requirements: 8.1, 8.2_


  - [x] 6.5 Add database recording for multi-platform results
    - Create post record in database before posting
    - Insert post_records entry for each platform attempt
    - Update post status based on overall results
    - Record platform_post_id, success status, and errors
    - _Requirements: 4.4, 7.2, 7.3_
  - [x] 6.6 Add integration tests for multi-platform posting

    - Test posting to all platforms with mock clients
    - Test selective platform posting
    - Test partial failure scenarios
    - Test concurrent execution timing
    - Test retry logic
    - _Requirements: 10.2, 10.6_

- [x] 7. Update plur-post binary for multi-platform support




  - [x] 7.1 Add --platform CLI flag


    - Add platform argument accepting multiple values
    - Support values: nostr, mastodon, bluesky
    - Allow multiple --platform flags
    - Default to config defaults if not specified
    - _Requirements: 4.2, 4.10_
  - [x] 7.2 Implement multi-platform output formatting


    - Output one line per platform in format "platform:post_id"
    - Write successful posts to stdout
    - Write errors to stderr with platform context
    - Show per-platform progress with --verbose flag
    - _Requirements: 4.6, 4.10, 8.6_
  - [x] 7.3 Update exit code logic for partial failures

    - Exit 0 if all platforms succeed
    - Exit 1 if at least one platform fails (non-auth)
    - Exit 2 if any platform has authentication error
    - Exit 3 for invalid input
    - _Requirements: 4.7, 4.8, 9.3_
  - [x] 7.4 Add content validation across all target platforms

    - Validate content against all selected platforms before posting
    - Report validation errors with platform-specific context
    - Provide helpful suggestions for fixing validation errors
    - _Requirements: 8.4, 8.7_
  - [x] 7.5 Add integration tests for plur-post multi-platform


    - Test posting to multiple platforms
    - Test --platform flag filtering
    - Test exit codes for various scenarios
    - Test output format
    - _Requirements: 10.2, 10.3_


- [x] 8. Implement plur-history binary




  - [x] 8.1 Create plur-history binary structure


    - Create `plur-history/` directory
    - Create `plur-history/Cargo.toml` with dependencies
    - Create `plur-history/src/main.rs` with basic CLI structure
    - Add binary to workspace members
    - _Requirements: 6.1_
  - [x] 8.2 Implement CLI argument parsing


    - Add --platform flag for filtering by platform
    - Add --since and --until flags for date range
    - Add --search flag for content search
    - Add --limit flag (default 20)
    - Add --format flag (text, json, jsonl, csv)
    - Add --help with comprehensive usage information
    - _Requirements: 6.2, 6.3, 6.4, 6.11, 9.4_
  - [x] 8.3 Implement database query logic


    - Create HistoryQuery struct with fields:
      - `platform: Option<String>`
      - `since: Option<i64>` (Unix timestamp)
      - `until: Option<i64>` (Unix timestamp)
      - `search: Option<String>` (content search term)
      - `limit: usize` (default 20)
    - Implement `query_history()` function that:
      - Builds SQL query with LEFT JOIN between posts and post_records
      - Adds WHERE clauses conditionally based on HistoryQuery fields
      - Uses SQLx bind parameters for all user inputs (prevent SQL injection)
      - Example: `WHERE (? IS NULL OR pr.platform = ?) AND (? IS NULL OR p.created_at >= ?)`
      - Uses `LIKE` for content search: `p.content LIKE '%' || ? || '%'`
      - Orders by `p.created_at DESC`
      - Applies LIMIT clause
    - Returns Vec<HistoryEntry> with grouped platform results per post
    - _Requirements: 6.2, 6.3, 6.4, 6.9_
  - [x] 8.4 Implement output formatters


    - Create `format_text()` function that:
      - Formats each post as: `{timestamp} | {post_id} | {content_preview}`
      - Shows platform results indented with ✓/✗ symbols
      - Example: `  ✓ nostr: note1abc...` or `  ✗ bluesky: Authentication failed`
      - Truncates long content with "..." for preview
    - Create `format_json()` function that:
      - Uses `serde_json::to_string_pretty()` to serialize Vec<HistoryEntry>
      - Outputs complete JSON array
    - Create `format_jsonl()` function that:
      - Outputs one JSON object per line using `serde_json::to_string()`
      - No array wrapper, just newline-separated objects
    - Create `format_csv()` function that:
      - Outputs header row: `post_id,timestamp,platform,success,platform_post_id,error,content`
      - One row per platform per post (flattened structure)
      - Escapes commas and quotes in content field
    - All formats include: post ID, timestamp, platform, status, content
    - _Requirements: 6.5, 6.6, 6.7, 6.8, 6.11_
  - [x] 8.5 Add error handling for plur-history


    - Handle missing database gracefully (exit code 1)
    - Handle invalid date formats with clear errors
    - Handle empty results (output nothing, exit 0)
    - Write errors to stderr
    - _Requirements: 6.10, 8.6, 9.2_
  - [x] 8.6 Add integration tests for plur-history


    - Test filtering by platform
    - Test date range filtering
    - Test search functionality
    - Test all output formats
    - Test empty results
    - _Requirements: 10.5_

- [x] 9. Add error handling enhancements





  - [x] 9.1 Add RateLimit variant to PlatformError

    - Add new error variant for rate limiting
    - Update error display messages
    - Update exit code logic if needed
    - _Requirements: 8.2_
  - [x] 9.2 Implement platform-specific error context


    - Include platform name in all error messages
    - Include operation attempted in error context
    - Add suggested remediation where possible
    - Format errors consistently across platforms
    - _Requirements: 8.3, 8.4, 8.7_
  - [x] 9.3 Add error handling tests


    - Test error message formatting
    - Test exit code mapping
    - Test error context inclusion
    - _Requirements: 10.3_


- [x] 10. Database enhancements





  - [x] 10.1 Add database indexes for performance


    - Add index on post_records.post_id for joins
    - Add index on posts.created_at for date filtering
    - Add index on post_records.platform for filtering
    - Create migration file for indexes
    - _Requirements: 7.3_
  - [x] 10.2 Add helper methods for multi-platform queries


    - Add method to query posts with all platform records
    - Add method to filter by platform
    - Add method to filter by date range
    - Add method to search content
    - _Requirements: 6.9, 7.3_
  - [x] 10.3 Add database tests


    - Test post creation with multiple platform records
    - Test querying with filters
    - Test concurrent writes
    - _Requirements: 10.6_

- [x] 11. Documentation and examples





  - [x] 11.1 Update README with multi-platform examples


    - Add examples for posting to multiple platforms
    - Add examples for selective platform posting
    - Add examples for querying history
    - Add configuration examples for each platform
    - _Requirements: 9.1, 9.5_
  - [x] 11.2 Create platform-specific setup guides


    - Document Mastodon OAuth token generation
    - Document Bluesky app password creation
    - Document configuration file format
    - Add troubleshooting section
    - _Requirements: 5.4, 8.3_
  - [x] 11.3 Add Unix composability examples

    - Show piping examples with plur-post
    - Show filtering examples with plur-history and jq
    - Show automation examples with shell scripts
    - Demonstrate agent-friendly features
    - _Requirements: 9.1, 9.6_
  - [x] 11.4 Update help text for all binaries


    - Ensure --help is comprehensive for plur-post
    - Ensure --help is comprehensive for plur-history
    - Include examples in help text
    - Document all flags and options
    - _Requirements: 9.4_

- [x] 12. Integration and end-to-end testing




  - [x] 12.1 Create mock platform implementations for testing


    - Create MockPlatform struct with configurable behavior
    - Support simulating failures and delays
    - Use in integration tests
    - _Requirements: 10.6_
  - [x] 12.2 Add end-to-end workflow tests


    - Test complete posting workflow with all platforms
    - Test posting with partial failures
    - Test querying history after posting
    - Test configuration loading and validation
    - _Requirements: 10.2, 10.3_
  - [x] 12.3 Add backward compatibility tests


    - Test Phase 1 configurations still work
    - Test existing database data is preserved
    - Test Nostr-only posting still works
    - _Requirements: 7.6_

- [x] 13. Final polish and release preparation






  - [x] 13.1 Review and refactor code for consistency

    - Ensure consistent error handling patterns
    - Ensure consistent naming conventions
    - Add missing documentation comments
    - Run clippy and fix warnings

  - [x] 13.2 Performance testing and optimization

    - Test concurrent posting performance
    - Test history query performance with large datasets
    - Optimize database queries if needed
    - Verify memory usage is reasonable

  - [x] 13.3 Security review

    - Verify credential file permissions
    - Ensure no credentials in logs
    - Password protect private key management 
    - Verify error messages don't leak sensitive data
    - Review authentication flows

  - [x] 13.4 Prepare for alpha release

    - Update version to 0.2.0-alpha
    - Create release notes
    - Tag release in git
    - Build binaries for distribution
