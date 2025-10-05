# Requirements Document

## Introduction

This specification covers the foundational components of Plurcast's Alpha MVP release. The goal is to establish the core infrastructure that all other tools will build upon: database schema, configuration management, authentication handling, and a basic posting capability to Nostr. This foundation must embody Unix philosophy principles while being agent-friendly and human-usable.

The Alpha MVP will enable users to post content to Nostr from the command line, with all data stored locally in SQLite and configuration managed through TOML files following XDG Base Directory standards.

## Requirements

### Requirement 1: Database Infrastructure

**User Story:** As a Plurcast user, I want my posts and platform records stored locally in a SQLite database, so that I have complete ownership and control over my data.

#### Acceptance Criteria

1. WHEN the application initializes THEN it SHALL create a SQLite database at `~/.local/share/plurcast/posts.db` if it does not exist
2. WHEN the database is created THEN it SHALL include a `posts` table with columns: id (TEXT PRIMARY KEY), content (TEXT NOT NULL), created_at (INTEGER NOT NULL), scheduled_at (INTEGER), status (TEXT DEFAULT 'pending'), metadata (TEXT)
3. WHEN the database is created THEN it SHALL include a `post_records` table with columns: id (INTEGER PRIMARY KEY), post_id (TEXT NOT NULL), platform (TEXT NOT NULL), platform_post_id (TEXT), posted_at (INTEGER), success (INTEGER DEFAULT 0), error_message (TEXT), and a foreign key constraint to posts(id)
4. WHEN the database is created THEN it SHALL include a `platforms` table with columns: name (TEXT PRIMARY KEY), enabled (INTEGER DEFAULT 1), config (TEXT)
5. WHEN database operations fail THEN the system SHALL return appropriate error codes and messages to stderr
6. WHEN the database path is specified via environment variable `PLURCAST_DB_PATH` THEN the system SHALL use that path instead of the default

### Requirement 2: Configuration Management

**User Story:** As a Plurcast user, I want my platform credentials and preferences stored in a standard configuration file, so that I can easily manage and version control my settings.

#### Acceptance Criteria

1. WHEN the application starts THEN it SHALL look for configuration at `~/.config/plurcast/config.toml`
2. IF the environment variable `PLURCAST_CONFIG` is set THEN the system SHALL use that path instead
3. WHEN the configuration file is missing THEN the system SHALL create a default configuration with sensible defaults
4. WHEN parsing the configuration THEN it SHALL support sections for [database], [nostr], [mastodon], [bluesky], and [defaults]
5. WHEN configuration parsing fails THEN the system SHALL exit with code 2 and output a clear error message to stderr
6. WHEN sensitive credential files are created THEN they SHALL have file permissions set to 600 (owner read/write only)
7. WHEN the configuration includes a nostr.keys_file path THEN it SHALL resolve relative paths from the config directory

### Requirement 3: Nostr Authentication and Key Management

**User Story:** As a Plurcast user, I want to authenticate with Nostr using my private key, so that I can post content to Nostr relays.

#### Acceptance Criteria

1. WHEN the nostr.keys_file is specified in config THEN the system SHALL read the private key from that file
2. WHEN the keys file contains a hex-encoded private key THEN the system SHALL parse it correctly
3. WHEN the keys file contains a bech32-encoded (nsec) private key THEN the system SHALL parse it correctly
4. IF the keys file does not exist THEN the system SHALL exit with code 2 and output an error message indicating missing authentication
5. WHEN the private key is invalid THEN the system SHALL exit with code 2 and output a clear error message
6. WHEN connecting to Nostr relays THEN the system SHALL use the relay list from config.toml [nostr.relays]
7. IF no relays are configured THEN the system SHALL use a default set of well-known relays

### Requirement 4: Basic Post Creation (plur-post)

**User Story:** As a Plurcast user, I want to post content to Nostr from the command line, so that I can share my thoughts on the decentralized social web.

#### Acceptance Criteria

1. WHEN content is provided via stdin THEN plur-post SHALL read and post that content
2. WHEN content is provided as a command-line argument THEN plur-post SHALL post that content
3. WHEN posting to Nostr THEN the system SHALL create a kind 1 (text note) event
4. WHEN posting succeeds THEN plur-post SHALL output the platform post ID in format `nostr:note1...` to stdout
5. WHEN posting succeeds THEN the system SHALL exit with code 0
6. WHEN posting fails on all platforms THEN the system SHALL exit with code 1
7. WHEN authentication fails THEN the system SHALL exit with code 2
8. WHEN input is invalid THEN the system SHALL exit with code 3
9. WHEN posting THEN the system SHALL record the post in the local database with status 'posted' or 'failed'
10. WHEN posting THEN the system SHALL create a post_record entry for each platform attempt

### Requirement 5: Platform Abstraction

**User Story:** As a Plurcast developer, I want a clean abstraction for platform clients, so that adding new platforms follows a consistent pattern.

#### Acceptance Criteria

1. WHEN implementing platform support THEN each platform SHALL implement a common trait with methods: post(), authenticate(), and validate_content()
2. WHEN a platform client is created THEN it SHALL accept configuration from the parsed TOML config
3. WHEN posting to a platform THEN the abstraction SHALL return a Result type with platform-specific post ID or error
4. WHEN validating content THEN each platform SHALL check content length and format constraints
5. IF content exceeds platform limits THEN the system SHALL return an error before attempting to post

### Requirement 6: Error Handling and Logging

**User Story:** As a Plurcast user, I want clear error messages and appropriate exit codes, so that I can script and automate my posting workflow.

#### Acceptance Criteria

1. WHEN any error occurs THEN error messages SHALL be written to stderr, not stdout
2. WHEN the application exits THEN it SHALL use exit code 0 for success, 1 for posting failure, 2 for authentication error, 3 for invalid input
3. WHEN verbose logging is enabled via `--verbose` flag THEN the system SHALL output detailed operation logs to stderr
4. WHEN an error occurs THEN the error message SHALL include context about what operation failed
5. WHEN multiple platforms are involved THEN errors SHALL clearly indicate which platform failed

### Requirement 7: Agent-Friendly Interface

**User Story:** As an AI agent or automation script, I want predictable interfaces and machine-readable output, so that I can reliably interact with Plurcast tools.

#### Acceptance Criteria

1. WHEN `--help` is provided THEN the tool SHALL output comprehensive usage information
2. WHEN `--format json` is provided THEN output SHALL be valid JSON
3. WHEN called from a pipe THEN the tool SHALL detect non-TTY and output plain text without colors or progress indicators
4. WHEN called from an interactive terminal THEN the tool MAY output colors and progress indicators
5. WHEN successful THEN stdout SHALL contain only the requested output (post IDs, query results, etc.)
6. WHEN processing THEN all diagnostic messages SHALL go to stderr, keeping stdout clean for piping

### Requirement 8: Unix Philosophy Compliance

**User Story:** As a Unix user, I want Plurcast tools to follow Unix conventions, so that they compose naturally with other command-line tools.

#### Acceptance Criteria

1. WHEN no input is provided and stdin is not a TTY THEN the tool SHALL read from stdin
2. WHEN the tool succeeds silently THEN it SHALL output only essential information
3. WHEN combining tools THEN output from one tool SHALL be usable as input to another via pipes
4. WHEN the tool completes THEN it SHALL exit immediately without unnecessary delays
5. WHEN environment variables are used THEN they SHALL follow the pattern `PLURCAST_*`
