# Plurcast Platform Setup Guide

This guide provides detailed instructions for setting up each platform supported by Plurcast.

## Table of Contents

- [Quick Setup (Recommended)](#quick-setup-recommended)
- [Credential Storage](#credential-storage)
- [Manual Platform Setup](#manual-platform-setup)
  - [Nostr Setup](#nostr-setup)
  - [Mastodon Setup](#mastodon-setup)
  - [Bluesky Setup](#bluesky-setup)
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
2. **Configuring each platform** (Nostr, Mastodon, Bluesky)
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

[... similar for Mastodon and Bluesky ...]

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
plur-creds set bluesky

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
    "wss://relay.nostr.band",
    "wss://relay.snort.social",
    "wss://relay.primal.net"
]
```

**Popular Nostr Relays**:
- `wss://relay.damus.io` - General purpose, high traffic
- `wss://nos.lol` - General purpose, reliable
- `wss://relay.nostr.band` - Aggregator with search
- `wss://relay.snort.social` - General purpose
- `wss://relay.primal.net` - Primal's relay
- `wss://nostr.wine` - Paid relay (spam-free)
- `wss://relay.nostr.info` - General purpose

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

## Bluesky Setup

Bluesky uses the AT Protocol (Authenticated Transfer Protocol) with DID-based identity.

### Prerequisites

- A Bluesky account (https://bsky.app)
- Your Bluesky handle

### Step-by-Step Instructions

#### 1. Generate an App Password

Bluesky uses app passwords for third-party applications. This is NOT your account password.

1. Log in to Bluesky at https://bsky.app
2. Click on **Settings** (gear icon in the left sidebar)
3. Navigate to **Privacy and Security**
4. Scroll down to **App Passwords**
5. Click **Add App Password**
6. Enter a name for the app password:
   - Name: `Plurcast` (or any descriptive name)
7. Click **Create App Password**
8. Copy the generated password immediately
   - Format: `xxxx-xxxx-xxxx-xxxx` (four groups of characters)
   - **Important**: You won't be able to see this password again!

**Security Note**: App passwords are safer than using your main password because:
- They can be revoked individually
- They don't grant access to account settings
- You can create multiple passwords for different apps

#### 2. Create the Auth File

The Bluesky auth file contains two lines: your handle and your app password.

```bash
# Create the auth file
touch ~/.config/plurcast/bluesky.auth

# Set proper permissions
chmod 600 ~/.config/plurcast/bluesky.auth

# Add your handle (line 1) and app password (line 2)
cat > ~/.config/plurcast/bluesky.auth << EOF
your-handle.bsky.social
xxxx-xxxx-xxxx-xxxx
EOF
```

**Example** (with fake credentials):
```
alice.bsky.social
abcd-efgh-ijkl-mnop
```

**Handle formats**:
- Standard: `username.bsky.social`
- Custom domain: `username.com` (if you've set up a custom handle)

#### 3. Configure Bluesky

Edit `~/.config/plurcast/config.toml`:

```toml
[bluesky]
enabled = true
handle = "your-handle.bsky.social"
auth_file = "~/.config/plurcast/bluesky.auth"
```

**Important**: The handle in config.toml should match the handle in your auth file (line 1).

#### 4. Test Your Setup

```bash
# Post a test message to Bluesky only
plur-post "Hello Bluesky! Testing Plurcast." --platform bluesky

# Expected output:
# bluesky:at://did:plc:abc123.../app.bsky.feed.post/xyz789

# This is an AT URI (AT Protocol URI)
```

#### 5. Verify on Bluesky

Visit your Bluesky profile at https://bsky.app to see the post.

### Character Limit

Bluesky has a **300 character limit** for posts. This is enforced by the protocol.

If your content exceeds 300 characters, you'll get an error:
```
Error: bluesky: Content validation failed: Post exceeds 300 character limit
```

**Solution**: Either shorten your content or exclude Bluesky from the post:
```bash
# Post to other platforms only
plur-post "Long content..." --platform nostr,mastodon
```

### Understanding AT URIs

Bluesky uses AT URIs to identify posts:

```
at://did:plc:abc123xyz.../app.bsky.feed.post/xyz789
```

- `at://` - AT Protocol scheme
- `did:plc:abc123xyz...` - Your DID (Decentralized Identifier)
- `app.bsky.feed.post` - Record type (a post)
- `xyz789` - Record key (unique ID for this post)

### Troubleshooting Bluesky

**"Invalid handle or password"**:
- Verify your handle is correct (e.g., `user.bsky.social`)
- Ensure you're using an app password, not your account password
- Check that the auth file has two lines: handle, then password
- Regenerate your app password if needed

**"Auth file format error"**:
- Ensure the auth file has exactly two lines
- Line 1: Your handle
- Line 2: Your app password
- No extra spaces or newlines

**"PDS unreachable"**:
- Check your internet connection
- Bluesky's servers may be temporarily down
- Try again in a few minutes

**"Content too long"**:
- Bluesky has a 300 character limit
- Shorten your content or exclude Bluesky from the post

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
    "wss://relay.nostr.band",
    "wss://relay.snort.social"
]

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
platforms = ["nostr", "bluesky"]  # Mastodon excluded
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

### General Issues

**"Configuration file not found"**:
- Plurcast will create a default config on first run
- Default location: `~/.config/plurcast/config.toml`
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
chmod 600 ~/.config/plurcast/bluesky.auth
```

This ensures only you can read these files.

### Credential Storage

- **Never** commit credential files to version control
- **Never** share your private keys or tokens
- **Never** post credentials in public forums or chat
- Use app passwords (Bluesky) instead of account passwords when possible
- Regenerate tokens/passwords if you suspect they've been compromised

### Backup

Backup your credential files securely:

```bash
# Create encrypted backup
tar czf - ~/.config/plurcast/*.keys ~/.config/plurcast/*.token ~/.config/plurcast/*.auth | \
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

**Bluesky**:
1. Go to Settings → Privacy and Security → App Passwords
2. Find the Plurcast app password
3. Click "Revoke"
4. Generate a new app password if needed

---

## Next Steps

After setting up your platforms:

1. **Test each platform individually**:
   ```bash
   plur-post "Test Nostr" --platform nostr
   plur-post "Test Mastodon" --platform mastodon
   plur-post "Test Bluesky" --platform bluesky
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
