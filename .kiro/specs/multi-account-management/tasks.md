# Implementation Plan: Multi-Account Credential Management

## Overview

This implementation plan breaks down the multi-account credential management feature into discrete, incremental coding tasks. Each task builds on previous work and references specific requirements from the requirements document.

---

## Phase 1: Core Account Management Infrastructure

- [x] 1. Create AccountManager module with state management





  - Create `libplurcast/src/accounts.rs` with AccountManager struct
  - Implement AccountState and PlatformAccounts data structures with serde support
  - Add account name validation function (alphanumeric, hyphens, underscores, max 64 chars)
  - Implement state file path resolution using XDG directories
  - _Requirements: 1.4, 8.1, 8.5_

- [x] 1.1 Implement account state persistence

  - Write save() method to serialize state to TOML file
  - Write load() method to deserialize state from TOML file
  - Handle missing file gracefully (return default state)
  - Handle corrupted file gracefully (log warning, return default state)
  - Set appropriate file permissions (644 on Unix)
  - _Requirements: 3.2, 3.5, 8.3_

- [x] 1.2 Implement account registry operations

  - Write register_account() to add account to platform registry
  - Write unregister_account() to remove account from platform registry
  - Write list_accounts() to return all accounts for a platform
  - Write account_exists() to check if account is registered
  - Ensure thread-safe access using Arc<RwLock<AccountState>>
  - _Requirements: 2.1, 2.2, 2.3_

- [x] 1.3 Implement active account management

  - Write get_active_account() returning "default" if not set
  - Write set_active_account() to update active account and persist state
  - Validate account exists before setting as active
  - _Requirements: 3.1, 3.3, 3.4_

- [x] 1.4 Write unit tests for AccountManager

  - Test account name validation (valid and invalid cases)
  - Test state persistence (save/load cycle)
  - Test missing and corrupted state file handling
  - Test account registry operations
  - Test active account management
  - Test thread safety with concurrent access
  - _Requirements: 1.4, 3.1, 8.1_

---

## Phase 2: Multi-Account Credential Storage

- [x] 2. Extend CredentialStore trait with multi-account methods





  - Add store_account() method with account parameter
  - Add retrieve_account() method with account parameter
  - Add delete_account() method with account parameter
  - Add exists_account() method with account parameter
  - Add list_accounts() method to return all accounts for service/key
  - Implement default methods that delegate to "default" account for backward compatibility
  - _Requirements: 1.1, 1.2, 1.3, 6.3_

- [x] 2.1 Implement multi-account support in KeyringStore


  - Write keyring_key() helper to build namespace: `plurcast.{platform}.{account}.{key}`
  - Implement store_account() using new namespace format
  - Implement retrieve_account() using new namespace format
  - Implement delete_account() using new namespace format
  - Implement exists_account() using new namespace format
  - Implement list_accounts() by querying account registry (keyring API doesn't support listing)
  - _Requirements: 1.1, 7.1_


- [x] 2.2 Implement multi-account support in EncryptedFileStore

  - Update get_file_path() to include account in filename: `{service}.{account}.{key}.age`
  - Implement store_account() with new filename format
  - Implement retrieve_account() with new filename format
  - Implement delete_account() with new filename format
  - Implement exists_account() with new filename format
  - Implement list_accounts() by scanning directory for matching files
  - _Requirements: 1.1, 7.2_

- [x] 2.3 Implement multi-account support in PlainFileStore


  - Update get_legacy_path() to include account: `{platform}.{account}.{key}`
  - Implement store_account() with new filename format
  - Implement retrieve_account() with new filename format
  - Implement delete_account() with new filename format
  - Implement exists_account() with new filename format
  - Implement list_accounts() by scanning directory for matching files
  - _Requirements: 1.1, 7.3_


- [x] 2.4 Write unit tests for multi-account storage backends


  - Test KeyringStore namespace derivation
  - Test EncryptedFileStore filename generation
  - Test PlainFileStore filename generation
  - Test account isolation (credentials don't leak between accounts)
  - Test backward compatibility (default account methods work)
  - _Requirements: 1.3, 6.1, 6.2, 7.1, 7.2, 7.3_

---

## Phase 3: Enhanced CredentialManager

- [x] 3. Add multi-account methods to CredentialManager





  - Implement store_account() that uses first available backend
  - Implement retrieve_account() that tries all backends in order
  - Implement delete_account() that removes from all backends
  - Implement exists_account() that checks all backends
  - Implement list_accounts() that aggregates from all backends
  - Integrate with AccountManager to register/unregister accounts
  - _Requirements: 1.1, 1.2, 1.3, 2.4, 7.4_

- [x] 3.1 Implement automatic migration from old namespace format


  - Write auto_migrate_if_needed() to detect old format credentials
  - Check if credential exists in old format: `plurcast.{platform}.{key}`
  - Check if already migrated to new format: `plurcast.{platform}.default.{key}`
  - If not migrated, read from old format and store in new format
  - Verify migration by retrieving from new format
  - Log migration success/failure
  - Keep old format for backward compatibility (don't delete)
  - _Requirements: 6.1, 6.2, 6.5_

- [x] 3.2 Implement manual migration command support


  - Write migrate_to_multi_account() method
  - Scan for old format credentials across all platforms
  - Display migration plan to user
  - Migrate each credential to "default" account
  - Return MigrationReport with success/failure details
  - _Requirements: 10.1, 10.2, 10.3, 10.4, 10.5_

- [x] 3.3 Write integration tests for CredentialManager multi-account


  - Test storing and retrieving credentials for multiple accounts
  - Test account isolation (test-account vs prod-account)
  - Test fallback logic across backends with multi-account
  - Test automatic migration from old format
  - Test manual migration command
  - Test list_accounts() aggregation across backends
  - _Requirements: 1.1, 1.2, 1.3, 6.1, 6.2, 7.4_

---

## Phase 4: CLI Integration - plur-creds

- [x] 4. Add --account flag to plur-creds commands





  - Add account field to Set command args with default value "default"
  - Add account field to Delete command args with default value "default"
  - Add account field to Test command args with default value "default"
  - Add platform filter to List command args (optional)
  - Validate account names on input using AccountManager::validate_account_name()
  - _Requirements: 1.4, 2.3, 4.1, 5.1, 9.1_

- [x] 4.1 Implement plur-creds use command


  - Create Use subcommand with platform and account parameters
  - Load AccountManager and validate account exists
  - Call set_active_account() to update state
  - Display success message with account details
  - Handle errors (account not found, invalid platform)
  - _Requirements: 3.1, 3.3, 8.2_

- [x] 4.2 Update plur-creds set command for multi-account

  - Accept --account parameter (default: "default")
  - Check if credentials already exist for this account
  - If exists, prompt for overwrite confirmation (interactive mode)
  - If exists in non-interactive mode, refuse and suggest delete first
  - Store credentials using CredentialManager::store_account()
  - Register account with AccountManager
  - Display success message with account and backend info
  - _Requirements: 1.1, 1.5, 4.1, 8.1_

- [x] 4.3 Update plur-creds list command for multi-account


  - Accept optional --platform filter
  - Load AccountManager to get active accounts
  - For each platform (or specified platform), list all accounts
  - Display account name, credential type, storage backend
  - Indicate which account is active with [active] marker
  - Handle case where no accounts exist
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_

- [x] 4.4 Update plur-creds delete command for multi-account


  - Accept --account parameter (default: "default")
  - Check if account exists before attempting delete
  - Prompt for confirmation unless --force flag used
  - If deleting active account, reset active to "default"
  - Delete credentials using CredentialManager::delete_account()
  - Unregister account with AccountManager
  - Display success message
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_

- [x] 4.5 Update plur-creds test command for multi-account


  - Accept --account parameter (default: active account)
  - If no account specified, use active account from AccountManager
  - Check if credentials exist for specified account
  - Display success/failure with account details
  - Handle account not found error
  - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5_

- [x] 4.6 Update plur-creds migrate command for multi-account


  - Add --to-multi-account flag to trigger multi-account migration
  - Call CredentialManager::migrate_to_multi_account()
  - Display migration plan before executing
  - Show migration report (migrated, failed, skipped)
  - Offer to delete old format credentials after successful migration
  - _Requirements: 10.1, 10.2, 10.3, 10.4, 10.5_

- [x] 4.7 Write CLI integration tests for plur-creds


  - Test plur-creds set --account test
  - Test plur-creds list --platform nostr
  - Test plur-creds use nostr --account prod
  - Test plur-creds delete --account test
  - Test plur-creds test --account test
  - Test error handling (invalid account names, account not found)
  - _Requirements: 1.4, 2.1, 3.1, 4.1, 5.1, 8.1, 8.2, 9.1_

---

## Phase 5: CLI Integration - plur-post

- [x] 5. Add --account flag to plur-post command





  - Add account field to Cli struct (optional)
  - If account not specified, use active account from AccountManager
  - Pass account parameter to create_platforms() function
  - Update platform creation to use account-specific credentials
  - Display which account is being used in verbose mode
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5_

- [x] 5.1 Update create_platforms() to accept account parameter


  - Add account parameter to function signature (Option<&str>)
  - For each platform, determine account to use (explicit or active)
  - Retrieve credentials using CredentialManager::retrieve_account()
  - Pass account-specific credentials to platform clients
  - Handle case where account has no credentials for a platform
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5_

- [x] 5.2 Write integration tests for plur-post with accounts


  - Test posting with explicit --account flag
  - Test posting with active account (no flag)
  - Test posting with account that has no credentials
  - Test multi-platform posting with different active accounts per platform
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5_

---

## Phase 6: Documentation and Migration

- [x] 6. Update documentation for multi-account feature





  - Update README.md with multi-account examples
  - Update ARCHITECTURE.md with account management design
  - Add multi-account section to user guide
  - Document account naming conventions and best practices
  - Add troubleshooting section for common account issues
  - _Requirements: All_

- [x] 6.1 Create migration guide for existing users


  - Document automatic migration behavior
  - Explain "default" account concept
  - Provide examples of adding additional accounts
  - Document how to check migration status
  - Explain backward compatibility guarantees
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_

- [x] 6.2 Update ADR 001 with implementation notes


  - Mark ADR as "Implemented" status
  - Add implementation date and version
  - Document any deviations from original design
  - Add lessons learned section
  - Link to relevant code files
  - _Requirements: All_

---

## Phase 7: Testing and Validation

- [x] 7. Write comprehensive integration tests



  - Test complete workflow: set → use → post → delete
  - Test multiple accounts per platform
  - Test account switching
  - Test backward compatibility with existing credentials
  - Test migration from old format to new format
  - Test error scenarios (invalid names, missing accounts, etc.)
  - _Requirements: All_

- [x] 7.1 Write process persistence tests (Windows)


  - Store credentials for multiple accounts
  - Spawn child process to verify persistence
  - Verify correct account retrieved after process restart
  - Test across system reboot (manual test)
  - _Requirements: 1.1, 1.2, 1.3_



- [ ] 7.2 Manual testing on all platforms
  - Test on Windows with Credential Manager
  - Test on macOS with Keychain
  - Test on Linux with Secret Service
  - Verify account isolation works correctly
  - Verify active account switching works
  - Verify migration works on all platforms
  - _Requirements: All_

---

## Success Criteria

- [ ] Users can store multiple accounts per platform
- [ ] Users can switch between accounts seamlessly
- [ ] Existing single-account users experience no breaking changes
- [ ] All credential storage backends support multi-account
- [ ] Account operations complete in under 100ms
- [ ] Clear error messages for all failure scenarios
- [ ] Documentation is complete and accurate
- [ ] All tests pass on Windows (other platforms need tested in the future)

---

## Notes

- **Account names**: Alphanumeric, hyphens, underscores only, max 64 characters
- **Default account**: Named "default", used for backward compatibility
- **Active account**: Stored in `~/.config/plurcast/accounts.toml`
- **Namespace format**: `plurcast.{platform}.{account}.{key}`
- **Migration**: Automatic on first use, manual command available
- **Backward compatibility**: Old namespace format still works, auto-migrates to "default"

---

## Dependencies

- Existing credential storage system (Phase 2 complete)
- Platform abstraction (Phase 2 complete)
- Configuration system (Phase 1 complete)

---

## Version

This implementation plan targets version **0.3.0-alpha2** of Plurcast.
