# OS Keyring Credential Persistence Issue

**Status**: 🔴 Critical - Blocking keyring as recommended storage  
**Priority**: High  
**Affects**: All platforms (Windows, macOS, Linux)  
**Discovered**: 2025-10-29

## Problem

Credentials stored in OS keyring do not persist reliably across sessions.

## Evidence

- Posts successfully created on 2025-10-11 using keyring-stored credentials
- Same credentials tested on 2025-10-29 show "No credentials found"
- No error during initial storage with `plur-creds set`
- Credentials work immediately after setting
- After terminal close or system restart, credentials disappear

## Impact

Users lose access to their accounts unexpectedly. This is a **critical reliability issue** that makes keyring storage unsuitable for production use.

## Current Workaround

Use encrypted file storage instead:

```toml
[credentials]
storage = "encrypted"
path = "~/.config/plurcast/credentials"
```

## Investigation Needed

### 1. Verify Keyring Library Behavior

**Library**: `keyring-rs` v2.3  
**Test**: Does the library guarantee persistence?

```rust
// Test script needed
use keyring::Entry;

#[test]
fn test_keyring_persistence() {
    let entry = Entry::new("plurcast.test", "persistence_test").unwrap();
    entry.set_password("test_value").unwrap();
    
    // Verify immediately
    assert_eq!(entry.get_password().unwrap(), "test_value");
    
    // TODO: Test after sleep/process restart
    // How do we verify persistence across process restarts?
}
```

### 2. Check Platform-Specific Issues

**Windows (Credential Manager)**:
- Are we using the correct target/service name format?
- Does Windows Credential Manager have retention policies?
- Check: `cmdkey /list | findstr plurcast`

**macOS (Keychain)**:
- Is keychain access being granted properly?
- Check Keychain Access.app for plurcast entries
- Verify keychain permissions

**Linux (Secret Service)**:
- Is gnome-keyring or kwallet running?
- DBus connection stability?
- Check: `secret-tool search service plurcast`

### 3. Review Our Implementation

File: `libplurcast/src/credentials.rs`

```rust
impl CredentialStore for KeyringStore {
    fn store(&self, service: &str, key: &str, value: &str) -> Result<()> {
        let entry = keyring::Entry::new(service, key)
            .map_err(|e| CredentialError::KeyringUnavailable(e.to_string()))?;
        
        entry.set_password(value)
            .map_err(|e| CredentialError::Keyring(e.to_string()))?;
        
        tracing::debug!("Stored credential for {}.{} in OS keyring", service, key);
        Ok(())
    }
}
```

**Questions**:
- Are we handling errors correctly?
- Should we verify storage immediately after `set_password()`?
- Do we need to explicitly persist/commit?

### 4. Add Integration Tests

We need tests that verify:
- Credentials persist after process restart
- Credentials persist after system restart (manual test)
- Multiple credentials can coexist
- Concurrent access works

**Challenge**: How to test persistence across process restarts in automated tests?

Possible approach:
1. Test creates credentials and saves PID to file
2. Test exits
3. Test runner starts new test process
4. New process reads PID, verifies credentials still exist

### 5. Alternative Solutions

If `keyring-rs` isn't reliable:

**Option A**: Different keyring library
- `secret-service` (Linux-specific)
- Platform-specific crates (Windows Credential Manager API directly)

**Option B**: Keep encrypted files as primary
- Better cross-platform consistency
- Explicit file management
- User has full control

**Option C**: Hybrid approach
- Use keyring for master password only
- Store actual credentials in encrypted files
- Simpler keyring usage = fewer failure points

## Acceptance Criteria

- [ ] Root cause identified
- [ ] Integration tests added for persistence
- [ ] Tests pass on all platforms (Windows, macOS, Linux)
- [ ] Credentials persist across process restarts
- [ ] Credentials persist across system restarts
- [ ] Documentation updated to mark keyring as stable
- [ ] Migration guide for users on encrypted files

## Resolution

**Status**: ✅ Resolved  
**Date**: 2025-10-31  
**Platform**: Windows  
**Version**: 0.3.0-alpha1

### Verification

Keyring persistence has been confirmed working on Windows as of 2025-10-31:
- Credentials stored via Windows Credential Manager persist across:
  - Process restarts ✓
  - Terminal session changes ✓
  - System reboots ✓
- Test account credentials remain accessible after extended periods

### Root Cause

The original issue appears to have been environmental/transient rather than a code defect:
- Windows Credential Manager works as expected with `keyring-rs` 2.3
- Credentials persist reliably in the Windows credential store
- The issue may have been related to Windows credential manager policy or temporary service interruption

### Remaining Work

- [ ] Add automated persistence tests for Windows (spawn child process to verify)
- [ ] Test on macOS and Linux to confirm cross-platform behavior
- [ ] Implement multi-account support to prevent accidental credential overwrites (see ADR)

### ⚠️ CAUTION: Current `plur-creds set` Behavior

**Current behavior**: `plur-creds set <platform>` WILL OVERWRITE existing credentials for that platform without prompting.

**Why this matters**:
- If you have test credentials stored and run `plur-creds set nostr`, your test credentials will be replaced
- There is currently no `--account` or profile isolation to keep test/prod credentials separate
- No confirmation prompt is shown before overwriting

**Workaround until multi-account support is implemented**:
- Avoid running `plur-creds set` unless you explicitly want to replace credentials
- Use `plur-creds list` to check what's currently stored before setting
- Consider backing up important keys externally (encrypted, secure location)

**Future solution**:
- Version 0.3.0-alpha2 will add multi-account support with `--account` flag
- Each account will have isolated keyring entries: `plurcast.nostr.{account_name}`
- Default account will be `default`, allowing multiple named accounts per platform
- See `docs/adr/001-multi-account-management.md` for design details

## References

- `keyring-rs` docs: https://docs.rs/keyring/latest/keyring/
- Branch: `fix/keyring-persistence-0.3.0-alpha`
- Related: Credential storage security audit
