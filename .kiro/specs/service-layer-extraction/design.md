# Design Document: Service Layer Extraction

## Overview

This document details the technical design for extracting business logic from CLI binaries into a dedicated service layer. The service layer provides a clean, testable API that can be consumed by multiple interfaces (CLI, TUI, GUI) without code duplication.

## Architecture

### High-Level Structure

```
libplurcast/
├── src/
│   ├── service/              # NEW: Service layer
│   │   ├── mod.rs           # PlurcastService facade
│   │   ├── posting.rs       # PostingService
│   │   ├── history.rs       # HistoryService
│   │   ├── draft.rs         # DraftService
│   │   ├── validation.rs    # ValidationService
│   │   └── events.rs        # EventBus
│   ├── platforms/           # Existing platform clients
│   ├── config.rs            # Existing configuration
│   ├── db.rs                # Existing database
│   ├── credentials.rs       # Existing credentials
│   ├── error.rs             # Existing error types
│   ├── poster.rs            # REFACTOR: Move logic to PostingService
│   └── types.rs             # Existing types
```

### Dependency Flow

```
┌─────────────────────────────────────────┐
│         CLI Binaries                    │
│  (plur-post, plur-history, etc.)        │
└──────────────┬──────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────┐
│      PlurcastService (Facade)           │
│  - Coordinates all services             │
│  - Manages shared state (Arc)           │
│  - Extensible for future services       │
└──────────────┬──────────────────────────┘
               │
       ┌───────┴───────┬───────────┬──────────┐
       ▼               ▼           ▼          ▼
┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐
│ Posting  │  │ History  │  │  Draft   │  │Validation│
│ Service  │  │ Service  │  │ Service  │  │ Service  │
└────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘
     │             │              │             │
     └─────────────┴──────────────┴─────────────┘
                   │
                   ▼
     ┌─────────────────────────────┐
     │    Core Components          │
     │  - Database (Arc)           │
     │  - Config (Arc)             │
     │  - Platform Clients         │
     │  - Credentials              │
     └─────────────────────────────┘

Future Services (Phase 4+):
┌──────────┐  ┌──────────┐  ┌──────────┐
│  Queue   │  │Embedding │  │ Account  │
│ Service  │  │ Service  │  │ Service  │
└──────────┘  └──────────┘  └──────────┘
```

**Extensibility**: The service layer is designed to accommodate future services without architectural changes. New services can be added to PlurcastService and share the same Database/Config instances via Arc.

## Components

### 1. PlurcastService (Facade)

**Purpose**: Single entry point for all service operations. Manages shared resources and provides access to sub-services.

**API Design**:

```rust
pub struct PlurcastService {
    posting: PostingService,
    history: HistoryService,
    draft: DraftService,
    validation: ValidationService,
    event_bus: EventBus,
}

impl PlurcastService {
    /// Create a new service with default configuration
    pub async fn new() -> Result<Self>;
    
    /// Create a service with custom configuration
    pub async fn from_config(config: Config) -> Result<Self>;
    
    /// Access posting service
    pub fn posting(&self) -> &PostingService;
    
    /// Access history service
    pub fn history(&self) -> &HistoryService;
    
    /// Access draft service
    pub fn draft(&self) -> &DraftService;
    
    /// Access validation service
    pub fn validation(&self) -> &ValidationService;
    
    /// Subscribe to events
    pub fn subscribe(&self) -> EventReceiver;
}
```

**Shared State**:
- `Arc<Database>` - Shared database connection pool
- `Arc<Config>` - Shared configuration (read-only)
- `EventBus` - Shared event distribution

### 2. PostingService

**Purpose**: Handle all posting operations including validation, multi-platform posting, retry logic, and progress tracking.

**API Design**:

```rust
pub struct PostingService {
    db: Arc<Database>,
    config: Arc<Config>,
    event_bus: EventBus,
}

#[derive(Debug, Clone)]
pub struct PostRequest {
    pub content: String,
    pub platforms: Vec<String>,
    pub draft: bool,
}

#[derive(Debug, Clone)]
pub struct PostResponse {
    pub post_id: String,
    pub results: Vec<PlatformResult>,
    pub overall_success: bool,
}

#[derive(Debug, Clone)]
pub struct PlatformResult {
    pub platform: String,
    pub success: bool,
    pub post_id: Option<String>,
    pub error: Option<String>,
}

impl PostingService {
    /// Post content to specified platforms
    pub async fn post(&self, request: PostRequest) -> Result<PostResponse>;
    
    /// Create a draft without posting
    pub async fn create_draft(&self, content: String) -> Result<String>;
    
    /// Retry a failed post
    pub async fn retry_post(&self, post_id: &str, platforms: Vec<String>) -> Result<PostResponse>;
    
    // Future: Phase 4 will add scheduling methods
    // pub async fn schedule_post(&self, request: ScheduleRequest) -> Result<String>;
    // pub async fn cancel_scheduled(&self, post_id: &str) -> Result<()>;
}
```

**Retry Logic**:
- Exponential backoff: 1s, 2s, 4s, 8s, 16s (max 5 attempts)
- Only retry transient errors (network, rate limit)
- Don't retry validation or authentication errors

**Event Emission**:
```rust
PostingStarted { post_id, platforms }
PostingProgress { post_id, platform, status }
PostingCompleted { post_id, results }
PostingFailed { post_id, error }
```

### 3. HistoryService

**Purpose**: Query and analyze post history with flexible filtering and pagination.

**API Design**:

```rust
pub struct HistoryService {
    db: Arc<Database>,
}

#[derive(Debug, Clone, Default)]
pub struct HistoryQuery {
    pub platform: Option<String>,
    pub status: Option<PostStatus>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub search: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct PostWithRecords {
    pub post: Post,
    pub records: Vec<PostRecord>,
}

#[derive(Debug, Clone)]
pub struct HistoryStats {
    pub total_posts: usize,
    pub platform_stats: HashMap<String, PlatformStats>,
}

#[derive(Debug, Clone)]
pub struct PlatformStats {
    pub total: usize,
    pub successful: usize,
    pub failed: usize,
    pub success_rate: f64,
}

impl HistoryService {
    /// List posts with filtering and pagination
    pub async fn list_posts(&self, query: HistoryQuery) -> Result<Vec<PostWithRecords>>;
    
    /// Get a single post by ID
    pub async fn get_post(&self, post_id: &str) -> Result<Option<PostWithRecords>>;
    
    /// Get statistics
    pub async fn get_stats(&self, query: HistoryQuery) -> Result<HistoryStats>;
    
    /// Count posts matching query
    pub async fn count_posts(&self, query: HistoryQuery) -> Result<usize>;
}
```

### 4. DraftService

**Purpose**: Manage draft posts (CRUD operations) and publish them.

**API Design**:

```rust
pub struct DraftService {
    db: Arc<Database>,
    posting: PostingService,
}

#[derive(Debug, Clone)]
pub struct Draft {
    pub id: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl DraftService {
    /// Create a new draft
    pub async fn create(&self, content: String) -> Result<Draft>;
    
    /// Update an existing draft
    pub async fn update(&self, id: &str, content: String) -> Result<Draft>;
    
    /// Delete a draft
    pub async fn delete(&self, id: &str) -> Result<()>;
    
    /// List all drafts
    pub async fn list(&self) -> Result<Vec<Draft>>;
    
    /// Get a single draft
    pub async fn get(&self, id: &str) -> Result<Option<Draft>>;
    
    /// Publish a draft (delegates to PostingService)
    pub async fn publish(&self, id: &str, platforms: Vec<String>) -> Result<PostResponse>;
}
```

### 5. ValidationService

**Purpose**: Validate content against platform requirements in real-time.

**API Design**:

```rust
pub struct ValidationService {
    config: Arc<Config>,
}

#[derive(Debug, Clone)]
pub struct ValidationRequest {
    pub content: String,
    pub platforms: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ValidationResponse {
    pub valid: bool,
    pub results: Vec<PlatformValidation>,
}

#[derive(Debug, Clone)]
pub struct PlatformValidation {
    pub platform: String,
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationService {
    /// Validate content for specified platforms
    pub fn validate(&self, request: ValidationRequest) -> ValidationResponse;
    
    /// Check if content is valid for all platforms
    pub fn is_valid(&self, content: &str, platforms: &[String]) -> bool;
    
    /// Get character limits for platforms
    pub fn get_limits(&self, platforms: &[String]) -> HashMap<String, Option<usize>>;
}
```

**Validation Rules**:
1. Content not empty or whitespace-only
2. Content size ≤ 100KB (MAX_CONTENT_LENGTH)
3. Character count ≤ platform limit (if applicable)
   - Nostr: No limit (warn if > 280 chars)
   - Mastodon: Instance-specific (default 500)
   - Bluesky: 300 characters

### 6. EventBus

**Purpose**: Distribute progress events to subscribers without blocking operations.

**API Design**:

```rust
pub struct EventBus {
    sender: broadcast::Sender<Event>,
}

pub type EventReceiver = broadcast::Receiver<Event>;

#[derive(Debug, Clone)]
pub enum Event {
    PostingStarted {
        post_id: String,
        platforms: Vec<String>,
    },
    PostingProgress {
        post_id: String,
        platform: String,
        status: String,
    },
    PostingCompleted {
        post_id: String,
        results: Vec<PlatformResult>,
    },
    PostingFailed {
        post_id: String,
        error: String,
    },
}

impl EventBus {
    /// Create a new event bus with capacity
    pub fn new(capacity: usize) -> Self;
    
    /// Subscribe to events
    pub fn subscribe(&self) -> EventReceiver;
    
    /// Emit an event (non-blocking)
    pub fn emit(&self, event: Event);
}
```

**Implementation Notes**:
- Use `tokio::sync::broadcast` for multi-subscriber support
- Default capacity: 100 events
- If no subscribers, events are dropped (no blocking)
- Subscribers can lag without blocking emitters

## Data Models

### Existing Types (No Changes)

```rust
// From types.rs
pub struct Post {
    pub id: String,
    pub content: String,
    pub created_at: i64,
    pub scheduled_at: Option<i64>,  // Already supports Phase 4 scheduling
    pub status: PostStatus,
    pub metadata: Option<String>,   // JSON field for extensibility
}

pub struct PostRecord {
    pub id: i64,
    pub post_id: String,
    pub platform: String,
    pub platform_post_id: Option<String>,
    pub posted_at: Option<i64>,
    pub success: bool,
    pub error_message: Option<String>,
}

pub enum PostStatus {
    Pending,
    Posted,
    Failed,
    Draft,
    // Future: Phase 4 will add Scheduled variant
}
```

**Future-Proofing Notes**:
- `scheduled_at` field already exists for Phase 4 scheduling
- `metadata` JSON field allows storing additional data (tags, intervals, etc.) without schema changes
- PostStatus enum can be extended with new variants (Scheduled, Processing, etc.)
- Database schema supports future tables (embeddings, accounts) via migrations

## Error Handling

**Principle**: Use existing `PlurcastError` types, add service-specific variants if needed.

```rust
// Potential additions to error.rs
pub enum ServiceError {
    PostNotFound(String),
    DraftNotFound(String),
    InvalidQuery(String),
    ConcurrentModification(String),
}
```

All service methods return `Result<T, PlurcastError>` for consistency.

## Testing Strategy

### Unit Tests

Each service has its own test module:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_posting_service_success() {
        // Test with in-memory database
        // Mock platform clients
        // Verify database records
        // Verify events emitted
    }
    
    #[tokio::test]
    async fn test_posting_service_retry() {
        // Test retry logic with transient failures
    }
}
```

### Integration Tests

Test service interactions:

```rust
#[tokio::test]
async fn test_draft_publish_flow() {
    let service = PlurcastService::new().await.unwrap();
    
    // Create draft
    let draft = service.draft().create("Test".to_string()).await.unwrap();
    
    // Publish draft
    let response = service.draft().publish(&draft.id, vec!["nostr".to_string()]).await.unwrap();
    
    // Verify in history
    let posts = service.history().list_posts(Default::default()).await.unwrap();
    assert_eq!(posts.len(), 1);
}
```

### Mock Platform Clients

```rust
pub struct MockPlatform {
    name: String,
    should_fail: bool,
    delay: Duration,
}

#[async_trait]
impl Platform for MockPlatform {
    async fn post(&self, content: &str) -> Result<String> {
        tokio::time::sleep(self.delay).await;
        if self.should_fail {
            Err(PlatformError::Network("Mock failure".to_string()).into())
        } else {
            Ok(format!("mock_id_{}", uuid::Uuid::new_v4()))
        }
    }
    // ... other methods
}
```

## CLI Refactoring

### plur-post Refactoring

**Before**:
```rust
async fn run(cli: Cli) -> Result<()> {
    let config = Config::load()?;
    let db = Database::new(&db_path).await?;
    let platforms = create_platforms(&config).await?;
    let poster = MultiPlatformPoster::new(platforms, db);
    let results = poster.post_to_selected(&post, &platform_names).await;
    // ... output handling
}
```

**After**:
```rust
async fn run(cli: Cli) -> Result<()> {
    let service = PlurcastService::new().await?;
    
    let request = PostRequest {
        content: get_content(&cli)?,
        platforms: determine_platforms(&cli)?,
        draft: cli.draft,
    };
    
    let response = service.posting().post(request).await?;
    
    output_results(&response, &cli.format)?;
    std::process::exit(determine_exit_code(&response));
}
```

### plur-history Refactoring

**Before**:
```rust
async fn run(cli: Cli) -> Result<()> {
    let config = Config::load()?;
    let db = Database::new(&db_path).await?;
    let posts = db.list_posts(...).await?;
    // ... formatting
}
```

**After**:
```rust
async fn run(cli: Cli) -> Result<()> {
    let service = PlurcastService::new().await?;
    
    let query = HistoryQuery {
        platform: cli.platform,
        since: cli.since,
        until: cli.until,
        search: cli.search,
        limit: Some(cli.limit),
        offset: Some(cli.offset),
    };
    
    let posts = service.history().list_posts(query).await?;
    
    output_posts(&posts, &cli.format)?;
    Ok(())
}
```

## Migration Strategy

### Phase 1: Create Service Layer (No Breaking Changes)
1. Create `libplurcast/src/service/` directory
2. Implement all services
3. Add comprehensive tests
4. Services coexist with existing code

### Phase 2: Refactor CLI Tools
1. Update `plur-post` to use PostingService
2. Update `plur-history` to use HistoryService
3. Verify all tests pass
4. Verify CLI behavior unchanged

### Phase 3: Cleanup (Optional)
1. Mark old `poster.rs` functions as deprecated
2. Consider removing unused code in future release

## Performance Considerations

### Shared State (Arc)
- Database connection pool already thread-safe
- Config is read-only, no contention
- Arc overhead is negligible (<10ns per clone)

### Event Bus
- Broadcast channel has O(1) send
- Dropped events if no subscribers (no allocation)
- Subscribers can lag without blocking

### Concurrent Posting
- Already implemented in MultiPlatformPoster
- Service layer maintains same concurrency model
- No performance regression expected

## Documentation

### Rustdoc Examples

```rust
/// Post content to multiple platforms
///
/// # Example
///
/// ```no_run
/// use libplurcast::service::PlurcastService;
/// use libplurcast::service::posting::PostRequest;
///
/// # async fn example() -> libplurcast::Result<()> {
/// let service = PlurcastService::new().await?;
///
/// let request = PostRequest {
///     content: "Hello world!".to_string(),
///     platforms: vec!["nostr".to_string(), "mastodon".to_string()],
///     draft: false,
/// };
///
/// let response = service.posting().post(request).await?;
/// println!("Posted to {} platforms", response.results.len());
/// # Ok(())
/// # }
/// ```
pub async fn post(&self, request: PostRequest) -> Result<PostResponse>
```

### SERVICE_LAYER.md Guide

Will include:
- Architecture overview with diagrams
- Usage examples for each service
- Event handling patterns
- Testing patterns
- Migration guide for future UI development

## Success Criteria

1. ✅ All services implemented with full API
2. ✅ Test coverage ≥ 80% for service layer
3. ✅ CLI tools refactored with zero behavioral changes
4. ✅ All existing tests pass
5. ✅ Documentation complete with examples
6. ✅ Ready for Phase 3.2 (TUI) development

## Future-Proofing for Roadmap Features

This section analyzes how the service layer design supports future roadmap features without requiring major architectural changes.

### Scheduled Posting (Phase 4)

**Roadmap Feature**: Queue posts for future delivery with specific DateTimes or random intervals.

**Service Layer Support**:

1. **PostingService Extension**:
```rust
impl PostingService {
    /// Schedule a post for future delivery
    pub async fn schedule_post(&self, request: ScheduleRequest) -> Result<String>;
    
    /// Cancel a scheduled post
    pub async fn cancel_scheduled(&self, post_id: &str) -> Result<()>;
    
    /// Reschedule an existing post
    pub async fn reschedule(&self, post_id: &str, new_time: DateTime<Utc>) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct ScheduleRequest {
    pub content: String,
    pub platforms: Vec<String>,
    pub scheduled_at: DateTime<Utc>,  // Specific time
    pub interval: Option<Duration>,    // For random spacing
}
```

2. **Database Schema** (already supports this):
   - `posts.scheduled_at` field already exists
   - `posts.status` can include "Scheduled" variant
   - No schema changes needed

3. **QueueService** (new service, Phase 4):
```rust
pub struct QueueService {
    db: Arc<Database>,
    posting: PostingService,
    event_bus: EventBus,
}

impl QueueService {
    /// Process pending scheduled posts
    pub async fn process_queue(&self) -> Result<Vec<PostResponse>>;
    
    /// Get upcoming scheduled posts
    pub async fn list_scheduled(&self, limit: usize) -> Result<Vec<Post>>;
}
```

4. **PlurcastService Integration**:
```rust
impl PlurcastService {
    pub fn queue(&self) -> &QueueService {
        &self.queue
    }
}
```

**Design Compatibility**: ✅ Current design fully supports scheduled posting
- PostingService already handles posting logic
- Database schema already has `scheduled_at` field
- EventBus can emit scheduling events
- New QueueService can be added without modifying existing services

### Vector Embeddings & Semantic Search (Post-1.0)

**Roadmap Feature**: Local semantic search using vector embeddings for discovering patterns in post history.

**Service Layer Support**:

1. **EmbeddingService** (new service, future):
```rust
pub struct EmbeddingService {
    db: Arc<Database>,
    model: Arc<EmbeddingModel>,  // Local model (candle/ort)
}

impl EmbeddingService {
    /// Generate embeddings for a post
    pub async fn embed_post(&self, post_id: &str) -> Result<Vec<f32>>;
    
    /// Generate embeddings for content
    pub async fn embed_content(&self, content: &str) -> Result<Vec<f32>>;
    
    /// Batch embed multiple posts
    pub async fn embed_batch(&self, post_ids: Vec<String>) -> Result<HashMap<String, Vec<f32>>>;
}
```

2. **SearchService** (new service, future):
```rust
pub struct SearchService {
    db: Arc<Database>,
    embedding: EmbeddingService,
}

#[derive(Debug, Clone)]
pub struct SemanticQuery {
    pub query: String,
    pub limit: usize,
    pub similarity_threshold: f32,
    pub filters: Option<HistoryQuery>,  // Reuse existing filters
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub post: PostWithRecords,
    pub similarity: f32,
}

impl SearchService {
    /// Semantic search over post history
    pub async fn search(&self, query: SemanticQuery) -> Result<Vec<SearchResult>>;
    
    /// Find similar posts to a given post
    pub async fn find_similar(&self, post_id: &str, limit: usize) -> Result<Vec<SearchResult>>;
    
    /// Suggest content based on draft
    pub async fn suggest(&self, draft_content: &str) -> Result<Vec<SearchResult>>;
}
```

3. **Database Extension**:
```sql
-- New table for embeddings (future)
CREATE TABLE embeddings (
    post_id TEXT PRIMARY KEY,
    embedding BLOB NOT NULL,  -- Serialized Vec<f32>
    model_version TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (post_id) REFERENCES posts(id)
);

-- Index for efficient lookups
CREATE INDEX idx_embeddings_model ON embeddings(model_version);
```

4. **HistoryService Integration**:
```rust
impl HistoryService {
    /// Check if embeddings exist for posts
    pub async fn has_embeddings(&self, post_ids: &[String]) -> Result<HashMap<String, bool>>;
}
```

5. **PlurcastService Integration**:
```rust
impl PlurcastService {
    /// Access embedding service (optional feature)
    #[cfg(feature = "embeddings")]
    pub fn embeddings(&self) -> &EmbeddingService {
        &self.embeddings
    }
    
    /// Access search service (optional feature)
    #[cfg(feature = "embeddings")]
    pub fn search(&self) -> &SearchService {
        &self.search
    }
}
```

**Design Compatibility**: ✅ Current design supports embeddings with minimal changes
- Services are independent and can be added incrementally
- Database can be extended with new tables
- HistoryService already provides post retrieval
- Optional feature flags keep core lightweight
- EventBus can emit embedding progress events

### Additional Future Features

**Multi-Account Support** (Phase 3.4):
```rust
pub struct AccountService {
    db: Arc<Database>,
    credentials: Arc<CredentialManager>,
}

impl AccountService {
    pub async fn list_accounts(&self, platform: &str) -> Result<Vec<Account>>;
    pub async fn switch_account(&self, platform: &str, account_id: &str) -> Result<()>;
    pub async fn get_active_account(&self, platform: &str) -> Result<Account>;
}
```

**Design Compatibility**: ✅ Can be added as new service
- PostingService can accept optional account_id parameter
- Database schema can be extended with accounts table
- No changes to existing service APIs

**Media Attachments** (Post-1.0):
```rust
#[derive(Debug, Clone)]
pub struct PostRequest {
    pub content: String,
    pub platforms: Vec<String>,
    pub draft: bool,
    pub attachments: Vec<Attachment>,  // New field
}

pub struct Attachment {
    pub path: PathBuf,
    pub alt_text: Option<String>,
    pub mime_type: String,
}
```

**Design Compatibility**: ✅ Can extend existing types
- PostRequest is already a struct (easy to add fields)
- ValidationService can validate attachment sizes/types
- PostingService can handle upload logic

### Design Principles for Future-Proofing

1. **Service Independence**: Each service has clear boundaries and can be extended independently
2. **Shared State via Arc**: New services can share Database/Config without refactoring
3. **Extensible Types**: Request/Response structs can add optional fields without breaking changes
4. **EventBus Flexibility**: New event types can be added without modifying existing code
5. **Optional Features**: Heavy features (embeddings) can be feature-gated
6. **Database Migrations**: SQLx migrations support schema evolution

### Migration Path for Future Features

**Phase 4 (Scheduling)**:
1. Add QueueService to service layer
2. Extend PostingService with scheduling methods
3. Create `plur-queue` and `plur-send` CLI tools
4. No changes to existing services

**Post-1.0 (Embeddings)**:
1. Add `embeddings` feature flag to Cargo.toml
2. Create EmbeddingService and SearchService
3. Add embeddings table via migration
4. Create `plur-embed` and `plur-search` CLI tools
5. No changes to existing services

**Phase 3.4 (Multi-Account)**:
1. Add AccountService to service layer
2. Extend PostingService to accept account_id
3. Add accounts table via migration
4. Update `plur-setup` and `plur-creds` for multi-account
5. Minimal changes to existing services

## Open Questions

None - design is complete and ready for implementation.
