# Requirements Document

## Introduction

Currently, Plurcast stores platform credentials (Nostr private keys, Mastodon OAuth tokens, Bluesky app passwords) in plain text files with only Unix file permissions (600) for protection. This poses security risks if the user's account is compromised, files are backed up without encryption, or the system is accessed by malicious software.

This feature will implement secure credential storage using OS-native keyrings (macOS Keychain, Windows Credential Manager, Linux Secret Service) as the primary method, with encrypted file storage as a fallback, and plain text files as a deprecated legacy option. The implementation must maintain backward compatibility with existing plain text credential files while providing a clear migration path.

## Requirements

### Requirement 1: OS-Native Keyring Integration

**User Story:** As a Plurcast user, I want my credentials stored in my operating system's secure keyring, so that they are protected using the same security mechanisms as my other passwords and sensitive data.

#### Acceptance Criteria

1. WHEN the user stores credentials THEN the system SHALL attempt to use the OS keyring first (macOS Keychain, Windows Credential Manager, or Linux Secret Service)
2. WHEN credentials are stored in the keyring THEN they SHALL be encrypted at rest by the operating system
3. WHEN the keyring is unavailable THEN the system SHALL fall back to encrypted file storage with a clear warning to the user
4. WHEN credentials are retrieved from the keyring THEN the system SHALL handle keyring access errors gracefully
5. IF the OS keyring requires user authentication THEN the system SHALL prompt appropriately and handle authentication failures

### Requirement 2: Encrypted File Storage Fallback

**User Story:** As a Plurcast user on a system without keyring support, I want my credentials encrypted with a master password, so that they remain secure even if someone gains access to my files.

#### Acceptance Criteria

1. WHEN the OS keyring is unavailable THEN the system SHALL offer encrypted file storage as an alternative
2. WHEN encrypted file storage is used THEN the system SHALL prompt for a master password on first use
3. WHEN credentials are stored in encrypted files THEN they SHALL use the `age` encryption format
4. WHEN the master password is entered THEN decrypted credentials SHALL be cached in memory for the session duration only
5. WHEN encrypted credential files are created THEN they SHALL have 600 file permissions (owner read/write only)
6. IF the master password is incorrect THEN the system SHALL provide clear error messages and allow retry

### Requirement 3: Backward Compatibility with Plain Text Files

**User Story:** As an existing Plurcast user, I want my current plain text credential files to continue working, so that I'm not locked out of my accounts during the upgrade.

#### Acceptance Criteria

1. WHEN plain text credential files exist THEN the system SHALL continue to read them successfully
2. WHEN plain text credentials are used THEN the system SHALL log a security warning
3. WHEN the system detects plain text credentials THEN it SHALL suggest migration to secure storage
4. IF the user has not migrated THEN the system SHALL not break existing functionality
5. WHEN new credentials are added THEN the system SHALL use secure storage by default, not plain text

### Requirement 4: Credential Migration Tool

**User Story:** As a Plurcast user with existing plain text credentials, I want an easy way to migrate to secure storage, so that I can improve my security without manual file manipulation.

#### Acceptance Criteria

1. WHEN the user runs the migration command THEN the system SHALL detect all plain text credential files
2. WHEN migration is initiated THEN the system SHALL copy credentials to secure storage (keyring or encrypted files)
3. WHEN migration completes successfully THEN the system SHALL offer to delete the plain text files
4. IF migration fails for any credential THEN the system SHALL report the error and leave the original file intact
5. WHEN migration is complete THEN the system SHALL verify that all credentials work in the new storage

### Requirement 5: Credential Management Commands

**User Story:** As a Plurcast user, I want command-line tools to manage my credentials, so that I can add, update, test, and remove credentials without manually editing files.

#### Acceptance Criteria

1. WHEN the user runs `plur-creds set <platform>` THEN the system SHALL prompt for credentials and store them securely
2. WHEN the user runs `plur-creds list` THEN the system SHALL show which platforms have stored credentials (without revealing the values)
3. WHEN the user runs `plur-creds delete <platform>` THEN the system SHALL remove the credentials from secure storage
4. WHEN the user runs `plur-creds test <platform>` THEN the system SHALL verify the credentials work by authenticating with the platform
5. WHEN the user runs `plur-creds migrate` THEN the system SHALL migrate plain text credentials to secure storage
6. IF a credential operation fails THEN the system SHALL provide clear error messages with suggested remediation

### Requirement 6: Security Audit Tool

**User Story:** As a Plurcast user, I want to audit my credential security, so that I can identify and fix potential vulnerabilities in my setup.

#### Acceptance Criteria

1. WHEN the user runs `plur-creds audit` THEN the system SHALL check for plain text credential files
2. WHEN plain text files are found THEN the audit SHALL report them as security issues
3. WHEN credential files exist THEN the audit SHALL verify file permissions are 600 or stricter
4. WHEN the audit completes THEN it SHALL provide a summary of security status and recommendations
5. IF security issues are found THEN the audit SHALL suggest specific remediation steps

### Requirement 7: Configuration for Storage Backend

**User Story:** As a Plurcast user, I want to choose my credential storage method, so that I can balance security and convenience based on my needs.

#### Acceptance Criteria

1. WHEN the configuration is created THEN it SHALL default to "keyring" storage
2. WHEN the user sets storage to "keyring" THEN the system SHALL use OS-native keyring with encrypted file fallback
3. WHEN the user sets storage to "encrypted" THEN the system SHALL use encrypted files with master password
4. WHEN the user sets storage to "plain" THEN the system SHALL use plain text files and display security warnings
5. IF an invalid storage backend is specified THEN the system SHALL reject the configuration with a clear error

### Requirement 8: Secure Credential Access in Platform Clients

**User Story:** As a developer, I want platform clients to retrieve credentials from the secure storage abstraction, so that the credential storage implementation can be changed without modifying platform code.

#### Acceptance Criteria

1. WHEN a platform client needs credentials THEN it SHALL call the CredentialManager API
2. WHEN credentials are retrieved THEN they SHALL be cached in memory for the session only
3. WHEN the application exits THEN cached credentials SHALL be cleared from memory
4. IF credential retrieval fails THEN the platform client SHALL receive a clear error
5. WHEN credentials are used THEN the system SHALL log access (without logging the credential values)

### Requirement 9: Interactive Setup Wizard

**User Story:** As a new Plurcast user, I want an interactive setup wizard, so that I can easily configure my credentials securely without reading extensive documentation.

#### Acceptance Criteria

1. WHEN the user runs `plur-setup` THEN the system SHALL guide them through credential setup for each platform
2. WHEN credentials are entered THEN the system SHALL validate them by testing authentication
3. WHEN authentication succeeds THEN the system SHALL store credentials in secure storage
4. IF authentication fails THEN the wizard SHALL allow the user to retry or skip the platform
5. WHEN setup completes THEN the system SHALL confirm which platforms are configured and ready to use

### Requirement 10: Security Documentation

**User Story:** As a Plurcast user, I want clear documentation about credential security, so that I understand the security model and can make informed decisions.

#### Acceptance Criteria

1. WHEN documentation is provided THEN it SHALL explain all three storage options (keyring, encrypted, plain)
2. WHEN migration is documented THEN it SHALL provide step-by-step instructions
3. WHEN security best practices are documented THEN they SHALL include recommendations for each operating system
4. WHEN the architecture is documented THEN it SHALL explain the security model and threat mitigation
5. IF security warnings are shown THEN they SHALL reference the documentation for more information
