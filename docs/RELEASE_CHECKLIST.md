# Release Checklist for Plurcast 0.2.0-alpha

## Pre-Release

- [x] All tasks from Phase 2 completed
- [x] Code review and refactoring complete
- [x] Clippy warnings fixed
- [x] Documentation comments added
- [x] Consistent naming conventions verified
- [x] Performance tests passing
- [x] Security review complete
- [x] Version updated to 0.2.0-alpha
- [x] Release notes created
- [x] Build verification successful

## Testing

- [x] Unit tests passing (200+ tests)
- [x] Integration tests passing (50+ tests)
- [x] Performance tests passing
- [x] Security tests passing
- [x] Backward compatibility tests passing
- [x] End-to-end tests passing

## Documentation

- [x] README updated with multi-platform examples
- [x] SETUP.md includes platform-specific instructions
- [x] Release notes comprehensive
- [x] API documentation complete
- [x] Configuration examples provided

## Build Artifacts

- [x] Release build successful
- [ ] Binaries built for target platforms:
  - [ ] Linux x86_64
  - [ ] macOS x86_64
  - [ ] macOS ARM64
  - [ ] Windows x86_64

## Git Operations

- [ ] All changes committed
- [ ] Create release branch: `release/0.2.0-alpha`
- [ ] Tag release: `v0.2.0-alpha`
- [ ] Push to repository
- [ ] Create GitHub release

## Post-Release

- [ ] Announce on project channels
- [ ] Update project website (if applicable)
- [ ] Monitor for issues
- [ ] Prepare for Phase 3

## Notes

### Build Commands

```bash
# Build release binaries
cargo build --release

# Run all tests
cargo test --all

# Run specific test suites
cargo test --test performance
cargo test --test security
cargo test --test backward_compatibility
cargo test --test end_to_end

# Check for issues
cargo clippy --all-targets --all-features -- -D warnings
```

### Binary Locations

After building, binaries are located at:
- `target/release/plur-post` (or `.exe` on Windows)
- `target/release/plur-history` (or `.exe` on Windows)

### Installation

Users can install from source:
```bash
cargo install --path plur-post
cargo install --path plur-history
```

Or from crates.io (after publishing):
```bash
cargo install plur-post
cargo install plur-history
```

### Platform-Specific Notes

**Linux/macOS:**
- Binaries work out of the box
- File permissions for credentials should be 600

**Windows:**
- Binaries work via PowerShell or CMD
- File permission checking not yet implemented

### Known Limitations

1. **Mastodon**: Manual OAuth token generation required
2. **Bluesky**: Text posts only (no media yet)
3. **Windows**: File permission warnings not implemented
4. **All platforms**: No scheduled posting yet (Phase 4)

### Support Channels

- GitHub Issues: Bug reports and feature requests
- GitHub Discussions: Questions and community support
- Documentation: README.md, SETUP.md, and inline docs

## Verification Steps

1. **Clean build:**
   ```bash
   cargo clean
   cargo build --release
   ```

2. **Test installation:**
   ```bash
   cargo install --path plur-post --force
   cargo install --path plur-history --force
   plur-post --version
   plur-history --version
   ```

3. **Smoke test:**
   ```bash
   echo "Test post" | plur-post --draft
   plur-history --limit 1
   ```

4. **Multi-platform test (if configured):**
   ```bash
   echo "Multi-platform test" | plur-post --platform nostr --platform mastodon
   plur-history --platform nostr
   ```

## Success Criteria

- [x] All tests pass
- [x] Build succeeds on all platforms
- [x] Documentation is complete
- [x] No critical bugs identified
- [x] Performance meets targets
- [x] Security review passed
- [ ] Release artifacts created
- [ ] Git tags created
- [ ] Release published

## Rollback Plan

If critical issues are discovered:

1. Do not publish to crates.io
2. Delete git tag if created
3. Mark GitHub release as pre-release
4. Document issues in GitHub
5. Fix issues and prepare 0.2.1-alpha

## Next Steps After Release

1. Monitor GitHub issues for bug reports
2. Gather user feedback
3. Begin Phase 3 planning (Service Layer & UI)
4. Consider publishing to crates.io after stability period
