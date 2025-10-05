## Project Organization

```
plurcast/
├── Cargo.toml              # Workspace manifest
├── src/
│   ├── lib.rs              # Shared library (libplurcast)
│   ├── db.rs               # Database operations
│   ├── config.rs           # Configuration management
│   ├── types.rs            # Shared types
│   └── platforms/          # Platform client wrappers
│       ├── mod.rs
│       ├── nostr.rs
│       ├── mastodon.rs
│       └── bluesky.rs
├── bin/                    # Separate binaries (Unix philosophy)
│   ├── plur-post.rs
│   ├── plur-queue.rs
│   ├── plur-send.rs
│   ├── plur-history.rs
│   ├── plur-import.rs
│   └── plur-export.rs
├── migrations/             # SQLx database migrations
│   └── 001_initial.sql
└── tests/
    └── integration.rs
```

## Architecture Patterns

**Separation of Concerns**:
- Each binary is a standalone tool (not subcommands)
- Shared logic in `src/lib.rs` (libplurcast)
- Platform-specific code isolated in `src/platforms/`
- Database layer abstracted in `src/db.rs`

**Data Flow**:
- Tools communicate via SQLite database and standard streams
- No in-memory shared state between binaries
- Configuration read from TOML files
- Credentials stored separately from config

**Module Responsibilities**:
- `lib.rs` - Public API for shared functionality
- `db.rs` - All SQLite operations, schema management
- `config.rs` - TOML parsing, XDG path resolution
- `types.rs` - Common structs (Post, Platform, etc.)
- `platforms/` - Trait implementations for each platform

## Coding Conventions

- Use `anyhow::Result` for application errors
- Use `thiserror` for library error types
- Async/await with Tokio runtime
- Structured logging with `tracing`
- Clap derive macros for CLI parsing
- SQLx compile-time checked queries
