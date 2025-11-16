# Undocumented Features in Plurcast Codebase

## Summary
This report identifies implemented features that are either not documented in README.md/CLAUDE.md or are mentioned only as "experimental" without proper usage documentation.

---

## 1. PLUR-POST: --schedule Flag

**Feature Name:** Post Scheduling

**Status:** Implemented and Functional

**Where It's Implemented:**
- `/home/user/plurcast/plur-post/src/main.rs` - Lines 133-138 (CLI flag definition)
- Lines 202-223 (Schedule parsing and timestamp handling)
- Lines 279-282 (Scheduled post output handling)

**Current Functionality:**
- Accepts schedule time in multiple formats:
  - Duration format: "30m", "2h", "1d"
  - Natural language: "tomorrow"
  - Random range: "random:10m-20m"
- Cannot be used together with `--draft` flag
- Parses schedule time using `libplurcast::scheduling::parse_schedule()`
- Returns output in format: `scheduled:<post_id>:for:<timestamp>`
- When scheduled, post is stored with `scheduled_at` timestamp

**CLI Definition:**
```rust
#[arg(short, long, value_name = "TIME")]
#[arg(
    help = "Schedule post for later. Supports duration (\"30m\", \"2h\", \"1d\"), natural language (\"tomorrow\"), or random (\"random:10m-20m\")"
)]
schedule: Option<String>,
```

**Documentation Status:**
- NOT mentioned in README.md
- NOT mentioned in CLAUDE.md (quick start section)
- Only exists in code help text

**Example Usage (Not in Docs):**
```bash
plur-post "Hello" --schedule "30m"
plur-post "Later post" --schedule "tomorrow"
plur-post "Random timing" --schedule "random:1h-2h"
```

---

## 2. PLUR-QUEUE: Complete Tool (Not in README)

**Feature Name:** Scheduled Post Queue Management

**Status:** Implemented and Fully Featured

**Where It's Implemented:**
- `/home/user/plurcast/plur-queue/src/main.rs` (entire file)

**Current Functionality:**

### Main Commands:
1. **list** - List scheduled posts
   - `--format` (text|json)
   - `--platform` (filter by platform)

2. **cancel** - Cancel scheduled post(s)
   - `post_id` (required unless `--all`)
   - `--all` (cancel all scheduled posts)
   - `--force` (skip confirmation)

3. **reschedule** - Change post schedule time
   - `post_id` (required)
   - `time` (new schedule time, supports relative like "+1h" or "-30m")

4. **now** - Post immediately
   - `post_id` (required)
   - Sets scheduled_at to None and status to Pending

5. **stats** - Queue statistics
   - `--format` (text|json)
   - Shows: total posts, by platform, by time bucket (next hour, today, this week, later)
   - Lists next 5 upcoming posts

6. **failed** - Failed post management (Subcommand)
   - `list` - List failed posts (`--format` text|json)
   - `clear` - Delete all failed posts (`--force` flag)
   - `delete` - Delete specific failed post (`--force` flag)

**Documentation Status:**
- Mentioned only as "experimental" in README.md line 36
- NO command reference, examples, or usage guide
- NO section in README.md dedicated to plur-queue
- NO mention in CLAUDE.md

**Example Usage (Not in Docs):**
```bash
plur-queue list
plur-queue list --format json --platform nostr
plur-queue cancel <post_id> --force
plur-queue reschedule <post_id> "+2h"
plur-queue now <post_id>
plur-queue stats
plur-queue stats --format json
plur-queue failed list
plur-queue failed clear --force
plur-queue failed delete <post_id>
```

---

## 3. PLUR-SEND: Background Daemon (Not in README)

**Feature Name:** Scheduled Post Daemon

**Status:** Implemented and Fully Featured

**Where It's Implemented:**
- `/home/user/plurcast/plur-send/src/main.rs` (entire file)

**Current Functionality:**
- Long-running daemon that polls database for scheduled posts
- Processes due posts at scheduled times
- Automatic retry logic for failed posts with configurable:
  - `max_retries` (default: 3)
  - `retry_delay` (default: 300s)
  - `inter_retry_delay` (default: 5s between retries)
  - `max_retries_per_iteration` (default: 10)
  - `startup_delay` (delay before first retry processing)
- Rate limiting per platform (from config)
- Graceful shutdown on SIGTERM/SIGINT

**CLI Flags:**
- `--poll-interval` (seconds between checks, default from config or 60)
- `--verbose` (debug logging)
- `--once` (hidden flag: process due posts once and exit, for testing)
- `--startup-delay` (seconds, overrides config)
- `--no-retry` (disable automatic retry processing)

**Configuration (in config.toml):**
```toml
[scheduling]
poll_interval = 60
max_retries = 3
retry_delay = 300
startup_delay = 0
inter_retry_delay = 5
max_retries_per_iteration = 10

[scheduling.rate_limits]
nostr = { posts_per_hour = 100 }
mastodon = { posts_per_hour = 300 }
```

**Documentation Status:**
- Mentioned only as "experimental" in README.md line 36
- NO usage guide or examples
- NO configuration documentation
- NO mention in CLAUDE.md
- Only internal long_about help text documents features

**Example Usage (Not in Docs):**
```bash
# Run daemon with default settings
plur-send

# Run with custom poll interval
plur-send --poll-interval 30

# Run verbose for debugging
plur-send --verbose

# Process once and exit (testing)
plur-send --once

# Disable retries, only process scheduled posts
plur-send --no-retry

# With startup delay before retries
plur-send --startup-delay 30
```

---

## 4. PLUR-IMPORT: Data Import Tool (Not in README)

**Feature Name:** Import Posts from Platform Exports

**Status:** Implemented but Limited

**Where It's Implemented:**
- `/home/user/plurcast/plur-import/src/main.rs`
- Module: `/home/user/plurcast/plur-import/src/ssb.rs`

**Current Functionality:**
- Only supports SSB feed import
- Imports existing posts from SSB feed into Plurcast database
- Preserves timestamps and metadata
- Supports `--account` flag for multi-account setup

**CLI:**
```
plur-import ssb [--account <NAME>] [--verbose]
```

**Documentation Status:**
- NOT mentioned in README.md
- NOT mentioned in CLAUDE.md
- In roadmap as `[ ] plur-import (data import)` (unchecked)
- No usage examples or guide

**Example Usage (Not in Docs):**
```bash
plur-import ssb
plur-import ssb --account custom-account
plur-import ssb --verbose
```

---

## 5. PLUR-EXPORT: Data Export Tool (Not in README)

**Feature Name:** Export Posts to Various Formats

**Status:** Implemented but Limited

**Where It's Implemented:**
- `/home/user/plurcast/plur-export/src/main.rs`
- Module: `/home/user/plurcast/plur-export/src/ssb.rs`

**Current Functionality:**
- Only supports SSB message format export (JSON lines)
- Exports Plurcast posts in platform-specific formats
- Useful for backup, migration, or tool integration

**CLI Flags:**
- `--format` (currently only "ssb" supported)
- `--output` (optional output file, default: stdout)
- `--verbose` (debug logging)

**Documentation Status:**
- NOT mentioned in README.md
- NOT mentioned in CLAUDE.md
- In roadmap as `[ ] plur-export (data export)` (unchecked)
- No usage examples or guide

**Example Usage (Not in Docs):**
```bash
plur-export --format ssb
plur-export --format ssb --output export.jsonl
plur-export --format ssb --verbose > backup.jsonl
```

---

## 6. PLUR-CREDS: Commands Not Documented in README

**Feature Name:** Credential Management - Advanced Commands

**Status:** Fully Implemented

**Where It's Implemented:**
- `/home/user/plurcast/plur-creds/src/main.rs` - Lines 93-97

**Documented Commands (in README):**
- ✅ `set` - Store credentials
- ✅ `list` - List stored credentials  
- ✅ `delete` - Delete credentials
- ✅ `test` - Test credentials
- ✅ `use` - Set active account

**Undocumented Commands:**

### a) migrate - Migrate credentials to multi-account format
- Usage: `plur-creds migrate`
- Migrates old single-account format to new multi-account format
- Mentioned in README only in context of plain text migration

### b) audit - Audit credential security
- Usage: `plur-creds audit`
- Checks:
  - Credential storage backend configuration
  - Plain text files and their permissions (Unix)
  - Security issues and recommendations
- Exit code: 1 if issues found, 0 if clean
- NOT mentioned in README anywhere

**Documentation Status:**
- `migrate` - Mentioned briefly in README for moving from plain text, but no dedicated section
- `audit` - COMPLETELY undocumented in README
- Both fully functional with help text in code

**Example Usage (Not Documented):**
```bash
plur-creds migrate    # Migrate to multi-account format
plur-creds audit      # Check credential security
```

---

## 7. PLUR-SETUP: Undocumented Flag

**Feature Name:** Interactive Setup - Non-Interactive Mode

**Status:** Implemented

**Where It's Implemented:**
- `/home/user/plurcast/plur-setup/src/main.rs` - Lines 14-16

**Current Functionality:**
- `--non-interactive` flag skips interactive prompts
- Uses default values where possible
- Still requires manual platform setup afterward

**Documentation Status:**
- NOT mentioned in README.md
- Implemented but not documented

**Example Usage (Not in Docs):**
```bash
plur-setup --non-interactive
```

---

## 8. PLUR-POST: --account Flag

**Feature Name:** Multi-Account Support in Posting

**Status:** Implemented

**Where It's Implemented:**
- `/home/user/plurcast/plur-post/src/main.rs` - Lines 114-119

**Current Functionality:**
- Allows specifying which account to use for posting
- If not specified, uses active account for each platform
- Works with multi-account setup

**Documentation Status:**
- Mentioned in code help text
- NOT clearly documented in README.md
- Only mentioned as footnote in CLAUDE.md in passing

**Example Usage (Not Well Documented):**
```bash
plur-post "Hello" --account work-account
plur-post "Personal" --account personal-nostr
```

---

## 9. PLUR-POST: --nostr-pow Flag

**Feature Name:** Proof of Work (NIP-13)

**Status:** Implemented and Partially Documented

**Where It's Implemented:**
- `/home/user/plurcast/plur-post/src/main.rs` - Lines 121-126
- Passed to Nostr platform: Line 263
- Extracted in plur-send: Lines 337-347

**Current Functionality:**
- Adds computational difficulty to Nostr posts
- Helps combat spam on relays with spam filters
- Accepts difficulty level (0-64)
- Recommended: 20-25 (takes 1-5 seconds)
- Only applies to Nostr platform

**Documentation Status:**
- PARTIALLY documented in README.md (lines 249-261)
- Has dedicated section "Proof of Work (--nostr-pow)"
- Examples provided
- However, not documented in CLAUDE.md quick reference

**Documentation Quality:**
- README has section but could be more detailed about difficulty recommendations
- Missing: best practices, relay-specific spam filters

---

## 10. PLUR-POST: Scheduled Post Output Format

**Feature Name:** Output Format for Scheduled Posts

**Status:** Implemented

**Where It's Implemented:**
- `/home/user/plurcast/plur-post/src/main.rs` - Lines 464-478

**Current Functionality:**
- Text format: `scheduled:<post_id>:for:<timestamp>`
- JSON format: `{"scheduled": true, "post_id": "...", "scheduled_at": <timestamp>}`
- Different from draft output format

**Documentation Status:**
- NOT documented in README.md
- NOT documented in CLAUDE.md
- Only in code

---

## Summary Statistics

### By Category:

**Completely Undocumented Tools (0 mentions in docs):**
1. plur-queue - Entire tool (6+ commands)
2. plur-send - Entire tool (daemon, configuration, flags)
3. plur-import - Entire tool
4. plur-export - Entire tool
5. plur-creds audit subcommand

**Partially Documented:**
1. plur-post --schedule flag (implemented, zero docs)
2. plur-post --account flag (vague documentation)
3. plur-post --nostr-pow flag (has README section, could be better)
4. plur-setup --non-interactive flag
5. plur-creds migrate (briefly mentioned)

**Total Undocumented Features:** 20+

### Documentation Gaps:
- Scheduling infrastructure is fully implemented but users would never know to use --schedule
- Queue management is feature-complete but undocumented
- Export/import functionality exists but no users would discover it
- Advanced credential management commands exist but are hidden

