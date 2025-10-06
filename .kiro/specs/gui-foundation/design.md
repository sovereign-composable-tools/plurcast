# Design Document: Service Layer & Progressive UI Enhancement

## Overview

Phase 3 refactors Plurcast to support multiple user interfaces through a natural progression: **CLI → Service Layer → Terminal UI → Desktop GUI**. The key architectural principle is **direct library integration** - all interfaces call the service layer as regular Rust functions within a single process.

This approach contrasts with traditional GUI architectures that require IPC, HTTP servers, or message passing. By keeping everything in-process, we achieve:
- **Simplicity**: No serialization, no process management, no network stack
- **Performance**: Direct function calls, shared memory, no marshaling overhead
- **Type Safety**: Compile-time guarantees across all interfaces
- **Development Speed**: Write business logic once, use everywhere

**Implementation Strategy:**
1. **Phase 3.1**: Extract service layer from CLI (better code structure)
2. **Phase 3.2**: Build TUI with Ratatui (validates service design, adds rich terminal UX)
3. **Phase 3.3**: Build Tauri GUI (native desktop app for broader audience)
4. **Phase 3.4**: Add multi-account support (when needed)

Each phase builds on the previous, delivering incremental value without architectural rewrites.

## Architecture

### High-Level Component Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    User Interfaces                          │
│                (All in same process)                        │
├──────────────┬──────────────────┬──────────────────────────┤
│  plur-post   │    plur-tui      │    plurcast-gui          │
│  plur-history│  (Ratatui)       │    (Tauri)               │
│              │                  │                          │
│ Direct Calls │  Direct Calls    │  Direct Calls            │
└──────┬───────┴────────┬─────────┴────────┬─────────────────┘
       │                │                  │
       └────────────────┴──────────────────┘
                        │
       ┌────────────────▼────────────────┐
       │      Service Layer              │
       │   (libplurcast/service/)        │
       ├─────────────────────────────────┤
       │  • PlurcastService (facade)     │
       │  • PostingService               │
       │  • AccountService               │
       │  • DraftService                 │
       │  • HistoryService               │
       │  • ValidationService            │
       │  • EventBus (in-process)        │
       └────────────────┬────────────────┘
                        │
       ┌────────────────▼────────────────┐
       │   Core Library (Phase 1-2)      │
       ├─────────────────────────────────┤
       │  • Platform Abstraction         │
       │  • Database (SQLite + sqlx)     │
       │  • Configuration (TOML)         │
       │  • Error Types                  │
       └─────────────────────────────────┘
```

### Key Design Principles

1. **Single Process**: All interfaces run in the same process
2. **Direct Calls**: Service methods are just regular async Rust functions
3. **Shared State**: Database and config accessed via Arc references
4. **In-Process Events**: Callbacks, not message passing
5. **Progressive Enhancement**: Each interface adds features, none remove them

## Service Layer Components

### 1. PlurcastService (Facade)

**Purpose**: Single entry point for all service operations

```rust
pub struct PlurcastService {
    posting: PostingService,
    accounts: AccountService,
    drafts: DraftService,
    history: HistoryService,
    validation: ValidationService,
    events: Arc<EventBus>,
    config: Arc<RwLock<Config>>,
    db: Arc<Database>,
}

impl PlurcastService {
    /// Create new service from config path
    pub async fn new(config_path: Option<PathBuf>) -> Result<Self>;

    /// Get posting service
    pub fn posting(&self) -> &PostingService;

    /// Get account service
    pub fn accounts(&self) -> &AccountService;

    /// Get draft service
    pub fn drafts(&self) -> &DraftService;

    /// Get history service
    pub fn history(&self) -> &HistoryService;

    /// Get validation service
    pub fn validation(&self) -> &ValidationService;

    /// Get event bus for subscribing to events
    pub fn events(&self) -> Arc<EventBus>;

    /// Reload configuration
    pub async fn reload_config(&self) -> Result<()>;
}
```

**Design Notes:**
- Services share config and database via Arc
- Event bus is shareable across threads
- Config can be hot-reloaded without restart

### 2. PostingService

**Purpose**: Multi-platform posting with progress tracking

```rust
pub struct PostingService {
    db: Arc<Database>,
    config: Arc<RwLock<Config>>,
    events: Arc<EventBus>,
}

impl PostingService {
    /// Post content to selected platforms
    pub async fn post(
        &self,
        content: String,
        platforms: Vec<String>,
        account_ids: HashMap<String, String>, // platform -> account_id
        cancellation: Option<CancellationToken>,
    ) -> Result<PostingResult>;

    /// Post to all enabled platforms with default accounts
    pub async fn post_to_all(
        &self,
        content: String,
        cancellation: Option<CancellationToken>,
    ) -> Result<PostingResult>;

    /// Publish a draft by ID
    pub async fn publish_draft(
        &self,
        draft_id: String,
        platforms: Vec<String>,
        cancellation: Option<CancellationToken>,
    ) -> Result<PostingResult>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostingResult {
    pub post_id: String,
    pub results: Vec<PlatformResult>,
    pub overall_status: PostStatus,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformResult {
    pub platform: String,
    pub account_id: Option<String>,
    pub success: bool,
    pub platform_post_id: Option<String>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PostStatus {
    Posted,      // All platforms succeeded
    Partial,     // Some succeeded, some failed
    Failed,      // All failed
    Cancelled,   // User cancelled
}
```

**Event Flow:**
```rust
// Events emitted during posting:
1. PostStarted { post_id, platforms }
2. For each platform:
   - PlatformStarted { post_id, platform }
   - PlatformCompleted { post_id, platform, success, result/error }
3. PostCompleted { post_id, overall_status, results }
```

### 3. AccountService

**Purpose**: Multi-account management

```rust
pub struct AccountService {
    db: Arc<Database>,
    config: Arc<RwLock<Config>>,
    credential_store: Arc<dyn CredentialStore>,
    events: Arc<EventBus>,
}

impl AccountService {
    /// List all configured accounts
    pub async fn list_accounts(&self) -> Result<Vec<Account>>;

    /// Add a new account
    pub async fn add_account(
        &self,
        platform: String,
        name: String,
        credentials: Credentials,
    ) -> Result<String>; // Returns account_id

    /// Remove an account
    pub async fn remove_account(&self, account_id: &str) -> Result<()>;

    /// Test account authentication
    pub async fn test_account(&self, account_id: &str) -> Result<AuthStatus>;

    /// Get default account for platform
    pub async fn get_default_account(&self, platform: &str) -> Result<Option<Account>>;

    /// Set default account for platform
    pub async fn set_default_account(&self, platform: &str, account_id: &str) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub platform: String,
    pub name: String,
    pub handle: Option<String>,
    pub is_default: bool,
    pub auth_status: AuthStatus,
    pub created_at: i64,
    pub last_used: Option<i64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AuthStatus {
    Valid,
    Invalid,
    Expired,
    Untested,
}

pub enum Credentials {
    NostrKeys { nsec: String },
    MastodonToken { instance: String, token: String },
    BlueskyPassword { handle: String, password: String },
}
```

**Credential Storage:**
```rust
pub trait CredentialStore: Send + Sync {
    async fn store(&self, key: &str, value: &str) -> Result<()>;
    async fn retrieve(&self, key: &str) -> Result<Option<String>>;
    async fn delete(&self, key: &str) -> Result<()>;
}

// Implementations:
// 1. KeyringStore - Uses OS keyring (preferred)
// 2. EncryptedFileStore - Encrypted with user password (fallback)
// 3. PlainFileStore - Plain text with 600 permissions (last resort)
```

### 4. DraftService

**Purpose**: Draft management

```rust
pub struct DraftService {
    db: Arc<Database>,
    events: Arc<EventBus>,
}

impl DraftService {
    /// Create a new draft
    pub async fn create_draft(&self, content: String) -> Result<Draft>;

    /// Update draft content
    pub async fn update_draft(&self, draft_id: &str, content: String) -> Result<()>;

    /// List all drafts
    pub async fn list_drafts(&self) -> Result<Vec<Draft>>;

    /// Get a specific draft
    pub async fn get_draft(&self, draft_id: &str) -> Result<Option<Draft>>;

    /// Delete a draft
    pub async fn delete_draft(&self, draft_id: &str) -> Result<()>;

    /// Set target platforms for draft
    pub async fn set_draft_platforms(
        &self,
        draft_id: &str,
        platforms: Vec<String>,
    ) -> Result<()>;

    /// Auto-save draft (updates timestamp without event)
    pub async fn auto_save_draft(&self, draft_id: &str, content: String) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Draft {
    pub id: String,
    pub content: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub target_platforms: Vec<String>,
    pub metadata: Option<String>,
}
```

### 5. HistoryService

**Purpose**: Query and analyze posting history

```rust
pub struct HistoryService {
    db: Arc<Database>,
}

impl HistoryService {
    /// Query posts with filters
    pub async fn query(&self, filter: HistoryFilter) -> Result<HistoryPage>;

    /// Get a specific post with all platform results
    pub async fn get_post(&self, post_id: &str) -> Result<Option<PostHistory>>;

    /// Get statistics
    pub async fn get_statistics(&self, filter: HistoryFilter) -> Result<Statistics>;

    /// Retry failed platforms for a post
    pub async fn retry_post(
        &self,
        post_id: &str,
        platforms: Vec<String>,
    ) -> Result<PostingResult>;
}

#[derive(Debug, Clone)]
pub struct HistoryFilter {
    pub platform: Option<String>,
    pub status: Option<PostStatus>,
    pub since: Option<i64>,
    pub until: Option<i64>,
    pub search: Option<String>,
    pub offset: usize,
    pub limit: usize,
    pub sort: SortOrder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryPage {
    pub posts: Vec<PostHistory>,
    pub total_count: usize,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostHistory {
    pub id: String,
    pub content: String,
    pub created_at: i64,
    pub status: PostStatus,
    pub platforms: Vec<PlatformRecord>,
}
```

### 6. ValidationService

**Purpose**: Real-time content validation

```rust
pub struct ValidationService {
    config: Arc<RwLock<Config>>,
    cache: Arc<RwLock<LruCache<u64, ValidationResult>>>, // hash -> result
}

impl ValidationService {
    /// Validate content against target platforms
    pub async fn validate(
        &self,
        content: &str,
        platforms: &[String],
    ) -> Result<ValidationResult>;

    /// Get platform limits
    pub async fn get_platform_limits(&self, platform: &str) -> Result<PlatformLimits>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub platform_results: Vec<PlatformValidation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformValidation {
    pub platform: String,
    pub valid: bool,
    pub character_count: usize,
    pub character_limit: Option<usize>,
    pub remaining: Option<i32>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}
```

### 7. EventBus (In-Process)

**Purpose**: Progress and state change notifications

```rust
pub struct EventBus {
    subscribers: Arc<RwLock<Vec<Box<dyn EventHandler>>>>,
}

impl EventBus {
    pub fn new() -> Self;

    /// Subscribe to events
    pub async fn subscribe(&self, handler: Box<dyn EventHandler>);

    /// Emit an event to all subscribers
    pub async fn emit(&self, event: Event);
}

pub trait EventHandler: Send + Sync {
    fn handle(&self, event: &Event);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    // Posting events
    PostStarted {
        post_id: String,
        platforms: Vec<String>,
        timestamp: i64,
    },
    PlatformStarted {
        post_id: String,
        platform: String,
        timestamp: i64,
    },
    PlatformCompleted {
        post_id: String,
        platform: String,
        success: bool,
        result: Option<String>,
        error: Option<String>,
        timestamp: i64,
    },
    PostCompleted {
        post_id: String,
        overall_status: PostStatus,
        results: Vec<PlatformResult>,
        timestamp: i64,
    },

    // Account events
    AccountAdded { account_id: String, platform: String, timestamp: i64 },
    AccountRemoved { account_id: String, timestamp: i64 },

    // Draft events
    DraftCreated { draft_id: String, timestamp: i64 },
    DraftUpdated { draft_id: String, timestamp: i64 },
    DraftDeleted { draft_id: String, timestamp: i64 },

    // Config events
    ConfigReloaded { timestamp: i64 },
}
```

## Interface Implementations

### 1. CLI Refactoring

**Before (Phase 2):**
```rust
// plur-post/src/main.rs
async fn run(cli: Cli) -> Result<()> {
    let content = get_content(&cli)?;
    let config = Config::load()?;
    let db = Database::new(&db_path).await?;

    // ... platform initialization
    // ... posting logic embedded in main
    // ... result formatting

    std::process::exit(exit_code);
}
```

**After (Phase 3):**
```rust
// plur-post/src/main.rs
async fn run(cli: Cli) -> Result<()> {
    let content = get_content(&cli)?;
    let service = PlurcastService::new(None).await?;

    let result = service.posting()
        .post_to_all(content, None)
        .await?;

    // Format output (same as before)
    output_results(&result, &cli.format)?;

    // Map to exit codes
    std::process::exit(result.overall_status.to_exit_code());
}

impl PostStatus {
    fn to_exit_code(&self) -> i32 {
        match self {
            PostStatus::Posted => 0,
            PostStatus::Partial => 1,
            PostStatus::Failed => 1,
            PostStatus::Cancelled => 1,
        }
    }
}
```

**Key Changes:**
- Business logic moved to PostingService
- CLI becomes thin wrapper
- Exit code mapping stays in CLI
- Output formatting unchanged
- All tests pass unchanged

### 2. Terminal UI (Ratatui)

**Architecture:**

```rust
// plur-tui/src/main.rs

use ratatui::{prelude::*, widgets::*};
use libplurcast::PlurcastService;

struct App {
    service: PlurcastService,
    current_screen: Screen,
    composer: ComposerState,
    history: HistoryState,
    drafts: DraftState,
    accounts: AccountState,
}

enum Screen {
    Composer,
    History,
    Drafts,
    Accounts,
    Settings,
}

struct ComposerState {
    content: String,
    cursor_position: usize,
    selected_platforms: Vec<String>,
    validation: Option<ValidationResult>,
    posting_progress: Option<PostingProgress>,
}

impl App {
    async fn new() -> Result<Self> {
        let service = PlurcastService::new(None).await?;

        // Subscribe to events
        let events = service.events();
        events.subscribe(Box::new(TuiEventHandler::new())).await;

        Ok(Self {
            service,
            current_screen: Screen::Composer,
            // ... initialize states
        })
    }

    async fn handle_post(&mut self) -> Result<()> {
        self.composer.posting_progress = Some(PostingProgress::new());

        let result = self.service.posting()
            .post(
                self.composer.content.clone(),
                self.composer.selected_platforms.clone(),
                HashMap::new(),
                None,
            )
            .await?;

        self.composer.posting_progress = None;
        self.show_result(result);
        Ok(())
    }

    async fn update_validation(&mut self) {
        let result = self.service.validation()
            .validate(&self.composer.content, &self.composer.selected_platforms)
            .await;
        self.composer.validation = result.ok();
    }

    fn render(&self, frame: &mut Frame) {
        match self.current_screen {
            Screen::Composer => self.render_composer(frame),
            Screen::History => self.render_history(frame),
            Screen::Drafts => self.render_drafts(frame),
            Screen::Accounts => self.render_accounts(frame),
            Screen::Settings => self.render_settings(frame),
        }
    }
}
```

**UI Layout:**

```
┌─────────────────────────────────────────────────────────────┐
│ Plurcast TUI                          [Tab] Composer        │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ Compose Post:                                               │
│ ┌───────────────────────────────────────────────────────┐   │
│ │ Hello, decentralized world!                           │   │
│ │ _                                                     │   │
│ │                                                       │   │
│ └───────────────────────────────────────────────────────┘   │
│                                                             │
│ Platforms:                                                  │
│ [x] Nostr       ✓ Valid (32,000 chars left)                │
│ [x] Mastodon    ✓ Valid (470 chars left)                   │
│ [x] Bluesky     ✓ Valid (270 chars left)                   │
│                                                             │
│ [Ctrl+P] Post   [Ctrl+D] Save Draft   [Ctrl+H] History     │
└─────────────────────────────────────────────────────────────┘
```

**Event Handling:**

```rust
struct TuiEventHandler {
    tx: mpsc::Sender<Event>,
}

impl EventHandler for TuiEventHandler {
    fn handle(&self, event: &Event) {
        // Send to TUI's event loop for rendering
        self.tx.send(event.clone()).ok();
    }
}
```

**Key Features:**
- Real-time validation as you type (debounced)
- Platform selection with visual indicators
- Character count per platform
- Progress display during posting
- Keyboard shortcuts
- Mouse support
- Works over SSH

### 3. Tauri Desktop Application

**Project Structure:**
```
plurcast-gui/
├── src-tauri/          # Rust backend
│   ├── src/
│   │   ├── main.rs     # Tauri commands
│   │   └── lib.rs
│   └── Cargo.toml
└── src/                # Frontend (Svelte/React/Vue)
    ├── App.svelte
    ├── Composer.svelte
    ├── History.svelte
    └── main.js
```

**Backend (Tauri Commands):**

```rust
// plurcast-gui/src-tauri/src/main.rs

use libplurcast::{PlurcastService, service::*};
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::{Manager, State};

struct AppState {
    service: PlurcastService,
}

#[tauri::command]
async fn post_content(
    state: State<'_, Arc<Mutex<AppState>>>,
    content: String,
    platforms: Vec<String>,
) -> Result<PostingResult, String> {
    let state = state.lock().await;
    state.service.posting()
        .post(content, platforms, HashMap::new(), None)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn validate_content(
    state: State<'_, Arc<Mutex<AppState>>>,
    content: String,
    platforms: Vec<String>,
) -> Result<ValidationResult, String> {
    state.service.validation()
        .validate(&content, &platforms)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn list_drafts(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<Vec<Draft>, String> {
    let state = state.lock().await;
    state.service.drafts()
        .list_drafts()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn list_accounts(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<Vec<Account>, String> {
    let state = state.lock().await;
    state.service.accounts()
        .list_accounts()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn query_history(
    state: State<'_, Arc<Mutex<AppState>>>,
    filter: HistoryFilter,
) -> Result<HistoryPage, String> {
    let state = state.lock().await;
    state.service.history()
        .query(filter)
        .await
        .map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Initialize service
            let service = tauri::async_runtime::block_on(async {
                PlurcastService::new(None).await
            })?;

            // Subscribe to events and forward to frontend
            let window = app.get_window("main").unwrap();
            let events = service.events();

            tauri::async_runtime::spawn(async move {
                events.subscribe(Box::new(move |event| {
                    window.emit("plurcast-event", event).ok();
                })).await;
            });

            app.manage(Arc::new(Mutex::new(AppState { service })));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            post_content,
            validate_content,
            list_drafts,
            list_accounts,
            query_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Frontend (Svelte example):**

```svelte
<!-- plurcast-gui/src/Composer.svelte -->
<script lang="ts">
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import { onMount } from 'svelte';

let content = '';
let platforms = ['nostr', 'mastodon', 'bluesky'];
let validation: ValidationResult | null = null;
let posting = false;

// Debounced validation
let validationTimeout;
$: {
    clearTimeout(validationTimeout);
    validationTimeout = setTimeout(() => {
        if (content) updateValidation();
    }, 300);
}

async function updateValidation() {
    validation = await invoke('validate_content', {
        content,
        platforms,
    });
}

async function post() {
    posting = true;
    try {
        const result = await invoke('post_content', {
            content,
            platforms,
        });
        // Show success
    } catch (e) {
        // Show error
    } finally {
        posting = false;
    }
}

onMount(() => {
    // Listen for events
    listen('plurcast-event', (event) => {
        console.log('Event:', event.payload);
        // Update UI based on event
    });
});
</script>

<div class="composer">
    <textarea
        bind:value={content}
        placeholder="What's happening?"
    />

    <div class="platforms">
        {#each platforms as platform}
            <label>
                <input type="checkbox" bind:value={platform} />
                {platform}
                {#if validation}
                    <span class="validation">
                        {validation.platform_results.find(p => p.platform === platform)?.remaining} chars left
                    </span>
                {/if}
            </label>
        {/each}
    </div>

    <button on:click={post} disabled={posting}>
        {posting ? 'Posting...' : 'Post'}
    </button>
</div>
```

**Key Advantages:**
- Direct Rust calls (no IPC overhead)
- Small binary (~5-10MB)
- Fast startup
- Native performance
- Type-safe frontend/backend communication

## Data Models

### Database Schema Additions

```sql
-- Accounts table (new)
CREATE TABLE accounts (
    id TEXT PRIMARY KEY,
    platform TEXT NOT NULL,
    name TEXT NOT NULL,
    handle TEXT,
    is_default INTEGER DEFAULT 0,
    credentials_ref TEXT,      -- Key in credential store
    created_at INTEGER NOT NULL,
    last_used INTEGER,
    metadata TEXT
);

CREATE INDEX idx_accounts_platform ON accounts(platform);
CREATE INDEX idx_accounts_default ON accounts(platform, is_default);

-- Update post_records to track account used
ALTER TABLE post_records ADD COLUMN account_id TEXT;
CREATE INDEX idx_post_records_account ON post_records(account_id);

-- Indexes for performance
CREATE INDEX idx_posts_created_at ON posts(created_at);
CREATE INDEX idx_posts_status ON posts(status);
CREATE INDEX idx_post_records_platform ON post_records(platform);
```

## Performance Considerations

### In-Process Advantages

**Direct Calls vs IPC:**
- No serialization overhead
- No process spawning
- No socket/pipe communication
- Shared memory (Arc)
- Compile-time type checking

**Benchmarks (estimated):**
- Service call latency: <1μs (direct call)
- Event propagation: <10μs (in-process)
- Validation: <100ms (includes platform checks)
- History query: <50ms (database I/O)

### Caching Strategy

```rust
// Validation cache
struct CachedValidation {
    cache: LruCache<u64, ValidationResult>, // content hash -> result
    max_size: usize,
}

// Platform instances cache
struct PlatformCache {
    instances: HashMap<String, Arc<dyn Platform>>,
}
```

## Migration from Phase 2

### Code Migration Steps

1. **Create service layer** (libplurcast/src/service/)
2. **Extract posting logic** from plur-post/src/main.rs
3. **Extract history logic** from plur-history/src/main.rs
4. **Update CLI binaries** to use services
5. **Verify all tests pass**

### User Migration

**Zero action required:**
- Existing configs work unchanged
- Database schema backwards compatible
- CLI behavior identical

## Success Metrics

### Functional
- ✅ Service layer handles all business logic
- ✅ CLI uses services without regressions
- ✅ TUI provides rich interactive experience
- ✅ Tauri app works with direct calls
- ✅ All interfaces share same data

### Performance
- Service overhead: <1ms per operation
- TUI launch: <500ms
- Tauri launch: <2s
- Validation: <100ms
- History load: <500ms

### Code Quality
- Service layer test coverage >80%
- Zero CLI regressions
- Clean architecture boundaries
- Comprehensive documentation

## Conclusion

Phase 3's progressive enhancement approach delivers value at each step while maintaining architectural simplicity. By using direct library integration instead of IPC or HTTP, we achieve:

- **Simpler code**: No serialization, no process management
- **Better performance**: Direct calls, shared memory
- **Type safety**: Compile-time guarantees
- **Faster development**: Write once, use everywhere
- **Better UX**: CLI, TUI, and GUI all feel native

This architecture positions Plurcast for future growth while keeping the codebase maintainable and the user experience excellent across all interfaces.
