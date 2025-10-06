# Plurcast: Design Document

**Project Name**: Plurcast  
**Version**: Alpha (0.1.0)  
**Philosophy**: Cast to many - Unix tools for the decentralized social web  
**Architecture**: Unix philosophy - small, focused tools that compose

## Project Vision

Plurcast is a collection of Unix command-line tools for scheduled cross-posting to decentralized social media platforms. Following the Unix philosophy, each tool does one thing well, communicating through standard streams and files. Built with mature, open-source Rust libraries.

## Core Principles

### Unix Philosophy
- **Do one thing well**: Each binary handles a single responsibility
- **Text streams**: Universal interface between components
- **Composability**: Tools combine via pipes and standard Unix utilities
- **Silence is golden**: Only output what's needed, errors to stderr
- **Exit codes**: Meaningful status codes for scripting
- **Agent-friendly**: LLM agents can operate the tools just like humans

### Agent-Aware Design Philosophy

Plurcast is built with an **agent-aware philosophy** - by following Unix principles, the tools are inherently accessible to both humans and AI agents:

**Why Unix Philosophy Enables AI Agents:**
- **Predictable interfaces**: Standard input/output streams are easy for agents to manipulate
- **Composable workflows**: Agents can chain commands just like shell scripts
- **Clear contracts**: `--help` text and exit codes provide discoverable interfaces
- **Stateless operations**: Each tool call is independent, easier to reason about
- **JSON output modes**: Machine-readable formats via `--format json`
- **No hidden state**: Configuration in files, not in-memory sessions

**Agent Capabilities:**
```bash
# Agent can discover capabilities
plur-post --help | agent-parse

# Agent can compose workflows  
agent: plur-history --since yesterday --format json | 
       jq '.[] | select(.platform=="nostr")' |
       plur-export --format markdown

# Agent can handle errors via exit codes
if ! plur-post "content"; then
  agent: retry with --platform nostr only
fi
```

**Human-Agent Parity:**
- What a human can do via CLI, an agent can automate
- Agents discover features through help text and man pages
- Tools respond identically whether called by human or agent
- No special "API mode" - Unix tools ARE the API

This agent-aware design means Plurcast works seamlessly with:
- Claude Code and other coding assistants
- Shell script automation
- CI/CD pipelines
- Custom agent workflows
- Future agentic tools

The Unix philosophy isn't just good design - it's **agent-native design**.

### Decentralized Values
- **Local-first**: All data stored locally in SQLite
- **Self-contained**: No external services required for core functionality
- **User ownership**: Complete control over data and configuration
- **Platform independence**: Easy import/export, no lock-in

## Architecture Overview

### Tool Suite

```
plurcast/
├── plur-post          # Post content to one or more platforms
├── plur-queue         # Schedule posts for later
├── plur-send          # Daemon that processes queue
├── plur-history       # Query posting history
├── plur-import        # Import from platform exports
├── plur-export        # Export posts to various formats
└── libplurcast        # Shared library (internal)
```

### Data Storage

**Location**: `~/.local/share/plurcast/`
- `posts.db` - SQLite database
- `embeddings.db` - Vector embeddings (optional, future)

**Configuration**: `~/.config/plurcast/`
- `config.toml` - Platform credentials and preferences
- `*.key` - Platform-specific authentication files

## Platform Support (Alpha Release)

### Nostr
**Library**: `nostr-sdk` v0.35+ (rust-nostr)
- **Status**: Alpha, actively maintained with strong ecosystem adoption
- **Features**: Complete protocol implementation, relay management, cryptographic key handling
- **Maturity**: High adoption in Nostr ecosystem, production-ready core
- **Key Capabilities**: 
  - Multiple relay support
  - Event signing and verification
  - NIP (Nostr Implementation Possibilities) support
  - Key management (hex/bech32)

### Bluesky
**Library**: `atrium-api` v0.24+ (atrium-rs)
- **Status**: Active development, protocol stabilizing
- **Features**: Complete AT Protocol implementation via XRPC
- **Maturity**: Growing ecosystem, protocol reached stability in 2024-2025
- **Key Capabilities**:
  - AT Protocol (authenticated transfer protocol)
  - DID-based identity
  - Lexicon schema support
  - PDS (Personal Data Server) federation

### Mastodon
**Library**: `megalodon` v0.14+
- **Status**: Stable, well-maintained
- **Features**: ActivityPub/Mastodon API with multi-platform Fediverse support
- **Maturity**: Battle-tested across multiple Fediverse implementations
- **Key Capabilities**:
  - Supports Mastodon, Pleroma, Friendica, Firefish, GoToSocial, Akkoma
  - Unified API across platforms
  - OAuth authentication
  - Media upload support

## Core Components

### Database Schema

```sql
-- Posts authored by user
CREATE TABLE posts (
    id TEXT PRIMARY KEY,              -- UUIDv4
    content TEXT NOT NULL,
    created_at INTEGER NOT NULL,      -- Unix timestamp
    scheduled_at INTEGER,             -- NULL = post immediately
    status TEXT DEFAULT 'pending',    -- pending, posted, failed
    metadata TEXT                     -- JSON: tags, attachments, etc.
);

-- Platform-specific post records
CREATE TABLE post_records (
    id INTEGER PRIMARY KEY,
    post_id TEXT NOT NULL,
    platform TEXT NOT NULL,           -- nostr, mastodon, bluesky
    platform_post_id TEXT,            -- Platform's ID for the post
    posted_at INTEGER,
    success INTEGER DEFAULT 0,        -- 0 or 1
    error_message TEXT,
    FOREIGN KEY (post_id) REFERENCES posts(id)
);

-- Platform configurations
CREATE TABLE platforms (
    name TEXT PRIMARY KEY,
    enabled INTEGER DEFAULT 1,
    config TEXT                       -- JSON: instance URLs, relay lists, etc.
);
```

### Configuration Format

```toml
# ~/.config/plurcast/config.toml

[database]
path = "~/.local/share/plurcast/posts.db"

[nostr]
enabled = true
keys_file = "~/.config/plurcast/nostr.keys"
relays = [
    "wss://relay.damus.io",
    "wss://nos.lol",
    "wss://relay.nostr.band"
]

[mastodon]
enabled = true
instance = "mastodon.social"
token_file = "~/.config/plurcast/mastodon.token"

[bluesky]
enabled = true
handle = "user.bsky.social"
auth_file = "~/.config/plurcast/bluesky.auth"

[defaults]
# Which platforms to post to by default (can override with --platform flag)
platforms = ["nostr", "mastodon", "bluesky"]
```

## Tool Specifications

### plur-post

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

### plur-queue

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
**Exit codes**: Same as sky-post

### plur-send

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

### plur-history

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

### plur-import

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

### plur-export

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

## Technology Stack

### Core Dependencies

```toml
[dependencies]
# Platform clients - mature, open-source libraries
nostr-sdk = "0.35"           # Nostr protocol (rust-nostr)
megalodon = "0.14"           # Mastodon/Fediverse (ActivityPub)
atrium-api = "0.24"          # Bluesky AT Protocol

# Data & persistence
sqlx = { version = "0.8", features = ["sqlite", "runtime-tokio"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"

# Async runtime
tokio = { version = "1", features = ["full"] }

# CLI & utilities
clap = { version = "4.5", features = ["derive"] }
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.10", features = ["v4", "serde"] }

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"
```

### Project Structure

```
plurcast/
├── Cargo.toml
├── README.md
├── LICENSE
├── src/
│   ├── lib.rs              # Shared library code
│   ├── db.rs               # Database operations
│   ├── config.rs           # Configuration management
│   ├── platforms/
│   │   ├── mod.rs
│   │   ├── nostr.rs        # Nostr client wrapper
│   │   ├── mastodon.rs     # Mastodon client wrapper
│   │   └── bluesky.rs      # Bluesky client wrapper
│   └── types.rs            # Shared types
├── bin/
│   ├── plur-post.rs
│   ├── plur-queue.rs
│   ├── plur-send.rs
│   ├── plur-history.rs
│   ├── plur-import.rs
│   └── plur-export.rs
├── migrations/
│   └── 001_initial.sql     # SQLx migrations
└── tests/
    └── integration.rs
```

## Development Phases

### Phase 1: Foundation (Alpha MVP) - **~85% COMPLETE**
- [x] Project naming and architecture design
- [x] Core database schema (SQLite with migrations)
- [x] Configuration system (TOML parsing, XDG paths, environment variables)
- [x] Basic `plur-post` for single platform (Nostr)
- [x] Authentication handling (Nostr keys: hex/bech32 formats)
- [x] Error types and exit codes (0, 1, 2, 3)
- [x] Unix philosophy implementation (stdin/stdout, pipes, composability)
- [x] Agent-friendly features (JSON output, help text, exit codes)
- [x] Draft mode (`--draft` flag)
- [x] Platform selection via CLI (`--platform` flag)
- [x] Verbose logging (`--verbose` flag)
- [x] Comprehensive README documentation
- [ ] Expanded test coverage (basic tests exist)
- [ ] Man pages (optional)
- [ ] Shell completion scripts (optional)

**Status**: Foundation is solid. Ready to move to Phase 2.

### Phase 2: Multi-Platform (Alpha Release) - **NOT STARTED**
- [ ] Platform abstraction trait (prepare for multi-platform)
- [ ] Mastodon integration (`megalodon` library)
- [ ] Bluesky integration (`atrium-api` library)
- [ ] Multi-platform posting in `plur-post` (infrastructure ready)
- [ ] `plur-history` basic queries (new binary)
- [ ] Alpha release to community

**Next Steps**: Create platform trait, implement Mastodon and Bluesky clients.

### Phase 3: Service Layer & Progressive UI Enhancement (Beta) - **PLANNED**

**Philosophy**: Build from what exists - CLI → Service Layer → TUI → Tauri GUI

#### Phase 3.1: Service Layer Extraction
- [ ] Extract business logic from CLI binaries into `libplurcast/service/`
- [ ] Create `PlurcastService` facade with sub-services:
  - PostingService (multi-platform posting)
  - AccountService (multi-account management)
  - DraftService (draft CRUD operations)
  - HistoryService (enhanced queries, retry, stats)
  - ValidationService (real-time content validation)
  - EventBus (in-process progress events)
- [ ] Refactor CLI tools to use service layer (zero behavioral changes)
- [ ] Comprehensive service layer testing

#### Phase 3.2: Terminal UI (Ratatui)
- [ ] Build `plur-tui` using Ratatui framework
- [ ] Interactive composer with real-time validation
- [ ] History browser with filtering and search
- [ ] Draft manager (create, edit, publish, delete)
- [ ] Keyboard and mouse support
- [ ] SSH-friendly (works over terminal)
- [ ] Direct service layer integration (no IPC)

#### Phase 3.3: Desktop GUI (Tauri)
- [ ] Build `plurcast-gui` using Tauri
- [ ] Native desktop app (Windows, macOS, Linux)
- [ ] Direct Rust integration (no IPC overhead)
- [ ] Modern UI with Svelte/React/Vue frontend
- [ ] Real-time validation and progress
- [ ] Small binary size (<15MB)
- [ ] Event system via Tauri's built-in events

#### Phase 3.4: Multi-Account Support (Optional)
- [ ] Multiple accounts per platform
- [ ] OS keyring integration for credentials
- [ ] Account switcher in TUI/GUI
- [ ] Default account per platform

**Key Architectural Decision**: All interfaces (CLI, TUI, GUI) call service layer as direct Rust functions within a single process. No IPC, no HTTP servers, no JSON-RPC complexity. Just clean function calls.

**See**: `.kiro/specs/gui-foundation/` for complete specification

### Phase 4: Scheduling (Stable) - **NOT STARTED**
- [ ] `plur-queue` implementation
- [ ] `plur-send` daemon
- [ ] Systemd service files
- [ ] Rate limiting per platform

### Phase 5: Data Portability (Stable) - **NOT STARTED**
- [ ] `plur-import` for major platforms
- [ ] `plur-export` with multiple formats
- [ ] Migration utilities
- [ ] 1.0 stable release

### Phase 5: Enhancement (Post-1.0) - **NOT STARTED**
- [ ] Semantic search (optional embeddings)
- [ ] Media attachment support
- [ ] Reply/thread handling
- [ ] Analytics and statistics

### Phase 6: Advanced Features (Stretch Goals) - **NOT STARTED**
- [ ] **Vector Embeddings & Semantic Search**
  - Local embedding generation (`plur-embed`)
  - Semantic post search (`plur-search`)
  - Similar post discovery (`plur-similar`)  
  - Creative writing assistance (`plur-suggest`)
  - Local thesaurus/dictionary integration
  - Serves consciousness rather than hijacking it
- [ ] **UI Extensions**
  - TUI with `ratatui` (`plur-tui`)
  - Optional GUI with `Tauri` or `iced` (`plur-gui`)
  - Local web server (`plur-server`)
  - All built on same Unix core
- [ ] **Advanced UX**
  - Configuration wizard (`plur-init --guided`)
  - Interactive prompts with validation
  - Progress indicators and colorized output
  - Comprehensive help system and man pages

## Extensibility & Future Architecture

### Vector Embeddings (Stretch Goal)

**Philosophy**: Semantic search should **serve consciousness** - helping users understand their own patterns rather than creating algorithmic dependency.

**Implementation**:
```bash
# Unix-native embedding tools
plur-embed          # Generate embeddings for posts
plur-search         # Semantic search over history  
plur-similar        # Find similar posts
plur-suggest        # Creative writing assistance
```

**Technical Stack**:
- **`candle`**: Hugging Face's Rust ML framework
- **`ort`**: ONNX Runtime bindings
- **`fastembed`**: Fast, lightweight embeddings
- Local models only, no external APIs

**Use Cases**:
```bash
# Discover your own patterns
echo "protocol design" | plur-search --format json

# Find similar historical posts
plur-similar --post-id abc123

# Subtle creative assistance while composing
plur-suggest --draft "Starting new thoughts on..."
```

**Data Storage**:
```
~/.local/share/plurcast/
├── posts.db           # Main database
├── embeddings.db      # Vector embeddings (optional)
└── models/           # Local embedding models
    └── sentence-transformer/
```

### UI Extensibility (Phase 3 Architecture)

**Progressive Enhancement Philosophy**: CLI → Service Layer → TUI → GUI

The architecture enables multiple interfaces through **direct library integration** - all UIs call the service layer as regular Rust functions within a single process. No IPC, no HTTP servers, no complexity.

**Plurcast Service-Based Architecture** (Phase 3):
```
┌─────────────────────────────────────────────────────────┐
│              User Interfaces                            │
│           (All in same process)                         │
├──────────────┬──────────────────┬──────────────────────┤
│  CLI         │    TUI           │    GUI               │
│ (plur-*)     │  (plur-tui)      │  (plurcast-gui)      │
│              │  Ratatui         │  Tauri               │
│ Direct Calls │  Direct Calls    │  Direct Calls        │
└──────┬───────┴────────┬─────────┴────────┬─────────────┘
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
       │  • Platform Abstraction         │
       │  • Database (SQLite + sqlx)     │
       │  • Configuration (TOML)         │
       └─────────────────────────────────┘
```

**Implementation Benefits**:

1. **CLI (plur-post, plur-history)**
   - Refactored to use service layer
   - Zero behavioral changes
   - Exit codes mapped from service results
   - Output formatting stays in CLI

2. **TUI (plur-tui)** - Terminal UI
   - Built with Ratatui
   - Rich interactive terminal interface
   - Direct service layer calls
   - SSH-friendly, works over terminal
   - Real-time validation and progress

3. **GUI (plurcast-gui)** - Desktop
   - Built with Tauri
   - Direct Rust integration (no IPC)
   - Svelte/React/Vue frontend
   - Small binary (<15MB)
   - Native performance

**Key Architectural Decisions**:

- **Single Process**: All interfaces run in same process
- **Direct Calls**: Service methods are regular async Rust functions
- **Shared State**: Database and config via Arc references
- **In-Process Events**: Callbacks, not message passing
- **No IPC Complexity**: No JSON-RPC, no HTTP servers, no process management

**Why This Works**:
- Service layer is framework-agnostic
- All types are Serialize/Deserialize
- Tauri auto-serializes Rust → TypeScript
- CLI maps results to exit codes
- TUI subscribes to events via channels
- GUI uses Tauri's event system

This is **simpler, faster, and more maintainable** than traditional GUI architectures.

### UX Enhancements (Unix-Compatible)

**Configuration Wizard**:
```bash
plur-init --guided     # Interactive setup

🌟 Welcome to Plurcast! Let's set up your platforms.

1. Nostr Configuration:
   Generate new keys? [Y/n]: y
   ✓ Generated keys: npub1abc...
   
2. Mastodon Configuration:
   Instance URL: mastodon.social
   ✓ Opening browser for OAuth...
   
✓ Configuration saved
Next: echo "Hello!" | plur-post
```

**Smart Validation**:
```bash
plur-post "Too long..." --platform nostr
❌ Error: Post exceeds Nostr's 280 character limit
   Current: 450 characters  
   Suggestion: Use --trim or split into thread
```

**Progress Indicators** (when TTY detected):
```bash
plur-send --verbose
⏳ Processing queue...
✓ Posted to nostr (note1abc...)
✓ Posted to mastodon (12345)
✓ 3/3 platforms successful
```

**Context-Aware Output**:
- TTY detected → colors, progress bars, emoji
- Pipe detected → plain text for scripting
- `--json` flag → machine-readable output
- `--help` flag → comprehensive guidance

**Best of Both Worlds**:
- Unix composability preserved
- Human-friendly when interactive
- Agent-friendly always

### Why SQLite?
- Zero-configuration
- Fast for local workloads
- ACID guarantees
- Built-in full-text search
- Easy backup (single file)

### Why Separate Binaries?
- Unix composability
- Test components independently
- Users install only what they need
- Clear separation of concerns
- Easier to understand and maintain

### Why Not a Monolithic CLI?
- Subcommands create complexity
- Harder to compose with Unix tools
- Violates "do one thing well"
- Each tool has focused documentation

### Configuration Philosophy
- XDG Base Directory compliance
- Environment variables for overrides
- Sensible defaults
- No required configuration for basic use

## Security Considerations

- Credentials stored in separate files (not in main config)
- File permissions: 600 for sensitive files
- No credentials in database
- Clear documentation about key management
- Support for system keyring (future)

## Future Considerations

### Optional Features (Not Phase 1)
- Semantic search with embeddings
- LLM-powered hashtag suggestions
- Analytics dashboard (TUI or web)
- Team/multi-user support
- Content drafting with templates
- Automated content threading
- Image/media optimization

### Ecosystem Integration
- Shell completion scripts
- Integration with RSS readers
- Bookmark managers (Pinboard, etc.)
- Note-taking tools (Obsidian plugins)

## Success Metrics

### Phase 1 (Alpha MVP) - Current Status: **~85% Complete**

**Achieved:**
- ✅ Post to Nostr from command line
- ✅ Clear, Unix-style documentation with comprehensive help system
- ✅ Zero external dependencies for core features
- ✅ Compiles to static binary
- ✅ Works on Linux, macOS, BSD (Windows via WSL)
- ✅ Clean separation between binaries following Unix philosophy
- ✅ Agent-discoverable interfaces (help text, JSON output, exit codes)
- ✅ Human-friendly UX when interactive (verbose logging, validation)
- ✅ Draft mode for saving without posting
- ✅ Platform selection via CLI flags
- ✅ Database persistence with SQLite

**Remaining for Phase 1:**
- ⏳ Expanded test coverage
- ⏳ Man pages (optional)
- ⏳ Shell completion scripts (optional)

### Phase 2 (Multi-Platform Alpha) - Target Goals:
- Post to all three platforms (Nostr, Bluesky, Mastodon) from command line
- View posting history with search/filters (`plur-history`)
- Multi-platform posting with platform abstraction trait
- Community alpha release

### Phase 3 (Service Layer & Progressive UI) - Target Goals:
- **Service Layer**: Extract business logic, create clean API
  - PostingService, AccountService, DraftService, HistoryService, ValidationService
  - In-process event system for progress tracking
  - CLI refactored to use services (zero behavioral changes)
- **Terminal UI (plur-tui)**: Rich interactive terminal interface
  - Composer with real-time validation
  - History browser with filtering
  - Draft manager
  - Direct service layer integration
- **Desktop GUI (plurcast-gui)**: Native desktop application
  - Tauri-based with modern frontend
  - Direct Rust calls (no IPC)
  - Small binary (<15MB)
  - Real-time progress and validation
- **Multi-Account Support** (optional): Multiple accounts per platform with OS keyring

### Post-1.0 Vision:
- Optional semantic search with local embeddings
- Configuration wizard for easy onboarding
- Works equally well for humans, scripts, and AI agents
- Serves consciousness rather than hijacking it
- Progressive enhancement: CLI → TUI → GUI, all on same foundation

## Non-Goals

- Real-time feed reading (use platform clients)
- Content recommendation algorithms
- Social graph analysis
- Centralized web service (local-first only)
- Mobile apps (Phase 1-3 focus on desktop/terminal)
- Complex IPC or HTTP layers (direct library integration instead)

## Licensing & Community

**License**: MIT or Apache 2.0 (TBD)
**Repository**: GitHub (plurcast/plurcast)
**Community**: Focus on users who value data ownership and Unix principles

## Name Etymology

**Plurcast** = Latin *plur(i)* (many) + *cast* (broadcast)

"Cast to many" - perfectly captures the essence of cross-posting to multiple decentralized platforms while maintaining a clean, Unix-friendly name.

## Design Philosophy Summary

Plurcast embodies three interlocking principles:

1. **Unix Philosophy**: Tools that do one thing well, compose via text streams, work for both humans and agents
2. **Decentralized Values**: Local-first, user-owned data, platform independence  
3. **Consciousness-Serving Technology**: Reveals patterns rather than manipulates, enhances awareness rather than creates dependency

This creates software that:
- Humans can learn and compose
- Agents can discover and automate  
- Serves authentic expression over algorithmic control
- Extends gracefully from CLI to TUI to GUI
- Works equally well in 2025 and 2035

---

**Version**: 0.1.0-alpha
**Last Updated**: 2025-10-05
**Status**: Active Development - Phase 1 (Foundation) ~85% Complete
**Next Milestone**: Phase 2 (Multi-Platform Alpha Release)
**Future Architecture**: Phase 3 will introduce service layer and progressive UI enhancement (CLI → TUI → Tauri GUI) via direct library integration. See `.kiro/specs/gui-foundation/` for details.