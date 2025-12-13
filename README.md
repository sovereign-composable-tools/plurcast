# Plurcast

**Unix tools for the decentralized social web**

Post to Nostr, Mastodon, and SSB from the command line. Each tool does one thing well and composes naturally with pipes and scripts.

## Features

- Post to multiple decentralized platforms simultaneously
- Multi-account support (test vs prod, personal vs work)
- Secure credential storage (OS keyring, encrypted files)
- Post scheduling with queue management
- JSON output for scripting and automation
- Nostr Proof of Work (NIP-13) support
- Unix-friendly: stdin/stdout, meaningful exit codes

## Quick Start

```bash
# Build from source
git clone https://github.com/sovereign-composable-tools/plurcast.git
cd plurcast && cargo build --release

# Run setup wizard
./target/release/plur-setup

# Post your first message
./target/release/plur-post "Hello decentralized world!"
```

### Try Without Setup

Test immediately using the shared Nostr account:

```bash
plur-post "Testing Plurcast!" --platform nostr --account shared-test
```

## Platforms

| Platform | Status | Notes |
|----------|--------|-------|
| Nostr | Stable | 7 relays, PoW support |
| Mastodon | Stable | All ActivityPub platforms |
| SSB | Experimental | Local posting works |

## Tools

| Tool | Purpose |
|------|---------|
| `plur-post` | Create and publish posts |
| `plur-history` | Query posting history |
| `plur-creds` | Manage credentials |
| `plur-queue` | Manage scheduled posts |
| `plur-send` | Daemon for scheduled posting |
| `plur-setup` | Interactive setup wizard |
| `plur-import` | Import from platforms |
| `plur-export` | Export post history |

## Usage Examples

```bash
# Post from stdin
echo "Hello world" | plur-post

# Post to specific platforms
plur-post "Nostr only" --platform nostr

# Schedule a post
plur-post "Post later" --schedule "30m"

# JSON output for scripting
plur-post "Hello" --format json

# Query history
plur-history --platform nostr --format json
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Platform error |
| 2 | Auth error |
| 3 | Invalid input |

## Documentation

- [Setup Guide](docs/SETUP.md) - Platform configuration
- [Usage Guide](docs/USAGE.md) - Detailed usage examples
- [Security](docs/SECURITY.md) - Credential storage
- [Troubleshooting](docs/TROUBLESHOOTING.md) - Common issues
- [Contributing](CONTRIBUTING.md) - Development guide

## Requirements

- Rust 1.70+
- SQLite 3.x (bundled)

## License

MIT

## Links

- [GitHub](https://github.com/sovereign-composable-tools/plurcast)
- [Issues](https://github.com/sovereign-composable-tools/plurcast/issues)
