# Plurcast

**Cast to many** - Unix tools for the decentralized social web

Plurcast is a collection of Unix command-line tools for posting to decentralized social media platforms like Nostr, Mastodon, and Bluesky. Following Unix philosophy, each tool does one thing well and composes naturally with other command-line utilities.

## Status

**Alpha Release (v0.2.0)** - Multi-platform support with Nostr and Mastodon

### Platform Support

- ✅ **Nostr** - Tested and stable
- ✅ **Mastodon** - Tested and stable
- 🚧 **Bluesky** - Implemented but not fully tested (stretch goal)

## Features

- ✅ Post to Nostr and Mastodon from command line
- ✅ Multi-platform posting with concurrent execution
- ✅ Query posting history with `plur-history`
- ✅ Secure credential storage (OS keyring, encrypted files, or plain text)
- ✅ Interactive setup wizard (`plur-setup`)
- ✅ Credential management (`plur-creds`)
- ✅ Local SQLite database for post history
- ✅ TOML-based configuration with XDG Base Directory support
- ✅ Unix-friendly: reads from stdin, outputs to stdout, meaningful exit codes
- ✅ Agent-friendly: JSON output mode, comprehensive help text
- 🚧 Bluesky support (implemented, needs testing)
- 🚧 Post scheduling (coming soon)

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/plurcast/plurcast.git
cd plurcast

# Build with cargo
cargo build --release

# Install binaries to ~/.cargo/bin
cargo install --path plur-post

# Or run directly
./target/release/plur-post --help
```

### Requirements

- Rust 1.70 or later
- SQLite 3.x (bundled via sqlx)

## Quick Start

### 1. Initial Setup

On first run, Plurcast will create a default configuration file:

```bash
# This will create ~/.config/plurcast/config.toml
plur-post "Hello world"
```

### 2. Configure Platforms

Edit your configuration file at `~/.config/plurcast/config.toml`:

```toml
[database]
path = "~/.local/share/plurcast/posts.db"

[nostr]
enabled = true
keys_file = "~/.config/plurcast/nostr.keys"
relays = [
    "wss://relay.damus.io",
    "wss://nos.lol",
    "wss://relay.nostr.band"
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
# Default platforms to post to (can override with --platform flag)
platforms = ["nostr", "mastodon"]
```

**Note**: Bluesky support is implemented but not fully tested. See the [Platform Setup Guides](#platform-setup-guides) section below for detailed instructions on obtaining credentials for each platform.

### 3. Set Up Nostr Keys

Create a keys file at `~/.config/plurcast/nostr.keys` with your Nostr private key:

**Option A: Hex format (64 characters)**
```
a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456
```

**Option B: Bech32 format (nsec)**
```
nsec1abc123def456...
```

**Important**: Set proper file permissions to protect your private key:
```bash
chmod 600 ~/.config/plurcast/nostr.keys
```

**Generating New Keys**: If you don't have a Nostr key yet, you can generate one using:
- [nak](https://github.com/fiatjaf/nak) - `nak key generate`
- [nostr-tools](https://github.com/nbd-wtf/nostr-tools) - JavaScript library
- Any Nostr client (Damus, Amethyst, etc.)

### 4. Post Your First Message

```bash
# Post to all enabled platforms
plur-post "Hello decentralized world!"
# Output:
# nostr:note1abc123...
# mastodon:12345
# bluesky:at://did:plc:xyz.../app.bsky.feed.post/abc

# Post from stdin
echo "Hello from stdin" | plur-post

# Post to specific platform only
echo "Nostr-only post" | plur-post --platform nostr

# Post to multiple specific platforms
echo "Nostr and Mastodon" | plur-post --platform nostr,mastodon
```

## Usage

### Basic Posting

```bash
# Post from argument
plur-post "Your message here"

# Post from stdin (pipe)
echo "Your message" | plur-post

# Post from file
cat message.txt | plur-post
```

### Content Size Limits

Plurcast enforces a maximum content length of **100KB (100,000 bytes)** to prevent memory exhaustion and ensure reliable posting across platforms.

**Why 100KB?**
- Sufficient for very long posts (≈50,000 words)
- Well above any platform's actual limits (Nostr: ~32KB, Mastodon: 500 chars default, Bluesky: 300 chars)
- Protects against memory exhaustion and DoS attacks
- Ensures database stability

**Examples**:

```bash
# Valid post (under 100KB)
plur-post "This is a normal post"
# Output: nostr:note1abc123...

# Oversized post (over 100KB) - REJECTED
plur-post "$(python -c 'print("x"*100001)')"
# Error: Content too large: 100001 bytes (maximum: 100000 bytes)
# Exit code: 3

# Large file exceeding limit - REJECTED
cat huge_file.txt | plur-post
# Error: Content too large: exceeds 100000 bytes (maximum: 100000 bytes)
# Exit code: 3
```

**Security Note**: The size limit is enforced before reading the entire input stream, preventing infinite stream attacks like `cat /dev/zero | plur-post`.

### Multi-Platform Posting

```bash
# Post to all enabled platforms (from config defaults)
plur-post "Hello everyone!"
# Output:
# nostr:note1abc123...
# mastodon:12345
# bluesky:at://did:plc:xyz.../app.bsky.feed.post/abc

# Post to specific platform only
plur-post "Nostr only" --platform nostr
# Output:
# nostr:note1abc123...

# Post to multiple specific platforms
plur-post "Nostr and Bluesky" --platform nostr,bluesky
# Output:
# nostr:note1abc123...
# bluesky:at://did:plc:xyz.../app.bsky.feed.post/abc

# Handle partial failures gracefully
plur-post "Test post" --platform nostr,mastodon,bluesky
# If Mastodon fails but others succeed:
# nostr:note1abc123...
# Error: mastodon: Authentication failed
# bluesky:at://did:plc:xyz.../app.bsky.feed.post/abc
# Exit code: 1 (partial failure)
```

### Draft Mode

```bash
# Save without posting
echo "Draft content" | plur-post --draft

# Output: draft:550e8400-e29b-41d4-a716-446655440000
```

### Output Formats

```bash
# Text format (default)
plur-post "Hello"
# Output: nostr:note1abc123...

# JSON format (machine-readable)
plur-post "Hello" --format json
# Output: [{"platform":"nostr","success":true,"post_id":"note1..."}]
```

### Verbose Logging

```bash
# Enable debug logging to stderr
plur-post "Debug post" --verbose
```

### Querying History

```bash
# View recent posts (default: last 20)
plur-history
# Output:
# 2025-10-05 14:30:00 | abc-123 | Hello world
#   ✓ nostr: note1abc...
#   ✓ mastodon: 12345
#   ✗ bluesky: Authentication failed

# Filter by platform
plur-history --platform nostr

# Filter by date range
plur-history --since "2025-10-01" --until "2025-10-05"

# Search content
plur-history --search "rust"

# Limit results
plur-history --limit 50

# JSON output for scripting
plur-history --format json
# Output: [{"post_id":"abc-123","content":"Hello world",...}]

# JSONL output (one JSON object per line)
plur-history --format jsonl

# CSV output for spreadsheets
plur-history --format csv
# Output: post_id,timestamp,platform,success,platform_post_id,error,content
```

## Configuration

### Configuration File Location

Default: `~/.config/plurcast/config.toml`

Override with environment variable:
```bash
export PLURCAST_CONFIG=/path/to/custom/config.toml
```

### Configuration Format

```toml
[database]
# Database location (supports ~ expansion)
path = "~/.local/share/plurcast/posts.db"

[nostr]
enabled = true
# Path to file containing private key (hex or nsec format)
keys_file = "~/.config/plurcast/nostr.keys"
# List of relay URLs
relays = [
    "wss://relay.damus.io",
    "wss://nos.lol",
    "wss://relay.nostr.band",
    "wss://relay.snort.social"
]

[mastodon]
enabled = true
# Mastodon instance URL (or other Fediverse platform)
instance = "mastodon.social"
# Path to file containing OAuth access token
token_file = "~/.config/plurcast/mastodon.token"

[bluesky]
enabled = true
# Your Bluesky handle
handle = "user.bsky.social"
# Path to file containing handle and app password (two lines)
auth_file = "~/.config/plurcast/bluesky.auth"

[defaults]
# Default platforms to post to (can override with --platform flag)
platforms = ["nostr", "mastodon", "bluesky"]
```

**Platform-specific notes**:

- **Nostr**: Requires private key (hex or nsec format) in keys_file
- **Mastodon**: Requires OAuth access token in token_file. Works with any Fediverse platform (Mastodon, Pleroma, Friendica, etc.)
- **Bluesky**: Requires handle (line 1) and app password (line 2) in auth_file

**Credential file formats**:

`~/.config/plurcast/nostr.keys`:
```
a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456
```

`~/.config/plurcast/mastodon.token`:
```
your-oauth-access-token-here
```

`~/.config/plurcast/bluesky.auth`:
```
your-handle.bsky.social
xxxx-xxxx-xxxx-xxxx
```

**Security**: All credential files should have 600 permissions:
```bash
chmod 600 ~/.config/plurcast/*.keys ~/.config/plurcast/*.token ~/.config/plurcast/*.auth
```

### Environment Variables

- `PLURCAST_CONFIG` - Override configuration file location
- `PLURCAST_DB_PATH` - Override database file location

Example:
```bash
export PLURCAST_CONFIG=~/my-config.toml
export PLURCAST_DB_PATH=~/my-posts.db
plur-post "Using custom paths"
```

## Exit Codes

Plurcast follows Unix conventions for exit codes:

- **0** - Success on all platforms
- **1** - Posting failed on at least one platform
- **2** - Authentication error (missing/invalid credentials)
- **3** - Invalid input (empty content, malformed arguments, content too large)

Example usage in scripts:
```bash
if plur-post "Test post"; then
    echo "Posted successfully"
else
    case $? in
        1) echo "Posting failed" ;;
        2) echo "Authentication error" ;;
        3) echo "Invalid input" ;;
    esac
fi
```

### Error Messages

Plurcast provides clear, actionable error messages:

**Content Too Large**:
```bash
$ plur-post "$(python -c 'print("x"*100001)')"
Error: Content too large: 100001 bytes (maximum: 100000 bytes)
$ echo $?
3
```

**Empty Content**:
```bash
$ echo "" | plur-post
Error: Content cannot be empty
$ echo $?
3
```

**Authentication Error**:
```bash
$ plur-post "Test" # with missing keys file
Error: Authentication failed: Could not read Nostr keys file
$ echo $?
2
```

All error messages are written to stderr, keeping stdout clean for piping and scripting.

## Platform Setup Guides

### Nostr Setup

Nostr uses cryptographic key pairs for identity. You'll need a private key to post.

**Step 1: Generate or obtain a Nostr private key**

If you don't have a Nostr key yet, you can generate one using:

- **nak** (recommended): `nak key generate`
- **nostr-tools**: JavaScript library
- **Any Nostr client**: Damus (iOS), Amethyst (Android), Snort (web), etc.

**Step 2: Create the keys file**

Create `~/.config/plurcast/nostr.keys` with your private key:

```bash
# Create the file
touch ~/.config/plurcast/nostr.keys
chmod 600 ~/.config/plurcast/nostr.keys

# Add your key (choose one format):
# Option A: Hex format (64 characters)
echo "a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456" > ~/.config/plurcast/nostr.keys

# Option B: Bech32 format (nsec)
echo "nsec1abc123def456..." > ~/.config/plurcast/nostr.keys
```

**Step 3: Configure relays**

Edit `~/.config/plurcast/config.toml`:

```toml
[nostr]
enabled = true
keys_file = "~/.config/plurcast/nostr.keys"
relays = [
    "wss://relay.damus.io",
    "wss://nos.lol",
    "wss://relay.nostr.band",
    "wss://relay.snort.social"
]
```

**Step 4: Test**

```bash
plur-post "Hello Nostr!" --platform nostr
```

**Finding your public key (npub)**:
```bash
# If you have nak installed
nak key public $(cat ~/.config/plurcast/nostr.keys)
```

### Mastodon Setup

Mastodon uses OAuth2 for authentication. You'll need to generate an access token.

**Step 1: Generate an access token**

1. Log in to your Mastodon instance (e.g., mastodon.social)
2. Go to **Settings** → **Development** → **New Application**
3. Fill in the application details:
   - **Application name**: Plurcast
   - **Scopes**: Select `write:statuses` (minimum required)
   - **Redirect URI**: `urn:ietf:wg:oauth:2.0:oob` (for command-line apps)
4. Click **Submit**
5. Copy the **Access Token** (starts with a long string of characters)

**Step 2: Create the token file**

```bash
# Create the file
touch ~/.config/plurcast/mastodon.token
chmod 600 ~/.config/plurcast/mastodon.token

# Add your token
echo "your-access-token-here" > ~/.config/plurcast/mastodon.token
```

**Step 3: Configure Mastodon**

Edit `~/.config/plurcast/config.toml`:

```toml
[mastodon]
enabled = true
instance = "mastodon.social"  # Change to your instance
token_file = "~/.config/plurcast/mastodon.token"
```

**Step 4: Test**

```bash
plur-post "Hello Mastodon!" --platform mastodon
```

**Supported Fediverse platforms**:
- Mastodon
- Pleroma
- Friendica
- Firefish
- GoToSocial
- Akkoma

Just change the `instance` URL to your platform's domain.

### Bluesky Setup

Bluesky uses app passwords for third-party applications.

**Step 1: Generate an app password**

1. Log in to Bluesky (https://bsky.app)
2. Go to **Settings** → **Privacy and Security** → **App Passwords**
3. Click **Add App Password**
4. Enter a name: **Plurcast**
5. Click **Create App Password**
6. Copy the generated password (format: `xxxx-xxxx-xxxx-xxxx`)

**Important**: This is NOT your account password. It's a special password for third-party apps.

**Step 2: Create the auth file**

```bash
# Create the file
touch ~/.config/plurcast/bluesky.auth
chmod 600 ~/.config/plurcast/bluesky.auth

# Add your handle and app password (one per line)
echo "your-handle.bsky.social" > ~/.config/plurcast/bluesky.auth
echo "xxxx-xxxx-xxxx-xxxx" >> ~/.config/plurcast/bluesky.auth
```

**Step 3: Configure Bluesky**

Edit `~/.config/plurcast/config.toml`:

```toml
[bluesky]
enabled = true
handle = "your-handle.bsky.social"
auth_file = "~/.config/plurcast/bluesky.auth"
```

**Step 4: Test**

```bash
plur-post "Hello Bluesky!" --platform bluesky
```

**Character limit**: Bluesky has a 300 character limit for posts.

## Security

Plurcast provides multiple options for storing your platform credentials securely.

### Credential Storage Backends

Plurcast supports three storage backends, with automatic fallback:

1. **OS Keyring (Recommended)** - Most secure, integrated with your operating system
   - **macOS**: Keychain via Security framework
   - **Windows**: Credential Manager via Windows API
   - **Linux**: Secret Service (GNOME Keyring/KWallet) via D-Bus

2. **Encrypted Files** - Password-protected files using age encryption
   - Good for systems without keyring support
   - Requires master password
   - Files stored in `~/.config/plurcast/credentials/`

3. **Plain Text** - Legacy compatibility only (not recommended)
   - Credentials stored in plain text files
   - Only for testing or backward compatibility
   - **Security risk** - use only if other options unavailable

### Configuration

Add a `[credentials]` section to your `config.toml`:

```toml
[credentials]
# Storage backend: "keyring" (OS native), "encrypted" (password-protected files), "plain" (not recommended)
storage = "keyring"
# Path for encrypted/plain file storage (keyring doesn't use files)
path = "~/.config/plurcast/credentials"
```

### Interactive Setup Wizard

The easiest way to configure credentials is using the interactive setup wizard:

```bash
plur-setup
```

This will guide you through:
1. Choosing a storage backend
2. Configuring credentials for each platform
3. Testing authentication
4. Saving your configuration

### Managing Credentials

Use `plur-creds` to manage your credentials:

```bash
# Set credentials for a platform
plur-creds set nostr
plur-creds set mastodon
plur-creds set bluesky

# List configured platforms (doesn't show credential values)
plur-creds list

# Test authentication
plur-creds test nostr
plur-creds test --all

# Delete credentials
plur-creds delete nostr

# Audit security (check for plain text files, file permissions, etc.)
plur-creds audit
```

### Migrating from Plain Text Files

If you're upgrading from an earlier version that used plain text files:

```bash
# Migrate credentials to secure storage
plur-creds migrate

# This will:
# 1. Detect plain text credential files
# 2. Copy them to secure storage (keyring or encrypted)
# 3. Verify authentication works
# 4. Optionally delete plain text files
```

### Master Password for Encrypted Storage

If using encrypted file storage, set your master password via:

**Option 1: Environment variable**
```bash
export PLURCAST_MASTER_PASSWORD="your_secure_password"
```

**Option 2: Interactive prompt**
```bash
# Plurcast will prompt for password when needed (if TTY available)
plur-post "Hello world"
# Enter master password: ********
```

**Password Requirements**:
- Minimum 8 characters
- Stored only in memory during session
- Never logged or written to disk

### Security Best Practices

1. **Use OS Keyring** - Most secure option, integrated with your system
2. **Set File Permissions** - Ensure credential files are readable only by you:
   ```bash
   chmod 600 ~/.config/plurcast/credentials/*
   chmod 600 ~/.config/plurcast/*.keys
   chmod 600 ~/.config/plurcast/*.token
   chmod 600 ~/.config/plurcast/*.auth
   ```
3. **Use Strong Master Password** - If using encrypted storage, choose a strong password
4. **Audit Regularly** - Run `plur-creds audit` to check for security issues
5. **Migrate from Plain Text** - If you have plain text credentials, migrate them:
   ```bash
   plur-creds migrate
   ```

### What's Protected

- **Nostr**: Private keys (hex or nsec format)
- **Mastodon**: Access tokens
- **Bluesky**: App passwords

### What's Not Sensitive

These are stored in `config.toml` (not encrypted):
- Mastodon instance URLs
- Bluesky handles
- Nostr relay URLs
- Database paths

### Troubleshooting Credentials

**Keyring not available**:
```bash
# Error: OS keyring not available
# Solution: Use encrypted storage instead
plur-creds set nostr  # Will automatically fall back to encrypted storage
```

**Forgot master password**:
```bash
# If you forget your master password, you'll need to:
# 1. Delete encrypted files
rm -rf ~/.config/plurcast/credentials/
# 2. Reconfigure credentials
plur-setup
```

**Migration failed**:
```bash
# Check what went wrong
plur-creds audit

# Try manual migration
plur-creds set nostr  # Enter credentials manually
plur-creds test nostr  # Verify it works
```

## Troubleshooting

### "Authentication failed: Could not read Nostr keys file"

**Cause**: Keys file doesn't exist or has wrong path

**Solution**:
1. Check the path in your config.toml: `keys_file = "~/.config/plurcast/nostr.keys"`
2. Ensure the file exists: `ls -la ~/.config/plurcast/nostr.keys`
3. Verify file permissions: `chmod 600 ~/.config/plurcast/nostr.keys`

### "Invalid private key format"

**Cause**: Keys file contains invalid key format

**Solution**:
- Ensure key is either 64-character hex OR bech32 nsec format
- Remove any whitespace or newlines
- Hex example: `a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456`
- Nsec example: `nsec1abc123def456...`

### "No content provided"

**Cause**: No content argument and stdin is a TTY

**Solution**:
```bash
# Provide content as argument
plur-post "Your message"

# OR pipe content via stdin
echo "Your message" | plur-post
```

### "Content too large"

**Cause**: Content exceeds 100KB (100,000 bytes) limit

**Error Example**:
```
Content too large: 150000 bytes (maximum: 100000 bytes)
```

**Solution**:
- Reduce content size to under 100KB
- Split long content into multiple posts
- Remove unnecessary whitespace or formatting
- For very long content, consider using a paste service and posting a link

**Prevention**:
```bash
# Check content size before posting
wc -c < message.txt
# If under 100000, safe to post
cat message.txt | plur-post
```

### "Failed to connect to relay" (Nostr)

**Cause**: Network issues or relay is down

**Solution**:
- Check your internet connection
- Try different relays in config.toml
- Use `--verbose` flag to see detailed connection logs
- Plurcast succeeds if ANY relay accepts the post

### "Authentication failed" (Mastodon)

**Cause**: Invalid or expired OAuth token

**Solution**:
1. Regenerate your access token in Mastodon settings
2. Update `~/.config/plurcast/mastodon.token` with the new token
3. Verify the instance URL is correct in config.toml
4. Check file permissions: `chmod 600 ~/.config/plurcast/mastodon.token`

### "Authentication failed" (Bluesky)

**Cause**: Invalid handle or app password

**Solution**:
1. Verify your handle is correct (e.g., `user.bsky.social`)
2. Regenerate your app password in Bluesky settings
3. Update `~/.config/plurcast/bluesky.auth` with handle and new password
4. Check file permissions: `chmod 600 ~/.config/plurcast/bluesky.auth`
5. Ensure the auth file has two lines: handle on line 1, password on line 2

### "Content validation failed: Post exceeds character limit"

**Cause**: Content is too long for the target platform

**Platform limits**:
- Nostr: ~32KB (practical limit)
- Mastodon: 500 characters (default, varies by instance)
- Bluesky: 300 characters

**Solution**:
- Shorten your content
- Use `--platform` to exclude platforms with stricter limits
- Split into multiple posts or threads (future feature)

Example:
```bash
# Long post fails on Bluesky
plur-post "Very long content..." --platform nostr,mastodon,bluesky
# Error: bluesky: Content validation failed: Post exceeds 300 character limit

# Post to platforms with higher limits
plur-post "Very long content..." --platform nostr,mastodon
```

### "Rate limit exceeded"

**Cause**: Too many posts in a short time

**Solution**:
- Wait before posting again (typically 1-5 minutes)
- Plurcast will automatically retry with exponential backoff
- Use `--verbose` to see retry attempts

### "Database error: unable to open database file"

**Cause**: Database directory doesn't exist or lacks permissions

**Solution**:
```bash
# Create directory
mkdir -p ~/.local/share/plurcast

# Check permissions
ls -la ~/.local/share/plurcast
```

### Configuration file not found

**Cause**: First run or config file deleted

**Solution**: Plurcast will create a default config on first run. If you need to recreate it:
```bash
mkdir -p ~/.config/plurcast
# Then run plur-post, it will create default config
```

## Database

Plurcast stores all data locally in SQLite:

**Location**: `~/.local/share/plurcast/posts.db`

**Schema**:
- `posts` - Your authored posts
- `post_records` - Platform-specific posting records
- `platforms` - Platform configurations

**Backup**:
```bash
# Simple file copy
cp ~/.local/share/plurcast/posts.db ~/backup/posts-$(date +%Y%m%d).db

# Or use SQLite backup
sqlite3 ~/.local/share/plurcast/posts.db ".backup ~/backup/posts.db"
```

## Unix Philosophy

Plurcast follows Unix principles:

- **Do one thing well**: Each tool has a single, focused purpose
- **Text streams**: Universal interface via stdin/stdout
- **Composability**: Tools work together via pipes
- **Silence is golden**: Only output what's needed
- **Exit codes**: Meaningful status for scripting
- **Agent-friendly**: Works equally well for humans and AI agents

### Unix Composability Examples

**Piping with plur-post**:

```bash
# Post with text preprocessing
cat draft.txt | sed 's/foo/bar/g' | plur-post

# Post with template substitution
echo "Hello from $(hostname) at $(date)" | plur-post

# Post from command output
fortune | plur-post --platform nostr

# Post with word count check
cat post.txt | tee >(wc -w >&2) | plur-post

# Chain multiple transformations
cat draft.txt | tr '[:upper:]' '[:lower:]' | sed 's/draft/final/g' | plur-post
```

**Filtering with plur-history and jq**:

```bash
# Get only successful posts
plur-history --format json | jq '.[] | select(.platforms[].success == true)'

# Extract post IDs from specific platform
plur-history --format json | jq -r '.[] | .platforms[] | select(.platform == "nostr") | .platform_post_id'

# Count posts per platform
plur-history --format json | jq '[.[] | .platforms[] | .platform] | group_by(.) | map({platform: .[0], count: length})'

# Find failed posts
plur-history --format json | jq '.[] | select(.platforms[] | .success == false)'

# Get posts from last week with errors
plur-history --since "7 days ago" --format json | jq '.[] | select(.platforms[].error != null)'
```

**CSV processing and analysis**:

```bash
# Export to CSV and analyze with standard tools
plur-history --format csv > posts.csv

# Count posts per platform
cut -d, -f3 posts.csv | tail -n +2 | sort | uniq -c

# Find all failures
grep ",false," posts.csv

# Get success rate per platform
awk -F, 'NR>1 {total[$3]++; if($4=="true") success[$3]++} END {for(p in total) print p, success[p]/total[p]*100"%"}' posts.csv
```

**Conditional posting based on content**:

```bash
# Post to different platforms based on content
if grep -q "urgent" message.txt; then
    cat message.txt | plur-post --platform nostr,mastodon,bluesky
else
    cat message.txt | plur-post --platform nostr
fi

# Post only if content is short enough for Bluesky
if [ $(wc -c < message.txt) -le 300 ]; then
    cat message.txt | plur-post --platform bluesky
else
    cat message.txt | plur-post --platform nostr,mastodon
fi
```

**Automation with shell scripts**:

```bash
#!/bin/bash
# daily-post.sh - Automated daily posting

# Generate content
CONTENT="Daily update: $(date +%Y-%m-%d) - $(uptime | awk '{print $3,$4}')"

# Post and handle errors
if plur-post "$CONTENT" --format json > /tmp/post-result.json; then
    echo "✓ Posted successfully"
    jq -r '.[] | "\(.platform): \(.post_id)"' /tmp/post-result.json
else
    EXIT_CODE=$?
    case $EXIT_CODE in
        1) echo "⚠ Partial failure - check logs" >&2 ;;
        2) echo "✗ Authentication error - check credentials" >&2 ;;
        3) echo "✗ Invalid input" >&2 ;;
    esac
    exit $EXIT_CODE
fi
```

**Agent-friendly workflows**:

```bash
# AI agent can discover capabilities
plur-post --help | grep -E "^\s+--"

# Agent can validate before posting
echo "$CONTENT" | plur-post --draft --format json | jq -r '.post_id'

# Agent can query history and make decisions
LAST_POST=$(plur-history --limit 1 --format json | jq -r '.[0].content')
if [ "$LAST_POST" != "$NEW_CONTENT" ]; then
    echo "$NEW_CONTENT" | plur-post
fi

# Agent can handle partial failures
plur-post "$CONTENT" --format json | jq -e '.[] | select(.success == false)' && {
    echo "Retrying failed platforms..." >&2
    # Retry logic here
}
```

**Integration with other tools**:

```bash
# Post from RSS feed
curl -s https://example.com/feed.xml | xmllint --xpath '//item[1]/title/text()' - | plur-post

# Post from clipboard (Linux)
xclip -o | plur-post

# Post from clipboard (macOS)
pbpaste | plur-post

# Post with notification
plur-post "Hello" && notify-send "Posted successfully"

# Scheduled posting with cron
# Add to crontab: 0 9 * * * echo "Good morning!" | /usr/local/bin/plur-post
```

## Development

### Project Structure

```
plurcast/
├── libplurcast/          # Shared library
│   ├── src/
│   │   ├── lib.rs
│   │   ├── config.rs     # Configuration management
│   │   ├── db.rs         # Database operations
│   │   ├── error.rs      # Error types
│   │   ├── types.rs      # Shared types
│   │   └── platforms/    # Platform implementations
│   │       ├── mod.rs
│   │       └── nostr.rs
│   └── migrations/       # SQLx migrations
├── plur-post/            # Post binary
│   └── src/
│       └── main.rs
└── Cargo.toml            # Workspace manifest
```

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Check without building
cargo check
```

## Roadmap

### Phase 1: Foundation (Complete)
- [x] Core database schema
- [x] Configuration system
- [x] Nostr platform support
- [x] Basic plur-post tool
- [x] Unix philosophy implementation
- [x] Agent-friendly features

### Phase 2: Multi-Platform Alpha (Current - Complete)
- [x] Platform abstraction trait
- [x] Mastodon integration
- [x] Bluesky integration
- [x] Multi-platform posting with concurrent execution
- [x] plur-history query tool
- [x] Comprehensive documentation
- [x] Platform setup guides

### Phase 3: CLI Polish & Service Layer (Next)
- [ ] Service layer extraction (for GUI reuse)
- [ ] CLI improvements (better error messages, progress indicators)
- [ ] Multi-account support
- [ ] Configuration validation and migration tools

### Phase 4: Desktop GUI (Planned)
- [ ] Tauri-based desktop application
- [ ] Visual post composer with platform preview
- [ ] History browser with search and filtering
- [ ] Draft management
- [ ] Account management UI

### Phase 5: Scheduling (Planned)
- [ ] plur-queue (scheduling)
- [ ] plur-send (daemon)
- [ ] Rate limiting per platform

### Phase 6: Data Portability (Planned)
- [ ] plur-import (data import)
- [ ] plur-export (data export)
- [ ] Migration utilities

### Future Enhancements
- [ ] Media attachments
- [ ] Thread support
- [ ] Semantic search with embeddings
- [ ] Reply handling

## Contributing

Contributions welcome! Please:

1. Follow Unix philosophy principles
2. Write tests for new features
3. Update documentation
4. Use conventional commits

## License

MIT OR Apache-2.0 (dual-licensed)

## Links

- **Repository**: https://github.com/plurcast/plurcast
- **Issues**: https://github.com/plurcast/plurcast/issues
- **Nostr**: [NIP-01](https://github.com/nostr-protocol/nips/blob/master/01.md)

## Acknowledgments

Built with mature, open-source Rust libraries:
- [nostr-sdk](https://github.com/rust-nostr/nostr) - Nostr protocol implementation
- [sqlx](https://github.com/launchbadge/sqlx) - Async SQL toolkit
- [tokio](https://github.com/tokio-rs/tokio) - Async runtime
- [clap](https://github.com/clap-rs/clap) - Command-line parser

---

**Plurcast** - Cast to many, own your data, follow Unix principles.
