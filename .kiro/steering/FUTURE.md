# Plurcast: Future Plans & Extensibility

**Related Documentation**:
- [Vision](./VISION.md) - Philosophy and design principles
- [Architecture](./ARCHITECTURE.md) - Technical implementation details
- [Roadmap](./ROADMAP.md) - Development phases and progress
- [Tools](./TOOLS.md) - Tool specifications and usage

---

## Advanced Features (Stretch Goals)

### Vector Embeddings & Semantic Search

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

### UI Extensions

**Progressive Enhancement Philosophy**: CLI → Service Layer → TUI → Tauri GUI

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

### Advanced UX Features

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

## Non-Goals

- Real-time feed reading (use platform clients)
- Content recommendation algorithms
- Social graph analysis
- Centralized web service (local-first only)
- Mobile apps (Phase 1-3 focus on desktop/terminal)
- Complex IPC or HTTP layers (direct library integration instead)

---

**Version**: 0.1.0-alpha
**Last Updated**: 2025-10-05
**Status**: Active Development - Phase 1 (Foundation) ~85% Complete
**Future Architecture**: Phase 3 will introduce service layer and progressive UI enhancement (CLI → TUI → Tauri GUI) via direct library integration. See `.kiro/specs/gui-foundation/` for details.
