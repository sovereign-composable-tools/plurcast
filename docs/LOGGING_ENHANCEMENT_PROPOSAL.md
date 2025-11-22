# Logging Enhancement Proposal for Plurcast

## Current State

Plurcast uses `tracing` with basic text-based logging to stderr. No JSON or structured logging is currently available.

**Current Features:**
- Environment variable filtering (`RUST_LOG`)
- Verbosity control (`--verbose` flag)
- Per-binary logging initialization
- Basic text output to stderr

**Limitations:**
- No JSON output for machine parsing
- No structured fields for filtering
- Duplicate logging setup across binaries
- Limited observability for production deployments

---

## Proposed Improvements

### Phase 1: Add JSON Logging Support

#### 1.1 Update Dependencies

**Current** (`Cargo.toml`):
```toml
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

**Proposed**:
```toml
tracing-subscriber = { version = "0.3", features = [
    "env-filter",
    "json",           # JSON formatting
    "fmt",            # Pretty formatting
    "ansi",           # Colored output
    "tracing-log",    # Compatibility with log crate
] }
```

#### 1.2 Add Global Flag for Log Format

Add to each CLI binary:

```rust
#[derive(Parser)]
struct Cli {
    // ... existing fields ...

    /// Log format (text, json, pretty)
    #[arg(
        long = "log-format",
        env = "PLURCAST_LOG_FORMAT",
        default_value = "text",
        help = "Log output format: text, json, or pretty"
    )]
    log_format: LogFormat,

    /// Log level filter (error, warn, info, debug, trace)
    #[arg(
        long = "log-level",
        env = "PLURCAST_LOG_LEVEL",
        default_value = "info",
        help = "Minimum log level to display"
    )]
    log_level: String,
}

#[derive(Clone, ValueEnum)]
enum LogFormat {
    /// Human-readable text output
    Text,
    /// Machine-parseable JSON (one JSON object per line)
    Json,
    /// Pretty-printed with colors (for development)
    Pretty,
}
```

#### 1.3 Centralized Logging Configuration

Create `libplurcast/src/logging.rs`:

```rust
//! Centralized logging configuration for all Plurcast binaries
//!
//! Provides consistent logging setup with support for:
//! - Text, JSON, and pretty-printed output
//! - Environment variable configuration
//! - Per-module log level filtering
//! - Request correlation IDs

use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    /// Human-readable text output (no colors)
    Text,
    /// Machine-parseable JSON (one JSON object per line)
    Json,
    /// Pretty-printed with colors (for development)
    Pretty,
}

impl std::str::FromStr for LogFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "text" => Ok(LogFormat::Text),
            "json" => Ok(LogFormat::Json),
            "pretty" => Ok(LogFormat::Pretty),
            _ => Err(format!("Invalid log format: {}", s)),
        }
    }
}

pub struct LoggingConfig {
    pub format: LogFormat,
    pub level: String,
    pub verbose: bool,
}

impl LoggingConfig {
    pub fn new(format: LogFormat, level: String, verbose: bool) -> Self {
        Self { format, level, verbose }
    }

    /// Initialize logging with the configured settings
    ///
    /// # Panics
    ///
    /// Panics if the logging subscriber has already been initialized
    pub fn init(&self) {
        let filter = if self.verbose {
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("debug"))
        } else {
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(&self.level))
        };

        match self.format {
            LogFormat::Json => {
                // JSON output for machine parsing (production/monitoring)
                tracing_subscriber::fmt()
                    .json()
                    .with_env_filter(filter)
                    .with_writer(std::io::stderr)
                    .with_current_span(true)
                    .with_span_list(true)
                    .flatten_event(true)
                    .with_target(true)
                    .with_line_number(true)
                    .with_file(true)
                    .init();
            }
            LogFormat::Pretty => {
                // Pretty output with colors for development
                tracing_subscriber::fmt()
                    .pretty()
                    .with_env_filter(filter)
                    .with_writer(std::io::stderr)
                    .with_target(true)
                    .with_line_number(true)
                    .with_file(true)
                    .init();
            }
            LogFormat::Text => {
                // Plain text output for piping/basic usage
                tracing_subscriber::fmt()
                    .with_env_filter(filter)
                    .with_writer(std::io::stderr)
                    .with_target(false) // Less verbose for end users
                    .init();
            }
        }
    }
}

/// Initialize logging with default settings
///
/// Respects RUST_LOG and PLURCAST_LOG_FORMAT environment variables
pub fn init_default() {
    let format = std::env::var("PLURCAST_LOG_FORMAT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(LogFormat::Text);

    let level = std::env::var("PLURCAST_LOG_LEVEL")
        .unwrap_or_else(|_| "info".to_string());

    LoggingConfig::new(format, level, false).init();
}
```

#### 1.4 Update Each Binary

**Before** (`plur-post/src/main.rs`):
```rust
if cli.verbose {
    tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_writer(std::io::stderr)
        .init();
} else {
    tracing_subscriber::fmt()
        .with_env_filter("error,nostr_sdk=off")
        .with_writer(std::io::stderr)
        .init();
}
```

**After**:
```rust
use libplurcast::logging::{LoggingConfig, LogFormat};

// Parse log format from CLI or env
let log_format = cli.log_format
    .or_else(|| std::env::var("PLURCAST_LOG_FORMAT")
        .ok()
        .and_then(|s| s.parse().ok()))
    .unwrap_or(LogFormat::Text);

let logging = LoggingConfig::new(
    log_format,
    cli.log_level.clone(),
    cli.verbose,
);
logging.init();
```

---

### Phase 2: Add Structured Logging

#### 2.1 Add Spans for Request Tracking

```rust
use tracing::{info_span, instrument};

#[instrument(skip(service))]
async fn post_with_progress(service: &PlurcastService, request: PostRequest) -> Result<PostResponse> {
    let span = info_span!("posting",
        post_id = %request.post_id,
        platforms = ?request.platforms,
        draft = request.draft,
    );

    let _enter = span.enter();

    info!("Starting post operation");
    let response = service.posting().post(request).await?;
    info!(success = response.overall_success, "Post operation completed");

    Ok(response)
}
```

#### 2.2 Add Structured Fields

**Instead of**:
```rust
info!("Posted to {} platforms", platforms.len());
```

**Use**:
```rust
info!(
    platform_count = platforms.len(),
    platforms = ?platforms,
    "Posted to multiple platforms"
);
```

**JSON Output Example**:
```json
{
  "timestamp": "2025-11-17T12:34:56.789Z",
  "level": "INFO",
  "target": "plur_post",
  "fields": {
    "message": "Posted to multiple platforms",
    "platform_count": 3,
    "platforms": ["nostr", "mastodon", "ssb"]
  },
  "span": {
    "post_id": "550e8400-e29b-41d4-a716-446655440000",
    "draft": false
  }
}
```

---

### Phase 3: Add Advanced Features

#### 3.1 Request Correlation IDs

```rust
use uuid::Uuid;

// In each request handler:
let request_id = Uuid::new_v4();
let span = info_span!("request", request_id = %request_id);
let _enter = span.enter();

// All logs within this span will include request_id
```

#### 3.2 Performance Metrics

```rust
use tracing::instrument;

#[instrument(skip(db))]
async fn save_post(db: &Database, post: &Post) -> Result<()> {
    let start = std::time::Instant::now();

    db.create_post(post).await?;

    let elapsed = start.elapsed();
    info!(
        duration_ms = elapsed.as_millis(),
        post_id = %post.id,
        "Post saved to database"
    );

    Ok(())
}
```

**JSON Output**:
```json
{
  "timestamp": "2025-11-17T12:34:56.790Z",
  "level": "INFO",
  "fields": {
    "message": "Post saved to database",
    "duration_ms": 42,
    "post_id": "550e8400-e29b-41d4-a716-446655440000"
  }
}
```

#### 3.3 Error Context

```rust
use tracing::error;

match platform.post(&post).await {
    Ok(post_id) => {
        info!(
            platform = platform.name(),
            platform_post_id = %post_id,
            "Post published successfully"
        );
    }
    Err(e) => {
        error!(
            platform = platform.name(),
            error = %e,
            error_type = ?std::error::Error::source(&e),
            "Failed to publish post"
        );
    }
}
```

---

## Usage Examples

### Text Format (Default)
```bash
$ plur-post "Hello world"
# Output to stderr:
# 2025-11-17T12:34:56Z INFO Starting post operation
# 2025-11-17T12:34:57Z INFO Posted to nostr: note1abc...
```

### JSON Format (Production)
```bash
$ plur-post "Hello world" --log-format json
# Output to stderr (one JSON object per line):
# {"timestamp":"2025-11-17T12:34:56.789Z","level":"INFO","message":"Starting post operation","post_id":"550e..."}
# {"timestamp":"2025-11-17T12:34:57.123Z","level":"INFO","message":"Posted to nostr","platform":"nostr","post_id":"note1abc..."}
```

### Pretty Format (Development)
```bash
$ plur-post "Hello world" --log-format pretty --verbose
# Output to stderr (colored, pretty-printed):
#   2025-11-17T12:34:56.789Z  INFO plur_post:23: Starting post operation
#     at plur_post/src/main.rs:234
#     in posting with post_id=550e..., platforms=["nostr"]
#
#   2025-11-17T12:34:57.123Z  INFO plur_post::platforms::nostr:45: Posted to nostr
#     at libplurcast/src/platforms/nostr.rs:312
#     with platform_post_id=note1abc...
```

### Environment Variable Configuration
```bash
# Use JSON logging for all commands
export PLURCAST_LOG_FORMAT=json
export PLURCAST_LOG_LEVEL=debug

$ plur-post "Test"
$ plur-history --limit 10
$ plur-send --once

# All will use JSON logging with debug level
```

### Integration with Log Aggregation

**Fluentd/Fluent Bit**:
```bash
plur-post "Deploy complete" --log-format json 2>&1 | fluent-bit -c fluent-bit.conf
```

**Datadog**:
```bash
plur-send --log-format json 2>&1 | datadog-agent
```

**ELK Stack**:
```bash
plur-post "Test" --log-format json 2>&1 | filebeat -e
```

---

## Migration Plan

### Step 1: Add Dependencies (Low Risk)
- Update `Cargo.toml` with JSON feature
- Add `libplurcast/src/logging.rs`
- No breaking changes

### Step 2: Update Binaries (Medium Risk)
- Add `--log-format` flag to each binary
- Replace inline logging init with `LoggingConfig`
- **Backward compatible**: defaults to current behavior

### Step 3: Add Structured Logging (Low Risk)
- Gradually add structured fields to existing logs
- Use spans for request tracking
- **Additive changes only**

### Step 4: Documentation (Low Risk)
- Update README.md with logging examples
- Add LOGGING.md guide
- Update CLI help text

---

## Benefits

### For Development
- **Pretty formatting** with colors for easy debugging
- **Structured fields** make logs searchable
- **Request correlation** tracks operations across components

### For Production
- **JSON output** integrates with log aggregation systems
- **Structured data** enables powerful queries
- **Performance metrics** built into logs
- **Error context** improves debugging

### For Operations
- **Environment variables** simplify configuration
- **Centralized setup** ensures consistency
- **Machine-parseable** logs enable automation

---

## Estimated Effort

| Phase | Effort | Impact |
|-------|--------|--------|
| Phase 1: JSON Support | 4-6 hours | High |
| Phase 2: Structured Logging | 3-4 hours | Medium |
| Phase 3: Advanced Features | 4-6 hours | Medium |
| **Total** | **11-16 hours** | **High** |

---

## Testing Strategy

1. **Unit Tests**: Test LoggingConfig initialization
2. **Integration Tests**: Verify JSON output is valid
3. **Manual Testing**: Test each log format with real binaries
4. **Performance**: Ensure JSON logging doesn't impact performance

---

## Alternatives Considered

### Option 1: Use `env_logger` instead of `tracing`
- **Pros**: Simpler, less overhead
- **Cons**: No structured logging, no spans, less powerful
- **Verdict**: ❌ Rejected - tracing is more powerful

### Option 2: Add OpenTelemetry support
- **Pros**: Industry standard, powerful tracing
- **Cons**: Heavy dependency, overkill for CLI tool
- **Verdict**: 🤔 Consider for future if needed

### Option 3: Custom JSON logger
- **Pros**: Full control
- **Cons**: Reinventing the wheel, maintenance burden
- **Verdict**: ❌ Rejected - tracing-subscriber is proven

---

## Recommendation

✅ **Implement Phase 1** (JSON support) immediately
- Low risk, high value
- Enables production monitoring
- Backward compatible

🔄 **Implement Phase 2** (structured logging) gradually
- Improve logs as you touch each module
- Add spans to new code first

⏳ **Implement Phase 3** (advanced features) as needed
- Add metrics when performance becomes a concern
- Add correlation IDs if debugging distributed issues

---

**End of Logging Enhancement Proposal**
