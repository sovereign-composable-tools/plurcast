# Requirements Document: Service Layer & Progressive UI Enhancement

## Introduction

Phase 3 of Plurcast enhances the user experience through a natural progression from command-line to rich interfaces, while extracting reusable business logic into a service layer. This phase follows a build-on-what-exists philosophy:

1. **Extract Service Layer** - Move business logic from CLI binaries to reusable library code
2. **Build Terminal UI** - Rich interactive terminal interface using Ratatui (validates service design)
3. **Build Desktop GUI** - Native application using Tauri with direct Rust integration

This progression leverages Rust's strengths at each step. Unlike traditional GUI architectures that require IPC or HTTP layers, all interfaces directly call the service layer as regular Rust functions. This keeps the codebase simple, performance high, and development velocity fast.

The design maintains Unix philosophy for CLI tools while providing progressively richer experiences for users who want them. Each interface uses the same tested service layer, ensuring consistency.

## Requirements

### Requirement 1: Service Layer Abstraction

**User Story:** As a developer building interfaces, I want a unified service layer API so that I can build CLI, TUI, and GUI without duplicating business logic.

#### Acceptance Criteria

1. WHEN calling service methods THEN the system SHALL provide high-level operations (post, save draft, get history)
2. WHEN using the service layer THEN it SHALL be framework-agnostic (no CLI, TUI, or GUI dependencies)
3. WHEN posting content THEN the service SHALL handle multi-platform orchestration, validation, and database persistence
4. WHEN errors occur THEN the service SHALL return structured error types suitable for any interface to display
5. IF the service is called from CLI, TUI, or GUI THEN it SHALL behave identically
6. WHEN the service completes operations THEN it SHALL return rich results (not exit codes)
7. WHEN integrating with interfaces THEN the service SHALL support async operations with optional progress callbacks

### Requirement 2: In-Process Event System

**User Story:** As a TUI or GUI developer, I want progress updates so that I can show users what's happening during multi-platform posts.

#### Acceptance Criteria

1. WHEN posting to platforms THEN the system SHALL emit events for each platform start, success, and failure
2. WHEN operations are in progress THEN the system SHALL provide callbacks for status updates
3. WHEN long-running operations execute THEN the system SHALL support cancellation via tokens
4. WHEN events are emitted THEN they SHALL include platform name, operation type, and timestamp
5. IF an interface subscribes to events THEN it SHALL receive all relevant updates in-process
6. WHEN errors occur THEN error events SHALL include detailed context for display
7. WHEN the service runs THEN events SHALL work within a single process (no IPC required)

### Requirement 3: CLI Refactoring

**User Story:** As an existing CLI user, I want the CLI tools to continue working exactly as before so that my workflows are not disrupted.

#### Acceptance Criteria

1. WHEN using plur-post THEN it SHALL behave identically to Phase 2
2. WHEN using plur-history THEN it SHALL behave identically to Phase 2
3. WHEN the CLI calls services THEN it SHALL map results to appropriate exit codes
4. WHEN errors occur THEN the CLI SHALL format them for terminal output
5. IF the service layer is refactored THEN all CLI integration tests SHALL pass unchanged
6. WHEN exit codes are needed THEN the CLI SHALL convert service errors to exit codes (0, 1, 2, 3)
7. WHEN output is formatted THEN the CLI SHALL continue to support text and JSON formats

### Requirement 4: Terminal UI (Ratatui) Implementation

**User Story:** As a power user, I want a rich terminal interface so that I can interact with Plurcast without leaving my terminal.

#### Acceptance Criteria

1. WHEN launching plur-tui THEN it SHALL display an interactive terminal interface
2. WHEN composing posts THEN the interface SHALL provide a text editor with real-time validation
3. WHEN selecting platforms THEN the interface SHALL show selectable list with status indicators
4. WHEN posting THEN the interface SHALL show real-time progress for each platform
5. IF errors occur THEN the interface SHALL display them with context and allow retry
6. WHEN viewing history THEN the interface SHALL provide searchable, filterable timeline
7. WHEN navigating THEN the interface SHALL support both keyboard and mouse input
8. WHEN running over SSH THEN the interface SHALL work without graphical requirements
9. WHEN managing drafts THEN the interface SHALL provide create, edit, delete, and publish actions
10. WHEN the TUI is packaged THEN it SHALL be a single binary like the CLI tools

### Requirement 5: Tauri Desktop Application

**User Story:** As a desktop user, I want a native application so that I can use Plurcast with a modern graphical interface.

#### Acceptance Criteria

1. WHEN launching the app THEN it SHALL start in under 2 seconds
2. WHEN composing posts THEN the app SHALL provide a rich text area with character counts
3. WHEN selecting platforms THEN the app SHALL show checkboxes with validation status
4. WHEN posting THEN the app SHALL show real-time progress per platform
5. IF errors occur THEN the app SHALL display user-friendly error messages
6. WHEN viewing history THEN the app SHALL provide a searchable timeline with filters
7. WHEN the app calls services THEN it SHALL use direct Rust function calls (no IPC)
8. WHEN events are emitted THEN the app SHALL receive them via Tauri's event system
9. WHEN packaged THEN the binary SHALL be under 15MB (leveraging system webview)
10. WHEN running THEN memory usage SHALL be under 150MB for typical workloads

### Requirement 6: Multi-Account Management

**User Story:** As a Plurcast user with multiple accounts per platform, I want to manage different identities so that I can post from various accounts.

#### Acceptance Criteria

1. WHEN configuring accounts THEN the system SHALL support multiple accounts per platform
2. WHEN posting THEN the user SHALL be able to select which account(s) to use
3. WHEN listing accounts THEN the system SHALL show account name, platform, and authentication status
4. WHEN switching accounts THEN the system SHALL load the correct credentials
5. IF authentication fails THEN the system SHALL indicate which specific account failed
6. WHEN adding accounts THEN the system SHALL validate credentials before saving
7. WHEN storing credentials THEN the system SHALL prefer OS keyring, fall back to encrypted files
8. WHEN accounts are managed THEN all interfaces (CLI, TUI, GUI) SHALL see the same accounts

### Requirement 7: Draft Management

**User Story:** As a Plurcast user, I want to save, edit, and publish drafts so that I can compose content over time.

#### Acceptance Criteria

1. WHEN creating drafts THEN the system SHALL save them to the database with status "draft"
2. WHEN listing drafts THEN the system SHALL return all unpublished posts
3. WHEN editing drafts THEN the system SHALL update content and metadata
4. WHEN publishing drafts THEN the system SHALL post to selected platforms
5. WHEN deleting drafts THEN the system SHALL remove from database
6. IF drafts have metadata THEN the system SHALL preserve platform selections and tags
7. WHEN drafts are managed THEN all interfaces SHALL provide draft operations

### Requirement 8: Enhanced History Service

**User Story:** As a Plurcast user, I want to browse my posting history with rich filtering so that I can review and analyze my activity.

#### Acceptance Criteria

1. WHEN querying history THEN the system SHALL support pagination
2. WHEN filtering THEN the system SHALL support platform, date range, status, and content search
3. WHEN viewing posts THEN the system SHALL include all platform results with success/failure status
4. WHEN sorting THEN the system SHALL support sort by date (ascending/descending)
5. IF a post has multiple platform records THEN they SHALL all be returned together
6. WHEN counting posts THEN the system SHALL provide total count for pagination
7. WHEN retrying failed posts THEN the system SHALL repost to only failed platforms

### Requirement 9: Real-Time Content Validation

**User Story:** As a user composing posts, I want real-time validation feedback so that I know if content will work on each platform before posting.

#### Acceptance Criteria

1. WHEN entering content THEN the system SHALL validate against all target platforms
2. WHEN content changes THEN validation SHALL update without requiring a post attempt
3. WHEN validation fails THEN the system SHALL indicate which platforms and why
4. WHEN character limits apply THEN the system SHALL show remaining characters per platform
5. IF content is valid for some platforms but not others THEN the system SHALL clearly indicate which
6. WHEN validation runs THEN it SHALL complete in under 100ms
7. WHEN validation results are cached THEN they SHALL be reused for identical content

### Requirement 10: State Management and Persistence

**User Story:** As a user, I want my state to persist across sessions so that I don't lose work.

#### Acceptance Criteria

1. WHEN composing in TUI/GUI THEN drafts SHALL auto-save periodically
2. WHEN the application closes THEN the current draft SHALL be saved
3. WHEN the application opens THEN the last draft SHALL be restored (if user wants)
4. WHEN configuration changes THEN the service layer SHALL reload without restart
5. IF the database is modified externally THEN the system SHALL detect changes
6. WHEN window state changes (TUI/GUI) THEN position and size SHALL be saved
7. WHEN errors occur THEN partial changes SHALL be rolled back

### Requirement 11: Testing Strategy

**User Story:** As a developer, I want comprehensive service tests so that all interfaces can rely on consistent behavior.

#### Acceptance Criteria

1. WHEN testing services THEN unit tests SHALL cover all methods
2. WHEN testing multi-platform operations THEN integration tests SHALL verify orchestration
3. WHEN testing events THEN tests SHALL verify all callbacks are invoked
4. WHEN testing state management THEN tests SHALL verify concurrency safety
5. IF services are mocked THEN interfaces can be tested independently
6. WHEN regression tests run THEN they SHALL cover CLI, TUI, and GUI
7. WHEN testing TUI THEN automated tests SHALL verify screen rendering

### Requirement 12: Security Considerations

**User Story:** As a security-conscious user, I want credentials handled securely so that my accounts are protected.

#### Acceptance Criteria

1. WHEN storing credentials THEN the system SHALL use OS keyring when available (Windows Credential Manager, macOS Keychain, Linux Secret Service)
2. WHEN keyring is unavailable THEN the system SHALL offer encrypted file storage with user password
3. WHEN falling back to plain files THEN the system SHALL warn users and set 600 permissions
4. WHEN displaying credentials THEN interfaces SHALL mask sensitive data
5. WHEN logging THEN the system SHALL never log credentials or tokens
6. IF credentials are in memory THEN they SHALL be cleared after use
7. WHEN config files are created THEN they SHALL have restricted permissions (600 on Unix)

### Requirement 13: Performance Requirements

**User Story:** As a user, I want the interface to be responsive so that operations feel instant.

#### Acceptance Criteria

1. WHEN opening TUI/GUI THEN it SHALL launch in under 2 seconds
2. WHEN composing posts THEN validation SHALL respond in under 100ms
3. WHEN posting THEN the UI SHALL remain responsive during network operations
4. WHEN loading history THEN the first page SHALL appear in under 500ms
5. IF operations take longer THEN progress indicators SHALL be shown
6. WHEN filtering history THEN results SHALL update in under 200ms
7. WHEN switching accounts THEN the UI SHALL update in under 300ms

### Requirement 14: Documentation and Examples

**User Story:** As a user or developer, I want comprehensive documentation so that I can use and extend Plurcast.

#### Acceptance Criteria

1. WHEN reviewing documentation THEN it SHALL include service API reference
2. WHEN building interfaces THEN examples SHALL demonstrate common patterns
3. WHEN using TUI THEN documentation SHALL include keyboard shortcuts reference
4. WHEN using GUI THEN documentation SHALL include user guide with screenshots
5. IF extending Plurcast THEN documentation SHALL explain service layer architecture
6. WHEN troubleshooting THEN documentation SHALL include common issues and solutions
7. WHEN migrating from CLI THEN documentation SHALL explain TUI/GUI advantages

### Requirement 15: Progressive Enhancement Philosophy

**User Story:** As a user, I want to choose my interface based on my needs without losing functionality.

#### Acceptance Criteria

1. WHEN using CLI THEN it SHALL provide full posting and history functionality
2. WHEN using TUI THEN it SHALL add interactive editing, live validation, and visual feedback
3. WHEN using GUI THEN it SHALL add mouse support, visual polish, and easier onboarding
4. IF a feature is added THEN it SHALL be available in all appropriate interfaces
5. WHEN switching interfaces THEN the same data and configuration SHALL be accessible
6. WHEN deploying THEN users SHALL be able to install just CLI, or CLI+TUI, or all three
7. WHEN documenting THEN the progression from CLI→TUI→GUI SHALL be clear

### Requirement 16: Backward Compatibility

**User Story:** As an existing user, I want upgrades to be seamless so that my workflows continue working.

#### Acceptance Criteria

1. WHEN upgrading from Phase 2 THEN existing configurations SHALL work unchanged
2. WHEN upgrading THEN existing database data SHALL be preserved
3. WHEN using single-account configs THEN they SHALL continue to work
4. IF new features are added THEN they SHALL be opt-in
5. WHEN running Phase 3 CLI THEN it SHALL behave identically to Phase 2
6. WHEN configuration is extended THEN old configs SHALL remain valid
7. WHEN database schema changes THEN migrations SHALL run automatically
