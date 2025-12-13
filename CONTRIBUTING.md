# Contributing to Plurcast

Thank you for your interest in contributing to Plurcast!

## Development Setup

```bash
# Clone the repository
git clone https://github.com/sovereign-composable-tools/plurcast.git
cd plurcast

# Build
cargo build

# Run tests
cargo test

# Check for issues
cargo clippy -- -D warnings
cargo fmt --check
```

## Development Workflow

### Test-Driven Development (TDD)

We follow TDD - write tests first:

1. **RED**: Write a failing test
2. **GREEN**: Write minimal code to pass
3. **REFACTOR**: Improve while keeping tests green

### Before Committing

```bash
cargo fmt          # Format code
cargo clippy       # Lint (must pass with no warnings)
cargo test         # Run all tests
```

All three must pass before committing.

## Code Quality Standards

### Function Size
- Ideal: 5-15 lines
- Maximum: 50 lines
- If longer, refactor into smaller functions

### File Size
- Maximum: 500 lines
- Split larger modules into submodules

### Documentation
- Document "why", not "what"
- Add examples for public APIs

### Zero Warnings Policy
- Treat all warnings as errors
- Fix or explicitly allow with comment explaining why

## Project Structure

```
plurcast/
├── libplurcast/          # Shared library (all business logic)
│   ├── src/
│   │   ├── config.rs     # Configuration
│   │   ├── credentials.rs # Credential storage
│   │   ├── db.rs         # Database operations
│   │   ├── error.rs      # Error types
│   │   └── platforms/    # Platform implementations
│   └── migrations/       # SQLx migrations
├── plur-post/            # Post binary
├── plur-history/         # History query binary
├── plur-creds/           # Credential management
├── plur-queue/           # Queue management
├── plur-send/            # Scheduling daemon
├── plur-setup/           # Setup wizard
├── plur-import/          # Import tool
└── plur-export/          # Export tool
```

## Adding a New Platform

1. Create `libplurcast/src/platforms/myplatform.rs`
2. Implement the `Platform` trait
3. Add config struct to `config.rs`
4. Add credential handling to `credentials.rs`
5. Write comprehensive tests

## Security Guidelines

- **Never** commit credentials
- Use `Secret<T>` from `secrecy` crate for sensitive data
- Validate all inputs at boundaries
- Don't expose secrets in error messages or logs
- Set file permissions 600 for credential files

## Commit Messages

Follow conventional commits:

```
feat: add scheduled posting
fix: handle invalid UTF-8 in content
docs: update setup guide
test: add multi-account integration tests
refactor: extract credential storage to module
```

## Pull Requests

1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Ensure all checks pass
5. Submit PR with clear description

## Unix Philosophy

All tools should:
- Do one thing well
- Use text streams (stdin/stdout)
- Compose via pipes
- Use meaningful exit codes
- Be agent-friendly (JSON output)

## Questions?

- Open an issue for bugs or feature requests
- Check existing issues before creating new ones

---

**Thank you for contributing!**
