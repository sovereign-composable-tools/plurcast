# Requirements Document: Service Layer Extraction

## Introduction

This specification defines the extraction of business logic from CLI binaries into a dedicated service layer within `libplurcast`. The service layer will provide a clean, testable API that can be consumed by multiple interfaces (CLI, TUI, GUI) without code duplication. This is Phase 3.1 of the Plurcast roadmap.

**Key Principle**: Zero behavioral changes to existing CLI tools. The service layer is a refactoring to enable future UI development, not a feature addition.

## Requirements

### Requirement 1: Service Layer Architecture

**User Story**: As a developer, I want a clean service layer architecture so that I can build multiple UIs (CLI, TUI, GUI) without duplicating business logic.

#### Acceptance Criteria

1. WHEN the service layer is implemented THEN it SHALL be located in `libplurcast/src/service/`
2. WHEN the service layer is implemented THEN it SHALL provide a `PlurcastService` facade that coordinates all sub-services
3. WHEN the service layer is implemented THEN it SHALL use `Arc<T>` for shared state (Database, Config) to enable concurrent access
4. WHEN the service layer is implemented THEN all service methods SHALL be async functions
5. WHEN the service layer is implemented THEN all services SHALL implement `Send + Sync` for thread safety
6. WHEN the service layer is implemented THEN services SHALL NOT depend on CLI-specific code (clap, stdout formatting, etc.)

### Requirement 2: PostingService

**User Story**: As a developer, I want a PostingService that handles all posting logic so that any UI can post to platforms without reimplementing the logic.

#### Acceptance Criteria

1. WHEN PostingService is implemented THEN it SHALL provide a `post()` method that accepts content and platform list
2. WHEN PostingService is implemented THEN it SHALL provide a `post_draft()` method that saves without publishing
3. WHEN PostingService is implemented THEN it SHALL validate content against platform limits before posting
4. WHEN PostingService is implemented THEN it SHALL post to multiple platforms concurrently
5. WHEN PostingService is implemented THEN it SHALL return structured results (success/failure per platform)
6. WHEN PostingService is implemented THEN it SHALL record all posts in the database
7. WHEN PostingService is implemented THEN it SHALL emit progress events via EventBus
8. WHEN posting fails on a platform THEN PostingService SHALL retry transient errors with exponential backoff
9. WHEN posting succeeds on any platform THEN PostingService SHALL mark the post as "Posted" even if other platforms fail

### Requirement 3: HistoryService

**User Story**: As a developer, I want a HistoryService that handles all history queries so that any UI can display post history without reimplementing query logic.

#### Acceptance Criteria

1. WHEN HistoryService is implemented THEN it SHALL provide a `list_posts()` method with filtering options
2. WHEN HistoryService is implemented THEN it SHALL support filtering by platform, date range, status, and content search
3. WHEN HistoryService is implemented THEN it SHALL support pagination with limit and offset
4. WHEN HistoryService is implemented THEN it SHALL return posts with their platform records (success/failure per platform)
5. WHEN HistoryService is implemented THEN it SHALL provide a `get_post()` method to retrieve a single post by ID
6. WHEN HistoryService is implemented THEN it SHALL provide statistics methods (total posts, success rate per platform, etc.)

### Requirement 4: DraftService

**User Story**: As a developer, I want a DraftService that manages draft posts so that any UI can create, edit, and publish drafts.

#### Acceptance Criteria

1. WHEN DraftService is implemented THEN it SHALL provide a `create_draft()` method
2. WHEN DraftService is implemented THEN it SHALL provide an `update_draft()` method
3. WHEN DraftService is implemented THEN it SHALL provide a `delete_draft()` method
4. WHEN DraftService is implemented THEN it SHALL provide a `list_drafts()` method
5. WHEN DraftService is implemented THEN it SHALL provide a `publish_draft()` method that delegates to PostingService
6. WHEN a draft is published THEN DraftService SHALL update its status to "Posted"

### Requirement 5: ValidationService

**User Story**: As a developer, I want a ValidationService that validates content in real-time so that UIs can provide immediate feedback before posting.

#### Acceptance Criteria

1. WHEN ValidationService is implemented THEN it SHALL provide a `validate_content()` method
2. WHEN ValidationService is implemented THEN it SHALL check content against platform character limits
3. WHEN ValidationService is implemented THEN it SHALL check for empty/whitespace-only content
4. WHEN ValidationService is implemented THEN it SHALL check content size against MAX_CONTENT_LENGTH (100KB)
5. WHEN ValidationService is implemented THEN it SHALL return detailed validation results per platform
6. WHEN content is valid for all platforms THEN ValidationService SHALL return success
7. WHEN content is invalid for any platform THEN ValidationService SHALL return specific error messages

### Requirement 6: EventBus

**User Story**: As a developer, I want an EventBus for progress events so that UIs can display real-time feedback during long operations.

#### Acceptance Criteria

1. WHEN EventBus is implemented THEN it SHALL use Rust channels (tokio::sync::broadcast) for event distribution
2. WHEN EventBus is implemented THEN it SHALL support multiple subscribers
3. WHEN EventBus is implemented THEN it SHALL provide event types for: PostingStarted, PostingProgress, PostingCompleted, PostingFailed
4. WHEN EventBus is implemented THEN events SHALL include relevant context (post_id, platform, message)
5. WHEN a service performs an operation THEN it SHALL emit appropriate events
6. WHEN no subscribers exist THEN EventBus SHALL not block or fail

### Requirement 7: PlurcastService Facade

**User Story**: As a developer, I want a single PlurcastService entry point so that I can easily access all services without managing multiple instances.

#### Acceptance Criteria

1. WHEN PlurcastService is implemented THEN it SHALL provide access to all sub-services
2. WHEN PlurcastService is implemented THEN it SHALL initialize shared resources (Database, Config) once
3. WHEN PlurcastService is implemented THEN it SHALL provide a `new()` constructor that loads config and initializes database
4. WHEN PlurcastService is implemented THEN it SHALL provide a `from_config()` constructor for custom configurations
5. WHEN PlurcastService is created THEN all sub-services SHALL share the same Database and Config instances

### Requirement 8: CLI Refactoring

**User Story**: As a user, I want the CLI tools to continue working exactly as before so that the refactoring doesn't break my workflows.

#### Acceptance Criteria

1. WHEN CLI tools are refactored THEN they SHALL use PlurcastService instead of direct database/platform calls
2. WHEN CLI tools are refactored THEN all existing functionality SHALL work identically
3. WHEN CLI tools are refactored THEN all exit codes SHALL remain the same
4. WHEN CLI tools are refactored THEN all output formats SHALL remain the same
5. WHEN CLI tools are refactored THEN all command-line flags SHALL work identically
6. WHEN CLI tools are refactored THEN error messages SHALL remain helpful and actionable
7. WHEN `plur-post` is refactored THEN it SHALL map service results to appropriate exit codes
8. WHEN `plur-history` is refactored THEN it SHALL format service results for text/json/csv output

### Requirement 9: Testing

**User Story**: As a developer, I want comprehensive service layer tests so that I can confidently refactor and extend the codebase.

#### Acceptance Criteria

1. WHEN service layer is implemented THEN each service SHALL have unit tests
2. WHEN service layer is implemented THEN integration tests SHALL verify service interactions
3. WHEN service layer is implemented THEN tests SHALL use in-memory databases where possible
4. WHEN service layer is implemented THEN tests SHALL mock platform clients for posting tests
5. WHEN service layer is implemented THEN test coverage SHALL be at least 80% for service layer code

### Requirement 10: Documentation

**User Story**: As a developer, I want clear documentation of the service layer so that I can understand how to use it for building new UIs.

#### Acceptance Criteria

1. WHEN service layer is implemented THEN each service SHALL have rustdoc comments
2. WHEN service layer is implemented THEN public methods SHALL have usage examples in rustdoc
3. WHEN service layer is implemented THEN a SERVICE_LAYER.md guide SHALL be created
4. WHEN service layer is implemented THEN the guide SHALL include architecture diagrams
5. WHEN service layer is implemented THEN the guide SHALL include example usage for each service

## Non-Functional Requirements

### Performance
- Service layer operations SHALL complete within the same time bounds as current CLI implementations
- EventBus SHALL not introduce measurable latency to operations
- Shared state (Arc) SHALL not cause contention under normal usage

### Maintainability
- Service layer code SHALL follow Rust best practices
- Services SHALL have clear separation of concerns
- Error handling SHALL use the existing PlurcastError types

### Compatibility
- Service layer SHALL work on all platforms supported by Plurcast (Windows, macOS, Linux)
- Service layer SHALL be compatible with Rust 1.70+

## Success Metrics

1. All CLI tools refactored to use service layer with zero behavioral changes
2. Service layer test coverage ≥ 80%
3. All existing integration tests pass without modification
4. Service layer documentation complete with examples
5. Foundation ready for Phase 3.2 (TUI) and Phase 3.3 (GUI) development

## Out of Scope

- Multi-account support (Phase 3.4)
- TUI implementation (Phase 3.2)
- GUI implementation (Phase 3.3)
- Scheduling features (Phase 4)
- New CLI features or behavioral changes
