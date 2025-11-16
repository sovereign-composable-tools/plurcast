# Plurcast

**Cast to many** - Unix tools for the decentralized social web

Plurcast is a collection of Unix command-line tools for posting to decentralized social media platforms like Nostr and Mastodon, with SSB (Secure Scuttlebutt) support planned. Following Unix philosophy, each tool does one thing well and composes naturally with other command-line utilities.

## Status

**Alpha Release (v0.3.0-alpha2)** - Multi-Account Support

### Platform Support

- ✅ **Nostr** - Tested and stable (with shared test account easter egg!)
- ✅ **Mastodon** - Tested and stable (supports all ActivityPub platforms)
- ⚗️ **SSB (Secure Scuttlebutt)** - Experimental reference implementation (local posting works, network replication limited)

**Platform Decision**: Removed Bluesky (centralized, banned test accounts). Replaced with SSB - truly decentralized, offline-first, and philosophically aligned with Plurcast values.

**SSB Status**: SSB integration is experimental and demonstrates the Platform trait architecture. Local posting, keypair management, and multi-account support work well. Network replication to pub servers is designed but not fully implemented. See [docs/SSB_SETUP.md](docs/SSB_SETUP.md) for details.

## Features

- ✅ Post to Nostr and Mastodon from command line
- ✅ Multi-platform posting with concurrent execution
- ✅ Multi-account support (test vs prod, personal vs work)
- ✅ Query posting history with `plur-history`
- ✅ Secure credential storage (OS keyring, encrypted files, or plain text)
- ✅ Interactive setup wizard (`plur-setup`)
- ✅ Credential management (`plur-creds`)
- ✅ Local SQLite database for post history
- ✅ TOML-based configuration with XDG Base Directory support
- ✅ Unix-friendly: reads from stdin, outputs to stdout, meaningful exit codes
- ✅ Agent-friendly: JSON output mode, comprehensive help text
- ✅ Shared test account easter egg (try `--account shared-test` on Nostr!)
- ⚗️ SSB support (experimental - local posting works, see docs for limitations)
- ⚗️ Post scheduling (experimental - `plur-queue` and `plur-send` implemented, needs real-world testing)

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

### 1. Run Interactive Setup (Recommended)

The easiest and most secure way to get started:

```bash
plur-setup
```

This interactive wizard will:
- Configure secure credential storage (OS keyring recommended)
- Guide you through platform setup (Nostr, Mastodon)
- Test authentication
- Create your configuration file

**Security**: The setup wizard uses your OS keyring by default (Windows Credential Manager, macOS Keychain, or Linux Secret Service), keeping your credentials secure.

### 2. Alternative: Manual Configuration

If you prefer manual setup, create `~/.config/plurcast/config.toml`:

```toml
[database]
path = "~/.local/share/plurcast/posts.db"

[credentials]
# Use OS keyring (recommended) or encrypted files
storage = "keyring"  # or "encrypted" with master password

[nostr]
enabled = true
relays = [
    "wss://relay.damus.io",
    "wss://nos.lol",
    "wss://relay.nostr.band"
]

[mastodon]
enabled = true
instance = "mastodon.social"

[defaults]
# Default platforms to post to (can override with --platform flag)
platforms = ["nostr", "mastodon"]
```

Then configure credentials securely:

```bash
# Store credentials in OS keyring
plur-creds set nostr
plur-creds set mastodon

# Test authentication
plur-creds test --all
```

**Note**: See the [Platform Setup Guides](#platform-setup-guides) section for instructions on obtaining credentials (Nostr keys, Mastodon tokens).

### 3. Post Your First Message

```bash
# Post to all enabled platforms
plur-post "Hello decentralized world!"
# Output:
# nostr:note1abc123...
# mastodon:12345

# Post from stdin
echo "Hello from stdin" | plur-post

# Post to specific platform only
echo "Nostr-only post" | plur-post --platform nostr

# Post to multiple specific platforms
echo "Nostr and Mastodon" | plur-post --platform nostr,mastodon

# Post to SSB (Phase 3 - when available)
echo "Hello SSB!" | plur-post --platform ssb
```

## Usage

### Try It Now! (No Setup Required) 🎉

Want to test Plurcast without setting up credentials? Use the shared test account:

```bash
# Post to the shared test account (publicly accessible)
plur-post "Testing Plurcast!" --platform nostr --account shared-test
```

This is a publicly known test account that anyone can post to. It's perfect for:
- Testing Plurcast without setup
- Demos and tutorials
- Community bulletin board for Plurcast users

**⚠️ Warning**: Anyone can post to this account! Don't use it for real posts.

**Public key**: `npub1qyv34w2prnz66zxrgqsmy2emrg0uqtrnvarhrrfaktxk9vp2dgllsajv05m`

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
- Well above any platform's actual limits (Nostr: ~32KB, Mastodon: 500 chars default)
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

# Post to specific platform only
plur-post "Nostr only" --platform nostr
# Output:
# nostr:note1abc123...

# Post to multiple specific platforms
plur-post "Multi-platform" --platform nostr,mastodon
# Output:
# nostr:note1abc123...
# mastodon:12345

# Post to SSB (Phase 3 - when available)
plur-post "Hello SSB!" --platform ssb
# Output:
# ssb:%abc123...

# Post to all three platforms
plur-post "Hello everyone!" --platform nostr,mastodon,ssb
# Output:
# nostr:note1abc123...
# mastodon:12345
# ssb:%def456...

# Handle partial failures gracefully
plur-post "Test post" --platform nostr,mastodon
# If Mastodon fails but Nostr succeeds:
# nostr:note1abc123...
# Error: mastodon: Authentication failed
# Exit code: 1 (partial failure)
```

### Nostr-Specific Features

#### Proof of Work (--nostr-pow)

Nostr supports Proof of Work (PoW) via NIP-13 to help combat spam. The `--nostr-pow` flag adds computational difficulty to your posts, making them more likely to be accepted by relays with spam filters.

```bash
# Add Proof of Work to Nostr posts
plur-post "Important message" --platform nostr --nostr-pow 20

# Higher difficulty for better spam protection (takes longer to compute)
plur-post "Critical announcement" --platform nostr --nostr-pow 25

# Only applies when posting to Nostr (ignored for other platforms)
plur-post "Cross-platform" --platform nostr,mastodon --nostr-pow 20
# Nostr post will have PoW, Mastodon post will not
```

**Difficulty Guidelines**:
- **Recommended**: 20-25 (takes 1-5 seconds on typical hardware)
- **Maximum**: 64 (extremely high difficulty, very slow)
- **Higher values**: More spam protection but longer computation time
- **Lower values**: Faster posting but less spam protection

**What is Proof of Work?**
Proof of Work requires your client to perform computational work before posting. The post's ID must have a certain number of leading zero bits. This makes it costly to spam the network while remaining cheap for legitimate users posting occasionally.

**When to use it**:
- Posting to relays with spam filters
- Important announcements that should be widely accepted
- When you want to ensure maximum deliverability

**When NOT to use it**:
- Testing or development (use `--draft` instead)
- High-frequency posting (PoW slows down each post)
- When posting to multiple platforms (PoW only applies to Nostr)

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

# Filter by platform
plur-history --platform nostr
plur-history --platform mastodon
plur-history --platform ssb  # Phase 3 - when available

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

**Recommended (Secure)**:

```toml
[database]
path = "~/.local/share/plurcast/posts.db"

[credentials]
# Secure credential storage (recommended)
storage = "keyring"  # OS keyring: Windows Credential Manager, macOS Keychain, Linux Secret Service
# Or: storage = "encrypted" with master password

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
instance = "mastodon.social"  # Your Mastodon instance

[defaults]
platforms = ["nostr", "mastodon"]
```

Then configure credentials using `plur-creds`:

```bash
plur-creds set nostr     # Stores in OS keyring
plur-creds set mastodon
```

**Legacy (Plain Text Files - Not Recommended)**:

For backward compatibility, you can still use plain text files, but this is **not secure**:

```toml
[nostr]
keys_file = "~/.config/plurcast/nostr.keys"  # Plain text (insecure)

[mastodon]
token_file = "~/.config/plurcast/mastodon.token"  # Plain text (insecure)
```

⚠️ **Security Warning**: Plain text credential files should have 600 permissions and be migrated to secure storage:

```bash
# Set permissions (if using plain text)
chmod 600 ~/.config/plurcast/*.keys ~/.config/plurcast/*.token ~/.config/plurcast/*.auth

# Migrate to secure storage (recommended)
plur-creds migrate
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

**Recommended**: Use `plur-setup` or `plur-creds` for secure, guided configuration.

The sections below explain how to obtain credentials from each platform. Once you have them, use `plur-creds set <platform>` to store them securely in your OS keyring.

### Nostr Setup

Nostr uses cryptographic key pairs for identity. You'll need a private key to post.

**Step 1: Generate or obtain a Nostr private key**

If you don't have a Nostr key yet, generate one using:

- **nak** (recommended): `nak key generate`
- **nostr-tools**: JavaScript library
- **Any Nostr client**: Damus (iOS), Amethyst (Android), Snort (web)

**Step 2: Store your key securely**

```bash
# Store in OS keyring (recommended)
plur-creds set nostr
# When prompted, enter your private key (hex or nsec format)
```

**Step 3: Configure relays**

Edit `~/.config/plurcast/config.toml`:

```toml
[nostr]
enabled = true
relays = [
    "wss://relay.damus.io",
    "wss://nos.lol",
    "wss://relay.nostr.band"
]
```

**Step 4: Test**

```bash
plur-creds test nostr
plur-post "Hello Nostr!" --platform nostr
```

### Mastodon Setup

Mastodon uses OAuth2 for authentication. You'll need to generate an access token.

**Step 1: Generate an access token**

1. Log in to your Mastodon instance (e.g., mastodon.social)
2. Go to **Settings** → **Development** → **New Application**
3. Fill in:
   - **Application name**: Plurcast
   - **Scopes**: `write:statuses` (minimum required)
   - **Redirect URI**: `urn:ietf:wg:oauth:2.0:oob`
4. Click **Submit**
5. Copy the **Access Token**

**Step 2: Store your token securely**

```bash
# Store in OS keyring (recommended)
plur-creds set mastodon
# When prompted, enter your access token
```

**Step 3: Configure instance**

Edit `~/.config/plurcast/config.toml`:

```toml
[mastodon]
enabled = true
instance = "mastodon.social"  # Change to your instance
```

**Step 4: Test**

```bash
plur-creds test mastodon
plur-post "Hello Mastodon!" --platform mastodon
```

**Supported Fediverse platforms**:
Mastodon, Pleroma, Friendica, Firefish, GoToSocial, Akkoma (just change the `instance` URL)

### SSB (Secure Scuttlebutt) Setup

**Status**: 🔮 Planned for Phase 3 - not yet implemented

SSB is a truly peer-to-peer, offline-first social protocol with no servers, no blockchain, and no corporate control.

**When implemented, setup will be:**

**Step 1: Generate or import SSB keypair**

```bash
# Generate new keypair
plur-creds set ssb --generate

# Or import from existing SSB installation
plur-creds set ssb --import ~/.ssb/secret
```

**Step 2: Configure SSB**

Edit `~/.config/plurcast/config.toml`:

```toml
[ssb]
enabled = true
feed_path = "~/.plurcast-ssb"  # Local feed database
pubs = [
    "net:hermies.club:8008~shs:base64-key-here",
    "net:pub.scuttlebutt.nz:8008~shs:base64-key-here"
]
```

**Step 3: Test**

```bash
plur-creds test ssb
plur-post "Hello SSB!" --platform ssb
```

**Why SSB instead of Bluesky?**
- Truly decentralized (not just one company)
- Offline-first (works without internet)
- No corporate control or banning
- Philosophically aligned with Plurcast values

**For detailed SSB documentation, see:**
- [SSB Setup Guide](docs/SSB_SETUP.md)
- [SSB Configuration Guide](docs/SSB_CONFIG.md)
- [SSB Troubleshooting Guide](docs/SSB_TROUBLESHOOTING.md)
- [SSB Comparison Guide](docs/SSB_COMPARISON.md)

See `.kiro/specs/ssb-integration/design.md` for the complete implementation plan.

## Security

Plurcast provides multiple options for storing your platform credentials securely.

### Credential Storage Backends

Plurcast supports three storage backends, with automatic fallback:

1. **Encrypted Files (Recommended)** - Password-protected files using age encryption
   - Reliable and cross-platform
   - Requires master password
   - Files stored in `~/.config/plurcast/credentials/`

2. **OS Keyring (Experimental)** - ⚠️ **Unstable - may lose credentials**
   - **Known Issue**: Credentials may not persist reliably across sessions
   - **macOS**: Keychain via Security framework
   - **Windows**: Credential Manager via Windows API
   - **Linux**: Secret Service (GNOME Keyring/KWallet) via D-Bus
   - **Recommendation**: Use encrypted files until keyring stability is verified

3. **Plain Text** - Legacy compatibility only (not recommended)
   - Credentials stored in plain text files
   - Only for testing or backward compatibility
   - **Security risk** - use only if other options unavailable

### Configuration

Add a `[credentials]` section to your `config.toml`:

```toml
[credentials]
# Storage backend: "encrypted" (recommended), "keyring" (experimental), "plain" (not recommended)
storage = "encrypted"  # Recommended: reliable and secure
# storage = "keyring"  # Experimental: may lose credentials
# Path for encrypted/plain file storage
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

1. **Use Encrypted Files** - Most reliable option (keyring support is experimental)
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
- **SSB** (Phase 3): Ed25519 keypairs

### What's Not Sensitive

These are stored in `config.toml` (not encrypted):
- Mastodon instance URLs
- Nostr relay URLs
- SSB server addresses (Phase 3)
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

## Multi-Account Management

Plurcast supports managing multiple accounts per platform, allowing you to maintain separate credentials for different purposes (test vs prod, personal vs work, etc.).

### Quick Start

```bash
# Store credentials for different accounts
plur-creds set nostr --account test
plur-creds set nostr --account prod

# List all accounts
plur-creds list --platform nostr
# Output:
#   ✓ nostr (default): Private Key (stored in keyring) [active]
#   ✓ nostr (test): Private Key (stored in keyring)
#   ✓ nostr (prod): Private Key (stored in keyring)

# Switch active account
plur-creds use nostr --account prod

# Post using active account
plur-post "Hello from prod account"

# Or specify account explicitly
plur-post "Test message" --account test
```

### The "default" Account

The **"default" account** is used for backward compatibility:
- When you omit `--account`, Plurcast uses the "default" account
- Existing credentials are automatically migrated to "default"
- Your existing workflows continue to work without changes

```bash
# These are equivalent:
plur-creds set nostr
plur-creds set nostr --account default

# These are equivalent:
plur-post "Hello"
plur-post "Hello" --account default
```

### Account Naming Rules

Account names must follow these rules:
- **Alphanumeric characters**: a-z, A-Z, 0-9
- **Hyphens and underscores**: `-` and `_`
- **Maximum length**: 64 characters
- **Case-sensitive**: `Test` and `test` are different

**Valid examples**: `default`, `test`, `prod`, `test-account`, `work`, `personal`

**Invalid examples**: `test account` (space), `test@account` (special char), `test.account` (period)

### Common Workflows

#### Developer: Test and Production

```bash
# Store test credentials
plur-creds set nostr --account test
# Enter test private key...

# Store prod credentials
plur-creds set nostr --account prod
# Enter prod private key...

# Set test as active for development
plur-creds use nostr --account test

# Post to test (uses active account)
plur-post "Testing new feature"

# Post to prod explicitly
plur-post "Production announcement" --account prod
```

#### Personal and Work Accounts

```bash
# Store personal Mastodon account
plur-creds set mastodon --account personal
# Enter personal access token...

# Store work Mastodon account
plur-creds set mastodon --account work
# Enter work access token...

# Switch to work account
plur-creds use mastodon --account work
plur-post "Team update"

# Switch to personal account
plur-creds use mastodon --account personal
plur-post "Weekend plans"
```

#### Multi-Platform with Different Accounts

```bash
# Configure different accounts for different platforms
plur-creds set nostr --account personal
plur-creds set mastodon --account work
plur-creds set nostr --account test

# Set active accounts
plur-creds use nostr --account personal
plur-creds use mastodon --account work

# Post to all platforms using their active accounts
plur-post "Cross-platform message"
# Uses: nostr (personal), mastodon (work)
```

### Account Management Commands

```bash
# List all accounts for a platform
plur-creds list --platform nostr

# List all accounts across all platforms
plur-creds list

# Set active account
plur-creds use <platform> --account <name>

# Delete an account
plur-creds delete nostr --account test

# Test account credentials
plur-creds test nostr --account test
```

### Account State

Active accounts are tracked in `~/.config/plurcast/accounts.toml`:

```toml
# Active account per platform
[active]
nostr = "test"
mastodon = "work"

# Registered accounts per platform
[accounts.nostr]
names = ["default", "test", "prod"]

[accounts.mastodon]
names = ["default", "work"]
```

### Best Practices

**Account Naming**:
- Use descriptive names: `test`, `prod`, `staging` instead of `a`, `b`, `c`
- Be consistent across platforms: use same naming scheme
- Keep names short and memorable
- Use hyphens or underscores for multi-word names: `test-account`, `prod_2024`

**Account Organization**:
- Use `default` for your primary/personal account
- Use `test` for development and testing
- Use `prod` for production deployments
- Use `work` and `personal` for separating contexts

**Safety**:
- Always verify active account before posting: `plur-creds list`
- Use explicit `--account` flag for critical posts to production
- Test new accounts before switching: `plur-creds test <platform> --account <name>`
- Keep test and prod credentials separate

**Workflow**:
- Set active accounts at the start of your work session
- Use `plur-creds use` to switch contexts (work/personal, test/prod)
- Use explicit `--account` flag for one-off posts to different accounts
- Review account list regularly: `plur-creds list`

### Migration from Single Account

If you're upgrading from an earlier version:

1. **Automatic migration**: Your existing credentials become the "default" account
2. **No action required**: Existing workflows continue to work
3. **Add accounts**: Use `--account` flag to add additional accounts

For detailed migration information, see [Multi-Account Migration Guide](docs/MULTI_ACCOUNT_MIGRATION.md).

### Troubleshooting Multi-Account

#### "Account not found"

**Error**: `Account 'test' not found for platform 'nostr'`

**Cause**: Account doesn't exist or hasn't been configured

**Solution**:
```bash
# List existing accounts
plur-creds list --platform nostr

# Create the account
plur-creds set nostr --account test
```

#### "Cannot delete active account"

**Error**: `Cannot delete active account 'test' for platform 'nostr'`

**Cause**: Trying to delete the currently active account

**Solution**:
```bash
# Switch to different account first
plur-creds use nostr --account default

# Now delete
plur-creds delete nostr --account test
```

#### "Invalid account name"

**Error**: `Invalid account name: 'test account'. Must be alphanumeric with hyphens/underscores, max 64 chars`

**Cause**: Account name contains invalid characters or is too long

**Solution**: Use only alphanumeric characters, hyphens, and underscores:
```bash
# Invalid
plur-creds set nostr --account "test account"  # Space not allowed
plur-creds set nostr --account "test@account"  # @ not allowed

# Valid
plur-creds set nostr --account "test-account"
plur-creds set nostr --account "test_account"
plur-creds set nostr --account "test2"
```

#### Switching between storage backends

If you want to switch from one storage backend to another (e.g., from plain files to keyring):

```bash
# Migrate existing credentials to keyring
plur-creds migrate --to keyring

# Or switch to encrypted file storage
plur-creds migrate --to encrypted
```

#### Migration failed

**Error**: `Migration failed for nostr.private_key`

**Cause**: Old credential file is corrupted or inaccessible

**Solution**:
```bash
# Check credential file exists and has correct permissions
ls -la ~/.config/plurcast/nostr.keys
chmod 600 ~/.config/plurcast/nostr.keys

# Try manual migration
plur-creds set nostr --account default
# Enter credentials manually

# Verify it works
plur-creds test nostr
```

#### Posting to wrong account

**Error**: Posted to wrong account unintentionally

**Prevention**: Always verify active account before posting:
```bash
# Check active accounts
plur-creds list

# Set correct active account
plur-creds use nostr --account prod

# Or specify account explicitly
plur-post "Important message" --account prod
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

### "Content validation failed: Post exceeds character limit"

**Cause**: Content is too long for the target platform

**Platform limits**:
- Nostr: ~32KB (practical limit)
- Mastodon: 500 characters (default, varies by instance)
- SSB: ~8KB (practical limit, Phase 3)

**Solution**:
- Shorten your content
- Use `--platform` to exclude platforms with stricter limits
- Split into multiple posts or threads (future feature)

Example:
```bash
# Long post works on Nostr but may fail on Mastodon
plur-post "Very long content..." --platform nostr,mastodon
# Error: mastodon: Content validation failed: Post exceeds 500 character limit

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
    cat message.txt | plur-post --platform nostr,mastodon
else
    cat message.txt | plur-post --platform nostr
fi

# Post only if content is short enough for Mastodon
if [ $(wc -c < message.txt) -le 500 ]; then
    cat message.txt | plur-post --platform nostr,mastodon
else
    cat message.txt | plur-post --platform nostr
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
- [x] Mastodon integration (supports all ActivityPub platforms)
- [x] Multi-account support with OS keyring
- [x] Multi-platform posting with concurrent execution
- [x] plur-history query tool
- [x] Shared test account easter egg (Nostr)
- [ ] SSB integration (Phase 3 - planned)
- [x] Comprehensive documentation
- [x] Platform setup guides

### Phase 3: CLI Polish & Library Stabilization (Next)
- [x] Multi-account support (completed in 0.3.0-alpha2)
- [x] Fix OS keyring credential persistence issue (resolved in 0.3.0-alpha1)
- [x] Add integration tests for credential storage backends (completed in 0.3.0-alpha2)
- [ ] Verify keyring stability on macOS and Linux
- [ ] Publish `libplurcast` to crates.io
- [ ] Stabilize public API for external consumers
- [ ] CLI improvements (better error messages, progress indicators)
- [ ] Configuration validation and migration tools
- [ ] Comprehensive API documentation

### Phase 4: Desktop GUI (Planned - Separate Project)
**Repository**: `plurcast-gui` (separate repo, depends on `libplurcast` from crates.io)
- [ ] Tauri-based desktop application
- [ ] Visual post composer with platform preview
- [ ] History browser with search and filtering
- [ ] Draft management
- [ ] Account management UI
- [ ] Cross-platform installers (Windows, macOS, Linux)

### Phase 5: Scheduling (Experimental)
- ⚗️ plur-queue (scheduling) - implemented, needs testing
- ⚗️ plur-send (daemon) - implemented, needs testing
- ⚗️ Rate limiting per platform - implemented, needs testing
- [ ] Human verification of real-world daemon behavior
- [ ] Long-running stability testing
- [ ] Network resilience testing

### Phase 6: Data Portability (Planned)
- [ ] plur-import (data import)
- [ ] plur-export (data export)
- [ ] Migration utilities

### Future Enhancements
- [ ] Media attachments
- [ ] Thread support
- [ ] Semantic search with embeddings
- [ ] Reply handling

## Known Issues

### OS Keyring Credential Storage

**Status**: ✅ Stable on Windows (as of 0.3.0-alpha1)  
**Testing Status**: macOS and Linux verification pending

**Windows**: Keyring persistence has been verified and is working reliably:
- Credentials persist across process restarts ✓
- Credentials persist across terminal sessions ✓
- Credentials persist across system reboots ✓

**macOS/Linux**: Testing in progress. If you experience issues, use encrypted file storage:

```toml
# In config.toml
[credentials]
storage = "encrypted"
path = "~/.config/plurcast/credentials"
```

**Note**: All three storage backends (keyring, encrypted, plain) are fully supported. Choose based on your security requirements and platform compatibility.

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
- **Documentation**:
  - [Multi-Account Migration Guide](docs/MULTI_ACCOUNT_MIGRATION.md)
  - [ADR 001: Multi-Account Management](docs/adr/001-multi-account-management.md)
- **Nostr**: [NIP-01](https://github.com/nostr-protocol/nips/blob/master/01.md)

## Acknowledgments

Built with mature, open-source Rust libraries:
- [nostr-sdk](https://github.com/rust-nostr/nostr) - Nostr protocol implementation
- [sqlx](https://github.com/launchbadge/sqlx) - Async SQL toolkit
- [tokio](https://github.com/tokio-rs/tokio) - Async runtime
- [clap](https://github.com/clap-rs/clap) - Command-line parser

---

**Plurcast** - Cast to many, own your data, follow Unix principles.
