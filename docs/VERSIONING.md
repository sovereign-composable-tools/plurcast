# Versioning Scheme

Plurcast follows semantic versioning with alpha/beta designations.

## Format

```
MAJOR.MINOR.PATCH-STAGE
```

- **MAJOR**: Breaking API changes
- **MINOR**: New features, non-breaking changes
- **PATCH**: Bug fixes, patches
- **STAGE**: `alpha`, `beta`, `rc` (release candidate), or omitted for stable

## Current Version: 0.3.0-alpha1

### Version History

**0.3.x-alpha** - Credential Storage Stability (Current)
- 0.3.0-alpha1 - Mark keyring as experimental, begin stability work
- 0.3.0-alpha2 - Add persistence tests (planned)
- 0.3.0-alpha3 - Fix keyring issues (planned)
- 0.3.0-alpha  - Stable keyring on all platforms (release target)

**0.2.0-alpha** - Multi-Platform Support (Released)
- ✅ Nostr, Mastodon, Bluesky support
- ✅ Concurrent posting
- ✅ History querying
- ✅ Credential management (keyring/encrypted/plain)
- ⚠️  Keyring persistence issues discovered

**0.1.0-alpha** - Foundation
- ✅ Core database schema
- ✅ Nostr-only posting
- ✅ Basic configuration

## Release Criteria

### Alpha → Beta
- [ ] All critical bugs fixed
- [ ] Core features complete and tested
- [ ] Documentation comprehensive
- [ ] Ready for wider testing

### Beta → RC
- [ ] No known critical bugs
- [ ] Performance optimized
- [ ] Security audited
- [ ] Migration guides complete

### RC → Stable (1.0)
- [ ] Production-tested by early adopters
- [ ] All platforms verified
- [ ] libplurcast published to crates.io
- [ ] Stable API guaranteed

## Version Milestones

### 0.3.0 - Credential Stability
**Goal**: Reliable credential storage across all platforms

- [ ] Fix keyring persistence issues
- [ ] Add integration tests for credential backends
- [ ] Verify on Windows, macOS, Linux
- [ ] Document stable storage recommendations

### 0.4.0 - Multi-Account Support
- [ ] Multiple profiles/accounts per platform
- [ ] Account switching
- [ ] Per-account configuration

### 0.5.0 - Library Stabilization
- [ ] Publish libplurcast to crates.io
- [ ] Stable public API
- [ ] Comprehensive API documentation
- [ ] External consumer examples

### 1.0.0 - Production Ready
- [ ] All core features stable
- [ ] Security audit complete
- [ ] Performance optimized
- [ ] Comprehensive testing
- [ ] Production deployment guide

## Version Bumping

### Patch Version (0.3.0 → 0.3.1)
- Bug fixes only
- No new features
- No breaking changes
- Update `Cargo.toml` [workspace.package] version

### Minor Version (0.3.x → 0.4.0)
- New features
- Non-breaking API additions
- Deprecations (with warnings)
- Update `Cargo.toml` and `README.md`

### Major Version (0.x → 1.0, 1.x → 2.0)
- Breaking API changes
- Major architectural changes
- Requires migration guide
- Update all documentation

## Commands

Check version:
```bash
plur-post --version
plur-history --version
plur-creds --version
plur-setup --version
```

All binaries share the same version from `[workspace.package]`.
