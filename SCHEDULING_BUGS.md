# Post Scheduling Bugs - Discovered 2025-11-15

## ⚗️ STATUS UPDATE (2025-11-15)

**All critical bugs have been FIXED in commit bc4e532**

- ✅ Poll interval now respected on startup (startup_delay configuration added)
- ✅ Failed posts now visible via `plur-queue failed list` command
- ✅ Rate limiting between retries implemented (inter_retry_delay, max_retries_per_iteration)
- ✅ 97/97 comprehensive tests passing (34 for plur-send, 63 for plur-queue)
- ✅ All functionality implemented and tested

**However**: Features marked as **EXPERIMENTAL** until human verified in real-world usage.

**Testing Status**:
- ✅ Automated tests pass
- ✅ Compiles without errors
- ⚠️ Needs real network testing (live Nostr/Mastodon relays)
- ⚠️ Needs long-running daemon stability testing
- ⚠️ Needs human verification of rate limiting accuracy

**Documentation Updated**:
- README.md now marks scheduling as experimental
- See Phase 5 roadmap for testing requirements

---

## Critical Issues (FIXED - See commit bc4e532)

### 1. plur-send Ignores Poll Interval on Startup
**Severity:** Critical
**Status:** ✅ FIXED (commit bc4e532)

**Problem:**
When `plur-send` starts, it immediately processes ALL failed posts for retry before entering the polling loop. This ignores the `--poll-interval` flag and posts everything as fast as possible.

**Observed Behavior:**
```bash
plur-send --poll-interval 300
# Immediately retries ALL failed posts without waiting
# Posts as fast as network allows
# Hits rate limits
```

**Expected Behavior:**
`plur-send` should respect the poll interval even on the first iteration, or at minimum, rate-limit retry attempts to prevent flooding.

**Code Location:** `plur-send/src/main.rs:200-235` (`run_daemon_loop`)

**Fix Required:**
- Add rate limiting to retry attempts
- Stagger retry attempts (don't retry all at once)
- Consider adding a startup delay before processing retries
- Add configuration option to disable automatic retries

---

### 2. No Visibility into Failed Posts
**Severity:** High
**Status:** ✅ FIXED (commit bc4e532)

**Problem:**
Users cannot see which posts are in "failed" status via `plur-queue` or `plur-history`. The only way to discover failed posts is when `plur-send` tries to retry them.

**Impact:**
- Old test posts accumulate in "failed" status
- No way to clean them up via CLI
- Surprise posting of old content when daemon starts

**Workaround:**
Created `cleanup_failed_posts.rs` example to view and delete failed posts:
```bash
cargo run -p libplurcast --example cleanup_failed_posts -- list
cargo run -p libplurcast --example cleanup_failed_posts -- delete
```

**Fix Required:**
- Add `plur-queue list --status failed` command
- Add `plur-queue cancel --status failed` command
- Show failed posts in `plur-history` with clear status indicator
- Add `--include-failed` flag to history commands

---

### 3. No Rate Limiting Between Retries
**Severity:** High
**Status:** ✅ FIXED (commit bc4e532)

**Problem:**
When retrying failed posts, `plur-send` posts them as fast as possible with no delay between attempts. This causes:
- Relay rate limiting (`rate-limited: you are noting too much`)
- Network flooding
- Poor user experience

**Expected Behavior:**
- Respect configured rate limits from `config.toml`
- Add delay between retry attempts (e.g., min 5 seconds)
- Exponential backoff for repeated failures

**Code Location:** `plur-send/src/main.rs:362-450` (`process_retry_posts`)

---

### 4. Scheduled Time Not Visible in plur-history
**Severity:** Medium
**Status:** By design, but confusing

**Problem:**
When users schedule posts, `plur-history` doesn't show when they're scheduled to post. Only `plur-queue list` shows this information.

**Impact:**
- User confusion about what tool to use
- Inconsistent interface between drafts, scheduled, and posted content

**Recommendation:**
- Make `plur-history` show ALL posts (including scheduled) with status column
- Or make it clear in documentation that scheduled posts are managed via `plur-queue`

---

### 5. Test Posts Accumulating in Database
**Severity:** Medium
**Status:** User issue + design issue

**Problem:**
During development/testing, posts that fail to post accumulate in the database with status="failed". There's no automatic cleanup or TTL.

**Impact:**
- Database grows unbounded
- Old test posts get retried unexpectedly
- No clear way to "start fresh"

**Recommendations:**
- Add `plur-queue purge` command to delete all scheduled/failed/draft posts
- Add TTL for failed posts (auto-delete after N days)
- Add `--no-retry` mode for plur-send to skip retry processing

---

## Recommendations for v0.3.0-alpha3

1. **Mark scheduling as UNSTABLE** in README and documentation
2. **Add warnings** to plur-send and plur-queue help text
3. **Document workarounds** for current issues
4. **Create cleanup tool** (or add to plur-queue)
5. **Add rate limiting** to retry logic
6. **Fix poll interval** behavior on startup

## Documentation Updates Needed

- README.md: Change "🚧 Post scheduling (coming soon)" to "⚠️ Post scheduling (alpha - unstable)"
- Add SCHEDULING.md with known issues
- Update plur-send --help to warn about retry behavior
- Add troubleshooting section to README

## Test Cases to Add

1. Test that poll_interval is respected on first iteration
2. Test that retry attempts are rate-limited
3. Test that failed posts can be viewed and deleted
4. Test that scheduling + retry don't conflict
5. Test plur-send behavior with empty queue
6. Test plur-send behavior with mix of scheduled + failed posts

---

Created: 2025-11-15
Reporter: User testing
Status: Documented, fixes needed
