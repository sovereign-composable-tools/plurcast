# Service Layer Architecture

## Overview

The Plurcast service layer provides a clean, testable API for business logic that can be consumed by multiple interfaces (CLI, TUI, GUI, API) without code duplication.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                     PlurcastService                          │
│                    (Facade Pattern)                          │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │ PostingService│  │HistoryService│  │ DraftService │     │
│  │              │  │              │  │              │     │
│  │ • post()     │  │ • list_posts()│  │ • create()   │     │
│  │ • retry()    │  │ • get_post() │  │ • update()   │     │
│  │ • draft()    │  │ • get_stats()│  │ • publish()  │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
│                                                              │
│  ┌──────────────┐  ┌──────────────┐                        │
│  │ValidationSvc │  │  EventBus    │                        │
│  │              │  │              │                        │
│  │ • validate() │  │ • emit()     │                        │
│  │ • is_valid() │  │ • subscribe()│                        │
│  └──────────────┘  └──────────────┘                        │
├─────────────────────────────────────────────────────────────┤
│            Shared Resources (Arc<T>)                         │
│  ┌──────────────┐              ┌──────────────┐            │
│  │   Database   │              │    Config    │            │
│  └──────────────┘              └──────────────┘            │
└─────────────────────────────────────────────────────────────┘
```

## Design Principles

### 1. Facade Pattern

`PlurcastService` acts as a single entry point, coordinating specialized sub-services:

- **PostingService**: Multi-platform posting with retry logic
- **HistoryService**: Query and analyze post history
- **DraftService**: Manage draft posts (CRUD + publishing)
- **ValidationService**: Real-time content validation
- **EventBus**: Progress event distribution

### 2. Shared State via Arc

All sub-services share the same `Arc<Database>` and `Arc<Config>` instances, enabling:
- Efficient concurrent access without duplication
- Consistent state across services
- Thread-safe operations

### 3. Event-Driven Progress Tracking

The EventBus enables:
- Real-time progress updates
- Multiple subscribers (UI, logging, monitoring)
- Non-blocking event emission
- Optional subscriptions (no overhead if unused)

## Getting Started

### Basic Usage

```rust
use libplurcast::service::{PlurcastService, posting::PostRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize service
    let service = PlurcastService::new().await?;

    // Post content
    let request = PostRequest {
        content: "Hello decentralized world!".to_string(),
        platforms: vec!["nostr".to_string(), "mastodon".to_string()],
        draft: false,
    };

    let response = service.posting().post(request).await?;
    
    println!("Posted to {} platforms", response.results.len());
    
    Ok(())
}
```

### With Custom Configuration

```rust
use libplurcast::{Config, service::PlurcastService};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load custom config
    let config = Config::load()?;
    
    // Initialize with custom config
    let service = PlurcastService::from_config(config).await?;
    
    // Use service...
    
    Ok(())
}
```

## Service Layer Components

### PostingService

Handles multi-platform posting with automatic retry logic and progress tracking.

#### Features:
- Concurrent posting to multiple platforms
- Automatic retry with exponential backoff
- Event emission for progress tracking
- Database recording of all results
- Draft mode support

#### Example: Post with Progress Tracking

```rust
use libplurcast::service::{PlurcastService, posting::PostRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = PlurcastService::new().await?;
    
    // Subscribe to events
    let mut events = service.subscribe();
    
    // Start event listener in background
    tokio::spawn(async move {
        while let Ok(event) = events.recv().await {
            match event {
                libplurcast::service::events::Event::PostingStarted { post_id, .. } => {
                    println!("⏳ Posting started: {}", post_id);
                }
                libplurcast::service::events::Event::PostingProgress { platform, .. } => {
                    println!("📤 Posting to {}...", platform);
                }
                libplurcast::service::events::Event::PostingCompleted { post_id, .. } => {
                    println!("✅ Posting completed: {}", post_id);
                }
                libplurcast::service::events::Event::PostingFailed { post_id, error } => {
                    println!("❌ Posting failed: {} - {}", post_id, error);
                }
            }
        }
    });
    
    // Post content
    let request = PostRequest {
        content: "Hello from Plurcast!".to_string(),
        platforms: vec!["nostr".to_string(), "mastodon".to_string()],
        draft: false,
    };
    
    let response = service.posting().post(request).await?;
    
    for result in response.results {
        if result.success {
            println!("✓ {}: {}", result.platform, result.post_id.unwrap());
        } else {
            println!("✗ {}: {}", result.platform, result.error.unwrap());
        }
    }
    
    Ok(())
}
```

### ValidationService

Provides real-time content validation before posting.

#### Features:
- Platform-specific character limits
- Content size validation (100KB max)
- Empty/whitespace detection
- Multi-platform validation in a single call

#### Example: Pre-Post Validation

```rust
use libplurcast::service::{PlurcastService, validation::ValidationRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = PlurcastService::new().await?;
    
    let request = ValidationRequest {
        content: "My post content".to_string(),
        platforms: vec!["nostr".to_string(), "bluesky".to_string()],
    };
    
    let response = service.validation().validate(request);
    
    if response.valid {
        println!("✅ Content is valid for all platforms");
    } else {
        println!("❌ Validation failed:");
        for result in response.results {
            if !result.valid {
                println!("  {}: {:?}", result.platform, result.errors);
            }
        }
    }
    
    Ok(())
}
```

### HistoryService

Query and analyze post history with flexible filtering.

#### Features:
- Filter by platform, date range, status
- Full-text search
- Pagination support
- Statistics generation

#### Example: Query Recent Posts

```rust
use libplurcast::service::{PlurcastService, history::HistoryQuery};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = PlurcastService::new().await?;
    
    let query = HistoryQuery {
        platform: Some("nostr".to_string()),
        status: None,
        since: None,
        until: None,
        search: Some("rust".to_string()),
        limit: Some(10),
        offset: None,
    };
    
    let posts = service.history().list_posts(query).await?;
    
    for post in posts {
        println!("{}: {}", post.post.id, post.post.content);
        for record in post.records {
            println!("  {} - {}", record.platform, 
                if record.success { "✓" } else { "✗" });
        }
    }
    
    Ok(())
}
```

### DraftService

Manage draft posts with full CRUD operations and publishing workflow.

#### Features:
- Create, read, update, delete drafts
- List all drafts
- Publish drafts (delegates to PostingService)
- Automatic draft deletion on successful publish

#### Example: Draft-to-Publish Workflow

```rust
use libplurcast::service::PlurcastService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = PlurcastService::new().await?;
    
    // Create a draft
    let draft = service.draft()
        .create("Draft content".to_string())
        .await?;
    
    println!("Draft created: {}", draft.id);
    
    // Update the draft
    let updated = service.draft()
        .update(&draft.id, "Updated content".to_string())
        .await?;
    
    println!("Draft updated: {}", updated.content);
    
    // Publish the draft
    let response = service.draft()
        .publish(&draft.id, vec!["nostr".to_string()])
        .await?;
    
    if response.overall_success {
        println!("✅ Draft published successfully!");
    }
    
    Ok(())
}
```

## Event System

### Event Types

```rust
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
```

### Subscribing to Events

```rust
use libplurcast::service::PlurcastService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = PlurcastService::new().await?;
    
    // Subscribe to events
    let mut receiver = service.subscribe();
    
    // Multiple subscribers are supported
    let mut receiver2 = service.subscribe();
    
    // Listen for events
    tokio::spawn(async move {
        while let Ok(event) = receiver.recv().await {
            println!("Event received: {:?}", event);
        }
    });
    
    Ok(())
}
```

## Testing Patterns

### Unit Testing Services

```rust
use libplurcast::service::{PlurcastService, posting::PostRequest};
use tempfile::TempDir;

#[tokio::test]
async fn test_posting_service() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    let config = libplurcast::Config {
        database: libplurcast::config::DatabaseConfig {
            path: db_path.to_str().unwrap().to_string(),
        },
        // ... configure platforms for testing
        ..Default::default()
    };
    
    let service = PlurcastService::from_config(config).await.unwrap();
    
    // Test posting in draft mode (no actual posting)
    let request = PostRequest {
        content: "Test content".to_string(),
        platforms: vec![],
        draft: true,
    };
    
    let response = service.posting().post(request).await.unwrap();
    assert!(response.overall_success);
}
```

### Integration Testing

See `libplurcast/tests/service_integration.rs` for comprehensive integration test examples.

## Migration Guide for UI Development

### From CLI to TUI/GUI

The service layer makes it easy to build new interfaces:

1. **Initialize service once**
   ```rust
   let service = Arc::new(PlurcastService::new().await?);
   ```

2. **Share across UI components**
   ```rust
   // In your UI framework (e.g., Ratatui, Iced)
   struct AppState {
       service: Arc<PlurcastService>,
   }
   ```

3. **Subscribe to events for real-time updates**
   ```rust
   let mut events = service.subscribe();
   
   // In your UI update loop
   if let Ok(event) = events.try_recv() {
       // Update UI based on event
       update_progress_bar(event);
   }
   ```

4. **Use service methods directly**
   ```rust
   // From button click handler
   async fn on_post_clicked(&mut self) {
       let request = PostRequest {
           content: self.text_input.value(),
           platforms: self.selected_platforms.clone(),
           draft: false,
       };
       
       let response = self.service.posting().post(request).await?;
       
       // Update UI with results
       self.show_results(response);
   }
   ```

### Best Practices for UI Integration

1. **Use Arc for shared service**
   - Services are cheap to clone (internal Arc)
   - Share one service instance across UI

2. **Subscribe to events early**
   - Set up event listeners before operations
   - Handle events in UI update loop

3. **Use async/await properly**
   - Don't block UI thread
   - Spawn tasks for long operations

4. **Handle errors gracefully**
   - All service methods return `Result`
   - Display user-friendly error messages

## Performance Considerations

### Concurrent Operations

All services support concurrent operations:

```rust
use futures::future::join_all;

let service = PlurcastService::new().await?;

// Post to multiple platforms concurrently (automatic in post())
let request = PostRequest {
    content: "Concurrent post".to_string(),
    platforms: vec!["nostr".to_string(), "mastodon".to_string()],
    draft: false,
};

// Platforms are posted to concurrently inside post()
let response = service.posting().post(request).await?;
```

### Database Connection Pooling

The Database struct uses SQLite with connection pooling automatically.

### Memory Efficiency

- `Arc<T>` for shared resources (no duplication)
- Streaming results for large queries (pagination)
- Event bus uses bounded channels (configurable capacity)

## Error Handling

All service methods return `Result<T, PlurcastError>`:

```rust
use libplurcast::error::PlurcastError;

match service.posting().post(request).await {
    Ok(response) => {
        // Handle success
    }
    Err(PlurcastError::Authentication(e)) => {
        // Handle authentication error (exit code 2)
    }
    Err(PlurcastError::InvalidInput(msg)) => {
        // Handle validation error (exit code 3)
    }
    Err(e) => {
        // Handle other errors (exit code 1)
    }
}
```

## Future Enhancements

The service layer is designed to support future features:

- **Scheduled Posts**: Framework in place (PostStatus::Pending, scheduled_at field)
- **Post Templates**: Easily added to DraftService
- **Analytics**: HistoryService provides foundation
- **Multi-User**: Add user_id to Post model
- **Webhooks**: EventBus enables webhook triggers

## API Stability

The service layer API is considered **stable** for:
- PlurcastService (facade)
- PostingService
- HistoryService
- DraftService
- ValidationService
- Event types

Internal implementation details may change, but public APIs will remain backward-compatible.

## Additional Resources

- [API Documentation](https://docs.rs/libplurcast) (run `cargo doc --open`)
- [Integration Tests](../libplurcast/tests/service_integration.rs)
- [CLI Implementation Examples](../plur-post/src/main.rs, ../plur-history/src/main.rs)
- [Project README](../README.md)
