# Plurcast Keyring Testing Checklist

Use this checklist to verify OS-level credential security is working correctly.

## Pre-Testing Setup

- [ ] Rust toolchain installed (1.70+)
- [ ] Git repository cloned
- [ ] Dependencies installed (`cargo build`)
- [ ] OS keyring available (Windows Credential Manager / macOS Keychain / Linux Secret Service)

## Phase 1: Build Verification

- [ ] `cargo build --release` completes successfully
- [ ] `plur-creds.exe` binary exists in `target/release/`
- [ ] `plur-post.exe` binary exists in `target/release/`
- [ ] Binaries run without errors (`--help` flag works)

**Commands:**
```powershell
cargo build --release
.\target\release\plur-creds.exe --help
.\target\release\plur-post.exe --help
```

## Phase 2: Unit Tests

- [ ] Standard credential tests pass
- [ ] Keyring-specific tests pass (or skip gracefully)
- [ ] Encrypted file store tests pass
- [ ] Plain file store tests pass
- [ ] Credential manager tests pass
- [ ] No test failures reported

**Commands:**
```powershell
cargo test --lib credentials
cargo test --lib credentials -- --ignored
```

**Expected:** All tests pass or skip gracefully

## Phase 3: Configuration

- [ ] Config directory exists: `~/.config/plurcast/`
- [ ] Config file exists: `~/.config/plurcast/config.toml`
- [ ] Keyring storage configured: `storage = "keyring"`
- [ ] Path configured: `path = "~/.config/plurcast/credentials"`

**Commands:**
```powershell
mkdir $env:USERPROFILE\.config\plurcast -Force
# Edit config.toml to add [credentials] section
```

**Config content:**
```toml
[credentials]
storage = "keyring"
path = "~/.config/plurcast/credentials"
```

## Phase 4: Credential Storage

### Nostr Credentials
- [ ] `plur-creds set nostr` prompts for key
- [ ] Valid key accepted (64 hex chars or nsec format)
- [ ] Invalid key rejected with error message
- [ ] Success message shows "keyring backend"
- [ ] Credential stored in OS keyring

**Commands:**
```powershell
.\target\release\plur-creds.exe set nostr
# Enter: 0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
```

**Expected output:**
```
✓ Stored nostr credentials securely using keyring backend
```

### Mastodon Credentials (Optional)
- [ ] `plur-creds set mastodon` prompts for token
- [ ] Token accepted and stored
- [ ] Success message shows "keyring backend"

### Bluesky Credentials (Optional)
- [ ] `plur-creds set bluesky` prompts for password
- [ ] Password accepted and stored
- [ ] Success message shows "keyring backend"

## Phase 5: Credential Listing

- [ ] `plur-creds list` shows stored credentials
- [ ] Nostr credential listed
- [ ] Backend shown as "keyring"
- [ ] No credential values displayed (security)
- [ ] Format is clean and readable

**Commands:**
```powershell
.\target\release\plur-creds.exe list
```

**Expected output:**
```
Stored credentials:

  ✓ nostr: Private Key (stored in keyring)
```

## Phase 6: Credential Retrieval

- [ ] `plur-creds test nostr` succeeds
- [ ] Success message displayed
- [ ] Exit code is 0
- [ ] No errors in output

**Commands:**
```powershell
.\target\release\plur-creds.exe test nostr
echo $LASTEXITCODE  # Should be 0
```

**Expected output:**
```
Testing nostr credentials...
✓ nostr credentials found
  Note: Full authentication testing requires platform client integration
```

### Test All Platforms
- [ ] `plur-creds test --all` runs
- [ ] Shows results for all configured platforms
- [ ] Exit code reflects overall success

## Phase 7: Security Audit

- [ ] `plur-creds audit` runs successfully
- [ ] Reports keyring storage in use
- [ ] No security issues found
- [ ] No plain text files detected
- [ ] Exit code is 0

**Commands:**
```powershell
.\target\release\plur-creds.exe audit
echo $LASTEXITCODE  # Should be 0
```

**Expected output:**
```
Auditing credential security...

Credential storage configuration:
  Backend: Keyring
  Path: ~/.config/plurcast/credentials

✓ Using secure credential storage: keyring

✓ Security audit complete: No issues found
```

## Phase 8: OS-Level Verification

### Windows
- [ ] Open Credential Manager: `control /name Microsoft.CredentialManager`
- [ ] Search for "plurcast"
- [ ] Entry "plurcast.nostr" exists
- [ ] Entry shows username "private_key"
- [ ] Password is masked

**Commands:**
```powershell
control /name Microsoft.CredentialManager
# Or: cmdkey /list | Select-String "plurcast"
```

### macOS
- [ ] Open Keychain Access
- [ ] Search for "plurcast"
- [ ] Entry "plurcast.nostr" exists
- [ ] Account name is "private_key"
- [ ] Password is protected

**Commands:**
```bash
security find-generic-password -s "plurcast.nostr" -a "private_key"
```

### Linux
- [ ] Secret Service is running
- [ ] D-Bus session active
- [ ] Entry "plurcast.nostr" exists
- [ ] Attribute "private_key" present

**Commands:**
```bash
secret-tool search service plurcast.nostr
```

## Phase 9: Integration Testing

### With plur-post
- [ ] `plur-post` can retrieve credentials
- [ ] Draft mode works with credentials
- [ ] Verbose mode shows credential retrieval
- [ ] No credential values in logs

**Commands:**
```powershell
$env:RUST_LOG = "debug"
echo "Test post from keyring" | .\target\release\plur-post.exe --platform nostr --draft --verbose
```

**Expected in logs:**
```
DEBUG libplurcast::credentials: Retrieved credential for plurcast.nostr.private_key from OS keyring
```

### Cross-Process Access
- [ ] Open two terminals
- [ ] Both can access credentials simultaneously
- [ ] No locking or blocking issues

**Commands (Terminal 1):**
```powershell
.\target\release\plur-creds.exe test nostr
```

**Commands (Terminal 2):**
```powershell
.\target\release\plur-creds.exe test nostr
```

## Phase 10: Credential Rotation

- [ ] Update existing credential with `set` command
- [ ] New credential overwrites old one
- [ ] Test retrieves new credential
- [ ] Old credential no longer accessible

**Commands:**
```powershell
# Store initial
.\target\release\plur-creds.exe set nostr
# Enter: old_key_value

# Update
.\target\release\plur-creds.exe set nostr
# Enter: new_key_value

# Verify
.\target\release\plur-creds.exe test nostr
```

## Phase 11: Credential Deletion

### With Confirmation
- [ ] `plur-creds delete nostr` prompts for confirmation
- [ ] Typing "y" deletes credential
- [ ] Typing "n" cancels deletion
- [ ] Success message displayed

**Commands:**
```powershell
.\target\release\plur-creds.exe delete nostr
# Enter: y
```

### Without Confirmation
- [ ] `plur-creds delete nostr --force` deletes immediately
- [ ] No prompt shown
- [ ] Success message displayed

**Commands:**
```powershell
.\target\release\plur-creds.exe delete nostr --force
```

### Verification
- [ ] `plur-creds list` no longer shows deleted credential
- [ ] `plur-creds test nostr` fails with "not found"
- [ ] Credential removed from OS keyring

## Phase 12: Migration Testing

### Setup
- [ ] Create test plain text files
- [ ] Files have correct permissions (600 on Unix)
- [ ] Files contain valid credential data

**Commands:**
```powershell
mkdir $env:USERPROFILE\.config\plurcast -Force
echo "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef" > $env:USERPROFILE\.config\plurcast\nostr.keys
```

### Migration
- [ ] `plur-creds migrate` detects plain text files
- [ ] Shows list of files to migrate
- [ ] Migrates successfully
- [ ] Reports migration results
- [ ] Prompts to delete plain text files
- [ ] Deletes files if confirmed

**Commands:**
```powershell
.\target\release\plur-creds.exe migrate
# Enter: y (to delete plain text files)
```

**Expected output:**
```
Migrating credentials from plain text files to secure storage...

Found 1 plain text credential file(s):
  - plurcast.nostr.private_key at C:\Users\[user]\.config\plurcast\nostr.keys

Migration complete:
  ✓ Migrated: 1
  ✗ Failed: 0
  ⊘ Skipped: 0

Successfully migrated:
  ✓ plurcast.nostr.private_key

Delete plain text files? [y/N]: y
✓ Deleted 1 plain text file(s)
```

### Verification
- [ ] Plain text files deleted
- [ ] Credentials now in keyring
- [ ] `plur-creds list` shows keyring backend
- [ ] `plur-creds audit` reports no issues

## Phase 13: Fallback Testing (Optional)

### Keyring Unavailable
- [ ] Stop keyring service (Linux only)
- [ ] Attempt to store credential
- [ ] Falls back to encrypted storage
- [ ] Prompts for master password
- [ ] Stores in encrypted file

**Commands (Linux):**
```bash
killall gnome-keyring-daemon
./target/release/plur-creds set nostr
# Should prompt for master password
```

### Encrypted Storage
- [ ] Configure `storage = "encrypted"`
- [ ] Set master password (env var or prompt)
- [ ] Store credential
- [ ] Encrypted file created (*.age)
- [ ] File permissions are 600
- [ ] Retrieve credential works

**Config:**
```toml
[credentials]
storage = "encrypted"
path = "~/.config/plurcast/credentials"
```

**Commands:**
```powershell
$env:PLURCAST_MASTER_PASSWORD = "test-password-123"
.\target\release\plur-creds.exe set nostr
```

## Phase 14: Performance Testing

- [ ] Store operation: < 100ms
- [ ] Retrieve operation: < 50ms
- [ ] List operation: < 100ms
- [ ] Delete operation: < 50ms
- [ ] Full post operation: < 2s

**Commands:**
```powershell
Measure-Command { .\target\release\plur-creds.exe set nostr }
Measure-Command { .\target\release\plur-creds.exe test nostr }
Measure-Command { .\target\release\plur-creds.exe list }
Measure-Command { .\target\release\plur-creds.exe delete nostr --force }
```

## Phase 15: Automated Testing

- [ ] `test-keyring.ps1` script runs
- [ ] All phases complete
- [ ] Test summary shows results
- [ ] Exit code is 0 (success)

**Commands:**
```powershell
.\test-keyring.ps1
```

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

## Phase 16: Cleanup

- [ ] Test credentials deleted
- [ ] No plain text files remain
- [ ] Config directory clean
- [ ] OS keyring entries removed (if desired)

**Commands:**
```powershell
.\target\release\plur-creds.exe delete nostr --force
.\target\release\plur-creds.exe delete mastodon --force
.\target\release\plur-creds.exe delete bluesky --force

# Verify cleanup
.\target\release\plur-creds.exe list
dir $env:USERPROFILE\.config\plurcast\*.keys
```

## Final Verification

### Security Checklist
- [ ] No credentials in plain text files
- [ ] All credentials in OS keyring or encrypted files
- [ ] File permissions correct (600 on Unix)
- [ ] Security audit passes
- [ ] No credential values in logs

### Functionality Checklist
- [ ] Store credentials works
- [ ] List credentials works
- [ ] Test credentials works
- [ ] Delete credentials works
- [ ] Migration works
- [ ] Integration with plur-post works

### Performance Checklist
- [ ] All operations complete within target times
- [ ] No noticeable delays
- [ ] Cross-process access works smoothly

### Documentation Checklist
- [ ] All issues documented
- [ ] Platform-specific quirks noted
- [ ] Performance results recorded
- [ ] Suggestions for improvement noted

## Issue Tracking

If any checklist item fails, document:

**Issue:** [Brief description]
**Phase:** [Phase number and name]
**Platform:** [Windows/macOS/Linux]
**Expected:** [What should happen]
**Actual:** [What actually happened]
**Error Message:** [Full error message if any]
**Workaround:** [If found]
**Resolution:** [How it was fixed]

## Sign-Off

**Tester:** ___________________  
**Date:** ___________________  
**Platform:** ___________________  
**Result:** ☐ Pass  ☐ Pass with issues  ☐ Fail  

**Notes:**
```
[Add any additional notes, observations, or recommendations]
```

---

**Version**: 0.2.0-alpha  
**Last Updated**: 2025-10-07  
**Status**: Ready for testing

**Next Steps After Completion:**
1. Review any failed items
2. Document platform-specific issues
3. Update documentation based on findings
4. Report bugs or suggestions
5. Proceed to next development phase
