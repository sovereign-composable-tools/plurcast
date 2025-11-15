# Phase 5: Post Scheduling - Task Breakdown

## Overview

Implementation tasks for adding Unix-style post scheduling to Plurcast.

**Total Estimated Time**: 2-3 weeks

## Phase 5.1: Database and Core Logic (2-3 days)

- [x] 1. Database schema review
  - Database already has `scheduled_at` field ✅
  - Post `status` field already exists ✅
  - No breaking changes needed ✅

- [x] 2. Create scheduling database migration ✅
  - Created `002_scheduling_enhancements.sql` ✅
  - Added index on `(scheduled_at, status)` for efficient queries ✅
  - Created `rate_limits` table for tracking post frequency ✅
  - Tested with in-memory SQLite (test_migration_creates_scheduling_indexes) ✅
  - Tested with in-memory SQLite (test_migration_creates_rate_limits_table) ✅

- [x] 3. Enhance Database module ✅
  - Added `get_scheduled_posts_due() -> Vec<Post>` ✅
  - Added `get_scheduled_posts() -> Vec<Post>` ✅
  - Added `update_post_schedule(id, scheduled_at)` ✅
  - Added `delete_post(id)` ✅
  - Added `get_rate_limit_count(platform, window_start) -> usize` ✅
  - Added `increment_rate_limit(platform, window_start)` ✅
  - Added `cleanup_rate_limits(before_timestamp)` ✅
  - Added `get_last_scheduled_timestamp() -> Option<i64>` (for random scheduling) ✅
  - Added 17 unit tests for all methods ✅

- [x] 4. Enhance HistoryService ✅
  - Added `get_scheduled_posts() -> Vec<Post>` (for plur-queue) ✅
  - Added `get_scheduled_posts_due() -> Vec<Post>` (for plur-send) ✅
  - Added 6 integration tests ✅
  - Updated matches_status() to include PostStatus::Scheduled ✅

## Phase 5.2: plur-post Enhancement (3-4 days)

- [x] 5. Add scheduling dependencies ✅
  - Added `chrono-english` for natural language parsing ✅
  - Added `humantime` for duration parsing ✅
  - Added `rand` for random interval generation ✅
  - Updated workspace and libplurcast Cargo.toml ✅

- [x] 6. Implement time parsing ✅
  - Created `libplurcast/src/scheduling.rs` module ✅
  - Implemented `parse_schedule(input, last_scheduled)` ✅
  - Supported formats:
    - Duration: "30m", "2h", "1d" ✅
    - Natural language: "tomorrow" ✅
    - Random: "random:10m-20m", "random:1h-2h" ✅
  - Added 18 unit tests (all passing) ✅
  - Random scheduling logic:
    - Parse "random:MIN-MAX" syntax ✅
    - Query last_scheduled for chaining ✅
    - Generate random offset between MIN and MAX ✅
    - Validation: 30s minimum, 30d maximum ✅

- [x] 7. Add --schedule flag to plur-post ✅
  - Added `schedule: Option<String>` to Cli struct ✅
  - Parse schedule time in run() using scheduling module ✅
  - Pass `scheduled_at` to PostRequest ✅
  - Validate --schedule and --draft are mutually exclusive ✅
  - Updated help text with examples ✅
  - Added output_schedule_result() for formatted output ✅

- [x] 8. Update PostingService for scheduling ✅
  - Added `scheduled_at: Option<i64>` to PostRequest ✅
  - When scheduled_at is set, create post with status=Scheduled ✅
  - Save to database without posting ✅
  - Return success response ✅
  - Updated draft.rs to include scheduled_at field ✅

- [x] 9. Update plur-post tests ✅
  - Created `plur-post/tests/scheduling_integration.rs` ✅
  - 16 integration tests (all passing):
    - Duration formats: 3 tests ✅
    - Natural language: 1 test ✅
    - Random scheduling: 2 tests ✅
    - Error handling: 4 tests ✅
    - Output formats: 2 tests ✅
    - Compatibility: 4 tests ✅

## Phase 5.3: plur-queue CLI (4-5 days)

- [x] 10. Create plur-queue project structure ✅
  - Created `plur-queue/` directory ✅
  - Created Cargo.toml with dependencies (clap, tokio, serde_json, chrono, uuid, humantime) ✅
  - Created src/main.rs with CLI skeleton ✅
  - Added to workspace Cargo.toml ✅

- [x] 11. Implement `plur-queue list` ✅
  - Query scheduled posts from database ✅
  - Format output (text and JSON) ✅
  - Show: ID, content preview, platforms, scheduled time, time until ✅
  - Filter by platform (--platform flag) ✅
  - Sort by scheduled_at ✅
  - 10 integration tests (all passing) ✅

- [x] 12. Implement `plur-queue cancel` ✅
  - Delete post from database by ID ✅
  - Support `--all` flag to cancel all scheduled posts ✅
  - Confirmation prompt (skip with --force) ✅
  - Output success message ✅
  - 9 integration tests (all passing) ✅

- [x] 13. Implement `plur-queue reschedule` ✅
  - Parse new schedule time (durations, natural language) ✅
  - Update `scheduled_at` in database ✅
  - Support relative adjustments: "+1h", "-30m" ✅
  - Output confirmation ✅
  - 9 integration tests (all passing) ✅

- [x] 14. Implement `plur-queue now` ✅
  - Change post status from 'scheduled' to 'pending' ✅
  - Clear `scheduled_at` timestamp ✅
  - Trigger immediate posting (or queue for next daemon run) ✅
  - Output confirmation ✅
  - 6 integration tests (all passing) ✅

- [x] 15. Implement `plur-queue stats` ✅
  - Count scheduled posts total ✅
  - Count by platform ✅
  - Show next 5 upcoming posts ✅
  - Show posts by time bucket (next hour, today, this week, later) ✅
  - Support text and JSON formats ✅
  - 9 integration tests (all passing) ✅

- [x] 16. Add plur-queue tests ✅
  - Integration tests for each command (43 total) ✅
  - Integration tests with test database ✅
  - Test JSON output format ✅
  - Test error cases ✅
  - Test coverage:
    - list: 10 tests ✅
    - cancel: 9 tests ✅
    - reschedule: 9 tests ✅
    - now: 6 tests ✅
    - stats: 9 tests ✅

- [x] 17. Add plur-queue documentation ✅
  - Comprehensive --help text with long_about ✅
  - Usage examples for all commands ✅
  - Common workflows (list, cancel, reschedule, now, stats) ✅
  - Configuration and exit codes documented ✅

## Phase 5.4: plur-send Daemon (5-7 days)

- [x] 18. Create plur-send project structure ✅
  - Created `plur-send/` directory ✅
  - Created Cargo.toml with tokio, tracing, signal-hook dependencies ✅
  - Created src/main.rs skeleton with daemon loop, signal handling ✅
  - Added to workspace Cargo.toml ✅
  - CLI: --poll-interval, --verbose, --once flags ✅
  - Graceful shutdown via SIGTERM/SIGINT ✅
  - Structured logging with tracing ✅

- [ ] 19. Implement configuration loading
  - Load config from `~/.config/plurcast/config.toml`
  - Parse `[scheduling]` section:
    - `poll_interval` (default: 60s)
    - `max_retries` (default: 3)
    - `retry_delay` (default: 300s)
  - Parse `[scheduling.rate_limits]` section
  - Validation and defaults

- [ ] 20. Implement rate limiting module
  - Create `RateLimiter` struct
  - Implement `check_and_record()` method
  - Use `rate_limits` database table
  - Handle window sliding
  - Unit tests for rate limiter

- [ ] 21. Implement daemon main loop
  - Initialize PlurcastService
  - Poll database every `poll_interval` seconds
  - Query scheduled posts where `scheduled_at <= now`
  - Process each due post
  - Handle errors gracefully
  - Log to stderr/journald

- [ ] 22. Implement post processing
  - Get platforms for each post
  - Check rate limits before posting
  - Create PostRequest from scheduled post
  - Call `service.posting().post(request)`
  - Update post status based on result
  - Record errors in database

- [ ] 23. Implement retry logic
  - Retry failed posts up to `max_retries`
  - Exponential backoff between retries
  - Track retry count in post metadata
  - Log retry attempts

- [ ] 24. Implement graceful shutdown
  - Handle SIGTERM and SIGINT signals
  - Finish current post before exiting
  - Don't start new posts during shutdown
  - Clean shutdown within 30 seconds
  - Log shutdown events

- [ ] 25. Implement logging
  - Structured logging with tracing
  - Log levels: info, warn, error
  - Log post scheduling events
  - Log rate limit hits
  - Log errors with context
  - Don't log credentials

- [ ] 26. Add plur-send tests
  - Unit tests for rate limiter
  - Unit tests for retry logic
  - Integration test: schedule → daemon picks up → posts
  - Integration test: rate limiting prevents over-posting
  - Integration test: graceful shutdown
  - Mock time for testing

## Phase 5.5: Systemd Integration (2-3 days)

- [ ] 27. Create systemd service file
  - Create `plur-send.service` template
  - User service (not system service)
  - Auto-restart on failure
  - Environment variables
  - Security hardening

- [ ] 28. Create installation script
  - Copy service file to `~/.config/systemd/user/`
  - Enable service: `systemctl --user enable plur-send`
  - Start service: `systemctl --user start plur-send`
  - Check status: `systemctl --user status plur-send`
  - Handle errors gracefully

- [ ] 29. Add daemon management commands
  - `plur-send install` - Install systemd service
  - `plur-send start` - Start daemon
  - `plur-send stop` - Stop daemon
  - `plur-send restart` - Restart daemon
  - `plur-send status` - Show status
  - `plur-send logs` - Show recent logs

- [ ] 30. Add health check (optional)
  - Daemon writes heartbeat to file
  - `plur-send health` checks heartbeat timestamp
  - Systemd watchdog integration
  - Metrics endpoint (optional)

- [ ] 31. Documentation
  - Installation guide
  - Configuration reference
  - Troubleshooting guide
  - Systemd integration guide
  - Update main README

## Phase 5.6: Testing and Polish (3-4 days)

- [ ] 32. End-to-end testing
  - Schedule post with plur-post
  - Verify in queue with plur-queue
  - Start plur-send daemon
  - Wait for scheduled time
  - Verify post was sent
  - Check post_records in database

- [ ] 33. Error scenario testing
  - Schedule post with invalid credentials
  - Verify daemon handles error gracefully
  - Schedule post that exceeds rate limit
  - Verify post is delayed appropriately
  - Kill daemon mid-post
  - Verify post continues on restart

- [ ] 34. Performance testing
  - Schedule 100 posts for same time
  - Verify daemon handles burst
  - Verify rate limiting works
  - Check memory usage
  - Check CPU usage

- [ ] 35. Documentation review
  - User guide complete
  - Developer guide complete
  - API documentation
  - Configuration examples
  - Troubleshooting section

- [ ] 36. Code review and cleanup
  - Remove debug prints
  - Add missing error messages
  - Ensure consistent code style
  - Add inline documentation
  - Run clippy and fix warnings

## Success Criteria

All tasks must be complete and passing:

- [ ] Can schedule posts with `plur-post --schedule`
- [ ] Can schedule posts with random intervals (`random:10m-20m`)
- [ ] Randomized queue builds correctly (each post after previous)
- [ ] Can list scheduled posts with `plur-queue list`
- [ ] Can cancel scheduled posts with `plur-queue cancel`
- [ ] Daemon posts at scheduled time (±1 minute accuracy)
- [ ] Rate limiting prevents over-posting
- [ ] Daemon runs as systemd service
- [ ] Graceful shutdown works correctly
- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] All end-to-end tests pass
- [ ] Documentation is complete
- [ ] Code review approved

## Dependencies

**Crates to Add:**
- `chrono-english` or `timeparse` - Natural language date parsing
- `signal-hook` - Unix signal handling for daemon
- `signal-hook-tokio` - Tokio integration for signals

**No Breaking Changes:**
- All existing functionality continues to work
- Database migration is additive only
- Configuration is optional (defaults provided)

## Timeline

| Phase | Duration | Tasks |
|-------|----------|-------|
| 5.1 - Database | 2-3 days | 1-4 |
| 5.2 - plur-post | 3-4 days | 5-9 |
| 5.3 - plur-queue | 4-5 days | 10-17 |
| 5.4 - plur-send | 5-7 days | 18-26 |
| 5.5 - Systemd | 2-3 days | 27-31 |
| 5.6 - Testing | 3-4 days | 32-36 |
| **Total** | **19-26 days** | **36 tasks** |

---

**Version**: 1.0
**Status**: Ready for Implementation
**Next Task**: Task 2 - Create scheduling database migration
