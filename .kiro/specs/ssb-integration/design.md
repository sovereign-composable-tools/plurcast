# SSB (Secure Scuttlebutt) Integration Design

**Status**: Planning
**Priority**: Phase 3 (Post Multi-Account & Scheduling)
**Complexity**: High (peer-to-peer architecture)

## Overview

Secure Scuttlebutt (SSB) is a peer-to-peer, offline-first social protocol that aligns perfectly with Plurcast's Unix philosophy and decentralization values. Unlike Bluesky's centralized "decentralization theater," SSB is genuinely peer-to-peer with no servers, no blockchain, and no corporate control.

## Why SSB?

**Philosophical Alignment:**
- ✅ **Truly peer-to-peer** - No servers, no relays, just peers gossiping
- ✅ **Offline-first** - Works without internet, syncs when connected
- ✅ **Local-first** - All data in local append-only logs
- ✅ **No blockchain** - Simple cryptographic keys and gossip protocol
- ✅ **Community-driven** - No company, no tokens, no VC funding
- ✅ **Unix philosophy** - Simple protocols, composable tools

**Technical Benefits:**
- Append-only logs (immutable, auditable)
- Works over any transport (TCP, LAN, sneakernet)
- Cryptographically signed messages
- Peer discovery via local network and pub servers
- Mature ecosystem with multiple implementations

## SSB Architecture

### Core Concepts

**Feed (Identity):**
- Each user has a cryptographic keypair (Ed25519)
- Public key is your identity: `@<base64-pubkey>.ed25519`
- Private key signs all your messages
- Feed is an append-only log of signed messages

**Messages:**
- JSON objects with content, timestamp, signature
- Immutably linked to previous message (hash chain)
- Types: `post`, `about`, `contact`, `vote`, etc.

**Replication:**
- Peers gossip and replicate feeds
- Follow graph determines what to replicate
- Works over TCP, WebSocket, Bluetooth, USB drives

**Pubs (Optional):**
- Public servers that help with peer discovery
- Not required, just helpful for internet connectivity
- Anyone can run a pub

### Example SSB Message

```json
{
  "previous": "%hash-of-previous-message",
  "author": "@pubkey.ed25519",
  "sequence": 42,
  "timestamp": 1635724800000,
  "hash": "sha256",
  "content": {
    "type": "post",
    "text": "Hello from Plurcast!",
    "mentions": []
  },
  "signature": "base64-signature"
}
```

## Rust SSB Libraries

### Available Crates

1. **`ssb-rs`** - Core SSB implementation
   - Feed management
   - Message signing/verification
   - Replication protocol
   - Status: Active but experimental

2. **`kuska-ssb`** - Alternative implementation
   - More modular design
   - Better async support
   - Used by some SSB clients

3. **`ssb-legacy-msg`** - Message format handling
   - Parsing and serialization
   - Signature verification
   - Well-maintained

### Recommended Approach

Use **`kuska-ssb`** for initial implementation:
- Better async/await support (works with tokio)
- Modular architecture fits Plurcast's design
- Active development and responsive maintainers

## Integration Design

### Phase 1: Basic Posting

**Goal**: Post to local SSB feed

**Components:**
```
plur-post → SSB Platform → Local SSB Server → Append to Feed
```

**Implementation:**
1. Connect to local SSB server (sbot)
2. Create signed message with post content
3. Append to local feed
4. Return message ID

**Configuration:**
```toml
[ssb]
enabled = true
server_address = "localhost:8008"  # Local sbot
keypair_file = "~/.ssb/secret"     # Standard SSB location
```

**Credentials:**
- SSB uses Ed25519 keypairs
- Standard location: `~/.ssb/secret`
- Format: JSON with public/private keys
- Can generate new keypair or use existing

### Phase 2: Server Management

**Challenge**: SSB requires a local server (sbot) to be running

**Options:**

1. **Assume External Server** (Phase 1)
   - User runs their own sbot
   - Plurcast just connects to it
   - Simplest, most Unix-like

2. **Embedded Server** (Phase 2)
   - Plurcast spawns sbot as child process
   - Manages lifecycle (start/stop)
   - More user-friendly

3. **Library Integration** (Phase 3)
   - Embed SSB library directly
   - No external process needed
   - Most complex, best UX

**Recommendation**: Start with Option 1, evolve to Option 2

### Phase 3: Replication & Discovery

**Peer Discovery:**
- Local network (mDNS/DNS-SD)
- Pub servers (invite codes)
- Manual peer addresses

**Replication:**
- Automatic via sbot
- Plurcast doesn't manage this
- Just posts and lets SSB handle propagation

### Phase 4: History & Queries

**Reading from SSB:**
```bash
# Query local SSB feed
plur-history --platform ssb

# Import SSB history into Plurcast DB
plur-import ssb --feed @pubkey.ed25519
```

**Implementation:**
- Query sbot for messages
- Filter by type (post)
- Store in Plurcast database
- Maintain mapping: SSB message ID ↔ Plurcast post ID

## Technical Challenges

### 1. Server Dependency

**Problem**: SSB requires a running server (sbot)

**Solutions:**
- Document sbot installation
- Provide systemd service files
- Eventually embed server

### 2. Async Replication

**Problem**: Posts don't appear immediately on other peers

**Solutions:**
- Set user expectations (offline-first)
- Show "posted locally, replicating..." status
- Don't wait for replication to complete

### 3. Key Management

**Problem**: SSB keys are in different format than Nostr

**Solutions:**
- Support standard `~/.ssb/secret` location
- Allow key generation via `plur-setup`
- Store in credential manager like other platforms

### 4. Message Size Limits

**Problem**: SSB messages have practical size limits (~8KB)

**Solutions:**
- Enforce content limits
- Support blob attachments for larger content
- Warn users about size constraints

## Implementation Plan

### Phase 3.1: Basic SSB Support (MVP)

**Prerequisites:**
- User has sbot installed and running
- User has SSB keypair

**Features:**
- Post to local SSB feed
- Basic error handling
- Configuration via config.toml

**Deliverables:**
- `SSBPlatform` implementation
- Connection to local sbot
- Message creation and signing
- Integration tests

**Estimated Effort**: 2-3 weeks

### Phase 3.2: Enhanced Integration

**Features:**
- Key generation via `plur-setup`
- Credential management via `plur-creds`
- Better error messages
- Status feedback

**Deliverables:**
- SSB key generation
- Secure key storage
- User documentation
- Example configurations

**Estimated Effort**: 1-2 weeks

### Phase 3.3: History & Import

**Features:**
- Query SSB feed history
- Import into Plurcast database
- Export to SSB format

**Deliverables:**
- `plur-history --platform ssb`
- `plur-import ssb`
- `plur-export --format ssb`

**Estimated Effort**: 1-2 weeks

### Phase 3.4: Server Management (Optional)

**Features:**
- Auto-start sbot if not running
- Lifecycle management
- Health checks

**Deliverables:**
- Process management
- Systemd integration
- Docker support

**Estimated Effort**: 2-3 weeks

## User Experience

### Setup Flow

```bash
# 1. Install sbot (user does this)
npm install -g ssb-server

# 2. Start sbot
sbot server

# 3. Configure Plurcast
plur-setup
# → Detects running sbot
# → Finds existing keys or generates new ones
# → Tests connection
# → Saves configuration

# 4. Post!
plur-post "Hello SSB!" --platform ssb
```

### Multi-Platform Posting

```bash
# Post to Nostr, Mastodon, and SSB
plur-post "Cross-posting to all platforms!"
# → Nostr: Instant (relay-based)
# → Mastodon: Instant (server-based)
# → SSB: Local immediately, replicates over time
```

## Documentation Needs

1. **SSB Primer** - What is SSB? Why use it?
2. **Installation Guide** - Setting up sbot
3. **Configuration** - Plurcast + SSB integration
4. **Key Management** - Generating and securing keys
5. **Troubleshooting** - Common issues and solutions
6. **Comparison** - SSB vs Nostr vs Mastodon

## Success Metrics

**Phase 3.1 (MVP):**
- ✅ Can post to local SSB feed
- ✅ Messages appear in SSB clients (Patchwork, Manyverse)
- ✅ Integration tests pass
- ✅ Documentation complete

**Phase 3.2 (Enhanced):**
- ✅ Key generation works
- ✅ Credential management integrated
- ✅ User feedback is positive
- ✅ No major bugs reported

**Phase 3.3 (History):**
- ✅ Can query SSB history
- ✅ Import/export works
- ✅ Database integration stable

**Phase 3.4 (Server Management):**
- ✅ Auto-start works reliably
- ✅ No zombie processes
- ✅ Systemd integration tested

## Future Enhancements

### Advanced Features (Post-1.0)

1. **Blob Support** - Attach images/files
2. **Thread Support** - Reply to SSB messages
3. **Follow Graph** - Manage SSB follows via Plurcast
4. **Pub Management** - Connect to pubs, manage invites
5. **Private Messages** - Encrypted DMs via SSB
6. **Rooms Support** - SSB Rooms for tunneling

### Integration Ideas

1. **SSB as Backend** - Use SSB for Plurcast's own data
2. **Hybrid Sync** - Sync Plurcast config via SSB
3. **Peer Discovery** - Find other Plurcast users on SSB
4. **Collaborative Features** - Shared drafts, team accounts

## Resources

**SSB Protocol:**
- [SSB Protocol Guide](https://ssbc.github.io/scuttlebutt-protocol-guide/)
- [SSB Handbook](https://handbook.scuttlebutt.nz/)

**Rust Libraries:**
- [kuska-ssb](https://github.com/Kuska-ssb/ssb)
- [ssb-rs](https://github.com/ssb-ngi-pointer/ssb-rs)

**SSB Clients (for testing):**
- [Patchwork](https://github.com/ssbc/patchwork) - Desktop
- [Manyverse](https://www.manyver.se/) - Mobile
- [Oasis](https://github.com/fraction/oasis) - Web

**Community:**
- [SSB Forum](https://ssb-forum.netlify.app/)
- [Scuttlebutt.nz](https://scuttlebutt.nz/)

---

**Version**: 0.3.0-alpha2
**Last Updated**: 2025-10-31
**Status**: Planning - Replacing Bluesky with SSB
**Target**: Phase 3 (Post Multi-Account & Scheduling)
