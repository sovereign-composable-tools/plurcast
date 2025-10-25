# Service Layer Extraction - Phase 3.1

**Status**: Requirements Complete, Ready for Design  
**Branch**: `feature/service-layer-extraction`  
**Phase**: 3.1 - Service Layer Extraction  
**Priority**: High (enables Phase 3.2 TUI and Phase 3.3 GUI)

## Overview

This spec defines the extraction of business logic from CLI binaries into a dedicated service layer. The service layer will provide a clean, testable API that can be consumed by multiple interfaces (CLI, TUI, GUI) without code duplication.

**Key Principle**: Zero behavioral changes to existing CLI tools.

## Architecture

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
       │      PlurcastService            │
       │         (Facade)                │
       ├─────────────────────────────────┤
       │  • PostingService               │
       │  • HistoryService               │
       │  • DraftService                 │
       │  • ValidationService            │
       │  • EventBus                     │
       └────────────────┬────────────────┘
                        │
       ┌────────────────▼────────────────┐
       │   Core Library (Phase 1-2)      │
       │  • Platform Abstraction         │
       │  • Database (SQLite + sqlx)     │
       │  • Configuration (TOML)         │
       │  • Credentials                  │
       └─────────────────────────────────┘
```

## Services

### PlurcastService (Facade)
- Entry point for all service operations
- Manages shared resources (Database, Config)
- Provides access to all sub-services

### PostingService
- Multi-platform posting with retry logic
- Draft creation and publishing
- Content validation
- Progress event emission

### HistoryService
- Post history queries with filtering
- Platform-specific filtering
- Statistics and analytics
- Pagination support

### DraftService
- Draft CRUD operations
- Draft publishing (delegates to PostingService)
- Draft listing and management

### ValidationService
- Real-time content validation
- Platform-specific limit checking
- Size validation (100KB max)
- Detailed error messages

### EventBus
- In-process event distribution
- Progress tracking for long operations
- Multiple subscriber support
- Non-blocking when no subscribers

## Implementation Plan

1. **Design Phase**: Create design.md with detailed architecture
2. **Implementation Phase**: Create tasks.md with step-by-step implementation
3. **Testing Phase**: Comprehensive unit and integration tests
4. **Documentation Phase**: Rustdoc and SERVICE_LAYER.md guide
5. **CLI Refactoring**: Update plur-post, plur-history to use services
6. **Validation Phase**: Ensure zero behavioral changes

## Success Criteria

- ✅ All CLI tools refactored with zero behavioral changes
- ✅ Service layer test coverage ≥ 80%
- ✅ All existing integration tests pass
- ✅ Complete documentation with examples
- ✅ Foundation ready for TUI/GUI development

## Next Steps

1. Review requirements.md
2. Create design.md with detailed service APIs
3. Create tasks.md with implementation checklist
4. Begin implementation

## Related Documentation

- [ROADMAP.md](../../steering/ROADMAP.md) - Phase 3 overview
- [ARCHITECTURE.md](../../steering/ARCHITECTURE.md) - System architecture
- [FUTURE.md](../../steering/FUTURE.md) - Progressive UI enhancement vision
