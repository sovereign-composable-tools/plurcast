+++
title = "Getting Started"
description = "Get posting to decentralized platforms in 5 minutes"
weight = 2
template = "section.html"
+++

# Getting Started

Get Plurcast running and post to decentralized platforms in under 5 minutes.

## Quick Installation

### Requirements
- **Rust** 1.70 or later ([install here](https://rustup.rs/))
- **Git** for cloning the repository

### Install from Source

```bash
# Clone the repository
git clone https://github.com/plurcast/plurcast.git
cd plurcast

# Build and install
cargo build --release
cargo install --path plur-post
cargo install --path plur-history
cargo install --path plur-creds
cargo install --path plur-setup

# Verify installation
plur-post --help
```

### Alternative: Run Directly

```bash
# Build once
cargo build --release

# Run tools directly
./target/release/plur-post --help
./target/release/plur-creds --help
./target/release/plur-setup
```

## Initial Setup

### Option 1: Interactive Setup (Recommended)

```bash
# Run the setup wizard
plur-setup

# Follow the prompts to:
# 1. Choose credential storage (OS keyring recommended)
# 2. Configure platforms (Nostr, Mastodon, Bluesky)
# 3. Test authentication
# 4. Make your first post
```

### Option 2: Manual Setup

#### 1. Create Configuration

On first run, Plurcast creates a default configuration:

```bash
# This creates ~/.config/plurcast/config.toml
plur-post "Hello world"
```

**Configuration locations:**
- **Linux/macOS**: `~/.config/plurcast/config.toml`
- **Windows**: `%APPDATA%\plurcast\config.toml`

#### 2. Configure Platforms

Edit your configuration file:

```toml
[database]
path = "~/.local/share/plurcast/posts.db"

[credentials]
storage = "keyring"  # or "encrypted" or "plain"

[nostr]
enabled = true
relays = [
    "wss://relay.damus.io",
    "wss://nos.lol",
    "wss://relay.nostr.band"
]

[mastodon]
enabled = true
instance = "mastodon.social"  # Change to your instance

[bluesky]
enabled = true
handle = "your-handle.bsky.social"  # Change to your handle

[defaults]
platforms = ["nostr", "mastodon", "bluesky"]
```

#### 3. Set Up Credentials

Use the credential manager to store your platform credentials securely:

```bash
# Set up Nostr (private key required)
plur-creds set nostr

# Set up Mastodon (OAuth token required)
plur-creds set mastodon

# Set up Bluesky (app password required)
plur-creds set bluesky

# Verify all credentials
plur-creds test --all
```

## Platform Setup Guides

### Nostr Setup

Nostr uses cryptographic key pairs for identity.

**Generate a new key** (if you don't have one):
```bash
# Using nak (recommended)
nak key generate

# Or use any Nostr client:
# - Damus (iOS/macOS)
# - Amethyst (Android) 
# - Snort (web)
```

**Configure Plurcast**:
```bash
plur-creds set nostr
# Enter your private key (hex or nsec format)
```

**Test posting**:
```bash
plur-post "Hello Nostr!" --platform nostr
```

### Mastodon Setup

Mastodon uses OAuth tokens for authentication.

**Get an access token**:
1. Log in to your Mastodon instance
2. Go to **Settings** → **Development** → **New Application**
3. Name: "Plurcast"
4. Scopes: `write:statuses`
5. Copy the **Access Token**

**Configure Plurcast**:
```bash
plur-creds set mastodon
# Enter your access token
```

**Test posting**:
```bash
plur-post "Hello Mastodon!" --platform mastodon
```

### Bluesky Setup

Bluesky uses app passwords for third-party applications.

**Generate an app password**:
1. Log in to [bsky.app](https://bsky.app)
2. Go to **Settings** → **Privacy and Security** → **App Passwords**
3. Click **Add App Password**
4. Name: "Plurcast"
5. Copy the generated password (format: `xxxx-xxxx-xxxx-xxxx`)

**Configure Plurcast**:
```bash
plur-creds set bluesky
# Enter your handle and app password
```

**Test posting**:
```bash
plur-post "Hello Bluesky!" --platform bluesky
```

## Your First Posts

### Basic Posting

```bash
# Post to all enabled platforms
plur-post "Hello decentralized world!"

# Output:
# nostr:note1abc123...
# mastodon:12345  
# bluesky:at://did:plc:xyz.../app.bsky.feed.post/abc

# Post to specific platform
plur-post "Nostr-specific content" --platform nostr

# Post to multiple specific platforms  
plur-post "Testing..." --platform nostr,mastodon
```

### Using Pipes (Unix Style)

```bash
# Post from stdin
echo "Hello from stdin" | plur-post

# Post file content
cat announcement.txt | plur-post

# Generate and post content
date +"Daily update: %Y-%m-%d" | plur-post

# Fun with fortune
fortune | plur-post --platform nostr
```

### JSON Output for Scripts

```bash
# Machine-readable output
plur-post "Test post" --format json
# [{"platform":"nostr","success":true,"post_id":"note1..."}]

# Use with jq for processing
plur-post "Test" --format json | jq '.[] | select(.success == true)'
```

## Viewing Your History

```bash
# View recent posts
plur-history

# Limit results
plur-history --limit 5

# Search content
plur-history --search "rust"

# Filter by platform
plur-history --platform nostr

# Date range
plur-history --since "2025-01-01" --until "2025-01-31"

# JSON output for processing
plur-history --format json | jq length
```

## Common Workflows

### Daily Status Updates

```bash
#!/bin/bash
# daily-status.sh

STATUS="Daily update $(date +%Y-%m-%d): $(uptime | awk '{print $3,$4}' | sed 's/,//')"

if plur-post "$STATUS"; then
    echo "✓ Posted daily status"
else
    echo "✗ Failed to post status" >&2
    exit 1
fi
```

### Conditional Posting

```bash
# Post to different platforms based on content length
if [ ${#message} -le 280 ]; then
    # Short message - post everywhere
    echo "$message" | plur-post
else
    # Long message - skip Twitter-like platforms
    echo "$message" | plur-post --platform nostr,mastodon
fi
```

### Content Preprocessing

```bash
# Process draft and post
cat draft.md | \
    pandoc -f markdown -t plain | \
    sed 's/^# //' | \
    plur-post --platform nostr
```

## Security Best Practices

### Credential Storage

```bash
# Audit your security setup
plur-creds audit

# Use OS keyring (most secure)
# Windows: Credential Manager
# macOS: Keychain  
# Linux: Secret Service (GNOME Keyring/KWallet)

# Check file permissions
ls -la ~/.config/plurcast/
```

### Content Validation

```bash
# Check content length before posting
wc -c < message.txt
# Should be under 100,000 bytes

# Test with draft mode first
echo "Test content" | plur-post --draft
```

## Troubleshooting

### Authentication Issues

```bash
# Test all credentials
plur-creds test --all

# Check specific platform
plur-creds test nostr

# Re-configure if needed
plur-creds set nostr
```

### Common Errors

**"Content too large"**:
```bash
# Check content size
wc -c < file.txt
# Must be under 100KB (100,000 bytes)
```

**"No default platforms configured"**:
```bash
# Edit config.toml, add:
[defaults]
platforms = ["nostr", "mastodon", "bluesky"]
```

**"Platform not configured"**:
```bash
# Set up credentials
plur-creds set platform_name

# Verify configuration
plur-creds list
```

### Getting Help

```bash
# Comprehensive help for each tool
plur-post --help
plur-history --help  
plur-creds --help
plur-setup --help

# Enable verbose logging
plur-post "test" --verbose
```

## Next Steps

Once you're posting successfully:

- **Read the [Philosophy](/philosophy/)** to understand the design principles
- **Explore [Documentation](/documentation/)** for advanced features
- **Check the [Roadmap](/roadmap/)** to see what's coming
- **Join the [Community](/community/)** to contribute and discuss

## Example Configurations

### Minimal Setup (Nostr Only)

```toml
[database]
path = "~/.local/share/plurcast/posts.db"

[credentials]
storage = "keyring"

[nostr]
enabled = true
relays = ["wss://relay.damus.io"]

[defaults]
platforms = ["nostr"]
```

### Power User Setup

```toml
[database]
path = "~/.local/share/plurcast/posts.db"

[credentials]
storage = "keyring"

[nostr]
enabled = true
relays = [
    "wss://relay.damus.io",
    "wss://nos.lol", 
    "wss://relay.nostr.band",
    "wss://relay.snort.social"
]

[mastodon]
enabled = true
instance = "hachyderm.io"  # Tech-focused instance

[bluesky]
enabled = true
handle = "user.bsky.social"

[defaults]
platforms = ["nostr", "mastodon", "bluesky"]
```

---

**You're ready to start posting!** 🚀

Try: `plur-post "Hello decentralized world!"` and watch your message appear across platforms.
