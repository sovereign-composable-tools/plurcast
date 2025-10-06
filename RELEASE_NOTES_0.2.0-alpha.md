# Plurcast 0.2.0-alpha Release Notes

**Release Date:** 2025-10-05

## Overview

Plurcast 0.2.0-alpha is the first multi-platform release, expanding from single-platform (Nostr) support to include Mastodon and Bluesky. This release introduces a clean platform abstraction layer, concurrent multi-platform posting, and the new `plur-history` tool for querying posting history.

## What's New

### Multi-Platform Support

- **Nostr**: Full support via `nostr-sdk` v0.35+
- **Mastodon**: Support for Mastodon and Fediverse platforms via `megalodon` v0.14+
  - Supports Mastodon, Pleroma, Friendica, Firefish, GoToSocial, and Akkoma
  - Instance-specific character limits
  - OAuth token authentication
- **Bluesky**: Support for AT Protocol via `atrium-api` v0.24+
  - DID-based identity
  - App password authentication
  - 300 character limit

### Platform Abstraction

- Clean `Platform` trait for unified platform interactions
- Async/await support for all platform operations
- Platform-specific character limits and validation
- Configuration checking via `is_configured()` method

### Enhanced plur-post

- Post to multiple platforms concurrently
- Selective platform posting via `--platform` flag
- Per-platform output format: `platform:post_id`
- Improved exit codes:
  - 0: All platforms succeeded
  - 1: At least one platform failed
  - 2: Authentication error
  - 3: Invalid input
- Partial failure handling (continues posting to remaining platforms)

### New plur-history Tool

Query your local posting history with powerful filtering:

```bash
# Recent posts (default: last 20)
plur-history

# Filter by platform
plur-history --platform nostr

# Date range filtering
plur-history --since "2025-10-01" --until "2025-10-05"

# Search content
plur-history --search "rust"

# Multiple output formats
plur-history --format json
plur-history --format jsonl
plur-history --format csv
```

### Concurrent Posting

- Posts to all platforms concurrently for better performance
- Retry logic with exponential backoff (up to 3 attempts)
- Transient error detection (network issues, rate limits)
- Per-platform result tracking in database

### Configuration Enhancements

New configuration sections for each platform:

```toml
[nostr]
enabled = true
keys_file = "~/.config/plurcast/nostr.keys"
relays = ["wss://relay.damus.io", "wss://nos.lol"]

[mastodon]
enabled = true
instance = "mastodon.social"
token_file = "~/.config/plurcast/mastodon.token"

[bluesky]
enabled = true
handle = "user.bsky.social"
auth_file = "~/.config/plurcast/bluesky.auth"

[defaults]
platforms = ["nostr", "mastodon", "bluesky"]
```

### Database Enhancements

- Optimized indexes for query performance
- Multi-platform post records
- Efficient filtering by platform, date range, and content
- Support for partial success tracking

### Error Handling

- New `RateLimit` error variant
- Platform-specific error context
- Suggested remediation in error messages
- No credential leakage in error messages

## Performance

- Concurrent posting: 2-3x faster than sequential
- History queries: <100ms for 1000 posts
- Database indexes for efficient filtering
- Memory usage: <50MB for typical workloads

## Security

- Credentials stored in separate files (not in config or database)
- No credentials in logs or error messages
- File permission checking (Unix systems)
- Secure authentication flows for all platforms

## Breaking Changes

None - this release is backward compatible with 0.1.0. Existing Nostr-only configurations continue to work unchanged.

## Migration Guide

### From 0.1.0 to 0.2.0-alpha

1. Update binaries:
   ```bash
   cargo install --path plur-post
   cargo install --path plur-history
   ```

2. (Optional) Add new platform configurations to `~/.config/plurcast/config.toml`

3. (Optional) Create credential files for new platforms:
   - Mastodon: `~/.config/plurcast/mastodon.token`
   - Bluesky: `~/.config/plurcast/bluesky.auth`

4. Existing posts and data are preserved - no database migration needed

## Known Issues

- Windows: File permission checking not yet implemented
- Mastodon: Interactive OAuth flow not yet available (manual token generation required)
- Bluesky: Limited to text posts (no media attachments yet)

## Documentation

- Updated README with multi-platform examples
- Platform-specific setup guides
- Configuration examples for each platform
- Troubleshooting section

## Testing

- 200+ unit tests
- 50+ integration tests
- Performance benchmarks
- Security tests
- Backward compatibility tests

## Dependencies

### New Dependencies

- `megalodon` v0.14+ - Mastodon/Fediverse client
- `atrium-api` v0.24+ - Bluesky AT Protocol client
- `futures` v0.3 - Async utilities

### Updated Dependencies

- `nostr-sdk` v0.35+ (from v0.35)
- All other dependencies remain the same

## Contributors

Thank you to all contributors who made this release possible!

## Next Steps

Phase 3 will focus on:
- Service layer extraction
- Terminal UI (TUI) with Ratatui
- Desktop GUI with Tauri
- Multi-account support
- Scheduled posting

## Feedback

This is an alpha release. Please report issues and provide feedback:
- GitHub Issues: https://github.com/plurcast/plurcast/issues
- Discussions: https://github.com/plurcast/plurcast/discussions

## License

Plurcast is dual-licensed under MIT OR Apache-2.0.

---

**Full Changelog**: https://github.com/plurcast/plurcast/compare/v0.1.0...v0.2.0-alpha
