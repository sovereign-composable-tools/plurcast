# SSB (Secure Scuttlebutt) Setup Guide

> **⚠️ EXPERIMENTAL STATUS**
>
> SSB integration in Plurcast is currently **experimental** and serves as a reference implementation
> for the Platform trait architecture. While core functionality is complete and well-tested, some
> features have limitations:
>
> **Working:**
> - ✅ Generate/import SSB keypairs
> - ✅ Configure SSB via plur-setup
> - ✅ Post to local SSB feed
> - ✅ Query posting history
> - ✅ Import/export SSB posts
> - ✅ Multi-platform posting (Nostr + Mastodon + SSB)
> - ✅ Multi-account support
>
> **Limitations:**
> - ⚠️ Network replication to pub servers is designed but not fully implemented
> - ⚠️ Posts may not appear in other SSB clients (Patchwork, Manyverse, etc.)
> - ⚠️ Feed database format differs from standard SSB implementations
>
> **Primary Use Cases:**
> - Demonstrating multi-platform architecture
> - Local-only SSB posting and storage
> - Testing platform abstraction
> - Foundation for future full SSB support
>
> For production use, we recommend focusing on **Nostr** and **Mastodon**, which are fully supported
> and regularly tested by the maintainers.

---

This guide provides detailed instructions for setting up SSB (Secure Scuttlebutt) with Plurcast.

## Table of Contents

- [What is SSB?](#what-is-ssb)
- [SSB Concepts](#ssb-concepts)
- [Prerequisites](#prerequisites)
- [Quick Setup (Recommended)](#quick-setup-recommended)
- [Manual Setup](#manual-setup)
- [Keypair Management](#keypair-management)
- [Pub Server Configuration](#pub-server-configuration)
- [Testing Your Setup](#testing-your-setup)
- [Next Steps](#next-steps)

---

## What is SSB?

**Secure Scuttlebutt (SSB)** is a peer-to-peer, offline-first social protocol that aligns perfectly with Plurcast's Unix philosophy and decentralization values.

### Why SSB?

**Philosophical Alignment**:
- ✅ **Truly peer-to-peer** - No servers, no relays, just peers gossiping
- ✅ **Offline-first** - Works without internet, syncs when connected
- ✅ **Local-first** - All data in local append-only logs
- ✅ **No blockchain** - Simple cryptographic keys and gossip protocol
- ✅ **Community-driven** - No company, no tokens, no VC funding
- ✅ **Unix philosophy** - Simple protocols, composable tools

**Technical Benefits**:
- Append-only logs (immutable, auditable)
- Works over any transport (TCP, LAN, sneakernet)
- Cryptographically signed messages
- Peer discovery via local network and pub servers
- Mature ecosystem with multiple implementations

### How SSB Differs from Other Platforms

| Feature | SSB | Nostr | Mastodon |
|---------|-----|-------|----------|
| Architecture | Peer-to-peer gossip | Relay-based | Server-based |
| Offline Support | ✅ Full | ❌ No | ❌ No |
| Data Storage | Local append-only log | Relays | Servers |
| Identity | Ed25519 keypair | secp256k1 keypair | Server account |
| Replication | Gossip protocol | Relay subscription | Federation |
| Servers Required | No (pubs optional) | Yes (relays) | Yes (instances) |

---

## SSB Concepts

Understanding these core concepts will help you use SSB effectively:

### Feed (Identity)

- Each user has a cryptographic keypair (Ed25519)
- Public key is your identity: `@<base64-pubkey>.ed25519`
- Private key signs all your messages
- Feed is an append-only log of signed messages

**Example SSB ID**:
```
@HSc+JVu3NfznJT8CJWqN9UhKd8DrY8+8kLPqLkmLR2Y=.ed25519
```

### Messages

Messages are JSON objects with content, timestamp, and signature:

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

**Key Properties**:
- **Immutable**: Once published, messages cannot be changed
- **Ordered**: Sequence numbers ensure chronological order
- **Linked**: Each message references the previous one (hash chain)
- **Signed**: Cryptographic signature proves authorship

### Replication

- Peers gossip and replicate feeds
- Follow graph determines what to replicate
- Works over TCP, WebSocket, Bluetooth, USB drives
- Asynchronous - posts don't appear instantly

### Pubs (Optional)

- Public servers that help with peer discovery
- Not required, just helpful for internet connectivity
- Anyone can run a pub
- Connect to pubs to reach the wider SSB network

**Popular Pubs**:
- `hermies.club` - Community pub
- `pub.scuttlebutt.nz` - New Zealand pub
- `ssb.celehner.com` - Celehner's pub

---

## Prerequisites

### System Requirements

- **Operating System**: Linux, macOS, or BSD (Windows via WSL)
- **Rust**: 1.70 or later (for building Plurcast)
- **Disk Space**: ~100MB for feed database (grows over time)

### No External Dependencies!

Unlike traditional SSB clients, Plurcast uses the `kuska-ssb` library to manage SSB feeds directly. This means:

- ✅ No need to install or run an external SSB server (sbot)
- ✅ No Node.js or npm required
- ✅ Feed database managed by Plurcast
- ✅ Simpler setup and maintenance

---

## Quick Setup (Recommended)

The easiest way to set up SSB is using the interactive setup wizard:

```bash
plur-setup
```

### Example Session

```bash
$ plur-setup

🌟 Welcome to Plurcast Setup!

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Step 1: Choose Credential Storage Backend
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[... credential storage selection ...]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Step 2: Configure Platform Credentials
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Configure SSB? [Y/n]: y

🔐 SSB Configuration
────────────────────────────────────────────────────────

Checking for existing SSB keypair at ~/.ssb/secret...
✓ Found existing SSB keypair

Do you want to import this keypair? [Y/n]: y
✓ SSB keypair imported successfully

Your SSB ID: @HSc+JVu3NfznJT8CJWqN9UhKd8DrY8+8kLPqLkmLR2Y=.ed25519

Feed database will be stored at: ~/.plurcast-ssb

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Step 3: Configure Pub Servers (Optional)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Pub servers help your posts reach the wider SSB network.
You can skip this and add pubs later.

Add default pub servers? [Y/n]: y

Adding pub servers:
  ✓ hermies.club
  ✓ pub.scuttlebutt.nz

Testing pub connectivity...
  ✓ hermies.club - reachable
  ✓ pub.scuttlebutt.nz - reachable

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🎉 SSB Setup Complete!
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Next steps:

  1. Post your first SSB message:
     echo "Hello SSB!" | plur-post --platform ssb

  2. View your SSB feed:
     plur-history --platform ssb

  3. Check replication status:
     plur-post --verbose --platform ssb
```

After running `plur-setup`, you're ready to start posting to SSB!

---

## Manual Setup

If you prefer to set up SSB manually, follow these steps:

### Step 1: Choose Keypair Source

You have two options:

**Option A: Import Existing SSB Keypair**

If you already use SSB clients (Patchwork, Manyverse, etc.), you can import your existing keypair:

```bash
# Check if you have an existing keypair
ls ~/.ssb/secret

# If it exists, Plurcast can import it
plur-creds set ssb --import ~/.ssb/secret
```

**Option B: Generate New Keypair**

Generate a new Ed25519 keypair for SSB:

```bash
# Generate and store new keypair
plur-creds set ssb --generate

# Your SSB ID will be displayed:
# ✓ Generated new SSB keypair
# Your SSB ID: @abc123...=.ed25519
```

### Step 2: Configure SSB

Create or edit `~/.config/plurcast/config.toml`:

```toml
[ssb]
enabled = true
feed_path = "~/.plurcast-ssb"  # Local feed database directory
pubs = [
    "net:hermies.club:8008~shs:base64-key-here",
    "net:pub.scuttlebutt.nz:8008~shs:base64-key-here"
]
```

**Configuration Options**:

- `enabled`: Set to `true` to enable SSB posting
- `feed_path`: Directory for local feed database (default: `~/.plurcast-ssb`)
- `pubs`: List of pub servers in multiserver address format (optional)

### Step 3: Initialize Feed Database

The feed database will be created automatically on first use:

```bash
# Post a test message to initialize the database
echo "Hello SSB!" | plur-post --platform ssb
```

This will:
1. Create the feed database directory at `feed_path`
2. Initialize the kuska-ssb library
3. Create your first SSB message
4. Return the message ID

---

## Keypair Management

### Understanding SSB Keypairs

SSB uses **Ed25519** cryptographic keypairs:

- **Private Key**: 32 bytes, used to sign messages
- **Public Key**: 32 bytes, your SSB identity
- **Format**: Base64-encoded with `.ed25519` suffix

**Example**:
```
Public Key (SSB ID):
@HSc+JVu3NfznJT8CJWqN9UhKd8DrY8+8kLPqLkmLR2Y=.ed25519

Private Key:
(64 bytes in base64, kept secret)
```

### Generating a New Keypair

```bash
# Interactive generation
plur-creds set ssb

# You'll be prompted:
# Generate new SSB keypair? [Y/n]: y
# ✓ Generated new SSB keypair
# Your SSB ID: @abc123...=.ed25519
```

### Importing from ~/.ssb/secret

If you have an existing SSB keypair from other clients:

```bash
# Standard SSB secret file location
plur-creds set ssb --import ~/.ssb/secret
```

The `~/.ssb/secret` file format:
```json
{
  "curve": "ed25519",
  "public": "HSc+JVu3NfznJT8CJWqN9UhKd8DrY8+8kLPqLkmLR2Y=.ed25519",
  "private": "base64-private-key-here.ed25519",
  "id": "@HSc+JVu3NfznJT8CJWqN9UhKd8DrY8+8kLPqLkmLR2Y=.ed25519"
}
```

### Viewing Your SSB ID

```bash
# Test SSB credentials (shows your SSB ID)
plur-creds test ssb

# Output:
# ✓ SSB authentication successful
# Your SSB ID: @HSc+JVu3NfznJT8CJWqN9UhKd8DrY8+8kLPqLkmLR2Y=.ed25519
```

### Security Best Practices

**Protect Your Private Key**:
- Never share your private key
- Use OS keyring or encrypted storage (not plain text)
- Backup your keypair securely
- Anyone with your private key can post as you

**Backup Your Keypair**:
```bash
# Export to encrypted backup
plur-creds export ssb | gpg -c > ssb-keypair-backup.gpg

# Restore from backup
gpg -d ssb-keypair-backup.gpg | plur-creds import ssb
```

---

## Pub Server Configuration

Pub servers help your posts reach the wider SSB network. They're optional but recommended for internet connectivity.

### What are Pubs?

- **Public SSB servers** that aid peer discovery
- Help replicate your feed to other users
- Not required for local-only or LAN usage
- Anyone can run a pub server

### Multiserver Address Format

Pubs are specified using the multiserver address format:

```
net:hostname:port~shs:public-key
```

**Components**:
- `net`: Protocol (TCP network)
- `hostname`: Domain or IP address
- `port`: Port number (usually 8008)
- `shs`: Secret handshake protocol
- `public-key`: Pub's public key in base64

**Example**:
```
net:hermies.club:8008~shs:gfW/+1nLKT+/+LNbGmHJQ8Pu7TMwCvXLPvqXbEN6kZk=
```

### Adding Pub Servers

**Option 1: Via Configuration File**

Edit `~/.config/plurcast/config.toml`:

```toml
[ssb]
enabled = true
feed_path = "~/.plurcast-ssb"
pubs = [
    "net:hermies.club:8008~shs:gfW/+1nLKT+/+LNbGmHJQ8Pu7TMwCvXLPvqXbEN6kZk=",
    "net:pub.scuttlebutt.nz:8008~shs:base64-key-here"
]
```

**Option 2: Via Setup Wizard**

```bash
plur-setup
# Select SSB configuration
# Choose "Add default pub servers"
```

### Popular Pub Servers

| Pub | Address | Notes |
|-----|---------|-------|
| hermies.club | `net:hermies.club:8008~shs:...` | Community pub, reliable |
| pub.scuttlebutt.nz | `net:pub.scuttlebutt.nz:8008~shs:...` | New Zealand pub |
| ssb.celehner.com | `net:ssb.celehner.com:8008~shs:...` | Celehner's pub |

**Finding More Pubs**:
- [SSB Pub List](https://github.com/ssbc/ssb-server/wiki/Pub-Servers)
- Ask in SSB community channels
- Run your own pub server

### Testing Pub Connectivity

```bash
# Test connection to configured pubs
plur-creds test ssb --check-pubs

# Output:
# Testing pub connectivity...
#   ✓ hermies.club - reachable (latency: 45ms)
#   ✓ pub.scuttlebutt.nz - reachable (latency: 120ms)
#   ✗ ssb.celehner.com - unreachable (timeout)
```

### Pub Invites

Some pubs require invite codes. If you have an invite:

```bash
# Accept pub invite
plur-creds accept-invite "invite-code-here"

# The pub will be added to your configuration automatically
```

**Getting Invites**:
- Ask in SSB community channels
- Some pubs offer public invites
- Friends can generate invites for you

---

## Testing Your Setup

### 1. Test Authentication

```bash
# Verify SSB credentials work
plur-creds test ssb

# Expected output:
# ✓ SSB authentication successful
# Your SSB ID: @abc123...=.ed25519
# Feed database: ~/.plurcast-ssb
# Messages in feed: 0
```

### 2. Post a Test Message

```bash
# Post to SSB only
echo "Hello SSB! Testing Plurcast." | plur-post --platform ssb

# Expected output:
# ssb:%abc123...
```

The output is an SSB message ID in the format `ssb:%<hash>`.

### 3. Verify in Feed Database

```bash
# Query your SSB history
plur-history --platform ssb

# Expected output:
# [2025-01-15 10:30:00] ssb:%abc123...
# Hello SSB! Testing Plurcast.
```

### 4. Check Replication Status

```bash
# Post with verbose logging to see replication
plur-post "Test replication" --platform ssb --verbose

# Output will show:
# [INFO] Initializing SSB platform
# [INFO] Feed database: ~/.plurcast-ssb
# [INFO] Creating SSB message...
# [INFO] Message signed: ssb:%abc123...
# [INFO] Appending to local feed...
# [INFO] Replicating to pubs...
# [INFO]   → hermies.club: connected
# [INFO]   → hermies.club: pushed message
# [INFO]   → pub.scuttlebutt.nz: connected
# [INFO]   → pub.scuttlebutt.nz: pushed message
# ✓ Posted to SSB: ssb:%abc123...
```

### 5. Verify in SSB Clients

Your posts should appear in other SSB clients:

- **Patchwork** (Desktop): https://github.com/ssbc/patchwork
- **Manyverse** (Mobile): https://www.manyver.se/
- **Oasis** (Web): https://github.com/fraction/oasis

Search for your SSB ID to find your feed.

---

## Next Steps

After setting up SSB:

### 1. Multi-Platform Posting

Post to SSB alongside Nostr and Mastodon:

```bash
# Post to all enabled platforms
echo "Hello from all platforms!" | plur-post

# Post to specific platforms
plur-post "SSB and Nostr only" --platform ssb,nostr
```

### 2. Query Your SSB History

```bash
# Recent SSB posts
plur-history --platform ssb --limit 10

# Search SSB posts
plur-history --platform ssb --search "keyword"

# Date range
plur-history --platform ssb --since "2025-01-01"

# JSON output for scripting
plur-history --platform ssb --format json | jq '.'
```

### 3. Import Existing SSB Posts

If you have existing SSB posts from other clients:

```bash
# Import from local SSB feed
plur-import ssb

# This will:
# - Query your local SSB feed database
# - Import all messages with type "post"
# - Store them in Plurcast's database
# - Skip duplicates
```

### 4. Export SSB Posts

```bash
# Export to SSB format
plur-export --format ssb --output ssb-posts.json

# Export to other formats
plur-export --format markdown --platform ssb
plur-export --format json --platform ssb
```

### 5. Explore SSB Features

**Follow Other Users** (future feature):
```bash
# Follow an SSB user
plur-follow @their-ssb-id.ed25519

# Your feed will replicate their posts
```

**Local Network Discovery** (future feature):
```bash
# Discover SSB peers on your local network
plur-discover --lan

# Replicate with local peers (no internet required)
```

**Offline Usage**:
```bash
# SSB works offline!
# Posts are stored locally and replicate when you reconnect

# Post while offline
echo "Offline post" | plur-post --platform ssb

# Later, when online, replication happens automatically
```

### 6. Advanced Configuration

See [SSB_CONFIG.md](SSB_CONFIG.md) for:
- Feed database management
- Replication settings
- Performance tuning
- Advanced pub configuration

### 7. Troubleshooting

If you encounter issues, see [SSB_TROUBLESHOOTING.md](SSB_TROUBLESHOOTING.md) for:
- Common problems and solutions
- Pub connectivity issues
- Replication failures
- Feed database corruption

---

## Understanding SSB Replication

### How Replication Works

1. **Post Locally**: Message is appended to your local feed
2. **Connect to Pubs**: Plurcast connects to configured pubs
3. **Push Messages**: New messages are pushed to pubs
4. **Gossip Protocol**: Pubs replicate to other peers
5. **Eventual Consistency**: Your post reaches the network over time

### Replication is Asynchronous

Unlike Nostr (instant) or Mastodon (instant), SSB replication takes time:

- **Local**: Instant (message in your feed)
- **Pubs**: Seconds to minutes (depends on connectivity)
- **Network**: Minutes to hours (gossip protocol)

**This is by design** - SSB prioritizes offline-first and eventual consistency over instant delivery.

### Checking Replication Status

```bash
# Verbose mode shows replication progress
plur-post "Test" --platform ssb --verbose

# Background sync status (future feature)
plur-sync-status ssb
```

---

## SSB Philosophy

### Why SSB is Different

**Offline-First**:
- Your data lives on your machine
- No internet required for posting
- Sync when you connect

**No Servers**:
- Pubs are optional helpers, not required infrastructure
- Anyone can run a pub
- No single point of failure

**Gossip Protocol**:
- Peers replicate feeds they're interested in
- Follow graph determines replication
- Organic, decentralized distribution

**Immutable Logs**:
- Messages can't be edited or deleted
- Cryptographically signed and linked
- Auditable history

### SSB vs. Other Platforms

**SSB vs. Nostr**:
- SSB: Gossip protocol, offline-first, local storage
- Nostr: Relay-based, always-online, relay storage

**SSB vs. Mastodon**:
- SSB: Peer-to-peer, no servers, append-only logs
- Mastodon: Federated servers, ActivityPub, mutable posts

**SSB vs. Bluesky**:
- SSB: Truly decentralized, no company, community-driven
- Bluesky: Centralized company, "decentralization theater"

---

## Resources

### SSB Protocol

- [SSB Protocol Guide](https://ssbc.github.io/scuttlebutt-protocol-guide/)
- [SSB Handbook](https://handbook.scuttlebutt.nz/)
- [Scuttlebutt.nz](https://scuttlebutt.nz/)

### SSB Clients

- [Patchwork](https://github.com/ssbc/patchwork) - Desktop client
- [Manyverse](https://www.manyver.se/) - Mobile client (iOS/Android)
- [Oasis](https://github.com/fraction/oasis) - Web client

### Community

- [SSB Forum](https://ssb-forum.netlify.app/)
- [SSB GitHub](https://github.com/ssbc)
- [SSB Pub List](https://github.com/ssbc/ssb-server/wiki/Pub-Servers)

### Rust Libraries

- [kuska-ssb](https://github.com/Kuska-ssb/ssb) - Plurcast's SSB library
- [ssb-rs](https://github.com/ssb-ngi-pointer/ssb-rs) - Alternative implementation

---

**Happy gossiping!** 🦀🔐

