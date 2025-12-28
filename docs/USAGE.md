# Plurcast Usage Guide

Comprehensive usage documentation for all Plurcast tools.

## Table of Contents

- [Basic Posting](#basic-posting)
- [Multi-Platform Posting](#multi-platform-posting)
- [Multi-Account Management](#multi-account-management)
- [Post Scheduling](#post-scheduling)
- [Nostr-Specific Features](#nostr-specific-features)
- [Querying History](#querying-history)
- [Import and Export](#import-and-export)
- [Output Formats](#output-formats)
- [Unix Composability](#unix-composability)

---

## Basic Posting

### Post from Argument

```bash
plur-post "Your message here"
```

### Post from stdin

```bash
echo "Your message" | plur-post
cat message.txt | plur-post
```

### Draft Mode

Save without posting (useful for testing):

```bash
plur-post "Draft content" --draft
# Output: draft:550e8400-e29b-41d4-a716-446655440000
```

### Content Size Limits

Maximum content: **100KB (100,000 bytes)**

```bash
# Valid post
plur-post "Normal post"

# Oversized post - REJECTED
plur-post "$(python -c 'print("x"*100001)')"
# Error: Content too large: 100001 bytes (maximum: 100000 bytes)
# Exit code: 3
```

---

## Multi-Platform Posting

### Post to All Enabled Platforms

```bash
plur-post "Hello everyone!"
# Output:
# nostr:note1abc123...
# mastodon:12345
```

### Post to Specific Platforms

```bash
plur-post "Nostr only" --platform nostr
plur-post "Multi-platform" --platform nostr,mastodon
```

### Handle Partial Failures

```bash
plur-post "Test post" --platform nostr,mastodon
# If Mastodon fails but Nostr succeeds:
# nostr:note1abc123...
# Error: mastodon: Authentication failed
# Exit code: 1 (partial failure)
```

---

## Multi-Account Management

Manage multiple accounts per platform (test vs prod, personal vs work).

### Store Credentials for Different Accounts

```bash
plur-creds set nostr --account test
plur-creds set nostr --account prod
```

### List All Accounts

```bash
plur-creds list --platform nostr
# Output:
#   nostr (default): Private Key (stored in keyring) [active]
#   nostr (test): Private Key (stored in keyring)
#   nostr (prod): Private Key (stored in keyring)
```

### Switch Active Account

```bash
plur-creds use nostr --account prod
```

### Post with Specific Account

```bash
# Uses active account
plur-post "Hello from prod"

# Override with explicit account
plur-post "Test message" --account test
```

### Account Naming Rules

- Alphanumeric, hyphens, underscores
- Maximum 64 characters
- Case-sensitive

Valid: `default`, `test`, `prod`, `test-account`, `work_2024`
Invalid: `test account`, `test@account`

---

## Post Scheduling

### Schedule a Post

```bash
# Schedule in 30 minutes
plur-post "Hello later!" --schedule "30m"

# Schedule for tomorrow
plur-post "Tomorrow's update" --schedule "tomorrow"

# Schedule for a specific date
plur-post "New Year!" --schedule "Jan 1 10:00"

# Explicit year (recommended for clarity)
plur-post "Next year!" --schedule "2026-01-01 10:00"

# Random time in range
plur-post "Random timing" --schedule "random:1h-2h"
```

**Supported formats:**
- Duration: `30m`, `2h`, `1d`
- Natural language: `tomorrow`, `next week`
- Absolute dates: `Jan 1 10:00`, `Dec 31 12:00`
- ISO format: `2026-01-01 10:00`
- Random range: `random:10m-20m`

**Year inference:** When scheduling with month/day without an explicit year
(e.g., `Jan 1 10:00`), if the date would be in the past, it automatically
schedules for the next occurrence (next year). Use explicit years like
`2026-01-01` for unambiguous scheduling.

### Manage Queue (plur-queue)

```bash
# List scheduled posts
plur-queue list
plur-queue list --format json

# View statistics
plur-queue stats

# Cancel a post
plur-queue cancel <post_id>
plur-queue cancel --all --force

# Reschedule
plur-queue reschedule <post_id> "+2h"   # Delay by 2 hours
plur-queue reschedule <post_id> "-30m"  # Move up

# Post immediately
plur-queue now <post_id>

# Manage failed posts
plur-queue failed list
plur-queue failed delete <post_id>
```

### Run the Daemon (plur-send)

The daemon processes scheduled posts automatically:

```bash
plur-send                      # Default settings
plur-send --poll-interval 30   # Check every 30 seconds
plur-send --verbose            # Detailed logging
plur-send --no-retry           # Disable auto-retry
```

**Configuration** (`~/.config/plurcast/config.toml`):

```toml
[scheduling]
poll_interval = 60
max_retries = 3
retry_delay = 300

[scheduling.rate_limits]
nostr = { posts_per_hour = 100 }
mastodon = { posts_per_hour = 300 }
```

---

## Nostr-Specific Features

### Proof of Work (--nostr-pow)

Add computational difficulty to combat spam (NIP-13):

```bash
plur-post "Important message" --platform nostr --nostr-pow 20
plur-post "Critical" --platform nostr --nostr-pow 25  # Higher difficulty
```

**Guidelines:**
- Recommended: 20-25 (1-5 seconds)
- Maximum: 64 (very slow)
- Only applies to Nostr platform

### Shared Test Account

Test without setup:

```bash
plur-post "Testing!" --platform nostr --account shared-test
```

Public key: `npub1qyv34w2prnz66zxrgqsmy2emrg0uqtrnvarhrrfaktxk9vp2dgllsajv05m`

---

## Querying History

### Basic Queries

```bash
plur-history                    # Last 20 posts
plur-history --limit 50         # Custom limit
plur-history --platform nostr   # Filter by platform
```

### Search and Filter

```bash
plur-history --search "rust"
plur-history --since "2025-10-01" --until "2025-10-05"
```

### Output Formats

```bash
plur-history --format json   # JSON array
plur-history --format jsonl  # One JSON object per line
plur-history --format csv    # CSV for spreadsheets
```

---

## Import and Export

### Export Posts (plur-export)

```bash
plur-export --format ssb
plur-export --format ssb --output backup.jsonl
```

### Import Posts (plur-import)

```bash
plur-import ssb
plur-import ssb --account work-account
```

---

## Output Formats

### Text (Default)

```bash
plur-post "Hello"
# nostr:note1abc123...
```

### JSON (Machine-Readable)

```bash
plur-post "Hello" --format json
# [{"platform":"nostr","success":true,"post_id":"note1..."}]
```

### Verbose Logging

```bash
plur-post "Debug" --verbose
```

---

## Unix Composability

### Piping

```bash
# Text preprocessing
cat draft.txt | sed 's/foo/bar/g' | plur-post

# Template substitution
echo "Hello from $(hostname) at $(date)" | plur-post

# From command output
fortune | plur-post --platform nostr
```

### JSON Processing with jq

```bash
# Get only successful posts
plur-history --format json | jq '.[] | select(.platforms[].success == true)'

# Extract Nostr post IDs
plur-history --format json | jq -r '.[] | .platforms[] | select(.platform == "nostr") | .platform_post_id'

# Count posts per platform
plur-history --format json | jq '[.[] | .platforms[] | .platform] | group_by(.) | map({platform: .[0], count: length})'
```

### CSV Analysis

```bash
plur-history --format csv > posts.csv
cut -d, -f3 posts.csv | tail -n +2 | sort | uniq -c  # Count by platform
grep ",false," posts.csv  # Find failures
```

### Conditional Posting

```bash
# Post to different platforms based on content
if grep -q "urgent" message.txt; then
    cat message.txt | plur-post --platform nostr,mastodon
else
    cat message.txt | plur-post --platform nostr
fi

# Post only if short enough for Mastodon
if [ $(wc -c < message.txt) -le 500 ]; then
    cat message.txt | plur-post --platform nostr,mastodon
else
    cat message.txt | plur-post --platform nostr
fi
```

### Shell Scripts

```bash
#!/bin/bash
# daily-post.sh

CONTENT="Daily update: $(date +%Y-%m-%d)"

if plur-post "$CONTENT" --format json > /tmp/result.json; then
    echo "Posted successfully"
    jq -r '.[] | "\(.platform): \(.post_id)"' /tmp/result.json
else
    case $? in
        1) echo "Partial failure" >&2 ;;
        2) echo "Auth error" >&2 ;;
        3) echo "Invalid input" >&2 ;;
    esac
fi
```

### Integration Examples

```bash
# Post from RSS feed
curl -s https://example.com/feed.xml | xmllint --xpath '//item[1]/title/text()' - | plur-post

# Post from clipboard
xclip -o | plur-post        # Linux
pbpaste | plur-post         # macOS

# Post with notification
plur-post "Hello" && notify-send "Posted successfully"
```

---

## Environment Variables

| Variable | Description |
|----------|-------------|
| `PLURCAST_CONFIG` | Config file path |
| `PLURCAST_DB_PATH` | Database file path |
| `PLURCAST_MASTER_PASSWORD` | Master password for encrypted storage |
| `PLURCAST_LOG_FORMAT` | Log format (text/json/pretty) |
| `PLURCAST_LOG_LEVEL` | Log level (error/warn/info/debug/trace) |

---

See also:
- [Setup Guide](SETUP.md) - Platform configuration
- [Security](SECURITY.md) - Credential storage
- [Troubleshooting](TROUBLESHOOTING.md) - Common issues
