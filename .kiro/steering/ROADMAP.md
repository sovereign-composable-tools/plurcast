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

### Phase 2: Multi-Platform (Alpha Release) - **~90% COMPLETE**
- [x] Platform abstraction trait (prepare for multi-platform)
- [x] Mastodon integration (`megalodon` library) - **Tested and stable**
- [x] Bluesky integration (`atrium-api` library) - **Implemented, needs testing**
- [x] Multi-platform posting in `plur-post`
- [x] `plur-history` basic queries
- [x] Secure credential storage system
- [x] Interactive setup wizard (`plur-setup`)
- [x] Credential management tool (`plur-creds`)
- [ ] Comprehensive Bluesky testing (stretch goal)
- [ ] Alpha release to community

**Status**: Nostr and Mastodon are tested and stable. Bluesky is implemented but marked as lower priority for testing.

**Next Steps**: Phase 3 (Service Layer & UI Enhancement) or Phase 4 (Scheduling).

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

### Phase 2 (Multi-Platform Alpha) - Current Status: **~90% Complete**
- ✅ Post to Nostr and Mastodon from command line (tested and stable)
- ✅ View posting history with search/filters (`plur-history`)
- ✅ Multi-platform posting with platform abstraction trait
- ✅ Secure credential storage (keyring, encrypted, plain text)
- ✅ Interactive setup wizard and credential management
- 🚧 Bluesky support (implemented, needs testing - stretch goal)
- ⏳ Community alpha release (pending)

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

**Version**: 0.2.0-alpha
**Last Updated**: 2025-10-11
**Status**: Active Development - Phase 2 (Multi-Platform) ~90% Complete
**Stable Platforms**: Nostr, Mastodon
**Next Milestone**: Phase 3 (Service Layer & UI) or Phase 4 (Scheduling)
