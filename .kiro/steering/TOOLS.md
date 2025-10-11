# Plurcast: Tool Specifications

**Related Documentation**:
- [Vision](./VISION.md) - Philosophy and design principles
- [Architecture](./ARCHITECTURE.md) - Technical implementation details
- [Roadmap](./ROADMAP.md) - Development phases and progress
- [Future](./FUTURE.md) - Extensibility and future plans

---

## plur-post

**Purpose**: Post content to platforms immediately or as draft

**Usage**:
```bash
# From stdin
echo "Hello decentralized world" | plur-post

# From arguments
plur-post "Hello decentralized world"

# Specific platforms only
echo "Nostr-only post" | plur-post --platform nostr

# Save as draft (don't post)
echo "Draft content" | plur-post --draft

# With metadata
plur-post "Tagged post" --tags rust,decentralization
```

**Output**:
- Success: Post ID (one per line if multiple platforms)
- Format: `platform:post_id` (e.g., `nostr:note1abc...`, `bluesky:at://...`)

**Exit codes**:
- 0: Success on all platforms
- 1: Failed on at least one platform
- 2: Authentication error
- 3: Invalid input

## plur-queue

**Purpose**: Schedule posts for future delivery

**Usage**:
```bash
# Schedule for specific time
echo "Good morning!" | plur-queue --at "2025-10-05T09:00:00Z"

# Schedule relative time
echo "Remember this later" | plur-queue --in "2 hours"

# Read from file with front matter
plur-queue < post.md
```

**Front matter format**:
```yaml
---
scheduled_at: 2025-10-05T14:00:00Z
platforms: [nostr, mastodon]
tags: [announcement, updates]
---
This is the post content.
It can be multiple lines.
```

**Output**: Queue ID
**Exit codes**: Same as plur-post

## plur-send

**Purpose**: Daemon that processes the queue

**Usage**:
```bash
# Run in foreground
plur-send

# Run with systemd
systemctl --user start plurcast

# One-shot mode (process queue once, then exit)
plur-send --once
```

**Behavior**:
- Polls database for pending posts every 60 seconds (configurable)
- Respects platform rate limits
- Updates post status and records results
- Logs to stderr or syslog

## plur-history

**Purpose**: Query local posting history

**Usage**:
```bash
# Recent posts (default: last 20)
plur-history

# Specific platform
plur-history --platform nostr

# Date range
plur-history --since "2025-10-01" --until "2025-10-05"

# Search content
plur-history --search "rust"

# JSON output for scripting
plur-history --format json | jq '.[] | .content'
```

**Output formats**: text (default), json, jsonl, csv

## plur-import

**Purpose**: Import existing posts from platform exports

**Usage**:
```bash
# Mastodon archive
plur-import mastodon --file archive.zip

# Nostr export (JSON)
plur-import nostr --file nostr-posts.json

# Bluesky export
plur-import bluesky --file bluesky-export.json
```

**Behavior**:
- Parses platform-specific export formats
- Preserves timestamps and metadata where possible
- Stores in local database with status='imported'
- Does not re-post to platforms

## plur-export

**Purpose**: Export local history to various formats

**Usage**:
```bash
# JSON export
plur-export --format json > posts.json

# CSV for analysis
plur-export --format csv > posts.csv

# Static HTML archive
plur-export --format html --output ./archive/

# Markdown files (one per post)
plur-export --format markdown --output ./posts/
```

---

**Version**: 0.2.0-alpha
**Last Updated**: 2025-10-11
**Status**: Active Development - Phase 2 (Multi-Platform) ~90% Complete
**Stable Platforms**: Nostr, Mastodon
