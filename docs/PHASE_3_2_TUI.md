# Phase 3.2: Terminal UI (plur-tui)

**Status**: 🚧 In Progress - Task 2 Complete  
**Milestone**: M1 - Composer MVP  
**Started**: 2025-10-25

## Overview

Interactive terminal UI for Plurcast using Ratatui. Following functional programming principles with immutable state and pure reducers.

## Architecture

### Functional Programming Principles

```
┌─────────────────────────────────────────┐
│         Functional Architecture         │
├─────────────────────────────────────────┤
│                                         │
│  User Input → Action → Reducer → State │
│       ↑          ↓         ↑        ↓   │
│       └─── Side Effects ───┘   Render  │
│           (Service Layer)               │
└─────────────────────────────────────────┘
```

**Key Principles**:
- **Immutability**: All state is immutable, transitions create new values
- **Purity**: Reducer is `(State, Action) -> State` with zero side effects
- **Separation**: Business logic lives in `PlurcastService`, not in UI
- **Testability**: State transitions are pure functions, easily testable

### Components

1. **Actions** (`app/actions.rs`)
   - Enum of all possible events
   - Immutable data structures
   - Examples: `ComposerInputChanged`, `ComposerPostRequested`, `Quit`

2. **State** (`app/state.rs`)
   - Immutable application state tree
   - `AppState` → `ComposerState`, `StatusBarState`, `UiConfig`
   - Defaults configured from environment

3. **Reducer** (`app/reducer.rs`)
   - Pure function: `fn reduce(AppState, Action) -> AppState`
   - No I/O, no side effects, deterministic
   - Keybindings mapped to actions here

4. **Services** (future: `services.rs`)
   - Thin adapter to `PlurcastService`
   - Handles I/O and business logic
   - Triggers actions based on results

5. **Event Loop** (future: `main.rs`)
   - Merges UI events (keyboard, mouse) + Service events (progress)
   - Dispatches actions to reducer
   - Renders new state

6. **UI** (future: `ui/`)
   - Pure rendering functions: `fn render(State) -> Frame`
   - No state mutation in rendering

## Progress

### ✅ Task 1: Approval
- [x] Scope approved: Composer MVP first, then History/Drafts

### ✅ Task 2: Workspace Scaffolding (Complete)
- [x] Added `plur-tui` to workspace members
- [x] Created `plur-tui/Cargo.toml` with dependencies:
  - `ratatui` 0.26
  - `crossterm` 0.27
  - `tui-textarea` 0.4
  - `crossbeam-channel` 0.5
  - `libplurcast` (service layer)
- [x] Created directory structure:
  - `src/app/` - Core architecture
  - `src/ui/` - Rendering (future)
  - `tests/` - Integration tests (future)
- [x] Implemented core architecture:
  - `error.rs` - TUI-specific error types
  - `app/actions.rs` - Action enum (98 lines)
  - `app/state.rs` - Immutable state (156 lines)
  - `app/reducer.rs` - Pure reducer with tests (332 lines)
- [x] Compilation verified:
  - `cargo build -p plur-tui` ✅
  - `cargo build --workspace` ✅
  - `cargo test -p plur-tui` ✅ (4 tests passing)
- [x] Created `plur-tui/README.md` documentation

### 🚧 Task 3: Architecture Documentation (Current)
The core architecture is complete and documented. The reducer pattern is implemented with:
- 4 unit tests verifying purity and behavior
- Comprehensive doc comments
- Clear separation of concerns

### ⏳ Remaining Tasks

**M1: Composer MVP**
- [ ] Task 4: Test harness with FakePorts
- [ ] Task 5: Event loop and terminal setup
- [ ] Task 6: Composer screen with tui-textarea
- [ ] Task 7: Posting flow with EventBus
- [ ] Task 8: Global keybindings
- [ ] Task 9: Error overlay

**M2: History Browser**
- [ ] Task 10: History list and filtering

**M3: Draft Manager**
- [ ] Task 11: Draft CRUD and publishing

**Documentation & QA**
- [ ] Task 12: Documentation and help
- [ ] Task 13: CI and packaging
- [ ] Task 14: Manual QA and SSH testing

## Design Decisions

### Reducer Purity

The reducer is strictly pure:
```rust
// ✅ Pure - returns new state
fn reduce(state: AppState, action: Action) -> AppState {
    match action {
        Action::Quit => AppState { should_quit: true, ..state },
        // ...
    }
}

// ❌ Would be impure
fn reduce(state: &mut AppState, action: Action) {
    state.should_quit = true; // Mutation!
}
```

### Side Effects

Side effects (I/O, service calls) happen **outside** the reducer:
1. Action dispatched: `ComposerPostRequested`
2. Side effect runs: Call `PostingService.post()`
3. Result dispatched as new action: `ComposerPostSucceeded` or `ComposerPostFailed`
4. Reducer updates state based on result

### SSH-Friendly Defaults

```toml
[defaults]
mouse = false         # Enable with 'm' key
colors = detect      # Honors NO_COLOR
unicode = detect     # ASCII fallback
alt_screen = true    # Fallback for dumb terminals
```

### Testing Strategy

```
Unit Tests (Reducer)
  ├─ test_reducer_is_pure
  ├─ test_quit_action
  ├─ test_composer_validation_result
  └─ test_posting_flow

Integration Tests (Future)
  ├─ test_app_boot
  ├─ test_keymap
  ├─ test_composer_state
  └─ test_posting_flow
```

## File Structure

```
plur-tui/
├── Cargo.toml                 # Dependencies
├── README.md                  # Crate documentation
├── src/
│   ├── main.rs               # Entry point (minimal stub)
│   ├── error.rs              # Error types ✅
│   ├── app/
│   │   ├── mod.rs            # Module exports ✅
│   │   ├── actions.rs        # Actions enum ✅
│   │   ├── state.rs          # State structures ✅
│   │   └── reducer.rs        # Pure reducer ✅
│   ├── ui/                   # Future: Rendering
│   └── services.rs           # Future: Service adapter
└── tests/                    # Future: Integration tests
```

## Next Session

Continue with **Task 3: Architecture Documentation** by updating this file with:
- Event loop design
- Service integration patterns
- UI rendering approach
- Testing patterns with FakePorts

Then proceed to **Task 4: Test Harness**.

## Acceptance Criteria

### Task 2 ✅
- [x] Workspace builds successfully
- [x] plur-tui compiles without errors
- [x] Core architecture implemented (actions, state, reducer)
- [x] Reducer tests pass (4/4)
- [x] README documentation created

### Future Tasks
- [ ] Event loop running and responsive
- [ ] Composer accepts input and validates
- [ ] Posting works with progress tracking
- [ ] Terminal properly restored on exit
- [ ] All keybindings functional

## References

- [Ratatui Documentation](https://ratatui.rs/)
- [crossterm Documentation](https://docs.rs/crossterm/)
- [tui-textarea](https://docs.rs/tui-textarea/)
- [Elm Architecture](https://guide.elm-lang.org/architecture/) - Inspiration for reducer pattern
- [Redux](https://redux.js.org/) - Similar state management pattern

---

**Last Updated**: 2025-10-25  
**Task 2 Completed**: 2025-10-25
