# Changelog

All notable changes to Plurcast will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0-alpha2] - 2025-10-31

### Major Platform Decision

**Removed Bluesky, Added SSB (Secure Scuttlebutt)**

After testing, we've made the strategic decision to remove Bluesky support and pivot to SSB (Secure Scuttlebutt) for Phase 3.

**Why we removed Bluesky:**
- Centralized infrastructure (one company controls almost everything)
- Banned our test account without explanation
- "Decentralization theater" - claims to be decentralized but acts centralized
- Not philosophically aligned with Plurcast values

**Why we're adding SSB:**
- Truly peer-to-peer (no servers, no central authority)
- Offline-first architecture
- No blockchain, no tokens, no corporate control
- Community-driven with mature protocol
- Philosophically aligned with Unix principles and decentralization values

### Added

- ✨ **Shared test account easter egg** - Built-in `--account shared-test` for Nostr
  - No credentials needed
  - Community bulletin board
  - Perfect for demos and testing
  - Public key: `npub1qyv34w2prnz66zxrgqsmy2emrg0uqtrnvarhrrfaktxk9vp2dgllsajv05m`

- 🔧 **Multi-account support** - Fully operational
  - Multiple accounts per platform
  - Account switching with `plur-creds use`
  - Per-post account override with `--account` flag
  - Account registry in `~/.config/plurcast/accounts.toml`

- 🐛 **Fixed `plur-creds list` bug** - Now properly shows keyring-stored credentials
  - Uses AccountManager instead of CredentialManager
  - Shows active account markers
  - Verifies credentials exist before listing

- 📡 **Improved Nostr relay coverage** - 7 relays for better propagation
  - Added: relay.primal.net, relay.snort.social, purplepag.es, relay.mostr.pub
  - Better overlap with mobile clients
  - Faster post propagation

- 📚 **SSB Integration Design** - Complete technical specification
  - Phase 3.1: Basic SSB support (MVP)
  - Phase 3.2: Enhanced integration
  - Phase 3.3: History & import
  - Phase 3.4: Server management (optional)
  - See `.kiro/specs/ssb-integration/design.md`

### Changed

- 📖 **Updated all documentation** - Removed Bluesky, added SSB
  - README.md: Platform support, setup guides, examples
  - ROADMAP.md: Phase 3 now SSB integration, renumbered phases
  - ARCHITECTURE.md: Platform list, config examples, credentials
  - TOOLS.md: Output formats, import examples
  - All version numbers updated to 0.3.0-alpha2

- 🎯 **Phase 2 marked complete** - Multi-platform and multi-account working
  - Nostr: Tested and stable
  - Mastodon: Tested and stable
  - Multi-account: Tested and stable

### Removed

- ❌ **Bluesky support** - Completely removed from codebase
  - Removed from platform list
  - Removed setup documentation
  - Removed config examples
  - Removed troubleshooting sections
  - Removed from roadmap and future plans

### Technical Details

**Platforms:**
- ✅ Nostr: Production-ready with shared test account
- ✅ Mastodon: Production-ready (supports all ActivityPub platforms)
- 🔮 SSB: Planned for Phase 3

**Multi-Account:**
- Account registry: `~/.config/plurcast/accounts.toml`
- Credential namespace: `plurcast.{platform}.{account}.{key}`
- Active account tracking per platform
- Backward compatible with "default" account

**Relay Coverage:**
- 7 Nostr relays (up from 3)
- Better mobile client overlap
- Improved propagation speed

## [0.2.0-alpha1] - 2025-10-11

### Added

- Initial multi-platform support (Nostr, Mastodon, Bluesky)
- Secure credential storage (OS keyring, encrypted files)
- Interactive setup wizard (`plur-setup`)
- Credential management tool (`plur-creds`)
- Post history queries (`plur-history`)
- Multi-platform posting with concurrent execution
- Platform abstraction trait
- Comprehensive error handling

### Changed

- Improved configuration system
- Enhanced logging and debugging
- Better error messages

## [0.1.0-alpha] - 2025-10-04

### Added

- Initial release
- Basic Nostr support
- SQLite database for post history
- TOML configuration
- Unix-friendly CLI design
- Agent-friendly features (JSON output, exit codes)
- Draft mode
- Platform selection

---

**Legend:**
- ✨ New feature
- 🔧 Enhancement
- 🐛 Bug fix
- 📚 Documentation
- 📖 Documentation update
- 🎯 Milestone
- ❌ Removal
- 📡 Infrastructure
- 🔮 Planned

