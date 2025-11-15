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

### Phase 2: Multi-Platform (Alpha Release) - **COMPLETE** ✅
- [x] Platform abstraction trait (prepare for multi-platform)
- [x] Mastodon integration (`megalodon` library) - **Tested and stable**
- [x] Multi-platform posting in `plur-post`
- [x] Multi-account support - **Tested and stable**
- [x] `plur-history` basic queries
- [x] Secure credential storage system (OS keyring)
- [x] Interactive setup wizard (`plur-setup`)
- [x] Credential management tool (`plur-creds`)
- [x] Shared test account easter egg (Nostr)
- [x] Alpha release to community

**Status**: Nostr and Mastodon are tested and stable. Multi-account system fully operational.

**Platform Decision**: Removed Bluesky (centralized, banned test accounts). Adding SSB (Secure Scuttlebutt) in Phase 3 - truly peer-to-peer and philosophically aligned.

**Next Steps**: Phase 3 (SSB Integration).

### Phase 3: SSB Integration (Peer-to-Peer) - **COMPLETE** ✅

**Philosophy**: Add truly peer-to-peer social networking via Secure Scuttlebutt (SSB)

**Why SSB?**
- Truly peer-to-peer (no servers, no blockchain)
- Offline-first architecture
- Local-first data storage
- Community-driven, no corporate control
- Philosophically aligned with Plurcast values

#### Phase 3.1: Basic SSB Support (MVP)
- [ ] SSB platform implementation using `kuska-ssb`
- [ ] Connect to local sbot (SSB server)
- [ ] Post to local SSB feed
- [ ] Message signing and verification
- [ ] Basic error handling
- [ ] Configuration via config.toml
- [ ] Integration tests

**Prerequisites**: User has sbot installed and running

#### Phase 3.2: Enhanced Integration
- [ ] SSB key generation via `plur-setup`
- [ ] Credential management via `plur-creds`
- [ ] Multi-account support for SSB
- [ ] Better error messages and status feedback
- [ ] User documentation
- [ ] Example configurations

#### Phase 3.3: History & Import
- [ ] Query SSB feed history
- [ ] Import SSB messages into Plurcast database
- [ ] Export to SSB format
- [ ] `plur-history --platform ssb`
- [ ] `plur-import ssb`
- [ ] `plur-export --format ssb`

#### Phase 3.4: Server Management (Optional)
- [ ] Auto-start sbot if not running
- [ ] Process lifecycle management
- [ ] Health checks and monitoring
- [ ] Systemd integration
- [ ] Docker support

**See**: `.kiro/specs/ssb-integration/design.md` for complete specification

### Phase 4: Service Layer & Desktop GUI (Beta) - **IN PROGRESS**

**Philosophy**: Build from what exists - CLI → Service Layer → Tauri GUI

**Note**: We are NOT building a TUI. The CLI tools are sufficient for terminal users, and a GUI provides better UX for visual composition and analytics.

#### Phase 4.1: Service Layer Extraction - **COMPLETE** ✅
- [x] Extract business logic from CLI binaries into `libplurcast/service/`
- [x] Create `PlurcastService` facade with sub-services:
  - PostingService (multi-platform posting)
  - DraftService (draft CRUD operations)
  - HistoryService (enhanced queries, retry, stats)
  - ValidationService (real-time content validation)
  - EventBus (in-process progress events)
- [x] Refactor CLI tools to use service layer (zero behavioral changes)
  - plur-post: Uses PostingService, ValidationService ✅
  - plur-history: Uses HistoryService ✅
  - plur-creds: Infrastructure tool (correct to use platforms directly) ✅
  - plur-setup: Infrastructure tool (correct to use platforms directly) ✅
  - plur-import: Data utility (correct to use database directly) ✅
  - plur-export: Data utility (correct to use database directly) ✅
- [x] Comprehensive service layer testing (42 service tests passing)

#### Phase 4.2: Desktop GUI (Tauri)
- [ ] Build `plurcast-gui` using Tauri
- [ ] Native desktop app (Windows, macOS, Linux)
- [ ] Direct Rust integration (no IPC overhead)
- [ ] Modern UI with Svelte/React/Vue frontend
- [ ] Real-time validation and progress
- [ ] Small binary size (<15MB)
- [ ] Event system via Tauri's built-in events
- [ ] Interactive composer with real-time validation
- [ ] History browser with filtering and search
- [ ] Draft manager (create, edit, publish, delete)
- [ ] Analytics dashboard

**Key Architectural Decision**: Both CLI and GUI call service layer as direct Rust functions within a single process. No IPC, no HTTP servers, no JSON-RPC complexity. Just clean function calls.

**See**: `.kiro/specs/gui-foundation/` for complete specification

### Phase 5: Scheduling (Stable) - **IN PROGRESS**

**Philosophy**: Unix-style scheduling with separate tools for queuing and sending. Daemon managed by systemd, human-friendly natural language scheduling.

- [ ] Database migrations and core logic
- [ ] `plur-post --schedule` enhancement (natural language time parsing)
- [ ] `plur-queue` CLI tool (list, cancel, reschedule, stats)
- [ ] `plur-send` daemon (polling, posting, rate limiting)
- [ ] Systemd service integration
- [ ] Rate limiting per platform
- [ ] Comprehensive testing

**See**: `.kiro/specs/scheduling/` for complete specification and task breakdown

### Phase 6: Data Portability (Stable) - **NOT STARTED**
- [ ] `plur-import` for major platforms (Nostr, Mastodon, SSB)
- [ ] `plur-export` with multiple formats
- [ ] Migration utilities
- [ ] 1.0 stable release

### Phase 7: Enhancement (Post-1.0) - **NOT STARTED**
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

### Phase 2 (Multi-Platform Alpha) - Current Status: **COMPLETE** ✅
- ✅ Post to Nostr and Mastodon from command line (tested and stable)
- ✅ View posting history with search/filters (`plur-history`)
- ✅ Multi-platform posting with platform abstraction trait
- ✅ Multi-account support (tested and stable)
- ✅ Secure credential storage (OS keyring)
- ✅ Interactive setup wizard and credential management
- ✅ Shared test account easter egg (Nostr)
- ✅ Community alpha release

**Platform Decision**: Removed Bluesky (centralized, banned test accounts). Replaced with SSB (truly peer-to-peer).

### Phase 3 (SSB Integration) - Target Goals:
- **Basic SSB Support**: Post to local SSB feed via sbot
- **Enhanced Integration**: Key generation, credential management, multi-account
- **History & Import**: Query SSB history, import/export
- **Server Management** (optional): Auto-start sbot, lifecycle management

### Phase 4 (Service Layer & Progressive UI) - Target Goals:
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

**Version**: 0.3.0-alpha2
**Last Updated**: 2025-11-15
**Status**: Active Development - Phase 3 Complete, Phase 4.1 Complete, Phase 5 In Progress
**Stable Platforms**: Nostr, Mastodon
**Experimental Platform**: SSB (Secure Scuttlebutt) - local posting works, network replication limited
**Removed Platform**: Bluesky (centralized, banned test accounts)
**Current Milestone**: Phase 5 (Post Scheduling) - Design complete, implementing database migrations
**Future Milestones**: Phase 4.2 (Desktop GUI with Tauri), Phase 6 (Data Portability)
