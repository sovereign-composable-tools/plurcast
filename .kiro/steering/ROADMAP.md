# Plurcast: Development Roadmap

**Related Documentation**:
- [Vision](./VISION.md) - Philosophy and design principles
- [Architecture](./ARCHITECTURE.md) - Technical implementation details
- [Tools](./TOOLS.md) - Tool specifications and usage
- [Future](./FUTURE.md) - Extensibility and future plans

---

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

### Phase 6: Enhancement (Post-1.0) - **NOT STARTED**
- [ ] Semantic search (optional embeddings)
- [ ] Media attachment support
- [ ] Reply/thread handling
- [ ] Analytics and statistics

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

---

**Version**: 0.1.0-alpha
**Last Updated**: 2025-10-05
**Status**: Active Development - Phase 1 (Foundation) ~85% Complete
**Next Milestone**: Phase 2 (Multi-Platform Alpha Release)
