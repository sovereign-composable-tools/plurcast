# Design Document: Multi-Platform Alpha Release

## Overview

Phase 2 transforms Plurcast from a single-platform tool (Nostr) into a true multi-platform cross-posting system supporting Nostr, Mastodon, and Bluesky. This design maintains the Unix philosophy established in Phase 1 while introducing a clean platform abstraction layer that makes adding future platforms straightforward.

The architecture leverages mature, open-source Rust libraries for each platform:
- **Nostr**: `nostr-sdk` v0.35+ (already integrated)
- **Mastodon**: `megalodon` v0.14+ (Fediverse support)
- **Bluesky**: `atrium-api` v0.24+ (AT Protocol)

Key deliverables:
1. Platform abstraction trait with async support
2. Mastodon and Bluesky platform implementations
3. Multi-platform posting with concurrent execution
4. New `plur-history` binary for querying posting history
5. Enhanced configuration system for multiple platforms
6. Comprehensive error handling and resilience

## Architecture

### High-Level Component Diagram

```
┌─────────────────────────────────────────────────────────┐
│                    CLI Binaries                         │
├──────────────────┬──────────────────┬───────────────────┤
│   plur-post      │  plur-history    │  (future tools)   │
│  (multi-platform)│  (query history) │                   │
└────────┬─────────┴────────┬─────────┴───────────────────┘
         │                  │
         └──────────┬───────┘
                    │
         ┌──────────▼──────────┐
         │    libplurcast      │
         │  (shared library)   │
         └──────────┬──────────┘
                    │
       ┌────────┴────────┬────────────┬────────────┐
       │                 │            │            │
   ┌───▼───┐      ┌──────▼─────┐ ┌───▼────┐  ┌───▼────┐
   │ Config│      │  Platform  │ │   DB   │  │ Types  │
   │Manager│      │ Abstraction│ │ Layer  │  │ & Error│
   └───────┘      └──────┬─────┘ └────────┘  └────────┘
                         │
          ┌──────────────┼──────────────┐
          │              │              │
     ┌────▼────┐   ┌─────▼─────┐  ┌────▼────┐
     │  Nostr  │   │ Mastodon  │  │Bluesky  │
     │ Client  │   │  Client   │  │ Client  │
     └────┬────┘   └─────┬─────┘  └────┬────┘
          │              │              │
     ┌────▼────┐   ┌─────▼─────┐  ┌────▼────┐
     │nostr-sdk│   │megalodon  │  │atrium   │
     │ v0.35+  │   │  v0.14+   │  │-api     │
     └─────────┘   └───────────┘  └─────────┘
```

### Platform Abstraction Layer

The existing `Platform` trait provides the foundation. We'll enhance it to support:
- Async operations (already using `async_trait`)
- Platform-specific configuration
- Retry logic and error handling
- Rate limiting awareness

**Enhanced Platform Trait:**

```rust
#[async_trait]
pub trait Platform: Send + Sync {
    /// Authenticate with the platform
    async fn authenticate(&mut self) -> Result<()>;

    /// Post content to the platform
    async fn post(&self, content: &str) -> Result<String>;

    /// Validate content before posting (character limits, format)
    fn validate_content(&self, content: &str) -> Result<()>;

    /// Get the platform name (e.g., "nostr", "mastodon", "bluesky")
    fn name(&self) -> &str;

    /// Get platform-specific character limit
    fn character_limit(&self) -> Option<usize>;

    /// Check if platform is properly configured
    fn is_configured(&self) -> bool;
}
```


## Components and Interfaces

### 1. Platform Implementations

#### Nostr Client (Existing - Minor Enhancements)

Already implemented using `nostr-sdk`. Enhancements needed:
- Implement new trait methods (`character_limit`, `is_configured`)
- Ensure consistent error mapping to `PlatformError`

**Key characteristics:**
- Character limit: None (but practical limit ~4000 chars)
- Authentication: Private key (hex or bech32 format)
- Post ID format: `note1...` (bech32 encoded event ID)
- Multiple relay support

#### Mastodon Client (New)

**Library**: `megalodon` v0.14+

**Configuration:**
```toml
[mastodon]
enabled = true
instance = "mastodon.social"
token_file = "~/.config/plurcast/mastodon.token"
```

**Implementation details:**
- Character limit: Instance-specific (typically 500, fetch from API)
- Authentication: OAuth2 access token
- Post ID format: Numeric ID from instance
- Supports multiple Fediverse platforms (Mastodon, Pleroma, etc.)

**Key methods:**
```rust
pub struct MastodonClient {
    client: Box<dyn megalodon::Megalodon + Send + Sync>,
    instance_url: String,
    character_limit: usize,
}

impl MastodonClient {
    pub async fn new(instance_url: String, token: String) -> Result<Self>;
    async fn fetch_instance_info(&mut self) -> Result<()>;
}
```

#### Bluesky Client (New)

**Library**: `atrium-api` v0.24+

**Configuration:**
```toml
[bluesky]
enabled = true
handle = "user.bsky.social"
auth_file = "~/.config/plurcast/bluesky.auth"
```

**Implementation details:**
- Character limit: 300 characters
- Authentication: DID-based identity with app password
- Post ID format: AT URI (`at://did:plc:.../app.bsky.feed.post/...`)
- Uses XRPC for communication

**Key methods:**
```rust
pub struct BlueskyClient {
    agent: atrium_api::agent::AtpAgent,
    did: String,
}

impl BlueskyClient {
    pub async fn new(handle: String, password: String) -> Result<Self>;
    async fn create_session(&mut self) -> Result<()>;
}
```


### 2. Multi-Platform Posting Orchestration

**Design pattern**: Concurrent posting with individual result tracking

```rust
pub struct MultiPlatformPoster {
    platforms: Vec<Box<dyn Platform>>,
    db: Database,
}

impl MultiPlatformPoster {
    pub async fn post_to_all(&self, post: &Post) -> Vec<PostResult>;
    pub async fn post_to_selected(&self, post: &Post, platforms: &[String]) -> Vec<PostResult>;
}

pub struct PostResult {
    pub platform: String,
    pub success: bool,
    pub platform_post_id: Option<String>,
    pub error: Option<String>,
}
```

**Posting flow:**
1. Validate content against all target platforms
2. Create post record in database with status `Pending`
3. Launch concurrent tasks for each platform
4. Each task:
   - Authenticates if needed
   - Posts content
   - Records result in `post_records` table
5. Collect all results
6. Update post status based on results
7. Return formatted output

**Concurrency strategy:**
- Use `tokio::spawn` for each platform
- Use `futures::join_all` to wait for all completions
- Continue on individual failures (don't short-circuit)

### 3. Configuration System Enhancements

**Existing structure** (from Phase 1):
```rust
pub struct Config {
    pub database: DatabaseConfig,
    pub nostr: Option<NostrConfig>,
}
```

**Enhanced structure:**
```rust
pub struct Config {
    pub database: DatabaseConfig,
    pub nostr: Option<NostrConfig>,
    pub mastodon: Option<MastodonConfig>,
    pub bluesky: Option<BlueskyConfig>,
    pub defaults: DefaultsConfig,
}

pub struct DefaultsConfig {
    pub platforms: Vec<String>,  // Default platforms to post to
}

pub struct MastodonConfig {
    pub enabled: bool,
    pub instance: String,
    pub token_file: String,
}

pub struct BlueskyConfig {
    pub enabled: bool,
    pub handle: String,
    pub auth_file: String,
}
```

**Configuration loading:**
- Read from `~/.config/plurcast/config.toml`
- Expand shell variables in paths
- Validate required fields per platform
- Provide helpful error messages for missing config


### 4. plur-history Binary

**Purpose**: Query local posting history with filtering and formatting options

**Architecture:**
```rust
// Main structure
pub struct HistoryQuery {
    pub platform: Option<String>,
    pub since: Option<i64>,
    pub until: Option<i64>,
    pub search: Option<String>,
    pub limit: usize,
}

pub struct HistoryEntry {
    pub post_id: String,
    pub content: String,
    pub created_at: i64,
    pub platforms: Vec<PlatformStatus>,
}

pub struct PlatformStatus {
    pub platform: String,
    pub success: bool,
    pub platform_post_id: Option<String>,
    pub error: Option<String>,
}
```

**Database queries:**
```sql
-- Basic query with joins
SELECT 
    p.id, p.content, p.created_at, p.status,
    pr.platform, pr.platform_post_id, pr.success, pr.error_message
FROM posts p
LEFT JOIN post_records pr ON p.id = pr.post_id
WHERE 1=1
    AND (? IS NULL OR pr.platform = ?)
    AND (? IS NULL OR p.created_at >= ?)
    AND (? IS NULL OR p.created_at <= ?)
    AND (? IS NULL OR p.content LIKE ?)
ORDER BY p.created_at DESC
LIMIT ?;
```

**Output formats:**

1. **Text (default)**: Human-readable
```
2025-10-05 14:30:00 | abc-123 | Hello world
  ✓ nostr: note1abc...
  ✓ mastodon: 12345
  ✗ bluesky: Authentication failed
```

2. **JSON**: Machine-readable array
```json
[
  {
    "post_id": "abc-123",
    "content": "Hello world",
    "created_at": 1728139800,
    "platforms": [
      {"platform": "nostr", "success": true, "platform_post_id": "note1abc..."},
      {"platform": "mastodon", "success": true, "platform_post_id": "12345"},
      {"platform": "bluesky", "success": false, "error": "Authentication failed"}
    ]
  }
]
```

3. **JSONL**: One JSON object per line (streaming-friendly)
4. **CSV**: Spreadsheet-compatible


### 5. Enhanced plur-post Binary

**Changes from Phase 1:**
- Support `--platform` flag for selective posting
- Handle multiple platform results
- Output format: one line per platform
- Exit code logic for partial failures

**CLI interface:**
```bash
plur-post [OPTIONS] [CONTENT]

Options:
  -p, --platform <PLATFORM>  Post to specific platform(s) [possible values: nostr, mastodon, bluesky]
  -d, --draft               Save as draft without posting
  -v, --verbose             Show detailed progress
  -h, --help                Print help
```

**Output format:**
```
nostr:note1abc123...
mastodon:12345
bluesky:at://did:plc:xyz.../app.bsky.feed.post/abc
```

**Exit codes:**
- 0: All platforms succeeded
- 1: At least one platform failed (but not auth)
- 2: Authentication error on any platform
- 3: Invalid input (empty content, validation failure)

## Data Models

### Database Schema (No Changes Required)

The existing schema from Phase 1 already supports multi-platform:

```sql
CREATE TABLE posts (
    id TEXT PRIMARY KEY,
    content TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    scheduled_at INTEGER,
    status TEXT DEFAULT 'pending',
    metadata TEXT
);

CREATE TABLE post_records (
    id INTEGER PRIMARY KEY,
    post_id TEXT NOT NULL,
    platform TEXT NOT NULL,
    platform_post_id TEXT,
    posted_at INTEGER,
    success INTEGER DEFAULT 0,
    error_message TEXT,
    FOREIGN KEY (post_id) REFERENCES posts(id)
);

CREATE TABLE platforms (
    name TEXT PRIMARY KEY,
    enabled INTEGER DEFAULT 1,
    config TEXT
);
```

**Usage pattern:**
- One row in `posts` per user post
- Multiple rows in `post_records` per post (one per platform)
- `platforms` table for runtime configuration (optional)


## Error Handling

### Error Type Hierarchy

Existing error types are sufficient, but we'll add platform-specific context:

```rust
#[derive(Error, Debug)]
pub enum PlatformError {
    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Content validation failed: {0}")]
    Validation(String),

    #[error("Posting failed: {0}")]
    Posting(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),  // New variant
}
```

### Retry Strategy

**Transient errors** (network issues, temporary unavailability):
- Retry up to 3 times
- Exponential backoff: 1s, 2s, 4s
- Log each retry attempt

**Permanent errors** (authentication, validation):
- No retry
- Immediate failure with clear message

**Implementation:**
```rust
async fn post_with_retry(platform: &dyn Platform, content: &str) -> Result<String> {
    let mut attempts = 0;
    let max_attempts = 3;
    
    loop {
        match platform.post(content).await {
            Ok(post_id) => return Ok(post_id),
            Err(e) if is_transient(&e) && attempts < max_attempts => {
                attempts += 1;
                let delay = Duration::from_secs(2_u64.pow(attempts - 1));
                tokio::time::sleep(delay).await;
            }
            Err(e) => return Err(e),
        }
    }
}
```

### Error Context

Each platform error should include:
- Platform name
- Operation attempted
- Underlying error message
- Suggested remediation (when possible)

**Example error messages:**
```
Error: Platform error: Authentication failed: Invalid Mastodon token
Suggestion: Run 'plur-config mastodon' to re-authenticate

Error: Platform error: Content validation failed: Post exceeds Bluesky's 300 character limit
Current length: 450 characters
Suggestion: Shorten content or use --platform to exclude bluesky
```


## Testing Strategy

### Unit Tests

**Platform implementations:**
- Test each trait method independently
- Mock external API calls
- Verify error handling and mapping
- Test character limit validation

**Configuration:**
- Test parsing valid and invalid TOML
- Test path expansion
- Test missing required fields
- Test platform enable/disable logic

**Database operations:**
- Test post creation and retrieval
- Test post_records insertion
- Test history queries with filters
- Test concurrent writes

### Integration Tests

**Multi-platform posting:**
- Test posting to all platforms
- Test selective platform posting
- Test partial failure scenarios
- Test concurrent execution

**plur-history:**
- Test filtering by platform
- Test date range filtering
- Test search functionality
- Test output formats (text, JSON, CSV)

**End-to-end flows:**
- Test complete posting workflow
- Test authentication flows
- Test error recovery
- Test database persistence

### Test Doubles

**Mock platforms:**
```rust
pub struct MockPlatform {
    pub name: String,
    pub should_fail: bool,
    pub delay: Duration,
}

#[async_trait]
impl Platform for MockPlatform {
    async fn post(&self, content: &str) -> Result<String> {
        tokio::time::sleep(self.delay).await;
        if self.should_fail {
            Err(PlatformError::Posting("Mock failure".into()).into())
        } else {
            Ok(format!("{}:mock-id-{}", self.name, content.len()))
        }
    }
    // ... other methods
}
```

**Test database:**
- Use in-memory SQLite (`:memory:`)
- Run migrations before each test
- Clean state between tests


## Dependencies

### New Dependencies to Add

**Workspace Cargo.toml additions:**
```toml
[workspace.dependencies]
# Existing dependencies...
nostr-sdk = "0.35"
sqlx = { version = "0.8", features = ["sqlite", "runtime-tokio", "migrate"] }
# ... other existing deps

# New dependencies for Phase 2
megalodon = "0.14"
atrium-api = "0.24"
futures = "0.3"
```

**libplurcast Cargo.toml additions:**
```toml
[dependencies]
# Existing dependencies...
nostr-sdk = { workspace = true }
# ... other existing deps

# New for Phase 2
megalodon = { workspace = true }
atrium-api = { workspace = true }
futures = { workspace = true }
```

### Library Maturity Assessment

**nostr-sdk (v0.35+)**
- Status: ✅ Mature, actively maintained
- Ecosystem: Strong Nostr ecosystem adoption
- Risk: Low - already integrated and working

**megalodon (v0.14+)**
- Status: ✅ Stable, well-maintained
- Ecosystem: Used across multiple Fediverse clients
- Risk: Low - battle-tested across platforms

**atrium-api (v0.24+)**
- Status: ⚠️ Active development, protocol stabilizing
- Ecosystem: Growing, official Rust implementation
- Risk: Medium - newer but protocol is stable
- Mitigation: Comprehensive error handling, version pinning

## Project Structure Updates

```
plurcast/
├── libplurcast/
│   ├── src/
│   │   ├── platforms/
│   │   │   ├── mod.rs          (enhanced trait)
│   │   │   ├── nostr.rs        (existing, minor updates)
│   │   │   ├── mastodon.rs     (NEW)
│   │   │   └── bluesky.rs      (NEW)
│   │   ├── config.rs           (enhanced for multi-platform)
│   │   ├── db.rs               (minor query additions)
│   │   ├── error.rs            (add RateLimit variant)
│   │   └── poster.rs           (NEW - multi-platform orchestration)
│   └── Cargo.toml              (add megalodon, atrium-api)
├── plur-post/
│   ├── src/
│   │   └── main.rs             (enhanced for multi-platform)
│   └── Cargo.toml              (no changes)
├── plur-history/               (NEW binary)
│   ├── src/
│   │   └── main.rs             (NEW)
│   └── Cargo.toml              (NEW)
└── Cargo.toml                  (add new dependencies)
```


## Implementation Phases

### Phase 2.1: Platform Abstraction Enhancement
- Enhance Platform trait with new methods
- Update Nostr implementation to match enhanced trait
- Add platform factory pattern for instantiation
- Add comprehensive unit tests

### Phase 2.2: Mastodon Integration
- Implement MastodonClient
- Add Mastodon configuration parsing
- Add authentication handling
- Test against public Mastodon instances

### Phase 2.3: Bluesky Integration
- Implement BlueskyClient
- Add Bluesky configuration parsing
- Add DID-based authentication
- Test against Bluesky network

### Phase 2.4: Multi-Platform Orchestration
- Implement MultiPlatformPoster
- Add concurrent posting logic
- Add retry and error handling
- Update plur-post binary
- Integration tests for multi-platform scenarios

### Phase 2.5: plur-history Implementation
- Create new binary
- Implement database queries
- Add filtering logic
- Implement output formatters (text, JSON, JSONL, CSV)
- Add CLI argument parsing
- Integration tests

### Phase 2.6: Testing & Documentation
- Comprehensive integration tests
- Update README with multi-platform examples
- Add configuration examples for each platform
- Add troubleshooting guide
- Performance testing

## Security Considerations

### Credential Storage

**Current approach (Phase 1):**
- Credentials in separate files with 600 permissions
- Not stored in database

**Phase 2 additions:**
- Mastodon OAuth tokens in `~/.config/plurcast/mastodon.token`
- Bluesky app passwords in `~/.config/plurcast/bluesky.auth`
- All credential files should have 600 permissions
- Validate file permissions on read

**Future consideration:**
- System keyring integration (Phase 4+)
- Encrypted credential storage

### API Key Exposure

- Never log credentials
- Redact tokens in error messages
- Don't include credentials in database
- Clear guidance in documentation

### Rate Limiting

- Respect platform rate limits
- Implement backoff on rate limit errors
- Log rate limit encounters
- Consider per-platform rate limit tracking (future)


## Performance Considerations

### Concurrent Posting

**Benefits:**
- Faster total posting time
- Better user experience
- Efficient resource utilization

**Implementation:**
```rust
async fn post_to_platforms(platforms: Vec<Box<dyn Platform>>, content: &str) -> Vec<PostResult> {
    let tasks: Vec<_> = platforms
        .into_iter()
        .map(|platform| {
            let content = content.to_string();
            tokio::spawn(async move {
                post_with_retry(&*platform, &content).await
            })
        })
        .collect();
    
    futures::future::join_all(tasks).await
}
```

**Considerations:**
- Each platform posts independently
- Failures don't block other platforms
- Database writes are sequential (SQLite limitation)

### Database Performance

**Query optimization:**
- Index on `post_id` in `post_records` (foreign key)
- Index on `created_at` in `posts` for date filtering
- Index on `platform` in `post_records` for filtering

**Connection pooling:**
- Use SQLx connection pool
- Configure appropriate pool size (default: 10)

### Memory Usage

- Stream large result sets (plur-history)
- Avoid loading all posts into memory
- Use iterators where possible

## Unix Philosophy Compliance

### Composability Examples

**Posting from file:**
```bash
cat post.txt | plur-post
```

**Filtering history:**
```bash
plur-history --format json | jq '.[] | select(.platforms[].success == false)'
```

**Counting posts per platform:**
```bash
plur-history --format csv | cut -d, -f3 | sort | uniq -c
```

**Automated posting:**
```bash
#!/bin/bash
if plur-post "Automated post"; then
    echo "Posted successfully"
else
    echo "Failed to post" >&2
    exit 1
fi
```

### Agent-Friendly Features

**Discoverable interfaces:**
- Comprehensive `--help` text
- Man pages (future)
- Consistent exit codes

**Machine-readable output:**
- JSON format for scripting
- JSONL for streaming
- CSV for data analysis

**Predictable behavior:**
- Deterministic output format
- No interactive prompts (unless TTY detected)
- Clear error messages to stderr


## Migration from Phase 1

### Backward Compatibility

**Existing users:**
- Phase 1 database schema is compatible (no migration needed)
- Existing Nostr configuration continues to work
- Default behavior: post to all enabled platforms

**Configuration migration:**
```toml
# Phase 1 config (still valid)
[nostr]
enabled = true
keys_file = "~/.config/plurcast/nostr.keys"
relays = ["wss://relay.damus.io"]

# Phase 2 additions (optional)
[mastodon]
enabled = false  # Disabled by default

[bluesky]
enabled = false  # Disabled by default

[defaults]
platforms = ["nostr"]  # Explicit default
```

**Upgrade path:**
1. Update binary
2. Existing functionality works unchanged
3. Add new platform configs as desired
4. Enable platforms incrementally

### Data Preservation

- Existing posts remain in database
- Existing post_records remain valid
- No data loss during upgrade

## Success Metrics

### Functional Metrics

- ✅ Post to Nostr, Mastodon, and Bluesky from single command
- ✅ Query posting history with filters
- ✅ Handle partial failures gracefully
- ✅ Concurrent posting completes faster than sequential
- ✅ All tests pass (unit + integration)

### Quality Metrics

- ✅ Zero regressions in Phase 1 functionality
- ✅ Clear error messages for all failure modes
- ✅ Documentation covers all platforms
- ✅ Configuration examples for each platform
- ✅ Backward compatible with Phase 1

### Performance Metrics

- Concurrent posting: 2-3x faster than sequential
- History queries: <100ms for 1000 posts
- Memory usage: <50MB for typical workloads

## Future Extensibility

### Adding New Platforms

The platform abstraction makes adding new platforms straightforward:

1. Implement `Platform` trait
2. Add configuration struct
3. Add to platform factory
4. Update documentation

**Example platforms for future:**
- Threads (Meta)
- Lens Protocol
- Farcaster
- RSS/Atom feeds

### Feature Additions

The architecture supports future features:
- Media attachments (trait method: `post_with_media`)
- Thread support (trait method: `post_thread`)
- Reply handling (trait method: `post_reply`)
- Scheduled posting (Phase 3)
- Analytics (Phase 5)

## Open Questions

1. **Rate limiting**: Should we implement per-platform rate limiting in Phase 2, or defer to Phase 3?
   - **Decision**: Basic retry with backoff in Phase 2, sophisticated rate limiting in Phase 3

2. **Authentication flow**: Should we provide interactive OAuth flow for Mastodon, or require manual token generation?
   - **Decision**: Manual token generation for Phase 2 (simpler), interactive flow in future

3. **Character limit handling**: Should we auto-truncate or fail on limit exceeded?
   - **Decision**: Fail with clear error message, let user decide

4. **Platform priority**: Should posting continue if validation fails on one platform?
   - **Decision**: Yes, validate all platforms first, then post to those that pass

## Conclusion

Phase 2 transforms Plurcast into a true multi-platform tool while maintaining the Unix philosophy and agent-friendly design established in Phase 1. The platform abstraction layer provides a clean foundation for future expansion, and the concurrent posting architecture ensures good performance.

Key achievements:
- Three platform support (Nostr, Mastodon, Bluesky)
- Clean platform abstraction
- New plur-history tool
- Backward compatible with Phase 1
- Comprehensive error handling
- Agent-friendly interfaces maintained

This design positions Plurcast for its alpha release to the community, demonstrating the viability of Unix-style tools for the decentralized social web.
