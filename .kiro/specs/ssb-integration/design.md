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

### Phase 1: Basic Posting with Replication

**Goal**: Post to local SSB feed and replicate to the network using kuska-ssb library

**Components:**
```
plur-post → SSB Platform → kuska-ssb Library → Local Feed Database
                                              ↓
                                         Replication
                                              ↓
                                    Pub Servers / Peers
```

**Implementation:**
1. Initialize kuska-ssb library with user's keypair
2. Create signed message with post content
3. Append to local feed database
4. Trigger replication with configured pubs/peers
5. Return message ID

**Configuration:**
```toml
[ssb]
enabled = true
feed_path = "~/.plurcast-ssb"  # Local feed database directory
pubs = [
    "net:hermies.club:8008~shs:...",  # Example pub server
]
# Optional: direct peer connections
peers = []
```

**Credentials:**
- SSB uses Ed25519 keypairs
- Stored in Plurcast credential manager (keyring/encrypted/plain)
- Format: Ed25519 private key (32 bytes)
- Can generate new keypair or import from `~/.ssb/secret`

### Phase 2: Replication Protocol

**Approach**: Full replication using kuska-ssb

**Implementation:**
- Use `kuska-ssb` library for replication protocol
- Connect to pub servers for network access
- Support direct peer connections (LAN, internet)
- Background replication process
- Gossip protocol for feed exchange

**Replication Strategy:**
1. **Immediate sync after posting** - Push new messages to pubs
2. **Background sync** - Periodic pull from pubs/peers
3. **Selective replication** - Only sync followed feeds (future)

**Benefits:**
- Full SSB network participation
- No external sbot dependency
- Direct control over replication
- Consistent with Plurcast's architecture

### Phase 3: Peer Discovery & Management

**Peer Discovery:**
- **Pub servers** - Connect via multiserver addresses
- **Local network** - mDNS/DNS-SD for LAN peers (future)
- **Manual peers** - Direct addresses in config
- **Invite codes** - Join pubs via invite system (future)

**Pub Server Integration:**
- Connect to well-known pubs (hermies.club, etc.)
- Authenticate using keypair
- Replicate feeds bidirectionally
- Handle connection failures gracefully

**Configuration:**
```toml
[ssb]
pubs = [
    "net:hermies.club:8008~shs:base64-key",
]
```

### Phase 4: History & Queries

**Reading from SSB:**
```bash
# Query local SSB feed
plur-history --platform ssb

# Import SSB history into Plurcast DB
plur-import ssb
```

**Implementation:**
- Query local feed database using kuska-ssb
- Filter by type (post)
- Store in Plurcast database
- Maintain mapping: SSB message ID ↔ Plurcast post ID

## Technical Challenges

### 1. Library Integration

**Challenge**: Integrate kuska-ssb library correctly

**Solutions:**
- Follow kuska-ssb documentation and examples
- Study existing SSB clients using kuska-ssb
- Start with basic message creation and signing
- Add feed database management incrementally

### 2. Replication Timing

**Challenge**: Replication is async - posts don't appear instantly on other peers

**Solutions:**
- Show "posted locally, replicating..." status
- Don't block on replication completion
- Background sync continues after plur-post exits
- Provide `--wait-for-sync` flag for testing (optional)

### 3. Key Management

**Challenge**: SSB keys are Ed25519 (different from Nostr's secp256k1)

**Solutions:**
- Support importing from standard `~/.ssb/secret` location
- Generate new Ed25519 keypairs via `plur-setup`
- Store in credential manager like other platforms
- Use kuska-ssb's key generation and validation

### 4. Message Size Limits

**Challenge**: SSB messages have practical size limits (~8KB)

**Solutions:**
- Validate content size before posting
- Warn users when content exceeds limits
- Skip SSB when posting to multiple platforms if oversized
- Blob attachments are a future enhancement

## Architecture

### SSB Platform Implementation

**Location**: `libplurcast/src/platforms/ssb.rs`

**Structure:**
```rust
pub struct SSBPlatform {
    keypair: Ed25519Keypair,
    feed_path: PathBuf,
    config: SSBConfig,
}

impl Platform for SSBPlatform {
    async fn post(&self, content: &str) -> Result<String>;
    async fn validate_content(&self, content: &str) -> Result<()>;
    fn name(&self) -> &str { "ssb" }
}
```

**Dependencies:**
```toml
[dependencies]
kuska-ssb = "0.x"  # SSB protocol implementation
ed25519-dalek = "2.0"  # Ed25519 cryptography (if needed)
```

### Feed Database Structure

**Location**: `~/.plurcast-ssb/`

**Contents:**
- `log.offset` - Append-only log of messages
- `flume/` - Flume database for indexing
- `secret` - Encrypted keypair (optional, we use credential manager)

**Managed by**: kuska-ssb library

### Message Flow

```
User Input
    ↓
plur-post CLI
    ↓
SSBPlatform::post()
    ↓
kuska-ssb::create_message()
    ↓
kuska-ssb::sign_message()
    ↓
kuska-ssb::append_to_feed()
    ↓
Local Feed Database
    ↓
Return message ID
```

### Integration with Existing Code

**Platform Trait** (already exists):
```rust
#[async_trait]
pub trait Platform: Send + Sync {
    async fn post(&self, content: &str) -> Result<String>;
    async fn validate_content(&self, content: &str) -> Result<()>;
    fn name(&self) -> &str;
}
```

**SSB Implementation**:
- Implements `Platform` trait like Nostr and Mastodon
- Uses credential manager for keypair storage
- Manages local feed database via kuska-ssb
- Returns SSB message ID in format `ssb:%<hash>`

## Implementation Plan

### Phase 3.1: Basic SSB Support with Replication (MVP)

**Prerequisites:**
- None! Library integration means no external dependencies

**Features:**
- Post to local SSB feed using kuska-ssb
- Connect to pub servers for replication
- Background sync after posting
- Basic error handling
- Configuration via config.toml
- Feed database initialization

**Deliverables:**
- `SSBPlatform` implementation
- kuska-ssb library integration
- Message creation and signing
- Feed database management
- Pub server connection and replication
- Background sync process
- Integration tests

**Estimated Effort**: 3-4 weeks (includes replication)

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

### Phase 3.4: Advanced Replication Features (Optional)

**Features:**
- Local network peer discovery (mDNS)
- Invite code system for joining pubs
- Selective replication (only followed feeds)
- Sync status UI/progress tracking
- LAN-only mode (no internet required)

**Deliverables:**
- mDNS peer discovery
- Invite code handling
- Follow graph management
- Replication statistics

**Estimated Effort**: 2-3 weeks

## User Experience

### Setup Flow

```bash
# 1. Configure Plurcast (no external dependencies!)
plur-setup
# → Checks for existing ~/.ssb/secret
# → Offers to import or generate new keypair
# → Initializes feed database at ~/.plurcast-ssb
# → Offers to add default pub servers
# → Tests connection to pubs
# → Saves configuration

# 2. Post!
plur-post "Hello SSB!" --platform ssb
# → Posted locally to ~/.plurcast-ssb
# → Replicating to pubs...
# → Returns: ssb:%abc123...

# 3. View history
plur-history --platform ssb
# → Shows posts from local feed database
# → Includes posts from followed feeds (after sync)
```

### Multi-Platform Posting

```bash
# Post to Nostr, Mastodon, and SSB
plur-post "Cross-posting to all platforms!"
# → Nostr: Instant (relay-based)
# → Mastodon: Instant (server-based)
# → SSB: Local + replicating to pubs
# Returns:
# nostr:note1abc...
# mastodon:12345
# ssb:%def456...
```

## Documentation Needs

1. **SSB Primer** - What is SSB? Why use it? Offline-first philosophy, gossip protocol
2. **Setup Guide** - Running plur-setup for SSB, importing existing keys, configuring pubs
3. **Configuration** - Plurcast + SSB integration, feed_path and pubs parameters
4. **Key Management** - Generating and securing Ed25519 keypairs
5. **Pub Servers** - What are pubs? How to find them? How to add them?
6. **Replication** - How SSB replication works, async nature, background sync
7. **Troubleshooting** - Common issues and solutions (pub connectivity, sync failures)
8. **Comparison** - SSB vs Nostr vs Mastodon (gossip vs relay vs server)

## Success Metrics

**Phase 3.1 (MVP):**
- ✅ Can post to local SSB feed
- ✅ Posts replicate to configured pubs
- ✅ Messages appear in SSB clients (Patchwork, Manyverse)
- ✅ Background sync receives updates from followed feeds
- ✅ Integration tests pass (including replication)
- ✅ Documentation complete (including pub setup)

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
