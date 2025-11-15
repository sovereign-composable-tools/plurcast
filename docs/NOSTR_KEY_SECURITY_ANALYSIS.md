# Nostr Private Key Security Analysis

**Date**: 2025-11-15
**Status**: Proposal
**Context**: Security review of Nostr private key handling in response to question about Nostr extensions

## Executive Summary

**Current Security Level**: ⭐⭐⭐⭐ (4/5) - Good
**Recommended Actions**: Memory protection improvements
**Nostr Extensions**: Not recommended for CLI tools

The current approach using OS keyring + encrypted files is **already secure** for a CLI application. Nostr extensions (NIP-07/NIP-46) are designed for different contexts and would not meaningfully improve security for Plurcast's use case.

---

## Current Security Posture

### ✅ What's Already Secure

1. **Credential Storage** (`libplurcast/src/credentials.rs`)
   - **Primary**: OS keyring (macOS Keychain, Windows Credential Manager, Linux Secret Service)
   - **Fallback**: Age-encrypted files with master password
   - **File permissions**: 600 (owner read/write only)
   - **Symlink protection**: Validates files are not symlinks before reading (`credentials.rs:558`)

2. **No Key Exposure in Logs**
   - ✅ Audit confirms no private key values logged
   - ✅ Only operation descriptions logged ("Stored credential", "Retrieved credential")
   - ✅ Key lengths logged for debugging, not actual values

3. **Key Format Support**
   - Accepts hex (64 chars) and bech32 (nsec) formats
   - Validates format before parsing
   - Proper error messages without exposing keys

### ⚠️ Current Vulnerabilities

1. **Memory Protection** (Medium Risk)
   - **Issue**: Private keys loaded into memory as plain `String`/`Keys` objects
   - **Risk**: Keys could be:
     - Read from memory dumps
     - Swapped to disk
     - Left in memory after use
   - **Location**: `libplurcast/src/platforms/nostr.rs:72-106`

   ```rust
   pub fn load_keys_from_string(&mut self, key_str: &str) -> Result<()> {
       // key_str is a plain String in memory
       let keys = Keys::parse(key_str)?;  // Keys object not zeroed
       self.client = Some(Client::new(keys.clone()));
       self.keys = Some(keys);  // Stored in struct without protection
       Ok(())
   }
   ```

2. **No Memory Locking** (Low Risk)
   - OS may swap memory pages containing keys to disk
   - No `mlock()` call to prevent swapping
   - Acceptable for most users, critical for high-security scenarios

3. **Key Lifetime** (Low Risk)
   - Keys remain in memory for the duration of the process
   - Could be zeroized after signing events
   - Current approach is simpler but less secure

---

## Nostr Extension Evaluation

### NIP-07 (Browser Extension Signing)

**Description**: Browser extensions (Alby, nos2x, Flamingo) that store keys and sign events without exposing the private key to websites.

**Applicability to Plurcast**: ❌ **Not Suitable**

**Reasoning**:
- Designed for browser/web app context
- Requires browser extension runtime
- Would need complex IPC (Inter-Process Communication) bridge
- CLI tools don't have browser context
- Implementation complexity far outweighs security benefit

**Verdict**: Do not implement

### NIP-46 (Nostr Connect / Remote Signing)

**Description**: Protocol for remote signing where private key lives on a separate device/service ("bunker"), and signing requests are sent over Nostr relays.

**Applicability to Plurcast**: ⚠️ **Optional Feature**

**Pros**:
- Keys never on local machine
- Useful for high-security scenarios
- Keys on air-gapped device possible

**Cons**:
- Requires separate bunker service running
- Network dependency (can't post offline)
- Increased latency (network round-trip)
- More complex setup for users
- Additional failure modes (bunker down, relay issues)

**Verdict**: Consider as **opt-in feature** for advanced users, not default

**Implementation Priority**: Low (after memory protection improvements)

---

## Recommended Security Improvements

### Priority 1: Memory Protection (High Priority)

**Objective**: Zero private key material from memory after use

**Approach**: Use `secrecy` crate for automatic zeroing

**Implementation**:

```rust
// Add to Cargo.toml
secrecy = "0.8"
zeroize = "1.7"

// In libplurcast/src/platforms/nostr.rs
use secrecy::{Secret, ExposeSecret, SecretString};
use zeroize::Zeroize;

pub struct NostrPlatform {
    client: Option<Client>,
    // Before: keys: Option<Keys>,
    keys: Option<Secret<Keys>>,  // Automatically zeroed on drop
    relays: Vec<String>,
    authenticated: bool,
}

pub fn load_keys_from_string(&mut self, key_str: &str) -> Result<()> {
    // Use SecretString for input
    let secret_key_str = SecretString::new(key_str.to_string());

    let keys = Keys::parse(secret_key_str.expose_secret())?;

    // Wrap in Secret for automatic zeroing
    self.client = Some(Client::new(keys.clone()));
    self.keys = Some(Secret::new(keys));

    // key_str will be zeroed when it goes out of scope
    Ok(())
}
```

**Benefits**:
- Private keys automatically zeroed when dropped
- Prevents key exposure in crash dumps
- Prevents keys from being swapped to disk unencrypted
- Minimal API changes (use `.expose_secret()` when needed)

**Effort**: Medium (2-3 days)
**Impact**: High (significantly improves security)

### Priority 2: Explicit Key Zeroing (Medium Priority)

**Objective**: Clear keys from memory as soon as they're no longer needed

**Approach**: Implement Drop trait for cleanup

```rust
impl Drop for NostrPlatform {
    fn drop(&mut self) {
        // Explicitly zero keys on drop
        if let Some(keys) = self.keys.take() {
            // Keys will be automatically zeroed by Secret<T>
            tracing::debug!("Zeroed Nostr private key from memory");
        }
    }
}
```

**Effort**: Low (1 day)
**Impact**: Medium (defense in depth)

### Priority 3: Hardware Key Support (Low Priority)

**Objective**: Support hardware signing devices (YubiKey, Ledger)

**Benefits**:
- Private key never leaves hardware device
- Best security for high-value accounts
- Physical security (need device to sign)

**Challenges**:
- Nostr ecosystem support varies
- Hardware wallet support for Nostr limited
- Additional dependency on device drivers
- More complex user setup

**Recommendation**: Research feasibility after memory protection implemented

**Potential Libraries**:
- `yubikey` crate (for YubiKey)
- `ledger-transport` crate (for Ledger)

**Effort**: High (2-3 weeks)
**Impact**: High for security-conscious users

### Priority 4: Memory Locking (Low Priority)

**Objective**: Prevent OS from swapping key material to disk

**Approach**: Use `mlock()` system call

```rust
#[cfg(unix)]
use libc::{mlock, munlock};

fn lock_memory(data: *const u8, len: usize) -> Result<()> {
    #[cfg(unix)]
    {
        unsafe {
            if mlock(data as *const libc::c_void, len) != 0 {
                return Err(PlatformError::Authentication(
                    "Failed to lock memory".to_string()
                ).into());
            }
        }
    }
    Ok(())
}
```

**Challenges**:
- Requires elevated permissions on some systems
- Platform-specific implementation
- May fail on resource-constrained systems
- Modest security benefit for most users

**Effort**: Medium (3-4 days)
**Impact**: Low (beneficial but not critical)

---

## Comparison: Current vs Proposed

| Security Feature | Current | With Memory Protection | With NIP-46 |
|-----------------|---------|----------------------|-------------|
| Storage at rest | ✅ Encrypted | ✅ Encrypted | ✅ Encrypted |
| Memory protection | ❌ Plain text | ✅ Zeroed on drop | ✅ Not in memory |
| Offline posting | ✅ Yes | ✅ Yes | ❌ No (needs bunker) |
| Setup complexity | ✅ Simple | ✅ Simple | ❌ Complex |
| Network dependency | ✅ Only for posting | ✅ Only for posting | ❌ For signing too |
| Suitable for CLI | ✅ Perfect fit | ✅ Perfect fit | ⚠️ Opt-in only |

---

## Implementation Plan

### Phase 1: Immediate (Week 1-2)
1. Add `secrecy` and `zeroize` crates
2. Implement memory protection for Nostr keys
3. Add integration tests for key zeroing
4. Update documentation

### Phase 2: Short-term (Week 3-4)
1. Extend memory protection to SSB keys
2. Implement explicit Drop handlers
3. Add security audit documentation
4. Create security best practices guide for users

### Phase 3: Medium-term (Month 2-3)
1. Research NIP-46 integration
2. Create design doc for opt-in NIP-46 support
3. Evaluate hardware key support
4. Implement memory locking if needed

### Phase 4: Long-term (Month 4+)
1. Implement NIP-46 support as opt-in feature
2. Add hardware key support if viable
3. Conduct security audit
4. Publish security white paper

---

## User Guidance

### Current Best Practices

**For Users Right Now**:

1. **Use OS Keyring** (automatic on most systems)
   - Credentials stored in OS-native secure storage
   - Keys encrypted at rest
   - Better than file-based storage

2. **Set Master Password** (if keyring unavailable)
   ```bash
   export PLURCAST_MASTER_PASSWORD="your-strong-password"
   ```
   - Encrypts credential files with age encryption
   - Use 12+ character password with mixed case, numbers, symbols

3. **Secure Your Machine**
   - Full disk encryption (FileVault, BitLocker, LUKS)
   - Lock screen when away
   - Keep OS updated
   - Use firewall

4. **Don't Use Public Test Key for Real Posts**
   - Test key is PUBLIC (anyone can post with it)
   - Generate your own key: `cargo run --example generate_nostr_key`

### After Memory Protection (Phase 1)

**Additional Security**:
- Keys automatically zeroed from memory after use
- Reduced risk from memory dumps/core dumps
- More secure against cold boot attacks

### After NIP-46 Support (Phase 3+)

**For High-Security Users**:
```bash
# Use remote bunker for key storage
plur-post "Hello world" --nostr-bunker "bunker://your-bunker-url"

# Keys never on posting machine
# All signing happens on bunker device
```

---

## Conclusion

**Answer to Original Question**:
> "Would enabling Nostr extensions for private key management be more secure?"

**Short Answer**: No, not for a CLI tool. The current OS keyring approach is already secure and more suitable for Plurcast.

**Better Security Improvements**:
1. ✅ **Memory protection** (Priority 1) - Add `secrecy` crate
2. ⚠️ **NIP-46 support** (Priority 3) - Opt-in feature for advanced users
3. ❌ **NIP-07 support** - Not suitable for CLI tools

**Recommended Next Steps**:
1. Implement memory protection with `secrecy` crate (Phase 1)
2. Document current security posture for users
3. Consider NIP-46 as future opt-in feature for high-security scenarios

**Security Rating After Phase 1**: ⭐⭐⭐⭐⭐ (5/5) - Excellent

---

## References

- [NIP-07: Browser Extension Signing](https://github.com/nostr-protocol/nips/blob/master/07.md)
- [NIP-46: Nostr Connect (Remote Signing)](https://github.com/nostr-protocol/nips/blob/master/46.md)
- [Secrecy Crate Documentation](https://docs.rs/secrecy/)
- [Zeroize Crate Documentation](https://docs.rs/zeroize/)
- [OWASP Key Management Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Key_Management_Cheat_Sheet.html)
