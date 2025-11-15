# Phase 5: Post Scheduling - Design Specification

## Overview

Add Unix-style post scheduling to Plurcast with a queue management CLI and background daemon.

**Philosophy**: Follow Unix conventions - separate tools for queuing and sending, daemon managed by systemd, human-friendly scheduling syntax.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    User Interface                        │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  plur-post --schedule "tomorrow 9am"                    │
│      ↓                                                   │
│  Creates post with scheduled_at timestamp               │
│      ↓                                                   │
│  Stores in database (status='scheduled')                │
│                                                          │
├─────────────────────────────────────────────────────────┤
│                    plur-queue                           │
│                  (CLI Tool)                              │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  plur-queue list        - Show scheduled posts          │
│  plur-queue cancel <id> - Cancel scheduled post         │
│  plur-queue reschedule  - Change schedule time          │
│  plur-queue now <id>    - Post immediately              │
│                                                          │
├─────────────────────────────────────────────────────────┤
│                    plur-send                             │
│                  (Daemon)                                │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  • Runs as systemd service                              │
│  • Polls database every 60 seconds                      │
│  • Finds posts where scheduled_at <= now                │
│  • Posts using PostingService                           │
│  • Updates post status to 'posted' or 'failed'          │
│  • Rate limiting per platform                           │
│  • Graceful shutdown on SIGTERM                         │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

## Database Schema

### Current Schema (Already Supports Scheduling!)

```sql
CREATE TABLE posts (
    id TEXT PRIMARY KEY,
    content TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    scheduled_at INTEGER,        -- Unix timestamp for scheduled posts
    status TEXT DEFAULT 'pending', -- 'draft', 'scheduled', 'posted', 'failed'
    metadata TEXT
);
```

**Status Values:**
- `draft` - Saved draft, not posted
- `scheduled` - Queued for future posting
- `pending` - Awaiting immediate posting
- `posted` - Successfully posted
- `failed` - Posting failed (with error in post_records)

### New Migration (002_scheduling_enhancements.sql)

```sql
-- Add index for efficient daemon queries
CREATE INDEX IF NOT EXISTS idx_posts_scheduled_at
ON posts(scheduled_at)
WHERE status = 'scheduled' AND scheduled_at IS NOT NULL;

-- Rate limiting tracking
CREATE TABLE IF NOT EXISTS rate_limits (
    platform TEXT NOT NULL,
    window_start INTEGER NOT NULL,
    post_count INTEGER DEFAULT 0,
    PRIMARY KEY (platform, window_start)
);

CREATE INDEX IF NOT EXISTS idx_rate_limits_platform
ON rate_limits(platform, window_start);
```

## Component Designs

### 1. plur-post (Enhancement)

**Add scheduling flag:**

```bash
# Human-friendly time parsing
plur-post "Good morning!" --schedule "tomorrow 9am"
plur-post "Weekly update" --schedule "next monday 10am"
plur-post "Announcement" --schedule "2025-11-20 15:00"

# Relative times
plur-post "In 1 hour" --schedule "1 hour"
plur-post "In 30 minutes" --schedule "30m"

# Recurring (future enhancement)
plur-post "Daily standup" --schedule "every day 9am"
```

**Implementation:**
- Parse schedule with `chrono` and natural language parser
- Set `scheduled_at` timestamp in database
- Set `status = 'scheduled'`
- Output: `scheduled:<post-id>:for:<timestamp>`

### 2. plur-queue (New CLI Tool)

**Commands:**

```bash
# List scheduled posts
plur-queue list
plur-queue list --platform nostr
plur-queue list --format json

# Cancel scheduled post
plur-queue cancel <post-id>
plur-queue cancel --all

# Reschedule
plur-queue reschedule <post-id> "tomorrow 3pm"
plur-queue reschedule <post-id> +1h  # Delay by 1 hour

# Post now (skip schedule)
plur-queue now <post-id>

# Show queue statistics
plur-queue stats
```

**Output Format (text):**

```
ID: abc123
Content: Good morning!
Platforms: nostr, mastodon
Scheduled: 2025-11-16 09:00:00 (in 8 hours)
Status: scheduled

ID: def456
Content: Weekly update
Platforms: nostr
Scheduled: 2025-11-18 10:00:00 (in 2 days)
Status: scheduled

Total: 2 scheduled posts
```

**Output Format (JSON):**

```json
[
  {
    "id": "abc123",
    "content": "Good morning!",
    "platforms": ["nostr", "mastodon"],
    "scheduled_at": 1731747600,
    "scheduled_human": "2025-11-16 09:00:00",
    "time_until": "8 hours",
    "status": "scheduled"
  }
]
```

### 3. plur-send (New Daemon)

**Architecture:**

```rust
// Main daemon loop
async fn run_daemon(config: Config) -> Result<()> {
    let service = PlurcastService::from_config(config).await?;
    let poll_interval = Duration::from_secs(60); // 1 minute

    loop {
        // Find posts due for sending
        let due_posts = service.history()
            .get_scheduled_posts_due()
            .await?;

        for post in due_posts {
            // Post using service layer
            let request = PostRequest {
                content: post.content,
                platforms: get_platforms_for_post(&post),
                draft: false,
                account: None,
            };

            match service.posting().post(request).await {
                Ok(response) => {
                    // Update post status
                    update_post_status(&post.id, "posted").await?;
                    log_success(&post, &response);
                }
                Err(e) => {
                    // Mark as failed, log error
                    update_post_status(&post.id, "failed").await?;
                    log_error(&post, &e);
                }
            }

            // Rate limiting
            apply_rate_limit(&post.platforms).await?;
        }

        // Sleep until next poll
        tokio::time::sleep(poll_interval).await;
    }
}
```

**Features:**

1. **Polling**: Check every 60 seconds for due posts
2. **Rate Limiting**: Configurable per platform
3. **Error Handling**: Retry logic with exponential backoff
4. **Logging**: Structured logs to journald
5. **Graceful Shutdown**: Handle SIGTERM/SIGINT
6. **Health Checks**: Expose status endpoint (optional)

**Configuration (config.toml):**

```toml
[scheduling]
enabled = true
poll_interval = 60  # seconds
max_retries = 3
retry_delay = 300   # seconds (5 minutes)

[scheduling.rate_limits]
nostr = 10    # posts per hour
mastodon = 5  # posts per hour
ssb = 20      # posts per hour
```

### 4. Systemd Integration

**Service File: `/etc/systemd/system/plur-send.service`**

```ini
[Unit]
Description=Plurcast Scheduled Post Daemon
After=network.target

[Service]
Type=simple
User=%u
ExecStart=%h/.cargo/bin/plur-send
Restart=on-failure
RestartSec=10

# Environment
Environment="RUST_LOG=info"

# Security
PrivateTmp=yes
NoNewPrivileges=yes

[Install]
WantedBy=default.target
```

**Installation:**

```bash
# Install daemon
cargo install --path plur-send

# Enable and start service
systemctl --user enable plur-send
systemctl --user start plur-send

# Check status
systemctl --user status plur-send
journalctl --user -u plur-send -f
```

## Implementation Phases

### Phase 5.1: Database and Core Logic
- [x] Database schema (already exists!)
- [ ] Add scheduled_at index migration
- [ ] Rate limiting table migration
- [ ] Add `get_scheduled_posts_due()` to HistoryService
- [ ] Add `update_post_status()` to Database

### Phase 5.2: plur-post Enhancement
- [ ] Add `--schedule` flag to plur-post
- [ ] Natural language time parsing (chrono-english or timeparse)
- [ ] Set scheduled_at and status='scheduled'
- [ ] Update tests
- [ ] Update documentation

### Phase 5.3: plur-queue CLI
- [ ] Create plur-queue binary
- [ ] Implement `list` command
- [ ] Implement `cancel` command
- [ ] Implement `reschedule` command
- [ ] Implement `now` command (post immediately)
- [ ] Implement `stats` command
- [ ] JSON output format
- [ ] Integration tests

### Phase 5.4: plur-send Daemon
- [ ] Create plur-send binary
- [ ] Main daemon loop with polling
- [ ] Query scheduled posts
- [ ] Post using PostingService
- [ ] Update post status
- [ ] Rate limiting implementation
- [ ] Graceful shutdown handling
- [ ] Logging and error handling
- [ ] Integration tests

### Phase 5.5: Systemd Integration
- [ ] Create systemd service file
- [ ] Installation script
- [ ] User guide documentation
- [ ] Health check endpoint (optional)

## Rate Limiting Strategy

### Per-Platform Limits

```rust
struct RateLimiter {
    platform: String,
    limit: usize,        // posts per window
    window: Duration,    // time window (e.g., 1 hour)
}

impl RateLimiter {
    async fn check_and_record(&self, db: &Database) -> Result<bool> {
        let now = Utc::now().timestamp();
        let window_start = now - (now % self.window.as_secs());

        // Get current count for this window
        let count = db.get_rate_limit_count(&self.platform, window_start).await?;

        if count >= self.limit {
            return Ok(false); // Rate limit exceeded
        }

        // Record this post
        db.increment_rate_limit(&self.platform, window_start).await?;
        Ok(true)
    }
}
```

### Default Limits

Based on platform best practices:
- **Nostr**: 10 posts/hour (no official limit, but be respectful)
- **Mastodon**: 5 posts/hour (avoid spam detection)
- **SSB**: 20 posts/hour (local-first, more flexible)

## Error Handling

### Retry Strategy

```rust
async fn post_with_retry(
    service: &PostingService,
    request: PostRequest,
    max_retries: usize,
) -> Result<PostResponse> {
    let mut attempts = 0;
    let mut delay = Duration::from_secs(300); // 5 minutes

    loop {
        match service.post(request.clone()).await {
            Ok(response) => return Ok(response),
            Err(e) if attempts < max_retries => {
                tracing::warn!(
                    "Post failed (attempt {}/{}): {}",
                    attempts + 1,
                    max_retries,
                    e
                );
                tokio::time::sleep(delay).await;
                delay *= 2; // Exponential backoff
                attempts += 1;
            }
            Err(e) => return Err(e),
        }
    }
}
```

## Testing Strategy

### Unit Tests
- Time parsing logic
- Rate limiter logic
- Post status transitions

### Integration Tests
- Schedule post → verify in database
- Cancel scheduled post
- Reschedule post
- Rate limit enforcement

### End-to-End Tests
- Schedule → daemon picks up → posts successfully
- Schedule → cancel before daemon runs
- Multiple posts scheduled at same time
- Rate limit prevents over-posting

## Security Considerations

1. **Daemon Runs as User**: No root privileges needed
2. **Rate Limiting**: Prevents accidental spam
3. **Graceful Shutdown**: SIGTERM doesn't lose posts
4. **Error Logging**: Don't leak credentials in logs
5. **Database Permissions**: Only user has access

## Future Enhancements (Post-Phase 5)

- **Recurring Posts**: `--schedule "every monday 9am"`
- **Timezones**: Better timezone handling
- **Web UI**: Visual calendar for scheduled posts
- **Conflict Detection**: Warn if too many posts at same time
- **Analytics**: Track success rate by time of day
- **Smart Scheduling**: AI suggests optimal posting times

## Documentation Requirements

### User Guide
- How to schedule posts
- Managing the queue
- Setting up the daemon
- Troubleshooting

### Developer Guide
- Architecture overview
- Adding new scheduling features
- Testing scheduled posts

## Success Criteria

- [ ] Can schedule posts with natural language
- [ ] Daemon reliably posts at scheduled time (±1 minute)
- [ ] Rate limiting prevents over-posting
- [ ] Systemd integration works on Linux
- [ ] All tests pass
- [ ] Documentation complete

## Timeline Estimate

- **Phase 5.1** (Database): 2-3 days
- **Phase 5.2** (plur-post): 3-4 days
- **Phase 5.3** (plur-queue): 4-5 days
- **Phase 5.4** (plur-send): 5-7 days
- **Phase 5.5** (Systemd): 2-3 days
- **Total**: 2-3 weeks

---

**Version**: 1.0
**Status**: Design Complete - Ready for Implementation
**Phase**: 5 (Post Scheduling)
