# Requirements Document

## Introduction

Phase 2 of Plurcast focuses on expanding from single-platform (Nostr) to multi-platform support, enabling users to post to Nostr, Mastodon, and Bluesky simultaneously from a unified command-line interface. This phase introduces a platform abstraction layer, integrates mature Rust libraries for each platform, implements the `plur-history` tool for querying posting history, and prepares the project for its first alpha release to the community.

The core Unix philosophy remains intact: each tool does one thing well, communicates through standard streams, and composes naturally with other Unix utilities. The multi-platform capability should feel seamless to users while maintaining the agent-friendly, local-first architecture established in Phase 1.

## Requirements

### Requirement 1: Platform Abstraction Layer

**User Story:** As a developer maintaining Plurcast, I want a unified platform abstraction trait so that adding new platforms is consistent and maintainable.

#### Acceptance Criteria

1. WHEN implementing platform support THEN the system SHALL define a Rust trait that abstracts common posting operations
2. WHEN a platform is added THEN it SHALL implement the trait with methods for authentication, posting, and error handling
3. WHEN posting content THEN the abstraction SHALL handle platform-specific formatting requirements transparently
4. WHEN errors occur THEN the abstraction SHALL map platform-specific errors to unified error types
5. IF a platform requires async operations THEN the trait SHALL support async/await patterns

### Requirement 2: Mastodon Platform Integration

**User Story:** As a Plurcast user, I want to post to Mastodon and other Fediverse platforms so that I can reach my ActivityPub network.

#### Acceptance Criteria

1. WHEN configuring Mastodon THEN the system SHALL use the `megalodon` v0.14+ library
2. WHEN authenticating THEN the system SHALL support OAuth token-based authentication
3. WHEN posting THEN the system SHALL support Mastodon, Pleroma, Friendica, Firefish, GoToSocial, and Akkoma instances
4. WHEN posting THEN the system SHALL respect instance-specific character limits
5. WHEN posting succeeds THEN the system SHALL return the platform-specific post ID
6. WHEN posting fails THEN the system SHALL provide clear error messages with platform context
7. IF the instance URL is invalid THEN the system SHALL return a configuration error before attempting to post
8. IF the OAuth token is expired or invalid THEN the system SHALL return an authentication error (exit code 2)

### Requirement 3: Bluesky Platform Integration

**User Story:** As a Plurcast user, I want to post to Bluesky so that I can participate in the AT Protocol network.

#### Acceptance Criteria

1. WHEN configuring Bluesky THEN the system SHALL use the `atrium-api` v0.24+ library
2. WHEN authenticating THEN the system SHALL support DID-based identity and app passwords
3. WHEN posting THEN the system SHALL use the AT Protocol XRPC interface
4. WHEN posting THEN the system SHALL respect Bluesky's 300 character limit for posts
5. WHEN posting succeeds THEN the system SHALL return the AT URI (e.g., `at://did:plc:...`)
6. WHEN posting fails THEN the system SHALL provide clear error messages with AT Protocol context
7. IF the handle or authentication is invalid THEN the system SHALL return an authentication error (exit code 2)
8. IF the PDS (Personal Data Server) is unreachable THEN the system SHALL return a network error

### Requirement 4: Multi-Platform Posting in plur-post

**User Story:** As a Plurcast user, I want to post to multiple platforms simultaneously so that I can efficiently cross-post content.

#### Acceptance Criteria

1. WHEN posting without platform flags THEN the system SHALL post to all enabled platforms from config
2. WHEN using `--platform` flag THEN the system SHALL post only to specified platforms
3. WHEN posting to multiple platforms THEN the system SHALL execute posts concurrently
4. WHEN posting to multiple platforms THEN the system SHALL record results for each platform in the database
5. WHEN any platform fails THEN the system SHALL continue posting to remaining platforms
6. WHEN posting completes THEN the system SHALL output one line per platform in format `platform:post_id`
7. WHEN all platforms succeed THEN the system SHALL exit with code 0
8. WHEN at least one platform fails THEN the system SHALL exit with code 1
9. IF authentication fails for any platform THEN the system SHALL exit with code 2
10. WHEN using `--verbose` flag THEN the system SHALL show per-platform progress and errors

### Requirement 5: Configuration Management for Multiple Platforms

**User Story:** As a Plurcast user, I want to configure multiple platforms easily so that I can control which platforms are active and how they're authenticated.

#### Acceptance Criteria

1. WHEN reading configuration THEN the system SHALL support sections for `[nostr]`, `[mastodon]`, and `[bluesky]`
2. WHEN a platform is disabled THEN the system SHALL skip it during posting
3. WHEN default platforms are specified THEN the system SHALL use them unless overridden by CLI flags
4. WHEN credentials are missing THEN the system SHALL provide clear guidance on authentication setup
5. IF configuration file is missing THEN the system SHALL create a default configuration with all platforms disabled
6. WHEN validating configuration THEN the system SHALL check for required fields per platform
7. IF sensitive credentials are in the main config THEN the system SHALL warn users to use separate credential files

### Requirement 6: plur-history Tool Implementation

**User Story:** As a Plurcast user, I want to query my posting history so that I can review what I've posted and when.

#### Acceptance Criteria

1. WHEN running `plur-history` without arguments THEN the system SHALL display the last 20 posts
2. WHEN using `--platform` flag THEN the system SHALL filter results to the specified platform
3. WHEN using `--since` and `--until` flags THEN the system SHALL filter by date range
4. WHEN using `--search` flag THEN the system SHALL filter posts by content matching
5. WHEN using `--format json` THEN the system SHALL output posts as JSON array
6. WHEN using `--format jsonl` THEN the system SHALL output posts as JSON lines
7. WHEN using `--format csv` THEN the system SHALL output posts as CSV with headers
8. WHEN using default format THEN the system SHALL output human-readable text
9. WHEN no posts match filters THEN the system SHALL output nothing and exit with code 0
10. WHEN database is missing THEN the system SHALL exit with error code 1
11. WHEN displaying posts THEN the system SHALL include post ID, timestamp, platform, status, and content preview

### Requirement 7: Database Schema Updates

**User Story:** As a developer, I want the database schema to support multi-platform posting so that we can track posts across all platforms accurately.

#### Acceptance Criteria

1. WHEN storing posts THEN the system SHALL use the existing `posts` and `post_records` tables
2. WHEN recording platform results THEN the system SHALL store platform name, platform post ID, timestamp, success status, and error messages
3. WHEN querying history THEN the system SHALL efficiently join posts with their platform records
4. WHEN a post is attempted on multiple platforms THEN the system SHALL create one `post_records` entry per platform
5. IF database migrations are needed THEN the system SHALL use SQLx migrations
6. WHEN the database schema changes THEN the system SHALL maintain backward compatibility with Phase 1 data

### Requirement 8: Error Handling and Resilience

**User Story:** As a Plurcast user, I want clear error messages and resilient behavior so that I understand what went wrong and can recover easily.

#### Acceptance Criteria

1. WHEN a platform is unreachable THEN the system SHALL retry with exponential backoff (up to 3 attempts)
2. WHEN rate limits are hit THEN the system SHALL report the rate limit error clearly
3. WHEN authentication fails THEN the system SHALL provide platform-specific guidance for re-authentication
4. WHEN content violates platform rules THEN the system SHALL report the specific validation error
5. WHEN posting to multiple platforms THEN the system SHALL report partial success clearly
6. WHEN errors occur THEN the system SHALL log to stderr with context (platform, error type, timestamp)
7. IF `--verbose` is enabled THEN the system SHALL show detailed error traces

### Requirement 9: Unix Philosophy and Agent-Friendly Design

**User Story:** As a user or AI agent, I want Plurcast to maintain Unix principles so that I can compose tools naturally and automate workflows.

#### Acceptance Criteria

1. WHEN tools are invoked THEN they SHALL read from stdin and write to stdout
2. WHEN errors occur THEN they SHALL be written to stderr
3. WHEN tools complete THEN they SHALL use meaningful exit codes (0, 1, 2, 3)
4. WHEN `--help` is used THEN the system SHALL provide comprehensive usage information
5. WHEN `--format json` is used THEN the system SHALL output machine-readable JSON
6. WHEN piping between tools THEN the system SHALL preserve composability
7. WHEN invoked by agents THEN the system SHALL behave identically to human invocation

### Requirement 10: Testing and Quality Assurance

**User Story:** As a developer, I want comprehensive tests so that multi-platform functionality is reliable and regressions are caught early.

#### Acceptance Criteria

1. WHEN platform traits are implemented THEN unit tests SHALL verify each method
2. WHEN posting to platforms THEN integration tests SHALL verify end-to-end flows
3. WHEN errors occur THEN tests SHALL verify error handling and exit codes
4. WHEN configuration is parsed THEN tests SHALL verify all valid and invalid configurations
5. WHEN `plur-history` queries data THEN tests SHALL verify filtering and formatting
6. IF external platform APIs are unavailable THEN tests SHALL use mocks or test doubles
7. WHEN running tests THEN they SHALL not require live platform credentials
