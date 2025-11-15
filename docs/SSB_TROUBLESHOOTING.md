# SSB Troubleshooting Guide

This guide helps you diagnose and resolve common issues with SSB (Secure Scuttlebutt) in Plurcast.

## Table of Contents

- [Quick Diagnostics](#quick-diagnostics)
- [Common Issues](#common-issues)
- [Pub Connectivity Problems](#pub-connectivity-problems)
- [Replication Failures](#replication-failures)
- [Feed Database Issues](#feed-database-issues)
- [Authentication Errors](#authentication-errors)
- [Performance Issues](#performance-issues)
- [Getting Help](#getting-help)

---

## Quick Diagnostics

### Run SSB Test

First step for any SSB issue:

```bash
plur-creds test ssb
```

**Expected Output** (healthy system):
```
✓ SSB credentials valid
✓ Feed database accessible: ~/.plurcast-ssb
✓ Messages in feed: 42
✓ Pub connectivity:
  ✓ hermies.club - reachable (45ms)
  ✓ pub.scuttlebutt.nz - reachable (120ms)
```

**Problem Indicators**:
- ✗ SSB credentials not found
- ✗ Feed database inaccessible
- ✗ All pubs unreachable
- ✗ Permission denied errors

### Enable Verbose Logging

Get detailed information about what's happening:

```bash
plur-post "Test" --platform ssb --verbose
```

**What to Look For**:
- Initialization messages
- Connection attempts
- Error messages with context
- Replication status

### Check Configuration

Verify your configuration is correct:

```bash
cat ~/.config/plurcast/config.toml
```

**Verify**:
- `[ssb]` section exists
- `enabled = true`
- `feed_path` is valid
- `pubs` array is properly formatted

---

## Common Issues

### Issue: "SSB credentials not configured"

**Error Message**:
```
Error: SSB credentials not configured - run plur-setup or plur-creds set ssb
```

**Cause**: No SSB keypair stored in credential manager

**Solution**:

```bash
# Option 1: Use setup wizard
plur-setup

# Option 2: Generate new keypair
plur-creds set ssb --generate

# Option 3: Import existing keypair
plur-creds set ssb --import ~/.ssb/secret
```

**Verification**:
```bash
plur-creds test ssb
# Should show: ✓ SSB credentials valid
```

---

### Issue: "Failed to open SSB feed database"

**Error Message**:
```
Error: Failed to open SSB feed database at ~/.plurcast-ssb
```

**Possible Causes**:
1. Directory doesn't exist
2. Permission denied
3. Disk full
4. Corrupted database

**Solutions**:

**1. Create Directory**:
```bash
mkdir -p ~/.plurcast-ssb
chmod 700 ~/.plurcast-ssb
```

**2. Check Permissions**:
```bash
ls -la ~/.plurcast-ssb
# Should show: drwx------ (700)

# Fix permissions
chmod 700 ~/.plurcast-ssb
chmod 600 ~/.plurcast-ssb/log.offset
```

**3. Check Disk Space**:
```bash
df -h ~/.plurcast-ssb
# Ensure sufficient free space (at least 100MB)
```

**4. Database Corruption** (see [Feed Database Issues](#feed-database-issues))

---

### Issue: "Invalid SSB keypair"

**Error Message**:
```
Error: Invalid SSB keypair - check credential format
```

**Cause**: Keypair format is incorrect or corrupted

**Solution**:

```bash
# Delete invalid credentials
plur-creds delete ssb

# Generate new keypair
plur-creds set ssb --generate

# Or import valid keypair
plur-creds set ssb --import ~/.ssb/secret
```

**Verification**:
```bash
plur-creds test ssb
# Should show your SSB ID: @abc123...=.ed25519
```

---

### Issue: "Post content exceeds SSB message size limit"

**Error Message**:
```
Error: Post content exceeds SSB message size limit (8KB)
```

**Cause**: SSB messages have a practical limit of ~8KB

**Solutions**:

**Option 1: Shorten Content**
```bash
# Edit your content to be shorter
echo "Shorter message" | plur-post --platform ssb
```

**Option 2: Exclude SSB**
```bash
# Post to other platforms only
plur-post "Long content..." --platform nostr,mastodon
```

**Option 3: Split into Thread** (future feature)
```bash
# Split long content into multiple messages
plur-thread "Long content..." --platform ssb
```

**Check Message Size**:
```bash
# Verbose mode shows message size
echo "Test content" | plur-post --platform ssb --verbose
# Output: [INFO] Message size: 245 bytes
```

---

### Issue: "SSB library error"

**Error Message**:
```
Error: SSB library error: <error-message>
```

**Cause**: Internal error in kuska-ssb library

**Solutions**:

**1. Check Library Version**:
```bash
cargo tree | grep kuska-ssb
# Ensure using compatible version
```

**2. Update Dependencies**:
```bash
cd plurcast
cargo update
cargo build --release
```

**3. Report Bug**:
- If error persists, it may be a library bug
- Report to [kuska-ssb](https://github.com/Kuska-ssb/ssb) or Plurcast
- Include full error message and verbose output

---

## Pub Connectivity Problems

### Issue: "All pubs unreachable"

**Error Message**:
```
Warning: All configured pubs unreachable
```

**Possible Causes**:
1. No internet connection
2. Firewall blocking connections
3. Pubs are down
4. Invalid pub addresses

**Diagnostics**:

**1. Check Internet Connection**:
```bash
ping 8.8.8.8
# Should show successful pings
```

**2. Test Pub Connectivity**:
```bash
plur-creds test ssb --check-pubs

# Output shows which pubs are reachable:
#   ✓ hermies.club - reachable (45ms)
#   ✗ pub.scuttlebutt.nz - unreachable (timeout)
```

**3. Test Direct Connection**:
```bash
# Try connecting to pub directly
nc -zv hermies.club 8008
# Should show: Connection to hermies.club 8008 port [tcp/*] succeeded!
```

**Solutions**:

**1. Check Firewall**:
```bash
# Linux (iptables)
sudo iptables -L | grep 8008

# macOS
sudo pfctl -s rules | grep 8008

# Windows
netsh advfirewall firewall show rule name=all | findstr 8008
```

**2. Try Different Pubs**:
```toml
# Edit ~/.config/plurcast/config.toml
[ssb]
pubs = [
    # Try different pubs
    "net:hermies.club:8008~shs:gfW/+1nLKT+/+LNbGmHJQ8Pu7TMwCvXLPvqXbEN6kZk=",
    "net:pub.scuttlebutt.nz:8008~shs:base64-key-here"
]
```

**3. Use Local-Only Mode**:
```toml
# Temporarily disable pub replication
[ssb]
pubs = []  # Empty array
```

---

### Issue: "Pub connection timeout"

**Error Message**:
```
Warning: Pub connection timeout: hermies.club
```

**Cause**: Pub is slow or unreachable

**Solutions**:

**1. Increase Timeout** (future feature):
```toml
[ssb.network]
timeout = 60  # Increase from default 30 seconds
```

**2. Remove Slow Pub**:
```toml
[ssb]
pubs = [
    # Remove slow pub
    # "net:slow-pub.example.com:8008~shs:..."
    "net:hermies.club:8008~shs:gfW/+1nLKT+/+LNbGmHJQ8Pu7TMwCvXLPvqXbEN6kZk="
]
```

**3. Check Pub Status**:
- Visit [SSB Pub List](https://github.com/ssbc/ssb-server/wiki/Pub-Servers)
- Check if pub is marked as down
- Try alternative pubs

---

### Issue: "Invalid pub address format"

**Error Message**:
```
Error: Invalid pub address format: <address>
```

**Cause**: Pub address doesn't match multiserver format

**Correct Format**:
```
net:hostname:port~shs:public-key
```

**Example**:
```
net:hermies.club:8008~shs:gfW/+1nLKT+/+LNbGmHJQ8Pu7TMwCvXLPvqXbEN6kZk=
```

**Common Mistakes**:

❌ Missing protocol:
```
hermies.club:8008~shs:gfW/+1nLKT+/+LNbGmHJQ8Pu7TMwCvXLPvqXbEN6kZk=
```

❌ Wrong separator:
```
net:hermies.club:8008:shs:gfW/+1nLKT+/+LNbGmHJQ8Pu7TMwCvXLPvqXbEN6kZk=
```

❌ Missing public key:
```
net:hermies.club:8008~shs:
```

✅ Correct:
```
net:hermies.club:8008~shs:gfW/+1nLKT+/+LNbGmHJQ8Pu7TMwCvXLPvqXbEN6kZk=
```

---

## Replication Failures

### Issue: "Replication failed"

**Error Message**:
```
Warning: Replication failed to hermies.club: <reason>
```

**Cause**: Failed to push messages to pub

**Behavior**:
- Post is still saved locally
- Other pubs may succeed
- Replication will retry later (future feature)

**Solutions**:

**1. Check Pub Status**:
```bash
plur-creds test ssb --check-pubs
```

**2. Retry Manually** (future feature):
```bash
plur-sync --force
```

**3. Check Verbose Output**:
```bash
plur-post "Test" --platform ssb --verbose
# Shows detailed replication status
```

---

### Issue: "Messages not appearing in other clients"

**Symptom**: Posts show in Plurcast but not in Patchwork/Manyverse

**Possible Causes**:
1. Replication hasn't completed yet
2. Pubs not replicating
3. Other clients not following you
4. Network issues

**Diagnostics**:

**1. Check Replication Status**:
```bash
plur-post "Test" --platform ssb --verbose
# Look for: "✓ Replication successful"
```

**2. Verify Message in Feed**:
```bash
plur-history --platform ssb --limit 1
# Should show your recent post
```

**3. Check SSB ID**:
```bash
plur-creds test ssb
# Note your SSB ID: @abc123...=.ed25519
```

**Solutions**:

**1. Wait for Replication**:
- SSB replication is asynchronous
- Can take minutes to hours
- Be patient!

**2. Verify Pub Connectivity**:
```bash
plur-creds test ssb --check-pubs
# Ensure at least one pub is reachable
```

**3. Check in Other Clients**:
- Search for your SSB ID in Patchwork/Manyverse
- Ensure you're following yourself
- Check pub connections in other clients

**4. Force Sync** (future feature):
```bash
plur-sync --force --platform ssb
```

---

### Issue: "Replication stuck"

**Symptom**: Messages not replicating despite pub connectivity

**Diagnostics**:

```bash
# Check replication status
plur-sync-status ssb  # Future feature

# Check pub connections
plur-creds test ssb --check-pubs
```

**Solutions**:

**1. Restart Replication** (future feature):
```bash
plur-sync --restart
```

**2. Check Feed Database**:
```bash
# Verify database integrity
plur-ssb verify  # Future feature
```

**3. Try Different Pub**:
```toml
# Add or switch to different pub
[ssb]
pubs = [
    "net:different-pub.example.com:8008~shs:base64-key-here"
]
```

---

## Feed Database Issues

### Issue: "Feed database corrupted"

**Error Message**:
```
Error: Feed database corrupted: <details>
```

**Symptoms**:
- Can't open database
- Errors when posting
- Missing messages

**Diagnostics**:

```bash
# Check database files
ls -la ~/.plurcast-ssb/

# Check file integrity
file ~/.plurcast-ssb/log.offset
```

**Solutions**:

**1. Restore from Backup**:
```bash
# If you have a backup
rm -rf ~/.plurcast-ssb
tar xzf ssb-feed-backup.tar.gz -C ~/
```

**2. Rebuild Database** (future feature):
```bash
# Rebuild from network
plur-ssb rebuild --from-network
```

**3. Start Fresh**:
```bash
# WARNING: This deletes all local messages!

# Backup first (if possible)
mv ~/.plurcast-ssb ~/.plurcast-ssb.backup

# Create new database
echo "Fresh start" | plur-post --platform ssb
```

**Prevention**:
```bash
# Regular backups
tar czf ssb-feed-backup-$(date +%Y%m%d).tar.gz ~/.plurcast-ssb/

# Automated backup (cron)
0 0 * * * tar czf ~/backups/ssb-feed-$(date +\%Y\%m\%d).tar.gz ~/.plurcast-ssb/
```

---

### Issue: "Permission denied accessing feed database"

**Error Message**:
```
Error: Permission denied: ~/.plurcast-ssb/log.offset
```

**Cause**: Incorrect file permissions

**Solution**:

```bash
# Fix directory permissions
chmod 700 ~/.plurcast-ssb

# Fix file permissions
chmod 600 ~/.plurcast-ssb/log.offset
chmod 600 ~/.plurcast-ssb/flume/log.offset

# Fix ownership (if needed)
sudo chown -R $USER:$USER ~/.plurcast-ssb
```

**Verification**:
```bash
ls -la ~/.plurcast-ssb/
# Should show: drwx------ (700) for directories
# Should show: -rw------- (600) for files
```

---

### Issue: "Feed database disk full"

**Error Message**:
```
Error: No space left on device
```

**Diagnostics**:

```bash
# Check disk space
df -h ~/.plurcast-ssb

# Check database size
du -sh ~/.plurcast-ssb
```

**Solutions**:

**1. Free Up Space**:
```bash
# Remove old files
# Clean package caches, etc.
```

**2. Move Database**:
```bash
# Move to larger partition
mv ~/.plurcast-ssb /mnt/large-disk/

# Update config
# Edit ~/.config/plurcast/config.toml:
# feed_path = "/mnt/large-disk/.plurcast-ssb"
```

**3. Compact Database** (future feature):
```bash
plur-ssb compact
```

---

## Authentication Errors

### Issue: "Failed to sign SSB message"

**Error Message**:
```
Error: Failed to sign SSB message - check keypair
```

**Cause**: Keypair is invalid or corrupted

**Solution**:

```bash
# Test keypair
plur-creds test ssb

# If invalid, regenerate
plur-creds delete ssb
plur-creds set ssb --generate
```

---

### Issue: "Keypair format error"

**Error Message**:
```
Error: Keypair format error: expected Ed25519 keypair
```

**Cause**: Wrong key format (not Ed25519)

**Solution**:

```bash
# SSB requires Ed25519 keys
# Generate new Ed25519 keypair
plur-creds set ssb --generate

# Or import valid Ed25519 keypair
plur-creds set ssb --import ~/.ssb/secret
```

**Note**: SSB uses Ed25519, not secp256k1 (Nostr) or RSA

---

## Performance Issues

### Issue: "Posting to SSB is slow"

**Symptoms**:
- Long delays when posting
- Timeouts
- High CPU usage

**Possible Causes**:
1. Many pubs configured
2. Slow pub connections
3. Large feed database
4. Network issues

**Diagnostics**:

```bash
# Time the operation
time plur-post "Test" --platform ssb

# Check with verbose output
plur-post "Test" --platform ssb --verbose
# Look for slow operations
```

**Solutions**:

**1. Reduce Number of Pubs**:
```toml
[ssb]
pubs = [
    # Keep only 2-3 fast pubs
    "net:hermies.club:8008~shs:gfW/+1nLKT+/+LNbGmHJQ8Pu7TMwCvXLPvqXbEN6kZk="
]
```

**2. Remove Slow Pubs**:
```bash
# Test pub latency
plur-creds test ssb --check-pubs

# Remove pubs with high latency (>500ms)
```

**3. Optimize Database** (future feature):
```bash
plur-ssb optimize
```

**4. Use Async Replication** (future feature):
```toml
[ssb.replication]
async = true  # Don't wait for replication
```

---

### Issue: "High disk usage"

**Symptom**: Feed database growing too large

**Diagnostics**:

```bash
# Check database size
du -sh ~/.plurcast-ssb

# Check message count
plur-creds test ssb
# Shows: Messages in feed: 1234
```

**Solutions**:

**1. Normal Growth**:
- ~1KB per message is normal
- 1000 messages ≈ 1MB
- Plan for long-term storage

**2. Compact Database** (future feature):
```bash
plur-ssb compact
```

**3. Archive Old Messages** (future feature):
```bash
plur-ssb archive --before 2024-01-01
```

---

## Getting Help

### Collect Diagnostic Information

Before asking for help, collect this information:

```bash
# 1. Plurcast version
plur-post --version

# 2. SSB test output
plur-creds test ssb > ssb-test.txt 2>&1

# 3. Verbose post output
plur-post "Test" --platform ssb --verbose > ssb-post.txt 2>&1

# 4. Configuration (redact sensitive info!)
cat ~/.config/plurcast/config.toml > config.txt

# 5. System information
uname -a > system.txt
```

### Where to Get Help

**Plurcast Issues**:
- GitHub: https://github.com/plurcast/plurcast/issues
- Include diagnostic information
- Redact any credentials!

**SSB Protocol Issues**:
- SSB Forum: https://ssb-forum.netlify.app/
- SSB GitHub: https://github.com/ssbc

**kuska-ssb Library Issues**:
- GitHub: https://github.com/Kuska-ssb/ssb

### Reporting Bugs

When reporting bugs, include:

1. **Error message** (full text)
2. **Steps to reproduce**
3. **Expected behavior**
4. **Actual behavior**
5. **Diagnostic output** (see above)
6. **System information**

**Template**:
```markdown
## Bug Report

**Error**: <error message>

**Steps to Reproduce**:
1. Run `plur-post "Test" --platform ssb`
2. See error

**Expected**: Post should succeed

**Actual**: Error: <details>

**Diagnostics**:
- Plurcast version: 0.3.0-alpha2
- OS: Linux 5.15.0
- SSB test output: <attach file>

**Configuration**:
<attach config.toml with credentials redacted>
```

---

## Troubleshooting Checklist

Use this checklist to systematically diagnose issues:

- [ ] Run `plur-creds test ssb`
- [ ] Check configuration file exists and is valid
- [ ] Verify SSB credentials are stored
- [ ] Check feed database directory exists and is writable
- [ ] Test pub connectivity
- [ ] Enable verbose logging
- [ ] Check disk space
- [ ] Verify file permissions
- [ ] Review error messages carefully
- [ ] Search existing issues on GitHub
- [ ] Collect diagnostic information
- [ ] Ask for help with details

---

## Quick Reference

### Common Commands

```bash
# Test SSB setup
plur-creds test ssb

# Test with pub connectivity
plur-creds test ssb --check-pubs

# Post with verbose output
plur-post "Test" --platform ssb --verbose

# View SSB history
plur-history --platform ssb

# Check configuration
cat ~/.config/plurcast/config.toml

# Check database
ls -la ~/.plurcast-ssb/
```

### Common Fixes

```bash
# Fix permissions
chmod 700 ~/.plurcast-ssb
chmod 600 ~/.plurcast-ssb/log.offset

# Regenerate credentials
plur-creds delete ssb
plur-creds set ssb --generate

# Create database directory
mkdir -p ~/.plurcast-ssb
chmod 700 ~/.plurcast-ssb

# Test internet connectivity
ping 8.8.8.8
nc -zv hermies.club 8008
```

---

**Troubleshooting Guide Version**: 0.3.0-alpha2  
**Last Updated**: 2025-01-15

For more information:
- [SSB Setup Guide](SSB_SETUP.md)
- [SSB Configuration Guide](SSB_CONFIG.md)
- [SSB Comparison Guide](SSB_COMPARISON.md)
