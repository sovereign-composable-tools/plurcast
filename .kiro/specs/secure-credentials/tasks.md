# Implementation Plan

- [x] 1. Add secure credential storage dependencies





  - Add `keyring = "2.3"` to workspace dependencies
  - Add `rpassword = "7.3"` to workspace dependencies  
  - Add `age = "0.10"` to workspace dependencies
  - Add `atty = "0.2"` to workspace dependencies
  - Update libplurcast Cargo.toml to include new dependencies
  - _Requirements: 1.1, 2.1_

- [x] 2. Implement credential storage abstraction






  - [x] 2.1 Create CredentialStore trait


    - Create `libplurcast/src/credentials.rs` module
    - Define CredentialStore trait with methods:
      - `store(service: &str, key: &str, value: &str) -> Result<()>`
      - `retrieve(service: &str, key: &str) -> Result<String>`
      - `delete(service: &str, key: &str) -> Result<()>`
      - `exists(service: &str, key: &str) -> Result<bool>`
      - `backend_name() -> &str`
    - Add comprehensive trait documentation with examples
    - _Requirements: 1.1, 8.1_
  
  - [x] 2.2 Implement KeyringStore (primary)


    - Create KeyringStore struct
    - Implement CredentialStore trait using `keyring` crate
    - Store credentials in OS keyring:
      - macOS: Keychain via Security framework
      - Windows: Credential Manager via Windows API
      - Linux: Secret Service (GNOME Keyring/KWallet) via D-Bus
    - Service name format: `plurcast.{platform}` (e.g., "plurcast.nostr")
    - Key format: `{credential_type}` (e.g., "private_key", "access_token")
    - Handle keyring unavailable gracefully with specific error type
    - Return `CredentialError::KeyringUnavailable` when OS keyring not accessible
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_
  
  - [x] 2.3 Implement EncryptedFileStore (fallback)


    - Create EncryptedFileStore struct with base_path and master_password fields
    - Implement `set_master_password()` method with password strength validation (min 8 chars)
    - Implement `encrypt()` method using `age::Encryptor::with_user_passphrase()`
    - Implement `decrypt()` method using `age::Decryptor`
    - Implement CredentialStore trait for encrypted file operations
    - Store encrypted credentials in `~/.config/plurcast/credentials/`
    - File format: `{service}.{key}.age` (e.g., `plurcast.nostr.private_key.age`)
    - Set file permissions to 600 on Unix systems
    - Cache decrypted credentials in memory during session only
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6_
  
  - [x] 2.4 Implement PlainFileStore (legacy/testing only)


    - Create PlainFileStore struct with base_path and warned fields
    - Implement CredentialStore trait for plain text file operations
    - Map service/key pairs to legacy file paths:
      - `plurcast.nostr/private_key` → `nostr.keys`
      - `plurcast.mastodon/access_token` → `mastodon.token`
      - `plurcast.bluesky/app_password` → `bluesky.auth`
    - Log deprecation warning on first use per credential
    - Mark as deprecated in documentation with security warnings
    - Set file permissions to 600 on Unix systems
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_
  
  - [x] 2.5 Create CredentialManager facade



    - Create CredentialManager struct with stores vector and config
    - Implement `new()` constructor that builds store priority list:
      1. Try KeyringStore (if configured and available)
      2. Try EncryptedFileStore (if master password set or can prompt)
      3. Fall back to PlainFileStore (with warnings)
    - Implement `store()` method that uses first available store
    - Implement `retrieve()` method that tries stores in order until success
    - Implement `delete()` method that removes from all stores
    - Implement `exists()` method that checks all stores
    - Log which backend is being used for each operation
    - Handle master password prompting for encrypted storage (if TTY available)
    - _Requirements: 1.3, 2.1, 2.2, 3.1, 8.1_

- [x] 3. Add credential error types





  - [x] 3.1 Define CredentialError enum


    - Add `NotFound(String)` variant
    - Add `KeyringUnavailable(String)` variant
    - Add `MasterPasswordNotSet` variant
    - Add `WeakPassword` variant
    - Add `DecryptionFailed` variant
    - Add `NoStoreAvailable` variant
    - Add `MigrationFailed(String)` variant
    - Implement `From<std::io::Error>` for CredentialError
    - Implement `From<keyring::Error>` for CredentialError
    - Add encryption error variant with descriptive messages
    - _Requirements: 1.4, 2.6, 6.6_
  
  - [x] 3.2 Integrate with PlurcastError


    - Add `Credential(CredentialError)` variant to PlurcastError enum
    - Implement `From<CredentialError>` for PlurcastError
    - Update error display messages to include credential context
    - _Requirements: 6.6, 8.4_

- [x] 4. Update configuration for secure credentials


  - [x] 4.1 Add credential storage configuration


    - Add `CredentialConfig` struct with fields:
      - `storage: StorageBackend` (enum: Keyring, Encrypted, Plain)
      - `path: String` (default: "~/.config/plurcast/credentials")
      - `master_password: Option<String>` (skipped in serialization)
    - Add `[credentials]` section to Config struct
    - Implement default values: storage=Keyring, path=default
    - Add validation for storage backend enum values
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5_
  
  - [x] 4.2 Update default configuration generation


    - Add `[credentials]` section to generated config.toml with comments:
      ```toml
      [credentials]
      # Storage backend: "keyring" (OS native), "encrypted" (password-protected files), "plain" (not recommended)
      storage = "keyring"
      # Path for encrypted/plain file storage (keyring doesn't use files)
      path = "~/.config/plurcast/credentials"
      ```
    - Include security recommendations in comments
    - _Requirements: 7.1, 10.1_
  
  - [x] 4.3 Add configuration parsing and validation



    - Parse `[credentials]` section from TOML
    - Validate storage backend is valid enum value
    - Expand shell variables in credential path
    - Check for master password in environment variable `PLURCAST_MASTER_PASSWORD`
    - Provide clear error messages for invalid configuration
    - _Requirements: 7.5, 10.5_
  
  - [x] 4.4 Add configuration tests


    - Test parsing valid credential configurations
    - Test default values when section is missing
    - Test invalid storage backend values
    - Test path expansion with ~ and environment variables
    - Test backward compatibility (no [credentials] section)
    - _Requirements: 7.5, 10.1_

- [x] 5. Update platform clients for secure credentials


  - [x] 5.1 Update NostrClient to use CredentialManager



    - Replace file reading with `credentials.retrieve("plurcast.nostr", "private_key")`
    - Support both hex and bech32 formats from credential store
    - Cache keys in memory during session
    - Handle CredentialError::NotFound with helpful message
    - Update NostrClient constructor to accept CredentialManager reference
    - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5_
  
  - [x] 5.2 Update MastodonClient to use CredentialManager


    - Replace token file reading with `credentials.retrieve("plurcast.mastodon", "access_token")`
    - Store instance URL in config (not sensitive, doesn't need credential store)
    - Update MastodonClient constructor to accept CredentialManager reference
    - Handle credential retrieval errors appropriately
    - _Requirements: 8.1, 8.2, 8.3, 8.4_
  
  - [x] 5.3 Update BlueskyClient to use CredentialManager


    - Replace auth file reading with `credentials.retrieve("plurcast.bluesky", "app_password")`
    - Store handle in config (not sensitive, doesn't need credential store)
    - Update BlueskyClient constructor to accept CredentialManager reference
    - Handle credential retrieval errors appropriately
    - _Requirements: 8.1, 8.2, 8.3, 8.4_
  
  - [x] 5.4 Update platform factory function

    - Update `create_platforms()` to create CredentialManager first
    - Pass CredentialManager reference to each platform client constructor
    - Remove direct file reading from platform factory
    - Handle credential errors and provide helpful messages
    - Maintain backward compatibility with plain text files
    - _Requirements: 3.4, 8.1, 8.2_

- [x] 6. Implement credential migration



  - [x] 6.1 Create migration detection logic


    - Implement `detect_plain_credentials()` function that:
      - Scans `~/.config/plurcast/` for plain text credential files
      - Checks for: nostr.keys, mastodon.token, bluesky.auth
      - Returns list of found credentials with file paths
    - _Requirements: 4.1, 4.2_
  
  - [x] 6.2 Implement migration logic


    - Create `MigrationReport` struct with fields:
      - `migrated: Vec<String>` (successfully migrated)
      - `failed: Vec<(String, String)>` (credential, error message)
      - `skipped: Vec<String>` (already in secure storage)
    - Implement `migrate_from_plain()` method in CredentialManager:
      - Detect all plain text credentials
      - For each credential:
        - Read from plain text file
        - Store in secure storage (first available store)
        - Verify by retrieving and comparing
        - Mark as migrated or failed
      - Return MigrationReport
    - _Requirements: 4.2, 4.3, 4.4, 4.5_
  
  - [x] 6.3 Add migration verification

    - After migration, test each credential by authenticating with platform
    - Use existing platform client authentication methods
    - Report authentication success/failure in migration report
    - Only mark migration as successful if authentication works
    - _Requirements: 4.5_
  
  - [x] 6.4 Add plain text file cleanup


    - Implement `cleanup_plain_files()` function that:
      - Takes list of successfully migrated credentials
      - Prompts user for confirmation (if TTY available)
      - Deletes plain text files only after confirmation
      - Reports which files were deleted
    - Never delete files if migration failed
    - _Requirements: 4.3_

- [x] 7. Create plur-creds binary



  - [x] 7.1 Create binary structure


    - Create `plur-creds/` directory
    - Create `plur-creds/Cargo.toml` with dependencies
    - Create `plur-creds/src/main.rs` with CLI structure using clap
    - Add binary to workspace members
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6_
  
  - [x] 7.2 Implement 'set' command


    - Add `plur-creds set <platform>` subcommand
    - Platform values: nostr, mastodon, bluesky
    - Prompt for credential value using `rpassword::prompt_password()`
    - For Nostr: validate hex or bech32 format
    - For Mastodon: prompt for instance URL and token
    - For Bluesky: prompt for handle and app password
    - Store using CredentialManager
    - Provide success confirmation or error message
    - _Requirements: 5.1_
  
  - [x] 7.3 Implement 'list' command


    - Add `plur-creds list` subcommand
    - Query CredentialManager for all known credentials
    - Display platform names and credential types (not values)
    - Show which storage backend each credential uses
    - Format output as table or simple list
    - _Requirements: 5.2_
  
  - [x] 7.4 Implement 'delete' command


    - Add `plur-creds delete <platform>` subcommand
    - Confirm deletion (unless --force flag provided)
    - Delete from all storage backends
    - Provide success confirmation or error message
    - _Requirements: 5.3_
  
  - [x] 7.5 Implement 'test' command


    - Add `plur-creds test <platform>` subcommand
    - Add `plur-creds test --all` flag to test all platforms
    - Retrieve credentials from CredentialManager
    - Create platform client and call authenticate() method
    - Report success or failure with specific error messages
    - Exit code 0 for success, 1 for failure
    - _Requirements: 5.4_
  
  - [x] 7.6 Implement 'migrate' command


    - Add `plur-creds migrate` subcommand
    - Call CredentialManager.migrate_from_plain()
    - Display migration progress and results
    - Show MigrationReport with migrated/failed/skipped credentials
    - Offer to delete plain text files after successful migration
    - Provide clear error messages for failures
    - _Requirements: 5.5, 4.1, 4.2, 4.3, 4.4, 4.5_
  
  - [x] 7.7 Implement 'audit' command


    - Add `plur-creds audit` subcommand
    - Check for plain text credential files
    - Verify file permissions on all credential files (should be 600)
    - Check which storage backend is configured
    - Report security issues found
    - Provide recommendations for improvements
    - Exit code 0 if no issues, 1 if issues found
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_
  
  - [x] 7.8 Add comprehensive help text

    - Add --help for main command with overview
    - Add --help for each subcommand with examples
    - Include security recommendations in help text
    - Document all flags and options
    - _Requirements: 5.6, 10.1_

- [x] 8. Create plur-setup binary




  - [x] 8.1 Create binary structure

    - Create `plur-setup/` directory
    - Create `plur-setup/Cargo.toml` with dependencies
    - Create `plur-setup/src/main.rs` with interactive wizard
    - Add binary to workspace members
    - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5_
  

  - [x] 8.2 Implement storage backend selection

    - Display welcome message and overview
    - Present storage backend options:
      1. OS Keyring (recommended)
      2. Encrypted files (password-protected)
      3. Plain text (not recommended)
    - Prompt for selection with default to keyring
    - Explain each option's security implications
    - Update configuration with selected backend
    - _Requirements: 9.1, 9.2_
  


  - [ ] 8.3 Implement platform credential setup
    - For each platform (Nostr, Mastodon, Bluesky):
      - Ask if user wants to configure this platform
      - Prompt for platform-specific credentials:
        - Nostr: private key (hex or nsec format)
        - Mastodon: instance URL and OAuth token
        - Bluesky: handle and app password
      - Validate credential format
      - Test authentication with platform
      - Store in CredentialManager if authentication succeeds
      - Allow retry if authentication fails
      - Allow skip if user doesn't want to configure
    - _Requirements: 9.2, 9.3, 9.4_


  
  - [ ] 8.4 Implement setup completion
    - Display summary of configured platforms
    - Show next steps (example commands to try)
    - Save configuration to config.toml
    - Provide helpful error messages if setup fails
    - _Requirements: 9.5_
  
  - [x] 8.5 Add setup wizard tests

    - Test storage backend selection
    - Test platform credential prompts
    - Test authentication validation
    - Test configuration saving
    - Mock user input for automated testing
    - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5_

- [x] 9. Add credential storage tests




  - [x] 9.1 Add KeyringStore tests

    - Test store/retrieve/delete/exists operations
    - Test keyring unavailable error handling
    - Mock keyring for CI/CD environments
    - Test service and key naming conventions
    - Test error messages are helpful
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_
  

  - [ ] 9.2 Add EncryptedFileStore tests
    - Test encryption/decryption with age
    - Test master password validation (min 8 chars)
    - Test file creation with correct permissions (600)
    - Test corrupted file handling
    - Test incorrect password error
    - Test file naming convention
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6_

  
  - [ ] 9.3 Add PlainFileStore tests
    - Test backward compatibility with legacy file paths
    - Test deprecation warnings are logged
    - Test file permissions (600)
    - Test warning is only logged once per credential

    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_
  
  - [ ] 9.4 Add CredentialManager tests
    - Test fallback logic (keyring → encrypted → plain)
    - Test store uses first available backend
    - Test retrieve tries all backends in order
    - Test migration from plain to secure storage
    - Test configuration parsing

    - Test master password prompting (mock TTY)
    - _Requirements: 1.3, 2.1, 3.1, 4.1, 4.2, 4.3, 4.4, 4.5_
  
  - [ ] 9.5 Add integration tests
    - Test end-to-end credential flow: store → retrieve → use in platform client
    - Test migration scenario: plain → keyring
    - Test migration scenario: plain → encrypted
    - Test backward compatibility: existing plain files work
    - Test platform clients with CredentialManager


    - _Requirements: 3.4, 4.5, 8.1, 8.2, 8.3, 8.4, 8.5_

- [x] 10. Update documentation

  - [x] 10.1 Update README with credential security

    - Add "Security" section explaining credential storage options
    - Add examples for each storage backend
    - Add migration instructions
    - Add troubleshooting section for credential issues
    - _Requirements: 10.1, 10.2, 10.3, 10.4, 10.5_
  

  - [x] 10.2 Create SECURITY.md document

    - Document threat model (what's protected, what's not)
    - Explain each storage backend's security properties
    - Provide best practices for each operating system
    - Document master password recommendations
    - Explain file permissions and their importance
    - _Requirements: 10.1, 10.2, 10.3, 10.4_
  

  - [x] 10.3 Update SETUP.md with credential setup

    - Add step-by-step credential setup instructions
    - Document plur-setup wizard usage
    - Document manual credential configuration
    - Add platform-specific credential generation guides:
      - Nostr: how to generate keys
      - Mastodon: how to get OAuth token
      - Bluesky: how to create app password
    - _Requirements: 10.1, 10.2, 10.5_
  

  - [x] 10.4 Update ARCHITECTURE.md with security model

    - Add "Credential Storage" section
    - Document CredentialStore trait and implementations
    - Explain fallback logic and priority
    - Document migration strategy
    - Add security considerations section
    - _Requirements: 10.4_
  
  - [x] 10.5 Add inline documentation

    - Add comprehensive doc comments to CredentialStore trait
    - Add doc comments to all credential-related structs and methods
    - Add examples in doc comments
    - Document error conditions and handling
    - _Requirements: 10.5_





- [ ] 11. Security hardening
  - [ ] 11.1 Verify file permissions
    - Ensure all credential files are created with 600 permissions on Unix
    - Add tests for file permission enforcement

    - Log warning if existing files have incorrect permissions
    - _Requirements: 2.5, 3.5, 6.2_
  
  - [x] 11.2 Ensure no credentials in logs

    - Audit all logging statements
    - Ensure credential values are never logged

    - Only log credential access events (service/key, not value)
    - Add tests to verify no credential leakage in logs
    - _Requirements: 8.5, 10.5_
  
  - [x] 11.3 Clear credentials from memory on exit

    - Implement Drop trait for CredentialManager to clear cached credentials

    - Use `zeroize` crate to securely clear sensitive data from memory
    - Test that credentials are cleared on normal exit
    - Test that credentials are cleared on panic
    - _Requirements: 8.3_
  
  - [x] 11.4 Verify error messages don't leak credentials

    - Audit all error messages
    - Ensure credential values are never included in errors
    - Only include metadata (service, key, operation) in errors
    - Add tests for error message content
    - _Requirements: 6.6, 10.5_

- [x] 12. Final integration and testing



  - [x] 12.1 Update plur-post to use secure credentials

    - Update plur-post to create CredentialManager
    - Pass CredentialManager to platform factory
    - Test posting with keyring credentials
    - Test posting with encrypted credentials
    - Test posting with plain text credentials (backward compatibility)
    - _Requirements: 3.4, 8.1, 8.2_
  
  - [x] 12.2 Update plur-history to use secure credentials

    - Update plur-history to create CredentialManager (if needed for future features)
    - Ensure no credential-related changes break existing functionality
    - _Requirements: 3.4_
  
  - [x] 12.3 Add end-to-end workflow tests

    - Test complete workflow: setup → store credentials → post → history
    - Test migration workflow: plain files → migrate → post
    - Test each storage backend end-to-end
    - Test error scenarios (wrong password, missing credentials, etc.)
    - _Requirements: 4.5, 8.1, 8.2, 9.1, 9.2, 9.3, 9.4, 9.5_
  
  - [x] 12.4 Add backward compatibility tests

    - Test that existing plain text files continue to work
    - Test that Phase 1 configurations work without [credentials] section
    - Test that migration doesn't break existing functionality
    - Test that users can gradually migrate one platform at a time
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_
  
  - [x] 12.5 Performance and security review

    - Verify keyring access performance is acceptable
    - Verify encryption/decryption performance is acceptable
    - Review all credential-related code for security issues
    - Run security audit tool and verify no issues
    - Test memory usage with cached credentials
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5, 10.4_
