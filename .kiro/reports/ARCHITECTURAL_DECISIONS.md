# Architectural Decisions for Test Implementation
**Date**: 2025-10-26  
**Context**: Test Health Assessment - Phase 3.2 TUI Foundation

## Security & Credentials

### 1. Credential Storage Precedence
**Decision**: Keyring (OS-level) is authoritative
- KeyringStore has highest priority
- EncryptedFileStore is fallback
- PlainFileStore is deprecated (migration only)

**Rationale**: OS-level security is highest priority

### 2. Credential Migration
**Decision**: Explicit migration only via `plur-creds migrate`
- No automatic migration on read
- User must be made aware of migration need
- After successful migration to Keyring, legacy encrypted file should be deleted
- If deletion fails, warn user but complete migration

**Test Implications**:
- Test explicit migration flow
- Test detection of legacy credentials
- Test user awareness/prompting
- Test cleanup behavior

### 3. Missing Credentials
**Decision**: Prompt user to configure credentials
- Fail with specific error that TUI can surface
- TUI should display: "Configure credentials" prompt
- Error code: 2 (authentication error)

**Test Implications**:
- Assert error code 2 on missing credentials
- Mock TUI to verify error message display
- Test credential configuration flow

## Input Validation & Sanitization

### 4. Cross-Platform Length Handling
**Decision**: Warn and offer options
- Detect when content exceeds any platform's limit
- Warn user with specific platform names
- Offer options:
  - Redraft (go back to editing)
  - Post to compatible platforms only (skip failing platforms)

**Test Implications**:
- Test length validation per platform
- Test warning message generation
- Mock user choice (redraft vs. partial post)
- Test partial posting workflow

### 5. Character Sanitization
**Decision**: Minimal sanitization
- Remove unsafe control characters
- Keep content as user intended otherwise
- No automatic CRLF → LF normalization (respect user input)

**Test Implications**:
- Property-based tests for control character removal
- Test preservation of user formatting
- Test newline handling (CRLF preserved)

## Multi-Platform Posting

### 6. Partial Success Handling
**Decision**: Report partial success, no rollback
- If Platform A succeeds and Platform B fails: report both outcomes
- Do NOT attempt to rollback/delete successful posts
- Present clear per-platform status to user

**Test Implications**:
- Test concurrent posting with mixed outcomes
- Test status aggregation (per-platform)
- Assert no rollback attempts
- Verify all platforms attempted even after first failure

### 7. Retry Policy
**Decision**: 3 attempts per platform with exponential backoff
- Retry count: 3 (initial + 2 retries)
- Backoff: exponential (e.g., 1s, 2s, 4s)
- Alert user of final failure after exhausting retries

**Test Implications**:
- Test retry mechanism with deterministic timing (tokio::time::pause)
- Test exponential backoff intervals
- Test max retry count enforcement
- Test failure notification after exhaustion

### 8. Idempotency
**Decision**: No automatic deduplication
- User responsible for avoiding duplicate submissions
- System does not dedupe within time window
- Each post request is independent

**Test Implications**:
- No deduplication tests needed
- Focus on per-request correctness

## TUI Error Handling

### 9. Error Display
**Decision**: Per-platform status lines
- Show individual status for each platform:
  - ✓ Nostr: Posted successfully
  - ✗ Mastodon: Authentication failed
  - ✓ Bluesky: Posted successfully
- Clear visual distinction between success/failure

**Test Implications**:
- Test status line generation from results
- Test rendering of mixed outcomes
- Verify emoji/icons for status

### 10. Retry Failed Platforms
**Decision**: Offer "retry failed platforms only" action
- After partial failure, user can retry just the failed platforms
- Original content preserved
- Only failed platforms re-attempted

**Test Implications**:
- Test failed platform filtering
- Test retry with subset of platforms
- Test content preservation across retries
- Test that successful platforms not re-posted

## Token Refresh
**Decision**: Out of scope for current test phase
- Library-specific requirements unknown
- Skip refresh simulation in tests
- Document as future work if needed

## Summary for Test Implementation

### P0 Security Tests Focus:
- Keyring precedence over file stores
- Explicit migration with user awareness
- Missing credentials → error code 2 + prompt

### P1 Posting Tests Focus:
- Length validation with user options (redraft/partial post)
- Partial success reporting (no rollback)
- 3 retries with exponential backoff
- Per-platform status display
- Retry failed platforms only

### Test Approach:
- Mock all external dependencies (OS keyring, network, platform APIs)
- Use deterministic time control (tokio::time::pause)
- Property-based tests for input validation
- Headless TUI simulation for user flows

---

**Document Owner**: Test Health Assessment  
**Status**: Approved for implementation  
**Next Action**: Begin Task #2 (Git management and branch setup)
