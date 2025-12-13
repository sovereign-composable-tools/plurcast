# Troubleshooting Guide

Common issues and solutions for Plurcast.

## Quick Diagnostic

```bash
# Check configuration
cat ~/.config/plurcast/config.toml

# Test credentials
plur-creds test --all

# Run with verbose logging
plur-post "Test" --verbose
```

---

## Exit Codes

| Code | Meaning | Common Cause |
|------|---------|--------------|
| 0 | Success | - |
| 1 | Platform error | Network, rate limit, relay down |
| 2 | Auth error | Missing/invalid credentials |
| 3 | Invalid input | Empty content, too large, malformed |

---

## Content Errors (Exit Code 3)

### "Content cannot be empty"

```bash
$ echo "" | plur-post
Error: Content cannot be empty
```

**Solution:** Provide non-empty content.

### "Content too large"

```bash
$ plur-post "$(python -c 'print("x"*100001)')"
Error: Content too large: 100001 bytes (maximum: 100000 bytes)
```

**Solution:**
- Reduce content to under 100KB
- Split into multiple posts

### "Post exceeds character limit"

**Platform limits:**
- Mastodon: 500 characters (varies by instance)
- Nostr: ~32KB
- SSB: ~8KB

**Solution:**
```bash
# Post to platforms with higher limits only
plur-post "Long content..." --platform nostr
```

---

## Authentication Errors (Exit Code 2)

### "Could not read Nostr keys file"

**Cause:** Keys file missing or wrong path.

**Solution:**
```bash
# Check path in config
grep keys_file ~/.config/plurcast/config.toml

# Verify file exists
ls -la ~/.config/plurcast/nostr.keys

# Fix permissions
chmod 600 ~/.config/plurcast/nostr.keys
```

### "Invalid private key format"

**Cause:** Key file contains invalid format.

**Valid formats:**
- Hex: 64 characters (`a1b2c3d4...`)
- nsec: starts with `nsec1`

**Solution:**
- Remove whitespace/newlines from key file
- Verify key format

### "Invalid access token" (Mastodon)

**Solution:**
1. Regenerate token in instance settings
2. Ensure `write:statuses` scope
3. Update token file:
   ```bash
   echo "new_token" > ~/.config/plurcast/mastodon.token
   chmod 600 ~/.config/plurcast/mastodon.token
   ```

### "Account not found"

```bash
Error: Account 'test' not found for platform 'nostr'
```

**Solution:**
```bash
# List existing accounts
plur-creds list --platform nostr

# Create the account
plur-creds set nostr --account test
```

---

## Platform Errors (Exit Code 1)

### "Failed to connect to relay" (Nostr)

**Cause:** Network issues or relay down.

**Solution:**
- Check internet connection
- Try different relays
- Plurcast succeeds if ANY relay accepts

### "Rate limit exceeded"

**Solution:**
- Wait 1-5 minutes
- Plurcast auto-retries with backoff
- Check `plur-queue stats` for rate limit status

### "Instance not found" (Mastodon)

**Cause:** Wrong instance URL.

**Solution:**
```toml
# In config.toml - use domain only, no https://
[mastodon]
instance = "mastodon.social"  # NOT https://mastodon.social
```

---

## Database Errors

### "Unable to open database file"

```bash
# Create directory
mkdir -p ~/.local/share/plurcast

# Check permissions
ls -la ~/.local/share/plurcast
```

### "Database is locked"

**Cause:** Another process has the database open.

**Solution:**
- Check for running `plur-send` daemon
- Kill other Plurcast processes

---

## Credential Storage Errors

### "OS keyring not available"

**Solution:** Use encrypted file storage instead:
```toml
[credentials]
storage = "encrypted"
path = "~/.config/plurcast/credentials"
```

### "Forgot master password"

For encrypted storage:
```bash
# Delete encrypted files and reconfigure
rm -rf ~/.config/plurcast/credentials/
plur-setup
```

### "Cannot delete active account"

```bash
# Switch to different account first
plur-creds use nostr --account default

# Then delete
plur-creds delete nostr --account test
```

---

## Configuration Errors

### "Configuration file not found"

Plurcast creates default config on first run at:
`~/.config/plurcast/config.toml`

Override with:
```bash
export PLURCAST_CONFIG=~/my-config.toml
```

### "Permission denied"

```bash
# Fix credential file permissions
chmod 600 ~/.config/plurcast/*.keys
chmod 600 ~/.config/plurcast/*.token
chmod 600 ~/.config/plurcast/credentials/*
```

---

## Multi-Account Errors

### "Invalid account name"

```bash
Error: Invalid account name: 'test account'
```

**Rules:**
- Alphanumeric, hyphens, underscores only
- Max 64 characters
- No spaces or special characters

Valid: `test`, `test-account`, `test_2024`
Invalid: `test account`, `test@work`

### "Posted to wrong account"

**Prevention:**
```bash
# Always verify active account
plur-creds list

# Or use explicit account
plur-post "Important" --account prod
```

---

## Scheduling Errors

### "Daemon not running"

Scheduled posts require the daemon:
```bash
plur-send  # Start daemon
```

### "Post stuck in queue"

```bash
# Check queue status
plur-queue list

# Post immediately
plur-queue now <post_id>

# Check failed posts
plur-queue failed list
```

---

## Getting Help

1. **Verbose mode:**
   ```bash
   plur-post "Test" --verbose
   ```

2. **Check configuration:**
   ```bash
   cat ~/.config/plurcast/config.toml
   plur-creds list
   ```

3. **Test credentials:**
   ```bash
   plur-creds test --all
   ```

4. **Security audit:**
   ```bash
   plur-creds audit
   ```

5. **Open an issue** at [GitHub](https://github.com/sovereign-composable-tools/plurcast/issues) with:
   - Error message (redact credentials!)
   - Platform(s) affected
   - Steps to reproduce

---

See also:
- [Setup Guide](SETUP.md) - Configuration help
- [Security](SECURITY.md) - Credential storage
- [Usage](USAGE.md) - Command reference
