# SSB Integration Requirements Document

## Introduction

This document specifies the requirements for integrating Secure Scuttlebutt (SSB) as a third platform in Plurcast, following Phase 3 of the development roadmap. SSB is a peer-to-peer, offline-first social protocol that aligns with Plurcast's Unix philosophy and decentralization values. Unlike centralized platforms, SSB operates through local append-only logs and peer-to-peer gossip replication, requiring no servers or blockchain infrastructure.

The integration will be implemented in phases, starting with basic posting capabilities (Phase 3.1), progressing through enhanced integration with credential management (Phase 3.2), adding history and import features (Phase 3.3), and optionally including server lifecycle management (Phase 3.4).

## Glossary

- **SSB (Secure Scuttlebutt)**: A peer-to-peer social protocol using append-only logs and gossip replication
- **kuska-ssb**: A Rust library for SSB protocol implementation with async support
- **Feed Database**: The local storage for SSB messages, managed by the kuska-ssb library
- **Feed**: An append-only log of signed messages associated with a cryptographic identity
- **Message**: A JSON object containing content, timestamp, and cryptographic signature
- **Pub**: A public SSB server that aids peer discovery and replication over the internet
- **Ed25519**: The elliptic curve cryptography algorithm used for SSB keypairs
- **Plurcast**: The command-line tool suite for cross-posting to decentralized platforms
- **Platform**: A social media protocol or service (Nostr, Mastodon, SSB)
- **Credential Manager**: The Plurcast component that securely stores authentication credentials
- **plur-post**: The Plurcast binary for posting content to platforms
- **plur-setup**: The Plurcast binary for interactive platform configuration
- **plur-creds**: The Plurcast binary for credential management
- **plur-history**: The Plurcast binary for querying posting history
- **plur-import**: The Plurcast binary for importing posts from platform exports
- **plur-export**: The Plurcast binary for exporting posts to various formats
- **Replication**: The process of synchronizing feeds between SSB peers via gossip protocol
- **Local Feed**: The user's own SSB message log stored on their local machine

## Requirements

### Requirement 1: Basic SSB Posting

**User Story:** As a Plurcast user, I want to post content to my local SSB feed, so that I can share messages with the SSB network alongside my Nostr and Mastodon posts.

#### Acceptance Criteria

1. WHEN the user invokes plur-post with SSB enabled, THE Plurcast System SHALL initialize the kuska-ssb library with the user's keypair
2. WHEN the user provides post content via stdin or command argument, THE Plurcast System SHALL create a valid SSB message with type "post" containing the provided content
3. WHEN the SSB message is created, THE Plurcast System SHALL sign the message using the user's Ed25519 private key via the kuska-ssb library
4. WHEN the signed message is ready, THE Plurcast System SHALL append the message to the user's local SSB feed database
5. WHEN the message is successfully appended, THE Plurcast System SHALL return the SSB message identifier in the format "ssb:%<message-hash>"
6. WHEN the message posting fails, THE Plurcast System SHALL emit an error message to stderr describing the failure reason
7. WHEN the message posting fails, THE Plurcast System SHALL exit with exit code 1

### Requirement 2: SSB Platform Configuration

**User Story:** As a Plurcast user, I want to configure SSB connection settings, so that Plurcast can communicate with my local sbot instance.

#### Acceptance Criteria

1. WHEN the user creates a config.toml file with an [ssb] section, THE Plurcast System SHALL parse the SSB configuration parameters
2. THE Plurcast System SHALL support a feed_path configuration parameter specifying the local SSB feed database directory
3. THE Plurcast System SHALL support an enabled boolean configuration parameter to enable or disable SSB posting
4. WHEN the feed_path parameter is omitted, THE Plurcast System SHALL default to "~/.plurcast-ssb"
5. WHEN the enabled parameter is omitted or set to false, THE Plurcast System SHALL skip SSB when posting to multiple platforms
6. THE Plurcast System SHALL create the feed database directory if it does not exist
7. THE Plurcast System SHALL set appropriate file permissions (700) on the feed database directory

### Requirement 3: SSB Credential Management

**User Story:** As a Plurcast user, I want to securely store my SSB keypair, so that my private key is protected from unauthorized access.

#### Acceptance Criteria

1. WHEN the user runs plur-creds set ssb, THE Plurcast System SHALL prompt for the SSB private key or offer to generate a new keypair
2. WHEN the user provides an SSB keypair, THE Plurcast System SHALL validate that the keypair contains valid Ed25519 keys
3. WHEN the keypair is valid, THE Plurcast System SHALL store the private key in the credential manager under service "plurcast.ssb" and key "keypair"
4. WHEN the user runs plur-creds list, THE Plurcast System SHALL display SSB in the list of configured platforms if SSB credentials exist
5. WHEN the user runs plur-creds delete ssb, THE Plurcast System SHALL remove the SSB keypair from the credential manager
6. WHEN the user runs plur-creds test ssb, THE Plurcast System SHALL initialize the kuska-ssb library and verify the keypair can sign messages
7. WHEN the SSB credential test succeeds, THE Plurcast System SHALL exit with exit code 0 and emit a success message with the public key

### Requirement 4: SSB Library Integration

**User Story:** As a Plurcast user, I want Plurcast to manage SSB feeds directly using the kuska-ssb library, so that I don't need to install or manage external SSB server processes.

#### Acceptance Criteria

1. WHEN plur-post initializes the SSB platform, THE Plurcast System SHALL load the user's SSB keypair from the credential manager
2. WHEN the SSB keypair is loaded, THE Plurcast System SHALL initialize the kuska-ssb library with the keypair
3. WHEN the kuska-ssb library is initialized, THE Plurcast System SHALL open or create the local SSB feed database at the configured path
4. WHEN the feed database initialization fails, THE Plurcast System SHALL emit an error message describing the failure reason
5. WHEN the feed database initialization fails, THE Plurcast System SHALL exit with exit code 2 indicating a configuration error
6. WHEN the feed database is successfully opened, THE Plurcast System SHALL proceed with message operations
7. WHEN verbose logging is enabled, THE Plurcast System SHALL log the feed database path and initialization status

### Requirement 5: SSB Message Format Compliance

**User Story:** As a Plurcast user, I want my SSB messages to be valid according to the SSB protocol, so that other SSB clients can read and display my posts correctly.

#### Acceptance Criteria

1. THE Plurcast System SHALL create SSB messages with a "previous" field containing the hash of the previous message in the feed
2. THE Plurcast System SHALL create SSB messages with an "author" field containing the user's public key in the format "@<base64-pubkey>.ed25519"
3. THE Plurcast System SHALL create SSB messages with a "sequence" field containing the message sequence number as an integer
4. THE Plurcast System SHALL create SSB messages with a "timestamp" field containing the Unix timestamp in milliseconds
5. THE Plurcast System SHALL create SSB messages with a "hash" field set to "sha256"
6. THE Plurcast System SHALL create SSB messages with a "content" object containing a "type" field set to "post"
7. THE Plurcast System SHALL create SSB messages with a "content" object containing a "text" field with the user's post content
8. THE Plurcast System SHALL create SSB messages with a "signature" field containing the Ed25519 signature in base64 format
9. WHEN the message is the first in the feed, THE Plurcast System SHALL set the "previous" field to null
10. WHEN the message is not the first in the feed, THE Plurcast System SHALL query sbot for the previous message hash

### Requirement 6: SSB Content Validation

**User Story:** As a Plurcast user, I want to be warned when my post content exceeds SSB's practical limits, so that I can adjust my content before posting.

#### Acceptance Criteria

1. WHEN the user provides post content, THE Plurcast System SHALL calculate the total message size including JSON structure and signature
2. WHEN the calculated message size exceeds 8192 bytes, THE Plurcast System SHALL emit a warning message to stderr
3. WHEN the calculated message size exceeds 8192 bytes, THE Plurcast System SHALL suggest splitting the content or using blob attachments
4. WHEN the user provides the --platform ssb flag with oversized content, THE Plurcast System SHALL post to other platforms but skip SSB
5. WHEN the user provides the --platform ssb flag exclusively with oversized content, THE Plurcast System SHALL exit with exit code 3 indicating invalid input
6. WHEN verbose logging is enabled, THE Plurcast System SHALL log the calculated message size for each platform

### Requirement 7: Multi-Platform SSB Integration

**User Story:** As a Plurcast user, I want to post to SSB alongside Nostr and Mastodon, so that I can reach audiences on all three platforms with a single command.

#### Acceptance Criteria

1. WHEN the user runs plur-post without the --platform flag, THE Plurcast System SHALL post to all enabled platforms including SSB
2. WHEN the user runs plur-post with --platform ssb, THE Plurcast System SHALL post only to SSB
3. WHEN the user runs plur-post with --platform nostr,ssb, THE Plurcast System SHALL post to both Nostr and SSB but not Mastodon
4. WHEN posting to multiple platforms including SSB, THE Plurcast System SHALL execute platform posts concurrently
5. WHEN SSB posting fails but other platforms succeed, THE Plurcast System SHALL exit with exit code 1 indicating partial failure
6. WHEN SSB posting succeeds, THE Plurcast System SHALL emit the SSB message ID to stdout in the format "ssb:%<hash>"
7. WHEN posting to multiple platforms, THE Plurcast System SHALL emit one message ID per line with platform prefix

### Requirement 8: SSB Interactive Setup

**User Story:** As a Plurcast user, I want to configure SSB through an interactive wizard, so that I can set up SSB without manually editing configuration files.

#### Acceptance Criteria

1. WHEN the user runs plur-setup and selects SSB, THE Plurcast System SHALL check for an existing keypair at ~/.ssb/secret
2. WHEN an existing keypair is found, THE Plurcast System SHALL ask the user if they want to import the existing keypair
3. WHEN no existing keypair is found or the user declines to import it, THE Plurcast System SHALL offer to generate a new keypair
4. WHEN the user chooses to generate a new keypair, THE Plurcast System SHALL create a valid Ed25519 keypair using the kuska-ssb library
5. WHEN the keypair is generated, THE Plurcast System SHALL save it to the credential manager under service "plurcast.ssb"
6. WHEN the keypair is configured, THE Plurcast System SHALL initialize the feed database and display the user's SSB public key
7. WHEN the setup succeeds, THE Plurcast System SHALL save the SSB configuration to config.toml with enabled set to true
8. WHEN the setup succeeds, THE Plurcast System SHALL display the feed database path and confirm SSB is ready to use

### Requirement 9: SSB History Queries

**User Story:** As a Plurcast user, I want to query my SSB posting history, so that I can review what I've posted to the SSB network.

#### Acceptance Criteria

1. WHEN the user runs plur-history --platform ssb, THE Plurcast System SHALL query the local database for posts with platform "ssb"
2. WHEN the user runs plur-history --platform ssb with no local records, THE Plurcast System SHALL offer to import from the local SSB feed
3. WHEN the user runs plur-history --since <date> --platform ssb, THE Plurcast System SHALL filter SSB posts by the specified date range
4. WHEN the user runs plur-history --search <term> --platform ssb, THE Plurcast System SHALL search SSB post content for the specified term
5. WHEN the user runs plur-history --format json --platform ssb, THE Plurcast System SHALL output SSB posts in JSON format
6. THE Plurcast System SHALL display SSB posts with the message ID, timestamp, content, and posting status
7. WHEN verbose logging is enabled, THE Plurcast System SHALL include the SSB message hash and sequence number in the output

### Requirement 10: SSB Feed Import

**User Story:** As a Plurcast user, I want to import my existing SSB posts into Plurcast's database, so that I have a complete history of my SSB activity.

#### Acceptance Criteria

1. WHEN the user runs plur-import ssb, THE Plurcast System SHALL initialize the kuska-ssb library and open the local feed database
2. WHEN the feed database is opened, THE Plurcast System SHALL query for all messages in the user's feed with type "post"
3. WHEN SSB messages are retrieved, THE Plurcast System SHALL parse each message and extract the content text
4. WHEN a message is parsed, THE Plurcast System SHALL create a Plurcast post record with status "imported"
5. WHEN a message is parsed, THE Plurcast System SHALL create a post_record entry linking the Plurcast post to the SSB message ID
6. WHEN a message already exists in the database, THE Plurcast System SHALL skip the duplicate and continue with the next message
7. WHEN the import completes, THE Plurcast System SHALL display the count of imported messages and any errors encountered
8. WHEN the import fails to open the feed database, THE Plurcast System SHALL exit with exit code 2 and emit a database error message

### Requirement 11: SSB Export Format

**User Story:** As a Plurcast user, I want to export my Plurcast posts in SSB-compatible format, so that I can migrate my content or share it with other SSB tools.

#### Acceptance Criteria

1. WHEN the user runs plur-export --format ssb, THE Plurcast System SHALL query the database for all posts with platform "ssb"
2. WHEN SSB posts are retrieved, THE Plurcast System SHALL format each post as a valid SSB message JSON object
3. THE Plurcast System SHALL include the original SSB message ID in the exported data if available
4. THE Plurcast System SHALL include the timestamp, content, and sequence number in the exported data
5. WHEN the user specifies --output <file>, THE Plurcast System SHALL write the exported data to the specified file
6. WHEN no output file is specified, THE Plurcast System SHALL write the exported data to stdout
7. WHEN the export completes, THE Plurcast System SHALL exit with exit code 0

### Requirement 12: SSB Multi-Account Support

**User Story:** As a Plurcast user, I want to manage multiple SSB identities, so that I can post from different SSB accounts for different purposes.

#### Acceptance Criteria

1. WHEN the user runs plur-creds set ssb --account <name>, THE Plurcast System SHALL store the SSB keypair under the specified account name
2. WHEN the user runs plur-creds list --platform ssb, THE Plurcast System SHALL display all configured SSB accounts
3. WHEN the user runs plur-creds use ssb --account <name>, THE Plurcast System SHALL set the specified account as the active SSB account
4. WHEN the user runs plur-post --account <name>, THE Plurcast System SHALL use the specified SSB account for posting
5. WHEN no account is specified, THE Plurcast System SHALL use the active SSB account from the account manager
6. WHEN no active account is set, THE Plurcast System SHALL default to the "default" account
7. THE Plurcast System SHALL store SSB credentials in the credential manager with namespace "plurcast.ssb.<account>.keypair"

### Requirement 13: SSB Error Handling

**User Story:** As a Plurcast user, I want clear error messages when SSB operations fail, so that I can understand and resolve issues quickly.

#### Acceptance Criteria

1. WHEN the feed database cannot be opened, THE Plurcast System SHALL emit the error message "Failed to open SSB feed database at <path>"
2. WHEN the SSB keypair is invalid, THE Plurcast System SHALL emit the error message "Invalid SSB keypair - check credential format"
3. WHEN the SSB keypair is not found, THE Plurcast System SHALL emit the error message "SSB credentials not configured - run plur-setup or plur-creds set ssb"
4. WHEN the message size exceeds limits, THE Plurcast System SHALL emit the error message "Post content exceeds SSB message size limit (8KB)"
5. WHEN message signing fails, THE Plurcast System SHALL emit the error message "Failed to sign SSB message - check keypair"
6. WHEN the kuska-ssb library returns an error, THE Plurcast System SHALL emit the error message "SSB library error: <error-message>"
7. WHEN verbose logging is enabled, THE Plurcast System SHALL include the full error context and stack trace in the log output

### Requirement 14: SSB Documentation

**User Story:** As a Plurcast user, I want comprehensive documentation for SSB integration, so that I can understand how to set up and use SSB with Plurcast.

#### Acceptance Criteria

1. THE Plurcast System SHALL include an SSB setup guide in the documentation explaining keypair generation and feed database initialization
2. THE Plurcast System SHALL include an SSB configuration guide explaining all config.toml parameters including feed_path
3. THE Plurcast System SHALL include an SSB troubleshooting guide with common issues and solutions
4. THE Plurcast System SHALL include an SSB comparison guide explaining differences between SSB, Nostr, and Mastodon
5. WHEN the user runs plur-post --help, THE Plurcast System SHALL include SSB in the list of supported platforms
6. WHEN the user runs plur-setup --help, THE Plurcast System SHALL include SSB in the list of configurable platforms
7. THE Plurcast System SHALL include example SSB commands in the README with expected output

### Requirement 15: SSB Replication

**User Story:** As a Plurcast user, I want my SSB posts to replicate to the network, so that other SSB users can see my content.

#### Acceptance Criteria

1. WHEN the user posts to SSB, THE Plurcast System SHALL append the message to the local feed database
2. WHEN the message is appended, THE Plurcast System SHALL initiate replication with configured pub servers
3. WHEN replication is initiated, THE Plurcast System SHALL connect to each configured pub using the multiserver protocol
4. WHEN connected to a pub, THE Plurcast System SHALL authenticate using the user's Ed25519 keypair
5. WHEN authenticated, THE Plurcast System SHALL push the new message to the pub server
6. WHEN the push succeeds, THE Plurcast System SHALL log the successful replication
7. WHEN the push fails, THE Plurcast System SHALL log the error but not fail the post operation
8. WHEN no pubs are configured, THE Plurcast System SHALL emit a warning that posts are local-only
9. WHEN verbose logging is enabled, THE Plurcast System SHALL log replication progress and connection details

### Requirement 16: SSB Pub Server Configuration

**User Story:** As a Plurcast user, I want to configure pub servers for replication, so that my posts reach the SSB network.

#### Acceptance Criteria

1. WHEN the user adds a pub server to config.toml, THE Plurcast System SHALL parse the multiserver address format
2. THE Plurcast System SHALL validate that the multiserver address contains protocol, host, port, and public key
3. WHEN the multiserver address is invalid, THE Plurcast System SHALL emit an error message describing the format error
4. WHEN plur-setup runs, THE Plurcast System SHALL offer to add default pub servers (hermies.club, etc.)
5. WHEN the user accepts default pubs, THE Plurcast System SHALL add them to the config.toml pubs array
6. WHEN the user runs plur-creds test ssb, THE Plurcast System SHALL test connectivity to all configured pubs
7. WHEN pub connectivity test succeeds, THE Plurcast System SHALL display which pubs are reachable

### Requirement 17: SSB Background Replication

**User Story:** As a Plurcast user, I want background replication to continue after posting, so that I receive updates from people I follow.

#### Acceptance Criteria

1. WHEN plur-post completes, THE Plurcast System SHALL spawn a background replication task
2. WHEN the background task runs, THE Plurcast System SHALL connect to configured pubs and pull new messages
3. WHEN new messages are received, THE Plurcast System SHALL validate signatures and append to local feed database
4. WHEN the background task completes, THE Plurcast System SHALL exit gracefully
5. WHEN the background task encounters errors, THE Plurcast System SHALL log errors but continue operation
6. WHEN the user provides the --no-sync flag, THE Plurcast System SHALL skip background replication
7. WHEN verbose logging is enabled, THE Plurcast System SHALL log received message counts and sync duration

### Requirement 18: SSB Testing

**User Story:** As a Plurcast developer, I want comprehensive tests for SSB integration, so that I can ensure the implementation is correct and maintainable.

#### Acceptance Criteria

1. THE Plurcast System SHALL include unit tests for SSB message creation and signing using kuska-ssb
2. THE Plurcast System SHALL include unit tests for SSB configuration parsing
3. THE Plurcast System SHALL include integration tests for posting to a test feed database
4. THE Plurcast System SHALL include integration tests for importing from a test SSB feed database
5. THE Plurcast System SHALL include integration tests for multi-platform posting including SSB
6. THE Plurcast System SHALL include integration tests for SSB credential management
7. WHEN all tests pass, THE Plurcast System SHALL exit with exit code 0

---

**Version**: 0.3.0-alpha2
**Last Updated**: 2025-10-31
**Status**: Requirements Complete - Ready for Design Review
**Phase**: Phase 3 (SSB Integration)
