# Testing Plurcast OS-Level Keyring Security

This guide walks you through testing the production credential system using OS-native secure storage (keyring).

## Overview

Plurcast's credential system uses a layered approach with automatic fallback:
1. **KeyringStore** (Primary) - OS-native secure storage
2. **EncryptedFileStore** (Fallback) - Password-protected files
3. **PlainFileStore** (Legacy) - Plain text files (deprecated)

This guide focuses on testing the **KeyringStore** backend, which provides the highest level of security.

## Prerequisites

### Platform-Specific Requirements

**Windows:**
- Windows Credential Manager (built-in)
- No additional setup required

**macOS:**
- Keychain Access (built-in)
- No additional setup required

**Linux:**
- Secret Service API (D-Bus)
- GNOME Keyring or KWallet installed
- D-Bus session running

**Linux Setup (if needed):**
```bash
# Ubuntu/Debian
sudo apt-get install gnome-keyring

# Fedora/RHEL
sudo dnf install gnome-keyring

# Arch
sudo pacman -S gnome-keyring

# Start keyring daemon (if not running)
gnome-keyring-daemon --start --components=secrets
```

## Testing Strategy

### Phase 1: Unit Tests (Automated)

The codebase includes comprehensive unit tests for all credential backends. These tests are marked with `#[ignore]` for keyring tests because they require OS-level access.

**Run all credential tests:**
```powershell
# Run standard tests (encrypted and plain file stores)
cargo test --lib credentials

# Run keyring tests (requires OS keyring access)
cargo test --lib credentials -- --ignored

# Run all tests including keyring
cargo test --lib credentials -- --include-ignored
```

**Expected output:**
```
running 15 tests
test credentials::keyring_store_tests::test_keyring_store_operations ... ok
test credentials::keyring_store_tests::test_keyring_multiple_platforms ... ok
test credentials::keyring_store_tests::test_keyring_service_naming ... ok
test credentials::encrypted_file_store_tests::test_encrypted_store_operations ... ok
test credentials::plain_file_store_tests::test_plain_store_operations ... ok
test credentials::credential_manager_tests::test_credential_manager_plain_backend ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Phase 2: CLI Tool Testing (Manual)

Test the `plur-creds` command-line tool with real OS keyring integration.

#### Step 1: Build the Tools

```powershell
# Build all binaries
cargo build --release

# Verify plur-creds is available
.\target\release\plur-creds.exe --help
```

#### Step 2: Configure Keyring Storage

Create or edit `~/.config/plurcast/config.toml`:

```toml
[credentials]
storage = "keyring"  # Use OS-native keyring
path = "~/.config/plurcast/credentials"  # Not used for keyring, but required
```

**Verify configuration:**
```powershell
# Check current configuration
type $env:USERPROFILE\.config\plurcast\config.toml
```

#### Step 3: Test Credential Storage

**Store credentials for each platform:**

```powershell
# Store Nostr credentials
.\target\release\plur-creds.exe set nostr
# Enter when prompted: nsec1... or 64-character hex key

# Store Mastodon credentials
.\target\release\plur-creds.exe set mastodon
# Enter when prompted: your OAuth access token

# Store Bluesky credentials
.\target\release\plur-creds.exe set bluesky
# Enter when prompted: your app password
```

**Expected output:**
```
✓ Stored nostr credentials securely using keyring backend
```

#### Step 4: List Stored Credentials

```powershell
.\target\release\plur-creds.exe list
```

**Expected output:**
```
Stored credentials:

  ✓ nostr: Private Key (stored in keyring)
  ✓ mastodon: Access Token (stored in keyring)
  ✓ bluesky: App Password (stored in keyring)
```

#### Step 5: Test Credential Retrieval

```powershell
# Test individual platform credentials
.\target\release\plur-creds.exe test nostr
.\target\release\plur-creds.exe test mastodon
.\target\release\plur-creds.exe test bluesky

# Test all platforms at once
.\target\release\plur-creds.exe test --all
```

**Expected output:**
```
Testing nostr credentials...
✓ nostr credentials found
  Note: Full authentication testing requires platform client integration
```

#### Step 6: Verify OS-Level Storage

**Windows:**
```powershell
# Open Credential Manager
control /name Microsoft.CredentialManager

# Look for entries named:
# - plurcast.nostr
# - plurcast.mastodon
# - plurcast.bluesky
```

**macOS:**
```bash
# Open Keychain Access
open -a "Keychain Access"

# Search for "plurcast" in the search bar
# You should see entries for each platform

# Or use command line:
security find-generic-password -s "plurcast.nostr" -a "private_key"
```

**Linux:**
```bash
# Using secret-tool (part of libsecret)
secret-tool search service plurcast.nostr

# Or use seahorse (GUI)
seahorse
```

#### Step 7: Test Credential Deletion

```powershell
# Delete a credential (with confirmation)
.\target\release\plur-creds.exe delete nostr

# Delete without confirmation
.\target\release\plur-creds.exe delete mastodon --force

# Verify deletion
.\target\release\plur-creds.exe list
```

**Expected output:**
```
Delete nostr credentials? [y/N]: y
✓ Deleted nostr credentials
```

### Phase 3: Integration Testing with plur-post

Test that the posting tool can retrieve credentials from the keyring.

#### Step 1: Store Test Credentials

```powershell
# Store a test Nostr key
.\target\release\plur-creds.exe set nostr
# Enter a valid nsec key or hex private key
```

#### Step 2: Test Posting

```powershell
# Enable verbose logging to see credential retrieval
$env:RUST_LOG="debug"

# Test posting (will attempt to retrieve credentials from keyring)
echo "Test post from keyring" | .\target\release\plur-post.exe --platform nostr --verbose
```

**Expected debug output:**
```
DEBUG libplurcast::credentials: Retrieved credential for plurcast.nostr.private_key from OS keyring
DEBUG libplurcast::platforms::nostr: Initialized Nostr client with credentials
INFO plur_post: Posted to nostr: note1abc...
```

### Phase 4: Security Audit

Run the built-in security audit to verify your setup.

```powershell
.\target\release\plur-creds.exe audit
```

**Expected output (secure setup):**
```
Auditing credential security...

Credential storage configuration:
  Backend: Keyring
  Path: ~/.config/plurcast/credentials

✓ Using secure credential storage: keyring

✓ Security audit complete: No issues found
```

**Expected output (issues found):**
```
Auditing credential security...

⚠ SECURITY ISSUE: Plain text credential files found:
  - C:\Users\[user]\.config\plurcast\nostr.keys (Nostr private key)
  Recommendation: Run 'plur-creds migrate' to move to secure storage

Security audit complete: Issues found
Follow the recommendations above to improve security.
```

### Phase 5: Migration Testing

If you have existing plain text credentials, test the migration process.

#### Step 1: Create Test Plain Text Files

```powershell
# Create test directory
mkdir $env:USERPROFILE\.config\plurcast -Force

# Create test plain text credentials
echo "test_nostr_key_12345678901234567890123456789012345678901234567890123456" > $env:USERPROFILE\.config\plurcast\nostr.keys
echo "test_mastodon_token" > $env:USERPROFILE\.config\plurcast\mastodon.token
echo "test_bluesky_password" > $env:USERPROFILE\.config\plurcast\bluesky.auth
```

#### Step 2: Run Migration

```powershell
.\target\release\plur-creds.exe migrate
```

**Expected output:**
```
Migrating credentials from plain text files to secure storage...

Found 3 plain text credential file(s):
  - plurcast.nostr.private_key at C:\Users\[user]\.config\plurcast\nostr.keys
  - plurcast.mastodon.access_token at C:\Users\[user]\.config\plurcast\mastodon.token
  - plurcast.bluesky.app_password at C:\Users\[user]\.config\plurcast\bluesky.auth

Migration complete:
  ✓ Migrated: 3
  ✗ Failed: 0
  ⊘ Skipped: 0

Successfully migrated:
  ✓ plurcast.nostr.private_key
  ✓ plurcast.mastodon.access_token
  ✓ plurcast.bluesky.app_password

Delete plain text files? [y/N]: y
✓ Deleted 3 plain text file(s)
```

#### Step 3: Verify Migration

```powershell
# List credentials (should show keyring backend)
.\target\release\plur-creds.exe list

# Verify plain text files are gone
dir $env:USERPROFILE\.config\plurcast\*.keys
dir $env:USERPROFILE\.config\plurcast\*.token
dir $env:USERPROFILE\.config\plurcast\*.auth
```

## Testing Scenarios

### Scenario 1: Fresh Installation

**Goal:** Verify keyring works on a clean system.

```powershell
# 1. Remove any existing credentials
.\target\release\plur-creds.exe delete nostr --force
.\target\release\plur-creds.exe delete mastodon --force
.\target\release\plur-creds.exe delete bluesky --force

# 2. Configure keyring storage
# Edit config.toml to set storage = "keyring"

# 3. Store new credentials
.\target\release\plur-creds.exe set nostr

# 4. Verify storage
.\target\release\plur-creds.exe list

# 5. Test retrieval
.\target\release\plur-creds.exe test nostr
```

### Scenario 2: Keyring Unavailable (Fallback Testing)

**Goal:** Verify graceful fallback when keyring is unavailable.

**On Linux (simulate unavailable keyring):**
```bash
# Stop keyring daemon
killall gnome-keyring-daemon

# Try to use keyring (should fall back to encrypted storage)
./target/release/plur-creds set nostr

# Should prompt for master password and use encrypted files
```

**Expected behavior:**
- Detects keyring unavailable
- Falls back to encrypted file storage
- Prompts for master password
- Stores credentials in `~/.config/plurcast/credentials/*.age`

### Scenario 3: Multi-Platform Credentials

**Goal:** Verify all three platforms can store/retrieve simultaneously.

```powershell
# Store all three platforms
.\target\release\plur-creds.exe set nostr
.\target\release\plur-creds.exe set mastodon
.\target\release\plur-creds.exe set bluesky

# List all
.\target\release\plur-creds.exe list

# Test all
.\target\release\plur-creds.exe test --all

# Verify in OS keyring (Windows example)
# Open Credential Manager and verify three entries exist
```

### Scenario 4: Credential Rotation

**Goal:** Test updating existing credentials.

```powershell
# Store initial credential
.\target\release\plur-creds.exe set nostr
# Enter: old_key_value

# Update with new credential
.\target\release\plur-creds.exe set nostr
# Enter: new_key_value

# Verify new credential is stored
.\target\release\plur-creds.exe test nostr

# Test posting with new credential
echo "Test with rotated key" | .\target\release\plur-post.exe --platform nostr
```

### Scenario 5: Cross-Process Access

**Goal:** Verify multiple processes can access keyring simultaneously.

```powershell
# Terminal 1: Store credential
.\target\release\plur-creds.exe set nostr

# Terminal 2: Retrieve credential (different process)
.\target\release\plur-creds.exe test nostr

# Terminal 3: Use credential for posting
echo "Multi-process test" | .\target\release\plur-post.exe --platform nostr
```

**Expected:** All processes should successfully access the keyring.

## Troubleshooting

### Issue: "OS keyring not accessible"

**Windows:**
- Ensure Credential Manager service is running
- Check Windows Event Viewer for credential errors

**macOS:**
- Unlock Keychain if locked
- Check Keychain Access permissions

**Linux:**
- Verify D-Bus is running: `ps aux | grep dbus`
- Start keyring daemon: `gnome-keyring-daemon --start --components=secrets`
- Check environment: `echo $DBUS_SESSION_BUS_ADDRESS`

### Issue: "Credential not found" after storing

**Diagnosis:**
```powershell
# Check which backend is actually being used
.\target\release\plur-creds.exe list --verbose

# Verify configuration
type $env:USERPROFILE\.config\plurcast\config.toml
```

**Solution:**
- Ensure `storage = "keyring"` in config.toml
- Check that keyring is available on your system
- Try encrypted storage as fallback

### Issue: Permission denied on Linux

**Solution:**
```bash
# Ensure user has access to Secret Service
dbus-send --session --print-reply --dest=org.freedesktop.secrets /org/freedesktop/secrets org.freedesktop.DBus.Properties.Get string:org.freedesktop.Secret.Service string:Collections

# If fails, restart keyring daemon
gnome-keyring-daemon --replace --components=secrets
```

## Security Verification Checklist

- [ ] Credentials stored in OS keyring (not plain text files)
- [ ] No `.keys`, `.token`, or `.auth` files in config directory
- [ ] `plur-creds audit` reports no issues
- [ ] Credentials visible in OS credential manager
- [ ] Credentials survive system reboot
- [ ] Multiple processes can access credentials
- [ ] Credentials deleted from keyring when using `plur-creds delete`
- [ ] Migration from plain text works correctly
- [ ] Fallback to encrypted storage works when keyring unavailable

## Performance Testing

### Benchmark Credential Operations

```powershell
# Time credential storage
Measure-Command { .\target\release\plur-creds.exe set nostr }

# Time credential retrieval (via test command)
Measure-Command { .\target\release\plur-creds.exe test nostr }

# Time posting (includes credential retrieval)
Measure-Command { echo "Benchmark post" | .\target\release\plur-post.exe --platform nostr }
```

**Expected performance:**
- Keyring store: < 100ms
- Keyring retrieve: < 50ms
- Full post operation: < 2s (network dependent)

## Automated Testing Script

Create a PowerShell script to run all tests:

```powershell
# test-keyring.ps1

Write-Host "=== Plurcast Keyring Testing Suite ===" -ForegroundColor Cyan

# Phase 1: Unit tests
Write-Host "`n[Phase 1] Running unit tests..." -ForegroundColor Yellow
cargo test --lib credentials -- --include-ignored

# Phase 2: CLI tests
Write-Host "`n[Phase 2] Testing CLI tools..." -ForegroundColor Yellow

# Clean up
Write-Host "Cleaning up existing credentials..."
.\target\release\plur-creds.exe delete nostr --force 2>$null
.\target\release\plur-creds.exe delete mastodon --force 2>$null
.\target\release\plur-creds.exe delete bluesky --force 2>$null

# Store test credentials
Write-Host "Storing test credentials..."
echo "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef" | .\target\release\plur-creds.exe set nostr

# List credentials
Write-Host "`nListing credentials..."
.\target\release\plur-creds.exe list

# Test credentials
Write-Host "`nTesting credentials..."
.\target\release\plur-creds.exe test --all

# Security audit
Write-Host "`n[Phase 3] Running security audit..." -ForegroundColor Yellow
.\target\release\plur-creds.exe audit

# Clean up
Write-Host "`n[Cleanup] Removing test credentials..." -ForegroundColor Yellow
.\target\release\plur-creds.exe delete nostr --force

Write-Host "`n=== Testing Complete ===" -ForegroundColor Green
```

**Run the script:**
```powershell
.\test-keyring.ps1
```

## Next Steps

After verifying keyring functionality:

1. **Document platform-specific quirks** in SECURITY.md
2. **Add CI/CD tests** for keyring (where available)
3. **Create user onboarding guide** for credential setup
4. **Implement `plur-setup` wizard** for guided configuration
5. **Add credential backup/restore** functionality

## References

- [ARCHITECTURE.md](./ARCHITECTURE.md) - Credential storage architecture
- [SECURITY.md](./SECURITY.md) - Security considerations
- [libplurcast/src/credentials.rs](./libplurcast/src/credentials.rs) - Implementation
- [plur-creds/src/main.rs](./plur-creds/src/main.rs) - CLI tool

---

**Version**: 0.2.0-alpha  
**Last Updated**: 2025-10-07  
**Status**: Active Development - Phase 2 (Multi-Platform) with Secure Credentials
