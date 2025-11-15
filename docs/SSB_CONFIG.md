# SSB Configuration Guide

This guide provides detailed information about configuring SSB (Secure Scuttlebutt) in Plurcast.

## Table of Contents

- [Configuration File Location](#configuration-file-location)
- [SSB Configuration Section](#ssb-configuration-section)
- [Configuration Parameters](#configuration-parameters)
- [Example Configurations](#example-configurations)
- [Feed Database Management](#feed-database-management)
- [Pub Server Configuration](#pub-server-configuration)
- [Advanced Settings](#advanced-settings)
- [Environment Variables](#environment-variables)

---

## Configuration File Location

Plurcast uses TOML format for configuration, following XDG Base Directory specification:

**Default Location**: `~/.config/plurcast/config.toml`

**Custom Location** (via environment variable):
```bash
export PLURCAST_CONFIG=~/my-config.toml
plur-post "Using custom config"
```

---

## SSB Configuration Section

The `[ssb]` section in `config.toml` controls all SSB-related settings:

```toml
[ssb]
enabled = true
feed_path = "~/.plurcast-ssb"
pubs = [
    "net:hermies.club:8008~shs:gfW/+1nLKT+/+LNbGmHJQ8Pu7TMwCvXLPvqXbEN6kZk="
]
```

---

## Configuration Parameters

### `enabled`

**Type**: Boolean  
**Default**: `false`  
**Required**: No

Controls whether SSB is enabled for posting.

```toml
[ssb]
enabled = true  # Enable SSB posting
```

**Behavior**:
- `true`: SSB will be included in multi-platform posts
- `false`: SSB will be skipped even if credentials exist

**Use Cases**:
- Temporarily disable SSB without removing configuration
- Test other platforms without SSB
- Disable SSB while troubleshooting

**Example**:
```bash
# With enabled = true
plur-post "Hello"  # Posts to all enabled platforms including SSB

# With enabled = false
plur-post "Hello"  # Posts to all platforms except SSB
```

---

### `feed_path`

**Type**: String (path)  
**Default**: `~/.plurcast-ssb`  
**Required**: No

Specifies the directory for the local SSB feed database.

```toml
[ssb]
feed_path = "~/.plurcast-ssb"
```

**Path Expansion**:
- `~` expands to user's home directory
- Relative paths are relative to current working directory
- Absolute paths are used as-is

**Directory Structure**:
```
~/.plurcast-ssb/
├── log.offset          # Append-only message log
├── flume/              # Flume database indexes
│   ├── log.offset
│   └── indexes/
└── .ssb-config         # Internal configuration
```

**Permissions**:
- Directory: `700` (owner read/write/execute only)
- Files: `600` (owner read/write only)

**Disk Usage**:
- Initial: ~1MB
- Growth: ~1KB per message
- Typical: 10-100MB after months of use

**Custom Locations**:
```toml
# Store on external drive
[ssb]
feed_path = "/mnt/external/ssb-feed"

# Store in project directory
[ssb]
feed_path = "./ssb-data"

# Store in XDG data directory
[ssb]
feed_path = "~/.local/share/plurcast/ssb"
```

**Backup Considerations**:
- Feed database is portable (copy entire directory)
- Backup regularly to prevent data loss
- Can be synced across machines (but use same keypair)

---

### `pubs`

**Type**: Array of strings  
**Default**: `[]` (empty)  
**Required**: No

List of pub servers in multiserver address format.

```toml
[ssb]
pubs = [
    "net:hermies.club:8008~shs:gfW/+1nLKT+/+LNbGmHJQ8Pu7TMwCvXLPvqXbEN6kZk=",
    "net:pub.scuttlebutt.nz:8008~shs:base64-key-here"
]
```

**Multiserver Address Format**:
```
net:hostname:port~shs:public-key
```

**Components**:
- `net`: Protocol (TCP network)
- `hostname`: Domain name or IP address
- `port`: Port number (typically 8008)
- `shs`: Secret handshake protocol
- `public-key`: Pub's Ed25519 public key (base64)

**Behavior**:
- Empty array: Local-only mode (no replication)
- One or more pubs: Replication enabled
- Unreachable pubs: Logged as warnings, don't block posting

**Replication Strategy**:
- Plurcast connects to all configured pubs
- Messages are pushed to each pub concurrently
- Failures on one pub don't affect others
- Background sync continues after posting

**Finding Pub Addresses**:
- [SSB Pub List](https://github.com/ssbc/ssb-server/wiki/Pub-Servers)
- Ask in SSB community channels
- Use `plur-setup` to add default pubs

---

## Example Configurations

### Minimal Configuration

Bare minimum for SSB support:

```toml
[ssb]
enabled = true
```

**Behavior**:
- Uses default feed path: `~/.plurcast-ssb`
- No pub servers (local-only mode)
- Posts stored locally, no network replication

**Use Cases**:
- Testing SSB integration
- Offline-only usage
- Local network only (future LAN discovery)

---

### Standard Configuration

Recommended for most users:

```toml
[ssb]
enabled = true
feed_path = "~/.plurcast-ssb"
pubs = [
    "net:hermies.club:8008~shs:gfW/+1nLKT+/+LNbGmHJQ8Pu7TMwCvXLPvqXbEN6kZk=",
    "net:pub.scuttlebutt.nz:8008~shs:base64-key-here"
]
```

**Behavior**:
- SSB enabled for multi-platform posting
- Feed stored in default location
- Replication to two reliable pubs
- Good balance of reliability and performance

---

### High-Availability Configuration

Maximum replication with multiple pubs:

```toml
[ssb]
enabled = true
feed_path = "~/.plurcast-ssb"
pubs = [
    # Primary pubs (high reliability)
    "net:hermies.club:8008~shs:gfW/+1nLKT+/+LNbGmHJQ8Pu7TMwCvXLPvqXbEN6kZk=",
    "net:pub.scuttlebutt.nz:8008~shs:base64-key-here",
    
    # Secondary pubs (geographic diversity)
    "net:ssb.celehner.com:8008~shs:base64-key-here",
    "net:pub.staltz.com:8008~shs:base64-key-here",
    
    # Backup pubs
    "net:pub.decent.land:8008~shs:base64-key-here"
]
```

**Advantages**:
- Higher replication redundancy
- Geographic diversity
- Faster propagation to network
- Resilient to pub outages

**Trade-offs**:
- Slower posting (more connections)
- Higher bandwidth usage
- More potential failure points

---

### Custom Feed Path Configuration

Store feed database in custom location:

```toml
[ssb]
enabled = true
feed_path = "/mnt/external/ssb-feed"
pubs = [
    "net:hermies.club:8008~shs:gfW/+1nLKT+/+LNbGmHJQ8Pu7TMwCvXLPvqXbEN6kZk="
]
```

**Use Cases**:
- External storage (USB drive, NAS)
- Separate partition for data
- Shared storage across machines
- Backup/archival purposes

**Important**:
- Ensure directory is writable
- Check filesystem supports required features
- Consider performance implications

---

### Offline-Only Configuration

No pub servers, local storage only:

```toml
[ssb]
enabled = true
feed_path = "~/.plurcast-ssb"
pubs = []  # Empty array = no replication
```

**Behavior**:
- Posts stored locally only
- No network replication
- Works completely offline
- Can add pubs later for replication

**Use Cases**:
- Offline environments
- Privacy-focused usage
- Testing and development
- Local network only (future)

---

### Multi-Account Configuration

Different accounts can have different settings:

```toml
# Default account
[ssb]
enabled = true
feed_path = "~/.plurcast-ssb/default"
pubs = [
    "net:hermies.club:8008~shs:gfW/+1nLKT+/+LNbGmHJQ8Pu7TMwCvXLPvqXbEN6kZk="
]

# Test account (separate feed)
[ssb.accounts.test]
feed_path = "~/.plurcast-ssb/test"
pubs = []  # Local-only for testing
```

**Note**: Multi-account SSB configuration is a future feature. Currently, use separate config files:

```bash
# Default account
plur-post "Production post" --platform ssb

# Test account (with custom config)
PLURCAST_CONFIG=~/.config/plurcast/test-config.toml \
  plur-post "Test post" --platform ssb
```

---

## Feed Database Management

### Database Location

The feed database is stored at the path specified by `feed_path`:

```bash
# Default location
~/.plurcast-ssb/

# Check current location
grep feed_path ~/.config/plurcast/config.toml
```

### Database Structure

```
~/.plurcast-ssb/
├── log.offset          # Main message log (append-only)
├── flume/              # Flume database (indexes)
│   ├── log.offset      # Index log
│   └── indexes/        # Various indexes
│       ├── sequence    # Message sequence index
│       ├── timestamp   # Timestamp index
│       └── type        # Message type index
└── .ssb-config         # Internal configuration
```

### Database Operations

**Initialize Database**:
```bash
# Database is created automatically on first post
echo "First post" | plur-post --platform ssb
```

**Check Database Status**:
```bash
# View database info
plur-creds test ssb

# Output includes:
# Feed database: ~/.plurcast-ssb
# Messages in feed: 42
```

**Backup Database**:
```bash
# Create backup
tar czf ssb-feed-backup.tar.gz ~/.plurcast-ssb/

# Restore from backup
tar xzf ssb-feed-backup.tar.gz -C ~/
```

**Move Database**:
```bash
# 1. Stop any SSB operations
# 2. Move directory
mv ~/.plurcast-ssb /new/location/

# 3. Update config
# Edit ~/.config/plurcast/config.toml:
# feed_path = "/new/location/.plurcast-ssb"

# 4. Verify
plur-creds test ssb
```

**Clean Database** (future feature):
```bash
# Compact database (remove unused space)
plur-ssb compact

# Rebuild indexes
plur-ssb reindex
```

### Database Maintenance

**Disk Space**:
- Monitor disk usage: `du -sh ~/.plurcast-ssb`
- Typical growth: ~1KB per message
- Plan for long-term storage

**Permissions**:
```bash
# Ensure correct permissions
chmod 700 ~/.plurcast-ssb
chmod 600 ~/.plurcast-ssb/log.offset
```

**Corruption Recovery**:
- See [SSB_TROUBLESHOOTING.md](SSB_TROUBLESHOOTING.md)
- Restore from backup if available
- Rebuild from network (future feature)

---

## Pub Server Configuration

### Adding Pub Servers

**Method 1: Configuration File**

Edit `~/.config/plurcast/config.toml`:

```toml
[ssb]
pubs = [
    "net:hermies.club:8008~shs:gfW/+1nLKT+/+LNbGmHJQ8Pu7TMwCvXLPvqXbEN6kZk=",
    "net:pub.scuttlebutt.nz:8008~shs:base64-key-here"
]
```

**Method 2: Setup Wizard**

```bash
plur-setup
# Select SSB configuration
# Choose "Add pub servers"
```

**Method 3: Accept Invite** (future feature)

```bash
plur-creds accept-invite "invite-code-here"
```

### Pub Server Format

**Multiserver Address**:
```
net:hostname:port~shs:public-key
```

**Example**:
```
net:hermies.club:8008~shs:gfW/+1nLKT+/+LNbGmHJQ8Pu7TMwCvXLPvqXbEN6kZk=
```

**Components**:
- `net`: TCP network protocol
- `hermies.club`: Hostname
- `8008`: Port (standard SSB port)
- `shs`: Secret handshake protocol
- `gfW/+1nL...`: Pub's Ed25519 public key (base64)

### Testing Pub Connectivity

```bash
# Test all configured pubs
plur-creds test ssb --check-pubs

# Output:
# Testing pub connectivity...
#   ✓ hermies.club - reachable (45ms)
#   ✓ pub.scuttlebutt.nz - reachable (120ms)
#   ✗ ssb.celehner.com - unreachable (timeout)
```

### Removing Pub Servers

Edit `~/.config/plurcast/config.toml` and remove the pub from the array:

```toml
[ssb]
pubs = [
    # Removed: "net:old-pub.example.com:8008~shs:..."
    "net:hermies.club:8008~shs:gfW/+1nLKT+/+LNbGmHJQ8Pu7TMwCvXLPvqXbEN6kZk="
]
```

### Pub Server Best Practices

**Number of Pubs**:
- Minimum: 0 (local-only)
- Recommended: 2-3 (redundancy)
- Maximum: 5-10 (diminishing returns)

**Selection Criteria**:
- Reliability (uptime)
- Geographic diversity
- Community reputation
- Latency to your location

**Monitoring**:
- Periodically test connectivity
- Remove consistently unreachable pubs
- Add new pubs as needed

---

## Advanced Settings

### Replication Settings (Future)

```toml
[ssb.replication]
enabled = true
interval = 300  # Sync every 5 minutes
max_concurrent = 3  # Max concurrent pub connections
timeout = 30  # Connection timeout in seconds
```

### Performance Tuning (Future)

```toml
[ssb.performance]
cache_size = 100  # Message cache size (MB)
index_cache = 50  # Index cache size (MB)
write_buffer = 10  # Write buffer size (MB)
```

### Network Settings (Future)

```toml
[ssb.network]
lan_discovery = true  # Enable LAN peer discovery
port = 8008  # Local SSB port
max_peers = 10  # Maximum peer connections
```

---

## Environment Variables

Override configuration with environment variables:

### `PLURCAST_CONFIG`

Override config file location:

```bash
export PLURCAST_CONFIG=~/my-config.toml
plur-post "Using custom config"
```

### `PLURCAST_SSB_FEED_PATH`

Override feed database path:

```bash
export PLURCAST_SSB_FEED_PATH=/tmp/test-feed
plur-post "Test post" --platform ssb
```

### `PLURCAST_SSB_ENABLED`

Override enabled setting:

```bash
# Disable SSB temporarily
export PLURCAST_SSB_ENABLED=false
plur-post "No SSB"

# Enable SSB
export PLURCAST_SSB_ENABLED=true
plur-post "With SSB"
```

### `PLURCAST_VERBOSE`

Enable verbose logging:

```bash
export PLURCAST_VERBOSE=1
plur-post "Verbose output" --platform ssb

# Shows detailed SSB operations:
# - Feed initialization
# - Message creation
# - Pub connections
# - Replication status
```

---

## Configuration Validation

### Validate Configuration

```bash
# Test SSB configuration
plur-creds test ssb

# Output shows:
# ✓ SSB credentials valid
# ✓ Feed database accessible
# ✓ Pub connectivity (if configured)
```

### Common Configuration Errors

**Invalid feed_path**:
```toml
[ssb]
feed_path = "/nonexistent/path"  # Error: directory doesn't exist
```

**Solution**: Create directory or use valid path

**Invalid pub address**:
```toml
[ssb]
pubs = [
    "invalid-format"  # Error: not a multiserver address
]
```

**Solution**: Use correct multiserver format

**Permission errors**:
```bash
# Error: Permission denied
```

**Solution**: Check directory permissions:
```bash
chmod 700 ~/.plurcast-ssb
```

---

## Configuration Examples by Use Case

### Personal Blog

```toml
[ssb]
enabled = true
feed_path = "~/.plurcast-ssb"
pubs = [
    "net:hermies.club:8008~shs:gfW/+1nLKT+/+LNbGmHJQ8Pu7TMwCvXLPvqXbEN6kZk="
]
```

### Development/Testing

```toml
[ssb]
enabled = true
feed_path = "/tmp/ssb-test"
pubs = []  # No replication during testing
```

### High-Availability

```toml
[ssb]
enabled = true
feed_path = "~/.plurcast-ssb"
pubs = [
    "net:hermies.club:8008~shs:gfW/+1nLKT+/+LNbGmHJQ8Pu7TMwCvXLPvqXbEN6kZk=",
    "net:pub.scuttlebutt.nz:8008~shs:base64-key-here",
    "net:ssb.celehner.com:8008~shs:base64-key-here"
]
```

### Offline/Private

```toml
[ssb]
enabled = true
feed_path = "~/.plurcast-ssb"
pubs = []  # No public replication
```

---

## Next Steps

- **Setup Guide**: See [SSB_SETUP.md](SSB_SETUP.md) for initial setup
- **Troubleshooting**: See [SSB_TROUBLESHOOTING.md](SSB_TROUBLESHOOTING.md) for common issues
- **Comparison**: See [SSB_COMPARISON.md](SSB_COMPARISON.md) for platform differences

---

**Configuration Reference Version**: 0.3.0-alpha2  
**Last Updated**: 2025-01-15
