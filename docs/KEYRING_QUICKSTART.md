# Keyring Quick Start Guide

Quick reference for testing Plurcast's OS-level credential security.

## 🚀 Quick Test (5 minutes)

```powershell
# 1. Build the tools
cargo build --release

# 2. Run automated test suite
.\test-keyring.ps1

# 3. Done! Check the summary for results.
```

## 📋 Manual Testing Checklist

### Basic Flow

```powershell
# Store a credential
.\target\release\plur-creds.exe set nostr
# Enter: 0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef

# List credentials
.\target\release\plur-creds.exe list
# Expected: ✓ nostr: Private Key (stored in keyring)

# Test credential
.\target\release\plur-creds.exe test nostr
# Expected: ✓ nostr credentials found

# Delete credential
.\target\release\plur-creds.exe delete nostr --force
# Expected: ✓ Deleted nostr credentials
```

### Verify OS Storage

**Windows:**
```powershell
# Open Credential Manager
control /name Microsoft.CredentialManager
# Look for: plurcast.nostr, plurcast.mastodon, plurcast.bluesky
```

**macOS:**
```bash
# Check keychain
security find-generic-password -s "plurcast.nostr" -a "private_key"
```

**Linux:**
```bash
# Check Secret Service
secret-tool search service plurcast.nostr
```

## 🔧 Configuration

**Enable keyring storage** (`~/.config/plurcast/config.toml`):

```toml
[credentials]
storage = "keyring"  # Use OS-native secure storage
path = "~/.config/plurcast/credentials"
```

**Alternative: Encrypted file storage:**

```toml
[credentials]
storage = "encrypted"  # Password-protected files
path = "~/.config/plurcast/credentials"
```

Set master password via environment:
```powershell
$env:PLURCAST_MASTER_PASSWORD = "your-secure-password"
```

## 🧪 Test Commands

### Unit Tests

```powershell
# All credential tests
cargo test --lib credentials

# Keyring-specific tests (requires OS keyring)
cargo test --lib credentials -- --ignored

# All tests including keyring
cargo test --lib credentials -- --include-ignored
```

### CLI Tests

```powershell
# Store credentials for all platforms
.\target\release\plur-creds.exe set nostr
.\target\release\plur-creds.exe set mastodon
.\target\release\plur-creds.exe set bluesky

# List all stored credentials
.\target\release\plur-creds.exe list

# Test all platforms
.\target\release\plur-creds.exe test --all

# Security audit
.\target\release\plur-creds.exe audit

# Migrate from plain text
.\target\release\plur-creds.exe migrate
```

### Integration Test

```powershell
# Test with plur-post (retrieves credentials from keyring)
echo "Test post" | .\target\release\plur-post.exe --platform nostr --draft --verbose
```

## 🔍 Debugging

### Enable verbose logging

```powershell
$env:RUST_LOG = "debug"
.\target\release\plur-creds.exe list
```

### Check which backend is being used

```powershell
.\target\release\plur-creds.exe list
# Look for: "(stored in keyring)" or "(stored in encrypted_file)"
```

### Verify configuration

```powershell
type $env:USERPROFILE\.config\plurcast\config.toml
```

### Check for plain text files

```powershell
dir $env:USERPROFILE\.config\plurcast\*.keys
dir $env:USERPROFILE\.config\plurcast\*.token
dir $env:USERPROFILE\.config\plurcast\*.auth
# Should return: File Not Found (if using secure storage)
```

## 🐛 Common Issues

### "OS keyring not accessible"

**Windows:**
- Credential Manager service not running
- Solution: Restart Windows Credential Manager service

**macOS:**
- Keychain locked
- Solution: Unlock Keychain Access

**Linux:**
- D-Bus not running or Secret Service unavailable
- Solution: `gnome-keyring-daemon --start --components=secrets`

### Credentials not persisting

- Check configuration: `storage = "keyring"` in config.toml
- Verify keyring is available: Run `.\target\release\plur-creds.exe audit`
- Try encrypted storage as fallback

### "Credential not found" after storing

- Check which backend was used: `.\target\release\plur-creds.exe list`
- Verify credential in OS: Use platform-specific tools above
- Check logs: `$env:RUST_LOG = "debug"`

## 📊 Expected Performance

| Operation | Expected Time |
|-----------|---------------|
| Store credential | < 100ms |
| Retrieve credential | < 50ms |
| List credentials | < 100ms |
| Delete credential | < 50ms |
| Full post (with retrieval) | < 2s |

## ✅ Success Criteria

After testing, verify:

- [ ] Credentials stored in OS keyring (not plain text)
- [ ] `plur-creds list` shows "stored in keyring"
- [ ] `plur-creds audit` reports no issues
- [ ] Credentials visible in OS credential manager
- [ ] Credentials survive system reboot
- [ ] `plur-post` can retrieve and use credentials
- [ ] No `.keys`, `.token`, or `.auth` files exist

## 🔐 Security Best Practices

1. **Use keyring storage** - Most secure option
2. **Avoid plain text** - Deprecated and insecure
3. **Audit regularly** - Run `plur-creds audit` periodically
4. **Migrate legacy files** - Use `plur-creds migrate`
5. **Strong master password** - If using encrypted storage (12+ chars)

## 📚 Full Documentation

- [TESTING_KEYRING.md](./TESTING_KEYRING.md) - Comprehensive testing guide
- [ARCHITECTURE.md](./ARCHITECTURE.md) - Credential system architecture
- [SECURITY.md](./SECURITY.md) - Security considerations

## 🎯 Quick Scenarios

### Scenario 1: Fresh Setup

```powershell
# Configure keyring
# Edit config.toml: storage = "keyring"

# Store credentials
.\target\release\plur-creds.exe set nostr

# Verify
.\target\release\plur-creds.exe list
.\target\release\plur-creds.exe audit
```

### Scenario 2: Migrate from Plain Text

```powershell
# Run migration
.\target\release\plur-creds.exe migrate

# Verify migration
.\target\release\plur-creds.exe list

# Audit security
.\target\release\plur-creds.exe audit
```

### Scenario 3: Rotate Credentials

```powershell
# Update credential (overwrites existing)
.\target\release\plur-creds.exe set nostr
# Enter new key

# Verify update
.\target\release\plur-creds.exe test nostr
```

### Scenario 4: Multi-Platform Setup

```powershell
# Store all platforms
.\target\release\plur-creds.exe set nostr
.\target\release\plur-creds.exe set mastodon
.\target\release\plur-creds.exe set bluesky

# Test all
.\target\release\plur-creds.exe test --all

# List all
.\target\release\plur-creds.exe list
```

## 🤖 Automated Testing

Run the full test suite:

```powershell
# Basic run
.\test-keyring.ps1

# With verbose output
.\test-keyring.ps1 -Verbose

# Skip build step
.\test-keyring.ps1 -SkipBuild

# Keep test credentials after run
.\test-keyring.ps1 -KeepCredentials
```

## 💡 Tips

- **Use `--verbose` flag** for detailed logging
- **Check exit codes** - 0 = success, non-zero = error
- **Test in clean environment** - Remove old credentials first
- **Verify OS-level storage** - Don't just trust the CLI
- **Test cross-process access** - Multiple terminals simultaneously

---

**Version**: 0.2.0-alpha  
**Last Updated**: 2025-10-07  
**Status**: Ready for testing
