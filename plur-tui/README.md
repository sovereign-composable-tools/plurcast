# plur-tui - Terminal UI for Plurcast

**Status**: 🚧 In Development - Milestone 1 (Composer MVP)

Interactive terminal interface for posting to decentralized social platforms. Built with Ratatui and following functional programming principles.

## Architecture

### Functional Programming Principles

plur-tui follows strict FP principles:

- **Immutable State**: All state is immutable; transitions create new state values
- **Pure Reducer**: `(State, Action) -> State` with no side effects
- **No Business Logic**: All business logic delegated to `PlurcastService`
- **Event-Driven**: UI events + Service events merged into action stream

### Structure

```
plur-tui/
├── src/
│   ├── main.rs           # Entry point and event loop
│   ├── error.rs          # TUI-specific errors
│   ├── app/              # Application architecture
│   │   ├── mod.rs        # Module exports
│   │   ├── actions.rs    # Action definitions (what can happen)
│   │   ├── state.rs      # State structures (what is true)
│   │   └── reducer.rs    # Pure reducer function
│   ├── ui/               # Rendering (pure view functions)
│   │   ├── mod.rs
│   │   ├── composer.rs   # Composer screen
│   │   ├── history.rs    # History browser (M2)
│   │   └── drafts.rs     # Draft manager (M3)
│   ├── services.rs       # Service layer adapters
│   └── keymap.rs         # Keybinding definitions
└── tests/                # Integration tests
    ├── test_app_boot.rs
    ├── test_composer_state.rs
    ├── test_posting_flow.rs
    └── test_keymap.rs
```

### State Management

```
     User Input
         ↓
     [Action] ──────────────┐
         ↓                   │
     [Reducer] ←─── Side Effects (I/O, Service calls)
    (Pure Fn)               │
         ↓                   │
    [New State]             │
         ↓                   │
    [Render] ───────────────┘
    (Pure Fn)
```

## Progress

### ✅ Completed (Task 2)

- [x] Workspace integration
- [x] Error handling (`error.rs`)
- [x] Action system (`app/actions.rs`)
- [x] Immutable state (`app/state.rs`)
- [x] Pure reducer (`app/reducer.rs`)
- [x] Reducer tests (4 tests, all passing)
- [x] Compilation verified

### 🚧 Next Steps

- [ ] Service layer adapter (`services.rs`)
- [ ] Event loop implementation
- [ ] Terminal setup (SSH-friendly)
- [ ] Composer screen UI
- [ ] Validation integration
- [ ] Posting flow with progress
- [ ] Global keybindings
- [ ] Error overlay

## Development

### Build

```bash
cargo build -p plur-tui
```

### Run

```bash
cargo run -p plur-tui
```

### Test

```bash
cargo test -p plur-tui
```

## Design Principles

### SSH-Friendly Defaults

- Mouse disabled by default (enable with `m`)
- Colors honor `NO_COLOR` and `PLUR_TUI_NO_COLOR`
- Unicode symbols have ASCII fallback
- Alt screen with fallback for basic terminals

### Keyboard-First

- Global keybindings:
  - `q`: Quit
  - `F1`: Help
  - `F2`: History
  - `F3`: Drafts
  - `m`: Toggle mouse
  
- Composer:
  - `Ctrl+S`: Post
  - `Ctrl+L`: Clear (after success)
  - `Esc`: Dismiss errors/overlays

### Environment Variables

- `NO_COLOR`: Disable colors
- `PLUR_TUI_NO_COLOR`: Disable colors (Plurcast-specific)
- `PLUR_TUI_TICK_MS`: Tick rate (default: 100ms)
- `PLUR_TUI_NO_ALT`: Disable alt screen

## Testing Strategy

Tests focus on state management without pixel rendering:

1. **Reducer Tests**: Pure function behavior
2. **State Tests**: Immutability and defaults
3. **Keymap Tests**: Action mapping
4. **Flow Tests**: Multi-step workflows

Mock services via `FakePorts` for deterministic testing.

## License

MIT OR Apache-2.0 (dual-licensed)
