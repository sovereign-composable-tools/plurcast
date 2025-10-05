# Plurcast

**Cast to many** - Unix tools for the decentralized social web

Plurcast is a collection of Unix command-line tools for posting to decentralized social media platforms like Nostr, Mastodon, and Bluesky. Following Unix philosophy, each tool does one thing well and composes naturally with other command-line utilities.

## Status

**Alpha MVP (v0.1.0)** - Foundation phase with basic Nostr support

## Features

- ✅ Post to Nostr from command line
- ✅ Local SQLite database for post history
- ✅ TOML-based configuration with XDG Base Directory support
- ✅ Unix-friendly: reads from stdin, outputs to stdout, meaningful exit codes
- ✅ Agent-friendly: JSON output mode, comprehensive help text
- 🚧 Mastodon support (coming soon)
- 🚧 Bluesky support (coming soon)
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

### 2. Configure Nostr

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

[defaults]
platforms = ["nostr"]
```

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
# Post from command line argument
plur-post "Hello decentralized world!"

# Post from stdin
echo "Hello from stdin" | plur-post

# Post to specific platform
echo "Nostr-only post" | plur-post --platform nostr
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

### Platform Selection

```bash
# Use default platforms from config
plur-post "Multi-platform post"

# Target specific platform
plur-post "Nostr only" --platform nostr

# Target multiple platforms (comma-separated)
plur-post "Multiple platforms" --platform nostr,mastodon
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
enabled = false
instance = "mastodon.social"
token_file = "~/.config/plurcast/mastodon.token"

[bluesky]
enabled = false
handle = "user.bsky.social"
auth_file = "~/.config/plurcast/bluesky.auth"

[defaults]
# Default platforms to post to (can override with --platform flag)
platforms = ["nostr"]
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

### "Failed to connect to relay"

**Cause**: Network issues or relay is down

**Solution**:
- Check your internet connection
- Try different relays in config.toml
- Use `--verbose` flag to see detailed connection logs
- Plurcast succeeds if ANY relay accepts the post

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

Example compositions:
```bash
# Post with preprocessing
cat draft.txt | sed 's/foo/bar/g' | plur-post

# Conditional posting
if grep -q "urgent" message.txt; then
    cat message.txt | plur-post --platform nostr,mastodon
else
    cat message.txt | plur-post --platform nostr
fi

# JSON processing
plur-post "Test" --format json | jq '.[] | select(.success == true)'
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

### Alpha (Current)
- [x] Core database schema
- [x] Configuration system
- [x] Nostr platform support
- [x] Basic plur-post tool
- [ ] Documentation and help text

### Beta
- [ ] Mastodon integration
- [ ] Bluesky integration
- [ ] plur-queue (scheduling)
- [ ] plur-send (daemon)
- [ ] plur-history (query tool)

### Stable (1.0)
- [ ] plur-import (data import)
- [ ] plur-export (data export)
- [ ] Comprehensive test coverage
- [ ] Man pages

### Future
- [ ] Media attachments
- [ ] Thread support
- [ ] Semantic search
- [ ] TUI interface

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
