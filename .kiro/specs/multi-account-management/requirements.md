# Requirements Document: Multi-Account Credential Management

## Introduction

This feature enables users to manage multiple accounts per platform (e.g., test vs prod Nostr keys, personal vs work accounts) with isolated credential storage and seamless account switching. The system prevents accidental credential overwrites while maintaining backward compatibility with existing single-account setups.

## Glossary

- **Account**: A named credential set for a specific platform (e.g., "test-account", "prod-account", "default")
- **Platform**: A social media service supported by Plurcast (Nostr, Mastodon, Bluesky)
- **Credential Store**: The backend system for storing credentials (OS keyring, encrypted files, or plain text)
- **Active Account**: The currently selected account for a platform, used when no explicit account is specified
- **Keyring Namespace**: The unique identifier used to store credentials in the OS keyring (format: `plurcast.{platform}.{account}.{key}`)
- **plur-creds**: The CLI tool for managing credentials
- **plur-post**: The CLI tool for posting content to platforms
- **Default Account**: The account named "default", used for backward compatibility with existing single-account setups
- **Account State File**: Configuration file tracking active accounts per platform (`~/.config/plurcast/accounts.toml`)

## Requirements

### Requirement 1: Named Account Storage

**User Story:** As a Plurcast user, I want to store multiple named accounts per platform, so that I can maintain separate credentials for different purposes (test, prod, personal, work).

#### Acceptance Criteria

1. WHEN a user executes `plur-creds set <platform> --account <name>`, THE Credential Store SHALL store the credentials under the namespace `plurcast.<platform>.<name>.<key>`
2. WHEN a user executes `plur-creds set <platform>` without the `--account` flag, THE Credential Store SHALL store the credentials under the namespace `plurcast.<platform>.default.<key>`
3. WHEN storing credentials for an account, THE Credential Store SHALL NOT affect credentials stored under different account names for the same platform
4. WHEN a user provides an account name, THE System SHALL accept alphanumeric characters, hyphens, and underscores with a maximum length of 64 characters
5. WHEN a user provides an invalid account name, THE System SHALL reject the operation and display an error message describing valid account name format

### Requirement 2: Account Listing and Discovery

**User Story:** As a Plurcast user, I want to list all configured accounts per platform, so that I can see what accounts are available and which credentials are stored.

#### Acceptance Criteria

1. WHEN a user executes `plur-creds list`, THE System SHALL display all platforms with at least one stored account
2. WHEN displaying account information, THE System SHALL show the platform name, account name, credential type, and storage backend for each account
3. WHEN a user executes `plur-creds list --platform <platform>`, THE System SHALL display only accounts for the specified platform
4. WHEN listing accounts, THE System SHALL indicate which account is currently active for each platform
5. WHEN no accounts exist for a platform, THE System SHALL display a message indicating no credentials are configured

### Requirement 3: Active Account Management

**User Story:** As a Plurcast user, I want to set an active account per platform, so that I can post without specifying the account every time.

#### Acceptance Criteria

1. WHEN a user executes `plur-creds use <platform> --account <name>`, THE System SHALL set the specified account as active for the platform
2. WHEN setting an active account, THE System SHALL persist the selection in the Account State File at `~/.config/plurcast/accounts.toml`
3. WHEN a user sets an active account that does not exist, THE System SHALL reject the operation and display an error message
4. WHEN no active account is set for a platform, THE System SHALL use the "default" account
5. WHEN the Account State File does not exist, THE System SHALL create it with appropriate file permissions (600 on Unix systems)

### Requirement 4: Posting with Account Selection

**User Story:** As a Plurcast user, I want to post content using a specific account or the active account, so that I can control which credentials are used for each post.

#### Acceptance Criteria

1. WHEN a user executes `plur-post <content> --account <name>`, THE System SHALL use credentials from the specified account
2. WHEN a user executes `plur-post <content>` without the `--account` flag, THE System SHALL use credentials from the active account for each enabled platform
3. WHEN posting with a non-existent account, THE System SHALL fail with an error message indicating the account does not exist
4. WHEN posting to multiple platforms with different active accounts, THE System SHALL use the correct active account for each platform
5. WHEN an account is specified that has no credentials for an enabled platform, THE System SHALL skip that platform and report the error

### Requirement 5: Account Deletion

**User Story:** As a Plurcast user, I want to delete specific accounts, so that I can remove credentials I no longer need.

#### Acceptance Criteria

1. WHEN a user executes `plur-creds delete <platform> --account <name>`, THE System SHALL remove all credentials for the specified account
2. WHEN deleting an account, THE System SHALL prompt for confirmation before removing credentials
3. WHEN deleting the active account, THE System SHALL reset the active account to "default" for that platform
4. WHEN a user attempts to delete a non-existent account, THE System SHALL display an error message
5. WHEN deleting an account, THE System SHALL remove credentials from all configured storage backends (keyring, encrypted files, plain text)

### Requirement 6: Backward Compatibility

**User Story:** As an existing Plurcast user, I want my current credentials to continue working after upgrading, so that I don't need to reconfigure my setup.

#### Acceptance Criteria

1. WHEN the System detects credentials stored in the old namespace format `plurcast.<platform>.<key>`, THE System SHALL automatically migrate them to `plurcast.<platform>.default.<key>`
2. WHEN migration occurs, THE System SHALL preserve the original credentials until migration is verified successful
3. WHEN no `--account` flag is provided in any command, THE System SHALL default to the "default" account
4. WHEN the Account State File does not exist, THE System SHALL treat "default" as the active account for all platforms
5. WHEN migration fails for any credential, THE System SHALL log the error and continue using the old namespace format for that credential

### Requirement 7: Credential Store Integration

**User Story:** As a Plurcast developer, I want the multi-account system to work with all credential storage backends, so that users can choose their preferred security model.

#### Acceptance Criteria

1. WHEN storing credentials with an account name, THE KeyringStore SHALL use the namespace format `plurcast.<platform>.<account>.<key>`
2. WHEN storing credentials with an account name, THE EncryptedFileStore SHALL use the filename format `plurcast.<platform>.<account>.<key>.age`
3. WHEN storing credentials with an account name, THE PlainFileStore SHALL use the filename format `<platform>.<account>.<key>`
4. WHEN listing accounts, THE System SHALL query all configured storage backends and aggregate results
5. WHEN retrieving credentials, THE System SHALL use the same fallback logic (keyring → encrypted → plain) for all accounts

### Requirement 8: Account Validation and Error Handling

**User Story:** As a Plurcast user, I want clear error messages when account operations fail, so that I can understand and fix issues.

#### Acceptance Criteria

1. WHEN an account operation fails, THE System SHALL display an error message describing the failure reason
2. WHEN credentials cannot be retrieved for an account, THE System SHALL indicate which account and platform failed
3. WHEN the Account State File is corrupted, THE System SHALL log a warning and use default accounts
4. WHEN multiple storage backends fail, THE System SHALL report all failures in the error message
5. WHEN an account name conflicts with reserved keywords, THE System SHALL reject the operation with a descriptive error

### Requirement 9: Testing Account Credentials

**User Story:** As a Plurcast user, I want to test credentials for a specific account, so that I can verify they work before posting.

#### Acceptance Criteria

1. WHEN a user executes `plur-creds test <platform> --account <name>`, THE System SHALL attempt to authenticate using the specified account's credentials
2. WHEN testing succeeds, THE System SHALL display a success message with account details
3. WHEN testing fails, THE System SHALL display an error message indicating the authentication failure reason
4. WHEN testing an account that does not exist, THE System SHALL display an error message
5. WHEN the `--account` flag is omitted, THE System SHALL test the active account for the platform

### Requirement 10: Account Migration Command

**User Story:** As a Plurcast user upgrading from a single-account setup, I want an explicit migration command, so that I can control when and how my credentials are migrated.

#### Acceptance Criteria

1. WHEN a user executes `plur-creds migrate --from-single-account`, THE System SHALL migrate all credentials from old namespace format to default account namespace
2. WHEN migration is executed, THE System SHALL display a summary of credentials to be migrated before proceeding
3. WHEN migration completes, THE System SHALL display a report of successful and failed migrations
4. WHEN migration is executed and no old-format credentials exist, THE System SHALL display a message indicating nothing to migrate
5. WHEN migration fails for any credential, THE System SHALL preserve the original credential and continue with remaining migrations

## Out of Scope

The following items are explicitly out of scope for this feature:

- **Account sharing or team features**: Multi-user access to the same account
- **Account synchronization**: Syncing accounts across multiple machines
- **Account templates**: Pre-configured account settings
- **Account groups**: Organizing accounts into hierarchical groups
- **Credential rotation**: Automatic credential expiration and renewal
- **Account permissions**: Fine-grained access control per account
- **GUI for account management**: Terminal UI or desktop GUI (covered in separate specs)

## Dependencies

- **Secure Credentials Spec**: Multi-account builds on the existing credential storage system
- **Phase 2 Multi-Platform**: Requires platform abstraction to be complete
- **Configuration System**: Uses existing TOML configuration infrastructure

## Success Metrics

- Users can store and switch between multiple accounts without credential loss
- Zero breaking changes for existing single-account users
- All credential storage backends support multi-account namespacing
- Account operations complete in under 100ms (excluding network authentication tests)
- Clear, actionable error messages for all failure scenarios
