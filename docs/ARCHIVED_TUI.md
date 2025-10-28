# TUI Development - Archived

**Date**: 2025-10-28  
**Branch**: `archive/tui-exploration`  
**Status**: Archived - Development discontinued

## Decision

Discontinued Terminal UI (plur-tui) development in favor of focusing on:
1. **CLI polish** - Existing tools (`plur-post`, `plur-history`, etc.)
2. **Desktop GUI** - Tauri-based application (broader appeal)

## Rationale

The TUI occupies an awkward middle ground:
- **CLI users** prefer scriptable, composable tools (already have this)
- **GUI users** want visual, mouse-driven interfaces (Tauri serves this better)
- **Complexity** adds maintenance burden and debugging difficulty
- **Narrow audience** between CLI power users and GUI casual users

## What Was Built

The archived branch contains:
- ✅ Functional architecture (immutable state, pure reducers)
- ✅ Action/State/Reducer pattern implementation
- ✅ 4 passing unit tests
- ✅ ~586 lines of working code
- ✅ Comprehensive documentation

## Preserved Learning

The functional architecture exploration (Redux/Elm-style) is preserved for:
- Future GUI state management inspiration
- Reference implementation of pure functional patterns in Rust
- Testing strategies for complex UI state

## If Reconsidering

Before reconsidering TUI development, ask:
1. Is there a specific user segment CLI doesn't serve well?
2. Would GUI not serve those users better?
3. Is the maintenance/complexity cost justified?

## Branch Access

```bash
# View archived work
git checkout archive/tui-exploration

# List changes
git log archive/tui-exploration

# Compare with main
git diff main..archive/tui-exploration
```

---

**Decision approved by**: User  
**Last TUI commit**: f88d116
