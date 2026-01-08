# Plurcast Platform Setup Guide

This guide provides detailed instructions for setting up each platform supported by Plurcast.

## Table of Contents

- [Quick Setup (Recommended)](#quick-setup-recommended)
- [Credential Storage](#credential-storage)
- [Manual Platform Setup](#manual-platform-setup)
  - [Nostr Setup](#nostr-setup)
  - [Mastodon Setup](#mastodon-setup)
  - [SSB Setup](#ssb-setup)
- [Configuration File Format](#configuration-file-format)
- [Troubleshooting](#troubleshooting)

---

## Quick Setup (Recommended)

The easiest way to set up Plurcast is using the interactive setup wizard:

```bash
plur-setup
```

This wizard will guide you through:
1. **Choosing a credential storage backend** (OS Keyring, Encrypted Files, or Plain Text)
2. **Configuring each platform** (Nostr, Mastodon, SSB)
3. **Testing authentication** to verify credentials work
4. **Saving your configuration**

### Example Session

```bash
$ plur-setup

🌟 Welcome to Plurcast Setup!

This wizard will help you configure Plurcast for posting to
decentralized social media platforms.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Step 1: Choose Credential Storage Backend
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Plurcast supports three storage backends for your credentials:

  1. OS Keyring (recommended)
     - macOS: Keychain
     - Windows: Credential Manager
     - Linux: Secret Service (GNOME Keyring/KWallet)
     - Most secure, integrated with your OS

  2. Encrypted Files
     - Password-protected files using age encryption
     - Good for systems without keyring support
     - Requires master password

  3. Plain Text (not recommended)
     - Credentials stored in plain text files
     - Only for testing or legacy compatibility
     - Security risk

Select storage backend [1-3] (default: 1): 1
✓ Storage backend set to: Keyring

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Step 2: Configure Platform Credentials
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Configure Nostr? [Y/n]: y

📡 Nostr Configuration
────────────────────────────────────────────────────────

You need a Nostr private key (hex or nsec format).
If you don't have one, you can generate it using:
  - Nostr clients like Damus, Amethyst, or Snort
  - Command line tools like 'nak' or 'nostr-tool'

Enter your Nostr private key: ********
✓ Nostr credentials stored
Testing Nostr authentication...
✓ Nostr authentication successful

[... similar for Mastodon ...]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🎉 Setup Complete!
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Next steps:

  1. Test your configuration:
     plur-creds test --all

  2. Post your first message:
     echo "Hello decentralized world!" | plur-post

  3. View your posting history:
     plur-history

  4. Manage credentials:
     plur-creds list
```

After running `plur-setup`, you're ready to start posting!

---

## Credential Storage

Plurcast provides secure credential storage with multiple backend options.

### Storage Backends

#### 1. OS Keyring (Recommended)

Uses your operating system's native credential storage:

- **macOS**: Keychain (AES-256 encryption, unlocked with user login)
- **Windows**: Credential Manager (DPAPI encryption)
- **Linux**: Secret Service (GNOME Keyring, KWallet, etc.)

**Advantages**:
- Most secure option
- Integrated with OS security
- No additional passwords needed
- Credentials protected by OS-level encryption

**Configuration**:
```toml
[credentials]
storage = "keyring"
```

#### 2. Encrypted Files

Password-protected files using age encryption:

- Files stored in `~/.config/plurcast/credentials/`
- Encrypted with age (ChaCha20-Poly1305)
- Requires master password (minimum 8 characters)

**Advantages**:
- Works on systems without keyring support
- Portable across systems
- Strong encryption (age format)

**Configuration**:
```toml
[credentials]
storage = "encrypted"
path = "~/.config/plurcast/credentials"
```

**Setting Master Password**:
```bash
# Option 1: Environment variable
export PLURCAST_MASTER_PASSWORD="your_secure_password"

# Option 2: Interactive prompt (if TTY available)
plur-post "Hello"  # Will prompt for password
```

#### 3. Plain Text (Not Recommended)

Legacy format for backward compatibility:

- Credentials stored in plain text files
- Only file permissions (600) for protection
- **Security risk** - use only for testing

**Configuration**:
```toml
[credentials]
storage = "plain"
path = "~/.config/plurcast"
```

### Managing Credentials

Use `plur-creds` to manage your credentials:

```bash
# Set credentials for a platform
plur-creds set nostr
plur-creds set mastodon

# List configured platforms (doesn't show values)
plur-creds list

# Test authentication
plur-creds test nostr
plur-creds test --all

# Delete credentials
plur-creds delete nostr

# Audit security
plur-creds audit

# Migrate from plain text to secure storage
plur-creds migrate
```

### Migrating from Plain Text

If you're upgrading from an earlier version with plain text credentials:

```bash
# Run migration wizard
plur-creds migrate

# This will:
# 1. Detect plain text credential files
# 2. Copy them to secure storage (keyring or encrypted)
# 3. Verify authentication works
# 4. Optionally delete plain text files
```

For more details, see [SECURITY.md](SECURITY.md).

---

## Manual Platform Setup

If you prefer to set up platforms manually instead of using `plur-setup`, follow these guides:

---

## Nostr Setup

Nostr (Notes and Other Stuff Transmitted by Relays) is a decentralized protocol that uses cryptographic key pairs for identity.

### Prerequisites

- A Nostr private key (hex or nsec format)
- Access to Nostr relays

### Step-by-Step Instructions

#### 1. Generate or Obtain a Nostr Private Key

If you don't have a Nostr key yet, you have several options:

**Option A: Using nak (Recommended)**

[nak](https://github.com/fiatjaf/nak) is a command-line tool for Nostr:

```bash
# Install nak (if you have Go installed)
go install github.com/fiatjaf/nak@latest

# Generate a new key pair
nak key generate

# Output will show:
# Private key (hex): a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456
# Private key (nsec): nsec1abc123def456...
# Public key (hex): ...
# Public key (npub): npub1xyz...
```

**Option B: Using a Nostr Client**

Most Nostr clients can generate keys:
- **Damus** (iOS): Settings → Account → View Keys
- **Amethyst** (Android): Profile → Security → Backup Keys
- **Snort** (Web): Settings → Keys → Export Keys
- **Iris** (Web): Settings → Account → Show Private Key

**Option C: Using nostr-tools (JavaScript)**

```javascript
import { generatePrivateKey, getPublicKey } from 'nostr-tools'

const sk = generatePrivateKey() // hex format
const pk = getPublicKey(sk)

console.log('Private key:', sk)
console.log('Public key:', pk)
```

#### 2. Create the Keys File

```bash
# Create the Plurcast config directory
mkdir -p ~/.config/plurcast

# Create the keys file
touch ~/.config/plurcast/nostr.keys

# Set proper permissions (important for security!)
chmod 600 ~/.config/plurcast/nostr.keys

# Add your private key (choose one format)
# Hex format (64 characters):
echo "a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456" > ~/.config/plurcast/nostr.keys

# OR nsec format:
echo "nsec1abc123def456..." > ~/.config/plurcast/nostr.keys
```

**Security Warning**: Your private key is like a password. Anyone with access to it can post as you. Never share it!

#### 3. Configure Relays

Edit `~/.config/plurcast/config.toml` and add your relay configuration:

```toml
[nostr]
enabled = true
keys_file = "~/.config/plurcast/nostr.keys"
relays = [
    "wss://relay.damus.io",
    "wss://nos.lol",
    "wss://relay.snort.social",
    "wss://relay.primal.net",
    "wss://nostr.mom"
]
```

**Popular Nostr Relays**:
- `wss://relay.damus.io` - General purpose, high traffic
- `wss://nos.lol` - General purpose, reliable
- `wss://relay.snort.social` - General purpose
- `wss://relay.primal.net` - Primal's relay
- `wss://nostr.mom` - General purpose, reliable
- `wss://nostr.wine` - Paid relay (spam-free)

**Specialized Relays** (may reject some post types):
- `wss://purplepag.es` - Profile relay only (rejects kind 1 posts)
- `wss://relay.mostr.pub` - Bridges to Mastodon (requires kind 0 profile first)

You can find more relays at [nostr.watch](https://nostr.watch).

#### 4. Test Your Setup

```bash
# Post a test message to Nostr only
plur-post "Hello Nostr! Testing Plurcast." --platform nostr

# Expected output:
# nostr:note1abc123def456...

# Verify on a Nostr client
# Search for your npub or the note ID on any Nostr client
```

#### 5. Find Your Public Key (npub)

```bash
# If you have nak installed
nak key public $(cat ~/.config/plurcast/nostr.keys)

# Or use an online converter (for nsec format)
# Visit: https://nostr.band/tools/converter
```

### Nostr Key Formats

Nostr supports two key formats:

**Hex Format** (64 characters):
```
a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456
```

**Bech32 Format** (starts with nsec):
```
nsec1abc123def456ghi789jkl012mno345pqr678stu901vwx234yz
```

Plurcast accepts both formats. The bech32 format (nsec) is more user-friendly and includes error detection.

---

## Mastodon Setup

Mastodon is part of the Fediverse, a network of federated social media platforms using ActivityPub.

### Prerequisites

- A Mastodon account (or account on any Fediverse platform)
- Access to your instance's settings

### Supported Platforms

Plurcast's Mastodon integration works with:
- **Mastodon** (mastodon.social, fosstodon.org, etc.)
- **Pleroma**
- **Friendica**
- **Firefish** (formerly Calckey)
- **GoToSocial**
- **Akkoma**

### Step-by-Step Instructions

#### 1. Generate an OAuth Access Token

**For Mastodon**:

1. Log in to your Mastodon instance (e.g., https://mastodon.social)
2. Click on **Settings** (gear icon) or go to Preferences
3. Navigate to **Development** in the left sidebar
4. Click **New Application**
5. Fill in the application details:
   - **Application name**: `Plurcast` (or any name you prefer)
   - **Application website**: `https://github.com/plurcast/plurcast` (optional)
   - **Redirect URI**: `urn:ietf:wg:oauth:2.0:oob`
     - This is required for command-line applications
   - **Scopes**: Select at minimum:
     - ✅ `write:statuses` (required for posting)
     - You can also select `read:statuses` if you plan to use future features
6. Click **Submit**
7. You'll be redirected to your application's page
8. Copy the **Your access token** field (long string of characters)

**For Other Fediverse Platforms**:

The process is similar, but the UI may differ:
- **Pleroma**: Settings → Applications → Create
- **Friendica**: Settings → API → Create new application
- **Firefish**: Settings → API → Generate Token

#### 2. Create the Token File

```bash
# Create the token file
touch ~/.config/plurcast/mastodon.token

# Set proper permissions
chmod 600 ~/.config/plurcast/mastodon.token

# Add your access token
echo "your-access-token-here" > ~/.config/plurcast/mastodon.token
```

**Example token** (yours will be different):
```
ZA-Xbk9mP3L2QwErTyUiOp4sNmVcBxDfGhJkLzAqWe1RtYuIo
```

#### 3. Configure Mastodon

Edit `~/.config/plurcast/config.toml`:

```toml
[mastodon]
enabled = true
instance = "mastodon.social"  # Change to your instance
token_file = "~/.config/plurcast/mastodon.token"
```

**Important**: Replace `mastodon.social` with your actual instance domain:
- `fosstodon.org`
- `mas.to`
- `mstdn.social`
- `your-custom-instance.com`

#### 4. Test Your Setup

```bash
# Post a test message to Mastodon only
plur-post "Hello Fediverse! Testing Plurcast." --platform mastodon

# Expected output:
# mastodon:123456789

# The number is your post ID on that instance
```

#### 5. Verify on Your Instance

Visit your Mastodon profile to see the post appear.

### Character Limits

Different Fediverse platforms have different character limits:
- **Mastodon**: 500 characters (default, can be higher on some instances)
- **Pleroma**: 5000 characters (default)
- **Friendica**: No hard limit
- **Firefish**: 3000 characters (default)

Plurcast will fetch the character limit from your instance automatically.

### Troubleshooting Mastodon

**"Invalid access token"**:
- Regenerate your token in the instance settings
- Ensure you copied the entire token (no spaces or newlines)
- Check that the token file has correct permissions (600)

**"Instance not found"**:
- Verify the instance URL in config.toml
- Don't include `https://` in the instance field
- Use just the domain: `mastodon.social`, not `https://mastodon.social`

**"Forbidden" or "Unauthorized"**:
- Check that your token has `write:statuses` scope
- Regenerate the token with correct scopes

---

## SSB Setup

SSB (Secure Scuttlebutt) is a truly peer-to-peer, offline-first social protocol with no servers, no blockchain, and no corporate control.

**Status**: Experimental - Local posting works, network replication is limited.

### Why SSB?

- **Truly decentralized** - No central servers or corporate control
- **Offline-first** - Works without internet
- **Cryptographic identity** - Ed25519 key pairs
- **Community-driven** - Mature protocol, active community

### Prerequisites

- SSB keypair (generated or imported)

### Step-by-Step Instructions

#### 1. Generate or Import SSB Keypair

```bash
# Option 1: Generate new keypair
plur-creds set ssb --generate

# Option 2: Import from existing SSB installation
plur-creds set ssb --import ~/.ssb/secret
```

#### 2. Configure SSB

Edit `~/.config/plurcast/config.toml`:

```toml
[ssb]
enabled = true
feed_path = "~/.plurcast-ssb"  # Local feed database
# Optional: pub servers for replication
pubs = [
    "net:hermies.club:8008~shs:base64-key-here"
]
```

#### 3. Test Your Setup

```bash
# Post a test message to SSB
plur-post "Hello SSB!" --platform ssb

# Expected output:
# ssb:%abc123...
```

### Current Limitations

SSB support is experimental:
- Local feed posting works
- Keypair management works
- Multi-account support works
- Network replication to pub servers is designed but not fully implemented

### Troubleshooting SSB

**"Keypair not found"**:
```bash
# Generate or import keypair
plur-creds set ssb --generate
```

**"Feed path not accessible"**:
```bash
# Create directory
mkdir -p ~/.plurcast-ssb
```

For more SSB details, see [SSB Comparison](SSB_COMPARISON.md).

---

## Configuration File Format

### Complete Example

Here's a complete `~/.config/plurcast/config.toml` with all platforms enabled:

```toml
[database]
path = "~/.local/share/plurcast/posts.db"

[nostr]
enabled = true
keys_file = "~/.config/plurcast/nostr.keys"
relays = [
    "wss://relay.damus.io",
    "wss://nos.lol",
    "wss://relay.snort.social",
    "wss://nostr.mom"
]

[mastodon]
enabled = true
instance = "mastodon.social"
token_file = "~/.config/plurcast/mastodon.token"

[ssb]
enabled = true
feed_path = "~/.plurcast-ssb"

[defaults]
platforms = ["nostr", "mastodon"]
```

### Disabling Platforms

To disable a platform, set `enabled = false`:

```toml
[mastodon]
enabled = false  # Won't post to Mastodon
instance = "mastodon.social"
token_file = "~/.config/plurcast/mastodon.token"
```

Or remove it from the defaults:

```toml
[defaults]
platforms = ["nostr"]  # Mastodon excluded
```

### Path Expansion

Plurcast supports `~` expansion in file paths:

```toml
[nostr]
keys_file = "~/.config/plurcast/nostr.keys"  # Expands to /home/user/.config/...
```

### Environment Variable Overrides

You can override configuration with environment variables:

```bash
# Override config file location
export PLURCAST_CONFIG=~/my-config.toml

# Override database location
export PLURCAST_DB_PATH=~/my-posts.db

plur-post "Using custom paths"
```

---

## Troubleshooting

### Configuration File Locations

Plurcast uses platform-specific paths for configuration:

| Platform | Config Location |
|----------|-----------------|
| **Linux/macOS** | `~/.config/plurcast/config.toml` |
| **Windows** | `%APPDATA%\plurcast\config.toml` (typically `C:\Users\<username>\AppData\Roaming\plurcast\config.toml`) |

**Important**: On Windows, the `~/.config/` path in documentation refers to the Windows AppData location above.

To find your actual config location, run with `--verbose`:
```bash
plur-post "test" --draft --verbose 2>&1 | head -20
```

### General Issues

**"Configuration file not found"**:
- Plurcast will create a default config on first run
- Default location: See table above for your platform
- You can specify a custom location with `PLURCAST_CONFIG` environment variable

**"Database error: unable to open database file"**:
```bash
# Create the directory
mkdir -p ~/.local/share/plurcast

# Check permissions
ls -la ~/.local/share/plurcast
```

**"Permission denied" when reading credential files**:
```bash
# Fix file permissions
chmod 600 ~/.config/plurcast/nostr.keys
chmod 600 ~/.config/plurcast/mastodon.token
chmod 600 ~/.config/plurcast/bluesky.auth
```

### Platform-Specific Issues

See the troubleshooting sections in each platform's setup guide above.

### Nostr Relay Issues

Nostr relays can go offline, expire SSL certificates, or change behavior. Common errors and fixes:

**"certificate expired"** or **"SSL error"**:
- The relay's SSL certificate has expired
- Remove the relay from your config and use an alternative
- Example: `relay.nostr.band` had an expired SSL certificate as of January 2026

**"No such host is known"** or **"DNS error"**:
- The relay domain no longer resolves
- Remove the relay and use an alternative
- Example: `relay.nostr.bg` had DNS issues as of January 2026

**"502 Bad Gateway"** or **"HTTP error"**:
- The relay server is down or misconfigured
- Try again later or use an alternative relay

**"blocked: kind 1 is not allowed"**:
- The relay only accepts certain event types (e.g., profiles)
- `purplepag.es` only accepts kind 0 (profile) events
- This is expected behavior for specialized relays

**"blocked: author is missing a kind 0 event"**:
- The relay requires you to publish a profile first
- `relay.mostr.pub` (Mastodon bridge) requires a kind 0 profile event
- Publish a profile or use a different relay

**Updating Relay Configuration**:

Edit your config file (see [Configuration File Locations](#configuration-file-locations)) and update the `relays` list:

```toml
[nostr]
relays = [
    "wss://relay.damus.io",
    "wss://nos.lol",
    "wss://relay.snort.social",
    "wss://relay.primal.net",
    "wss://nostr.mom"
]
```

**Recommended General-Purpose Relays** (as of January 2026):
- `wss://relay.damus.io` - High traffic, reliable
- `wss://nos.lol` - Reliable
- `wss://relay.snort.social` - General purpose
- `wss://relay.primal.net` - Primal's relay
- `wss://nostr.mom` - Reliable

Find more relays at [nostr.watch](https://nostr.watch).

### Getting Help

If you encounter issues not covered here:

1. Run with `--verbose` flag to see detailed logs:
   ```bash
   plur-post "Test" --verbose
   ```

2. Check the error message carefully - Plurcast provides actionable error messages

3. Verify your configuration file syntax:
   ```bash
   cat ~/.config/plurcast/config.toml
   ```

4. Check file permissions:
   ```bash
   ls -la ~/.config/plurcast/
   ```

5. Open an issue on GitHub with:
   - Error message (redact any credentials!)
   - Platform(s) affected
   - Steps to reproduce

---

## Security Best Practices

### File Permissions

Always set proper permissions on credential files:

```bash
chmod 600 ~/.config/plurcast/nostr.keys
chmod 600 ~/.config/plurcast/mastodon.token
```

This ensures only you can read these files.

### Credential Storage

- **Never** commit credential files to version control
- **Never** share your private keys or tokens
- **Never** post credentials in public forums or chat
- Use app passwords or tokens instead of account passwords when possible
- Regenerate tokens/passwords if you suspect they've been compromised

### Backup

Backup your credential files securely:

```bash
# Create encrypted backup
tar czf - ~/.config/plurcast/*.keys ~/.config/plurcast/*.token | \
    gpg -c > plurcast-credentials-backup.tar.gz.gpg

# Restore from backup
gpg -d plurcast-credentials-backup.tar.gz.gpg | tar xzf - -C ~/
```

### Revoking Access

If you need to revoke access:

**Nostr**: Generate a new key pair and update your profile on Nostr clients

**Mastodon**:
1. Go to Settings → Development
2. Find the Plurcast application
3. Click "Revoke" or "Delete"
4. Generate a new token if needed

**SSB**: Generate a new keypair (old identity will be abandoned)

---

## Next Steps

After setting up your platforms:

1. **Test each platform individually**:
   ```bash
   plur-post "Test Nostr" --platform nostr
   plur-post "Test Mastodon" --platform mastodon
   ```

2. **Try multi-platform posting**:
   ```bash
   plur-post "Hello from all platforms!"
   ```

3. **Query your history**:
   ```bash
   plur-history --limit 10
   ```

4. **Explore Unix composability**:
   ```bash
   echo "Piped content" | plur-post
   plur-history --format json | jq '.'
   ```

5. **Read the main README** for more examples and advanced usage

---

**Happy posting!** 🚀
