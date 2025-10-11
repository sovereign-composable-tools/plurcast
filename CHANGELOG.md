# Changelog

All notable changes to Plurcast will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0-alpha] - 2025-10-11

### Added
- **Multi-platform support**: Post to Nostr and Mastodon simultaneously
- **Mastodon integration**: Full support for Mastodon and Fediverse platforms via `megalodon`
- **Secure credential storage**: Three-tier system (OS keyring, encrypted files, plain text)
- **Interactive setup wizard**: `plur-setup` for easy platform configuration
- **Credential management**: `plur-creds` tool for managing platform credentials
- **Platform selection**: `--platform` flag to target specific platforms
- **JSON output mode**: Machine-readable output with `--format json`
- **Verbose logging**: `--verbose` flag for debugging
- **Draft mode**: Save posts without publishing with `--draft`
- **History queries**: `plur-history` with platform filtering and search
- **Character limit validation**: Pre-flight validation for platform-specific limits
- **Concurrent posting**: Posts to multiple platforms simultaneously
- **Bluesky implementation**: Basic Bluesky support (not fully tested)

### Changed
- **Configuration format**: Enhanced TOML config with platform-specific sections
- **Error handling**: Improved error messages with actionable suggestions
- **Logging**: Suppressed noisy relay messages from Nostr SDK (unless verbose)
- **URL normalization**: Automatic `https://` prefix for Mastodon instances

### Fixed
- **Mastodon instance URLs**: Properly handle URLs with and without `https://` prefix
- **Nostr duplicate messages**: Suppressed confusing relay-level duplicate warnings
- **Default platforms**: Config now properly respects default platform list

### Platform Status
- ✅ **Nostr**: Tested and stable
- ✅ **Mastodon**: Tested and stable
- 🚧 **Bluesky**: Implemented but not fully tested (stretch goal)

### Documentation
- Updated README with platform stability status
- Updated ROADMAP to reflect Phase 2 completion (~90%)
- Updated ARCHITECTURE with platform testing status
- Clarified Bluesky as lower priority stretch goal

## [0.1.0-alpha] - 2025-10-05

### Added
- Initial release with Nostr support
- `plur-post` binary for posting to Nostr
- `plur-history` binary for querying post history
- SQLite database for local post storage
- TOML-based configuration with XDG Base Directory support
- Unix-friendly design: stdin/stdout, pipes, exit codes
- Agent-friendly: JSON output, comprehensive help text
- Basic error handling and validation
- Nostr relay management
- Support for hex and bech32 (nsec) key formats

### Platform Status
- ✅ **Nostr**: Basic support implemented

---

**Legend**:
- ✅ Tested and stable
- 🚧 Implemented but needs testing
- ⏳ Planned/In progress
- ❌ Not yet implemented
