# Implementation Plan: Service Layer Extraction

## Overview

This implementation plan converts the service layer design into actionable coding tasks. Each task builds incrementally on previous work, following test-driven development principles where appropriate. The plan focuses exclusively on code implementation tasks.

---

## Phase 1: Core Service Infrastructure

- [x] 1. Create service layer directory structure and module system





  - Create `libplurcast/src/service/` directory
  - Create `libplurcast/src/service/mod.rs` with module declarations
  - Export service types in `libplurcast/src/lib.rs`
  - _Requirements: 1.1, 1.2_

- [x] 2. Implement EventBus for progress tracking





  - Create `libplurcast/src/service/events.rs`
  - Implement `Event` enum with variants: PostingStarted, PostingProgress, PostingCompleted, PostingFailed
  - Implement `EventBus` struct using `tokio::sync::broadcast`
  - Add `new()`, `subscribe()`, and `emit()` methods
  - Ensure non-blocking behavior when no subscribers exist
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5, 6.6_

- [x] 2.1 Write unit tests for EventBus

  - Test event emission and subscription
  - Test multiple subscribers
  - Test behavior with no subscribers
  - Test event cloning and serialization
  - _Requirements: 6.1, 6.2, 6.6_

---

## Phase 2: Validation Service

- [x] 3. Implement ValidationService





  - Create `libplurcast/src/service/validation.rs`
  - Implement `ValidationService` struct with `Arc<Config>`
  - Implement `ValidationRequest` and `ValidationResponse` types
  - Implement `PlatformValidation` type
  - Add `validate()` method with platform-specific rules
  - Add `is_valid()` convenience method
  - Add `get_limits()` method for character limits
  - Implement validation rules: empty content, MAX_CONTENT_LENGTH (100KB), platform character limits
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.7_

- [x] 3.1 Write unit tests for ValidationService


  - Test validation for each platform (Nostr, Mastodon, Bluesky)
  - Test empty content rejection
  - Test MAX_CONTENT_LENGTH enforcement
  - Test character limit enforcement per platform
  - Test multi-platform validation
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_

---

## Phase 3: History Service

- [ ] 4. Implement HistoryService
  - Create `libplurcast/src/service/history.rs`
  - Implement `HistoryService` struct with `Arc<Database>`
  - Implement `HistoryQuery` struct with filtering options
  - Implement `PostWithRecords` type (may already exist in db.rs)
  - Implement `HistoryStats` and `PlatformStats` types
  - Add `list_posts()` method using existing `Database::query_posts_with_records()`
  - Add `get_post()` method using existing `Database::get_post()` and `Database::get_post_records()`
  - Add `get_stats()` method to calculate statistics from query results
  - Add `count_posts()` method
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6_

- [ ] 4.1 Write unit tests for HistoryService
  - Test list_posts with various filters
  - Test get_post for existing and non-existing posts
  - Test get_stats calculation
  - Test count_posts
  - Test pagination with limit and offset
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_

---

## Phase 4: Posting Service

- [ ] 5. Implement PostingService
  - Create `libplurcast/src/service/posting.rs`
  - Implement `PostingService` struct with `Arc<Database>`, `Arc<Config>`, and `EventBus`
  - Implement `PostRequest` and `PostResponse` types
  - Implement `PlatformResult` type
  - Add `post()` method that:
    - Creates platform instances using existing `create_platforms()`
    - Posts to platforms concurrently (reuse logic from `MultiPlatformPoster`)
    - Emits events via EventBus
    - Records results in database
    - Implements retry logic with exponential backoff
  - Add `create_draft()` method that saves post without posting
  - Add `retry_post()` method for retrying failed posts
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 2.8, 2.9_

- [ ] 5.1 Write unit tests for PostingService
  - Test successful posting to single platform
  - Test successful posting to multiple platforms
  - Test partial failure (some platforms succeed, some fail)
  - Test retry logic with transient errors
  - Test draft creation
  - Test event emission
  - Use mock platforms from `libplurcast/src/platforms/mock.rs`
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 2.8, 2.9_

---

## Phase 5: Draft Service

- [ ] 6. Implement DraftService
  - Create `libplurcast/src/service/draft.rs`
  - Implement `DraftService` struct with `Arc<Database>` and reference to `PostingService`
  - Implement `Draft` type
  - Add `create()` method to create drafts
  - Add `update()` method to update draft content
  - Add `delete()` method to delete drafts
  - Add `list()` method to list all drafts
  - Add `get()` method to retrieve single draft
  - Add `publish()` method that delegates to PostingService
  - Note: Drafts are posts with `status = PostStatus::Draft` in the database
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6_

- [ ] 6.1 Write unit tests for DraftService
  - Test draft CRUD operations
  - Test draft publishing
  - Test draft status updates
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6_

---

## Phase 6: PlurcastService Facade

- [ ] 7. Implement PlurcastService facade
  - Update `libplurcast/src/service/mod.rs`
  - Implement `PlurcastService` struct with all sub-services
  - Add `new()` constructor that:
    - Loads config using `Config::load()`
    - Initializes database using `Database::new()`
    - Creates shared `Arc<Database>` and `Arc<Config>`
    - Initializes EventBus
    - Creates all sub-services with shared state
  - Add `from_config()` constructor for custom configurations
  - Add accessor methods: `posting()`, `history()`, `draft()`, `validation()`, `subscribe()`
  - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5_

- [ ] 7.1 Write integration tests for PlurcastService
  - Test service initialization
  - Test draft-to-publish workflow
  - Test history queries after posting
  - Test validation before posting
  - Test event subscription
  - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5_

---

## Phase 7: CLI Refactoring - plur-post

- [ ] 8. Refactor plur-post to use PlurcastService
  - Update `plur-post/src/main.rs`
  - Replace direct `create_platforms()` and `MultiPlatformPoster` usage with `PlurcastService`
  - Update `run()` function to:
    - Create `PlurcastService` instance
    - Use `ValidationService` for content validation
    - Use `PostingService` for posting
    - Use `DraftService` for draft mode
  - Ensure all existing functionality works identically
  - Ensure all exit codes remain the same (0, 1, 2, 3)
  - Ensure all output formats remain the same (text, json)
  - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5, 8.6, 8.7_

- [ ] 8.1 Verify plur-post behavior with existing tests
  - Run existing plur-post integration tests
  - Verify exit codes are unchanged
  - Verify output formats are unchanged
  - Verify error messages are helpful
  - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5, 8.6, 8.7_

---

## Phase 8: CLI Refactoring - plur-history

- [ ] 9. Refactor plur-history to use PlurcastService
  - Update `plur-history/src/main.rs`
  - Replace direct database queries with `HistoryService`
  - Update `query_history()` function to use `HistoryService::list_posts()`
  - Map `HistoryQuery` to service layer types
  - Ensure all existing functionality works identically
  - Ensure all output formats remain the same (text, json, jsonl, csv)
  - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5, 8.8_

- [ ] 9.1 Verify plur-history behavior with existing tests
  - Run existing plur-history integration tests
  - Verify output formats are unchanged
  - Verify filtering works correctly
  - Verify date parsing works correctly
  - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5, 8.8_

---

## Phase 9: Documentation

- [ ] 10. Add rustdoc documentation to service layer
  - Add module-level documentation to `service/mod.rs`
  - Add comprehensive rustdoc comments to all public types and methods
  - Add usage examples in rustdoc for each service
  - Ensure examples compile with `cargo test --doc`
  - _Requirements: 10.1, 10.2_

- [ ] 11. Create SERVICE_LAYER.md guide
  - Create `docs/SERVICE_LAYER.md`
  - Document architecture overview with diagrams
  - Provide usage examples for each service
  - Document event handling patterns
  - Document testing patterns
  - Include migration guide for future UI development
  - _Requirements: 10.3, 10.4, 10.5_

---

## Phase 10: Cleanup and Verification

- [ ] 12. Mark deprecated code and verify tests
  - Add `#[deprecated]` attributes to old `poster.rs` functions if appropriate
  - Run full test suite: `cargo test --all`
  - Run clippy: `cargo clippy --all-targets --all-features`
  - Verify all existing integration tests pass
  - Verify service layer test coverage ≥ 80%
  - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5_

---

## Notes

- **Incremental Development**: Each task builds on previous tasks - complete them in order
- **Zero Breaking Changes**: All CLI refactoring must maintain exact behavior, exit codes, and output formats
- **Shared State**: Use `Arc<Database>` and `Arc<Config>` for shared state across services
- **Event System**: EventBus is in-process, non-blocking, and optional (no subscribers = no overhead)
- **Future-Proofing**: Service layer design supports Phase 4 (scheduling) and future features without architectural changes

---

**Total Tasks**: 19 tasks (12 implementation + 7 testing)
**Estimated Complexity**: Medium-High (refactoring existing code with zero behavioral changes)
**Success Criteria**: All CLI tools work identically, service layer test coverage ≥ 80%, documentation complete
