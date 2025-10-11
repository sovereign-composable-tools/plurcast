# Plurcast Keyring Testing - Complete Guide

**Quick Links:**
- [Quick Start](./KEYRING_QUICKSTART.md) - 5-minute testing guide
- [Full Testing Guide](./TESTING_KEYRING.md) - Comprehensive testing documentation
- [Testing Flow Diagrams](./docs/keyring-testing-flow.md) - Visual testing architecture
- [Automated Test Script](./test-keyring.ps1) - PowerShell testing automation

---

## What You're Testing

Plurcast's **OS-level credential security system** that stores platform credentials (Nostr keys, Mastodon tokens, Bluesky passwords) securely using:

1. **KeyringStore** (Primary) - OS-native secure storage
   - Windows: Credential Manager
   - macOS: Keychain
   - Linux: Secret Service (GNOME Keyring/KWallet)

2. **EncryptedFileStore** (Fallback) - Password-protected files using age encryption

3. **PlainFileStore** (Legacy) - Plain text files (deprecated, for backward compatibility)

## Why This Matters

**Security Benefits:**
- Credentials encrypted at OS level
- Protected by system authentication
- Survives process termination
- Accessible across multiple processes
- No plain text files on disk

**User Benefits:**
- Seamless credential management
- No manual file editing
- Automatic fallback if keyring unavailable
- Migration from legacy plain text files
- Security audit tools

## Testing Approaches

### 1. Quick Automated Test (Recommended)

**Time:** 2-5 minutes

```powershell
# Run the automated test suite
.\test-keyring.ps1
```

**What it tests:**
- ✓ Unit tests for all credential backends
- ✓ CLI tool functionality (store, list, test, delete)
- ✓ OS-level credential verification
- ✓ Security audit
- ✓ Integration with plur-post

**Expected output:**
```
╔═══════════════════════════════════════════════════════════╗
║                      Test Summary                         ║
╚═══════════════════════════════════════════════════════════╝
  Passed:  15
  Failed:  0
  Skipped: 2
  Total:   17

✓ All tests passed!
```

### 2. Manual Step-by-Step Test

**Time:** 10-15 minutes

Follow the [Quick Start Guide](./KEYRING_QUICKSTART.md) for manual testing:

```powershell
# 1. Build
cargo build --release

# 2. Store credential
.\target\release\plur-creds.exe set nostr

# 3. List credentials
.\target\release\plur-creds.exe list

# 4. Test retrieval
.\target\release\plur-creds.exe test nostr

# 5. Security audit
.\target\release\plur-creds.exe audit

# 6. Verify in OS
control /name Microsoft.CredentialManager  # Windows

# 7. Cleanup
.\target\release\plur-creds.exe delete nostr --force
```

### 3. Comprehensive Testing

**Time:** 30-60 minutes

Follow the [Full Testing Guide](./TESTING_KEYRING.md) for:
- All unit tests (including ignored keyring tests)
- All CLI scenarios
- Migration testing
- Integration testing
- Performance benchmarking
- Security verification

## Test Coverage

### Unit Tests (Automated)

**Location:** `libplurcast/src/credentials/tests.rs`

**Coverage:**
- KeyringStore operations (store, retrieve, delete, exists)
- EncryptedFileStore operations
- PlainFileStore operations (legacy)
- CredentialManager fallback logic
- Multi-platform credential handling
- Error handling (weak passwords, missing credentials, etc.)
- File permissions (Unix)
- Service naming conventions

**Run:**
```powershell
# Standard tests
cargo test --lib credentials

# Including keyring tests (requires OS keyring)
cargo test --lib credentials -- --include-ignored
```

### CLI Tests (Manual/Scripted)

**Tool:** `plur-creds` binary

**Coverage:**
- `set` - Store credentials for each platform
- `list` - Display stored credentials and backend used
- `test` - Verify credentials exist and are retrievable
- `delete` - Remove credentials from storage
- `migrate` - Migrate from plain text to secure storage
- `audit` - Security audit of credential configuration

**Run:**
```powershell
# Automated
.\test-keyring.ps1

# Manual
.\target\release\plur-creds.exe --help
```

### Integration Tests

**Coverage:**
- `plur-post` retrieves credentials from keyring
- Cross-process credential access
- Credential rotation (updating existing credentials)
- Multi-platform simultaneous access

**Run:**
```powershell
# Store credential
.\target\release\plur-creds.exe set nostr

# Use in plur-post
echo "Test post" | .\target\release\plur-post.exe --platform nostr --draft
```

### OS-Level Verification

**Coverage:**
- Windows Credential Manager entries
- macOS Keychain entries
- Linux Secret Service entries
- Credential persistence across reboots
- Credential isolation per service/key

**Verify:**
```powershell
# Windows
control /name Microsoft.CredentialManager

# macOS
security find-generic-password -s "plurcast.nostr"

# Linux
secret-tool search service plurcast.nostr
```

## Success Criteria

After testing, you should verify:

### ✅ Functional Requirements
- [ ] Credentials can be stored via `plur-creds set`
- [ ] Credentials can be listed via `plur-creds list`
- [ ] Credentials can be retrieved via `plur-creds test`
- [ ] Credentials can be deleted via `plur-creds delete`
- [ ] Migration from plain text works via `plur-creds migrate`
- [ ] Security audit passes via `plur-creds audit`

### ✅ Security Requirements
- [ ] Credentials stored in OS keyring (not plain text files)
- [ ] No `.keys`, `.token`, or `.auth` files in config directory
- [ ] Credentials visible in OS credential manager
- [ ] File permissions are 600 on Unix (if using file storage)
- [ ] Credentials survive system reboot
- [ ] Credentials isolated per platform

### ✅ Integration Requirements
- [ ] `plur-post` can retrieve credentials from keyring
- [ ] Multiple processes can access credentials simultaneously
- [ ] Credential rotation works (updating existing credentials)
- [ ] Fallback to encrypted storage works when keyring unavailable

### ✅ Performance Requirements
- [ ] Store operation: < 100ms
- [ ] Retrieve operation: < 50ms
- [ ] List operation: < 100ms
- [ ] Full post operation: < 2s

## Common Test Scenarios

### Scenario 1: Fresh Installation
**Goal:** Verify keyring works on clean system

```powershell
# Configure keyring storage
# Edit config.toml: storage = "keyring"

# Store and test
.\target\release\plur-creds.exe set nostr
.\target\release\plur-creds.exe test nostr
.\target\release\plur-creds.exe audit
```

### Scenario 2: Migration from Plain Text
**Goal:** Verify migration process

```powershell
# Create test plain text files
echo "test_key" > $env:USERPROFILE\.config\plurcast\nostr.keys

# Run migration
.\target\release\plur-creds.exe migrate

# Verify
.\target\release\plur-creds.exe list
.\target\release\plur-creds.exe audit
```

### Scenario 3: Multi-Platform Setup
**Goal:** Verify all platforms work together

```powershell
# Store all platforms
.\target\release\plur-creds.exe set nostr
.\target\release\plur-creds.exe set mastodon
.\target\release\plur-creds.exe set bluesky

# Test all
.\target\release\plur-creds.exe test --all
```

### Scenario 4: Keyring Unavailable (Fallback)
**Goal:** Verify graceful fallback

```powershell
# On Linux: Stop keyring daemon
killall gnome-keyring-daemon

# Try to store (should fall back to encrypted)
.\target\release\plur-creds.exe set nostr
# Should prompt for master password
```

### Scenario 5: Credential Rotation
**Goal:** Verify updating credentials

```powershell
# Store initial credential
.\target\release\plur-creds.exe set nostr
# Enter: old_key

# Update credential
.\target\release\plur-creds.exe set nostr
# Enter: new_key

# Verify update
.\target\release\plur-creds.exe test nostr
```

## Troubleshooting

### Issue: "OS keyring not accessible"

**Diagnosis:**
```powershell
# Check configuration
type $env:USERPROFILE\.config\plurcast\config.toml

# Try manual keyring access (Windows)
cmdkey /list | Select-String "plurcast"
```

**Solutions:**
- **Windows:** Restart Credential Manager service
- **macOS:** Unlock Keychain Access
- **Linux:** Start keyring daemon: `gnome-keyring-daemon --start --components=secrets`

### Issue: Tests failing in CI/CD

**Cause:** Keyring not available in headless environments

**Solution:** Configure fallback to encrypted storage:
```toml
[credentials]
storage = "encrypted"
master_password = "${PLURCAST_MASTER_PASSWORD}"  # From env var
```

### Issue: Credentials not persisting

**Diagnosis:**
```powershell
# Check which backend is being used
.\target\release\plur-creds.exe list

# Check for plain text files
dir $env:USERPROFILE\.config\plurcast\*.keys
```

**Solution:** Ensure `storage = "keyring"` in config.toml

## Performance Benchmarking

```powershell
# Benchmark credential operations
Measure-Command { .\target\release\plur-creds.exe set nostr }
Measure-Command { .\target\release\plur-creds.exe test nostr }
Measure-Command { .\target\release\plur-creds.exe list }

# Benchmark full post operation
Measure-Command { echo "Test" | .\target\release\plur-post.exe --platform nostr --draft }
```

**Expected results:**
- Store: 50-100ms
- Retrieve: 20-50ms
- List: 50-100ms
- Post: 1-2s (network dependent)

## Next Steps After Testing

1. **Document platform-specific issues** encountered during testing
2. **Update CI/CD configuration** to handle keyring unavailability
3. **Create user onboarding guide** based on testing experience
4. **Implement `plur-setup` wizard** for guided credential configuration
5. **Add credential backup/restore** functionality
6. **Performance optimization** if benchmarks show issues

## Resources

### Documentation
- [ARCHITECTURE.md](./ARCHITECTURE.md) - Credential system architecture
- [SECURITY.md](./SECURITY.md) - Security considerations and best practices
- [ROADMAP.md](./ROADMAP.md) - Development phases and progress

### Code
- [libplurcast/src/credentials.rs](./libplurcast/src/credentials.rs) - Core implementation
- [libplurcast/src/credentials/tests.rs](./libplurcast/src/credentials/tests.rs) - Unit tests
- [plur-creds/src/main.rs](./plur-creds/src/main.rs) - CLI tool

### External References
- [keyring-rs](https://github.com/hwchen/keyring-rs) - Rust keyring library
- [age](https://github.com/str4d/rage) - Rust age encryption
- [Windows Credential Manager](https://docs.microsoft.com/en-us/windows/security/identity-protection/credential-guard/)
- [macOS Keychain](https://developer.apple.com/documentation/security/keychain_services)
- [Linux Secret Service](https://specifications.freedesktop.org/secret-service/)

## Quick Reference

### Test Commands
```powershell
# Automated testing
.\test-keyring.ps1                          # Full test suite
.\test-keyring.ps1 -Verbose                 # With detailed output
.\test-keyring.ps1 -SkipBuild              # Skip build step

# Unit testing
cargo test --lib credentials                # Standard tests
cargo test --lib credentials -- --ignored   # Keyring tests only

# Manual testing
.\target\release\plur-creds.exe set nostr   # Store credential
.\target\release\plur-creds.exe list        # List credentials
.\target\release\plur-creds.exe test nostr  # Test credential
.\target\release\plur-creds.exe audit       # Security audit
.\target\release\plur-creds.exe migrate     # Migrate from plain text
```

### Configuration
```toml
# ~/.config/plurcast/config.toml
[credentials]
storage = "keyring"  # or "encrypted" or "plain"
path = "~/.config/plurcast/credentials"
```

### Environment Variables
```powershell
$env:RUST_LOG = "debug"                     # Enable debug logging
$env:PLURCAST_MASTER_PASSWORD = "password"  # Set master password
```

---

**Version**: 0.2.0-alpha  
**Last Updated**: 2025-10-07  
**Status**: Complete testing documentation ready for use

**Feedback:** If you encounter issues or have suggestions for improving the testing process, please document them for future updates.
