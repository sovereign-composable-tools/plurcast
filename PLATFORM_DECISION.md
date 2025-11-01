# Platform Decision: Bluesky → SSB

**Date**: 2025-10-31  
**Version**: 0.3.0-alpha2  
**Status**: Complete

## Executive Summary

Plurcast has made the strategic decision to remove Bluesky support and pivot to SSB (Secure Scuttlebutt) for Phase 3. This decision was made after testing revealed that Bluesky's centralized infrastructure and corporate control are fundamentally incompatible with Plurcast's values of true decentralization and user sovereignty.

## The Problem with Bluesky

### What Happened

During testing, our Bluesky test account was banned without explanation or warning. This revealed a fundamental issue: despite claims of decentralization, Bluesky operates as a centralized platform with a single point of control.

### Why This Matters

1. **Centralized Infrastructure**: One company (Bluesky PBC) controls almost all infrastructure
2. **Corporate Control**: Can ban accounts arbitrarily without due process
3. **Decentralization Theater**: Claims to be decentralized but acts centralized
4. **Philosophical Misalignment**: Not compatible with Plurcast's Unix and decentralization values

### Technical Reality

- **AT Protocol**: Theoretically federated, practically centralized
- **PDS (Personal Data Server)**: Few exist outside Bluesky's control
- **DID Resolution**: Centralized through Bluesky's infrastructure
- **Moderation**: Centralized, opaque, no appeals process

## The Solution: SSB (Secure Scuttlebutt)

### What is SSB?

Secure Scuttlebutt is a truly peer-to-peer, offline-first social protocol with no servers, no blockchain, and no corporate control.

### Why SSB?

1. **Truly Peer-to-Peer**: No servers, no central authority, just peers gossiping
2. **Offline-First**: Works without internet, syncs when connected
3. **Local-First**: All data stored locally in append-only logs
4. **No Corporate Control**: Community-driven, no company, no tokens
5. **Mature Protocol**: Battle-tested with active community
6. **Philosophically Aligned**: Embodies Unix principles and true decentralization

### Technical Advantages

- **Ed25519 Cryptography**: Simple, secure identity
- **Append-Only Logs**: Immutable, auditable history
- **Gossip Protocol**: Efficient peer-to-peer replication
- **Transport Agnostic**: Works over TCP, LAN, Bluetooth, USB drives
- **No Blockchain**: Simple, efficient, no mining or tokens

## Implementation Plan

### Phase 3: SSB Integration

**Phase 3.1: Basic SSB Support (MVP)**
- Connect to local sbot (SSB server)
- Post to local SSB feed
- Message signing and verification
- Basic error handling

**Phase 3.2: Enhanced Integration**
- SSB key generation via `plur-setup`
- Credential management via `plur-creds`
- Multi-account support
- Better error messages

**Phase 3.3: History & Import**
- Query SSB feed history
- Import SSB messages into Plurcast database
- Export to SSB format

**Phase 3.4: Server Management (Optional)**
- Auto-start sbot if not running
- Process lifecycle management
- Systemd integration

### Technical Stack

- **Library**: `kuska-ssb` (Rust SSB implementation)
- **Server**: sbot (user-installed)
- **Protocol**: SSB protocol (mature, stable)
- **Identity**: Ed25519 keypairs

## Documentation Updates

All documentation has been comprehensively updated:

### Updated Files

- ✅ `README.md` - Platform support, setup guides, examples
- ✅ `ROADMAP.md` - Phase 3 now SSB integration, phases renumbered
- ✅ `ARCHITECTURE.md` - Platform list, config examples, credentials
- ✅ `TOOLS.md` - Output formats, import examples
- ✅ `CHANGELOG.md` - Complete change history
- ✅ `.kiro/specs/ssb-integration/design.md` - Complete technical specification

### Removed Content

- ❌ All Bluesky setup instructions
- ❌ Bluesky configuration examples
- ❌ Bluesky troubleshooting sections
- ❌ Bluesky from platform lists
- ❌ Bluesky from roadmap and future plans

## Platform Comparison

| Feature | Nostr | Mastodon | Bluesky | SSB |
|---------|-------|----------|---------|-----|
| **Decentralized** | ✅ Yes | ✅ Federated | ❌ No | ✅ Yes |
| **Servers** | Relays | Instances | Centralized | None |
| **Corporate Control** | ❌ No | ❌ No | ✅ Yes | ❌ No |
| **Offline-First** | ❌ No | ❌ No | ❌ No | ✅ Yes |
| **Censorship Resistant** | ✅ Yes | ⚠️ Instance-level | ❌ No | ✅ Yes |
| **Banning Possible** | ❌ No | ⚠️ Per-instance | ✅ Yes | ❌ No |
| **Blockchain** | ❌ No | ❌ No | ❌ No | ❌ No |
| **Plurcast Status** | ✅ Stable | ✅ Stable | ❌ Removed | 🔮 Phase 3 |

## Impact on Users

### Current Users

- **No Breaking Changes**: Existing Nostr and Mastodon functionality unchanged
- **Multi-Account Works**: Fully tested and stable
- **Shared Test Account**: New easter egg feature for Nostr

### Future Users

- **Better Platform Choice**: SSB offers true decentralization
- **Offline Capability**: SSB works without internet
- **No Banning Risk**: Peer-to-peer means no central authority
- **Community-Driven**: No corporate control or profit motive

## Lessons Learned

### What We Learned

1. **Test Early**: Testing revealed Bluesky's true nature
2. **Values Matter**: Technical features don't matter if values don't align
3. **Decentralization is Binary**: Either truly decentralized or not
4. **Community Over Corporate**: Community-driven protocols are more resilient

### What We're Doing Differently

1. **Thorough Evaluation**: Deep dive into SSB before implementation
2. **Clear Documentation**: Complete technical specification before coding
3. **Value Alignment**: Ensure platforms align with Plurcast philosophy
4. **User Sovereignty**: Prioritize user control and data ownership

## Timeline

- **2025-10-31**: Decision made, documentation updated
- **Phase 3.1**: SSB MVP (estimated 2-3 weeks)
- **Phase 3.2**: Enhanced integration (estimated 1-2 weeks)
- **Phase 3.3**: History & import (estimated 1-2 weeks)
- **Phase 3.4**: Server management (optional, estimated 2-3 weeks)

## Conclusion

This decision represents a commitment to Plurcast's core values: true decentralization, user sovereignty, and Unix philosophy. By removing Bluesky and adding SSB, we're choosing substance over hype, community over corporate, and real decentralization over theater.

SSB embodies what we believe social networking should be: peer-to-peer, offline-first, community-driven, and free from corporate control. This is the future we're building.

---

**For More Information:**
- SSB Design Spec: `.kiro/specs/ssb-integration/design.md`
- Changelog: `CHANGELOG.md`
- Roadmap: `.kiro/steering/ROADMAP.md`
- Architecture: `.kiro/steering/ARCHITECTURE.md`

**Community:**
- Shared Test Account: `--account shared-test` (Nostr)
- GitHub: https://github.com/plurcast/plurcast
- SSB Community: https://scuttlebutt.nz/

