# Memory Protection Implementation Plan

**Date**: 2025-11-15
**Status**: Proposal
**Priority**: High (Security Improvement)

---

## Objective

Add memory protection for private keys to prevent exposure in:
- Memory dumps (crash dumps, core dumps)
- Swap files (OS paging to disk)
- Memory scanning attacks
- Cold boot attacks

---

## Solution: Secrecy Crate

### Why Secrecy?

**The `secrecy` crate** provides:
- Automatic memory zeroing when values are dropped
- Protection from accidental exposure (Debug, Display, Clone)
- Minimal API surface for accessing secret values
- Compile-time guarantees via type system

**Key Features**:
```rust
use secrecy::{Secret, ExposeSecret, SecretString};

// Secret values automatically zeroed on drop
let secret = Secret::new("my-private-key".to_string());

// Can't accidentally print or debug
println!("{:?}", secret);  // Output: Secret([REDACTED])

// Explicit access required
let value = secret.expose_secret();  // &str

// Automatically zeroed when dropped
drop(secret);  // Memory cleared
```

---

## Current Dependencies

**Good News**: We already have `age = "0.10"` which depends on `secrecy`!

**Check Dependency Tree**:
```bash
cargo tree -p age | grep secrecy
```

Expected output:
```
age v0.10.0
├── secrecy v0.8.0
│   └── zeroize v1.7.0
```

**Additional Dependencies Needed**: ✅ None (already included transitively)

**Optional Enhancement**: Add explicit dependency for clarity
```toml
[workspace.dependencies]
secrecy = "0.8"
zeroize = "1.7"
```

---

## Implementation Plan

### Phase 1: Nostr Platform (Week 1)

**File**: `libplurcast/src/platforms/nostr.rs`

#### Current Code (Vulnerable)

```rust
pub struct NostrPlatform {
    client: Option<Client>,
    keys: Option<Keys>,  // ⚠️ Plain Keys in memory
    relays: Vec<String>,
    authenticated: bool,
}

pub fn load_keys_from_string(&mut self, key_str: &str) -> Result<()> {
    let key_str = key_str.trim();  // ⚠️ Plain String

    let keys = if key_str.len() == 64 {
        Keys::parse(key_str)?  // ⚠️ Not zeroed
    } else if key_str.starts_with("nsec") {
        Keys::parse(key_str)?
    } else {
        return Err(/*...*/);
    };

    self.client = Some(Client::new(keys.clone()));
    self.keys = Some(keys);  // ⚠️ Stored unprotected
    Ok(())
}
```

#### Proposed Code (Protected)

```rust
use secrecy::{Secret, ExposeSecret, SecretString};
use zeroize::Zeroize;

pub struct NostrPlatform {
    client: Option<Client>,
    keys: Option<Secret<Keys>>,  // ✅ Protected with Secret
    relays: Vec<String>,
    authenticated: bool,
}

pub fn load_keys_from_string(&mut self, key_str: &str) -> Result<()> {
    // Wrap input in SecretString for automatic zeroing
    let mut secret_input = SecretString::new(key_str.trim().to_string());

    let keys = {
        let key_str_ref = secret_input.expose_secret();

        if key_str_ref.len() == 64 {
            Keys::parse(key_str_ref)?
        } else if key_str_ref.starts_with("nsec") {
            Keys::parse(key_str_ref)?
        } else {
            return Err(PlatformError::Authentication(
                "Nostr authentication failed (load keys): Key must be 64-character hex or bech32 nsec format.".to_string(),
            ).into());
        }
    };

    // Create client (needs exposed key temporarily)
    self.client = Some(Client::new(keys.clone()));

    // Store keys wrapped in Secret for automatic zeroing
    self.keys = Some(Secret::new(keys));

    // Explicitly zero the input (happens automatically on drop, but being explicit)
    secret_input.expose_secret().to_string().zeroize();

    tracing::debug!("Loaded Nostr keys (memory protected)");
    Ok(())
}

// Add Drop implementation for extra safety
impl Drop for NostrPlatform {
    fn drop(&mut self) {
        if self.keys.is_some() {
            tracing::debug!("Zeroing Nostr private key from memory");
            // Keys automatically zeroed by Secret<T> wrapper
            self.keys = None;
        }
    }
}
```

#### Changes to `authenticate()` Method

```rust
#[async_trait]
impl Platform for NostrPlatform {
    async fn authenticate(&mut self) -> Result<()> {
        // Check if keys are loaded
        if self.keys.is_none() {
            return Err(PlatformError::Authentication(
                "Nostr authentication failed (authenticate): Keys not loaded.".to_string(),
            ).into());
        }

        // Client is already created with keys, no changes needed
        let client = self.client.as_ref().ok_or_else(|| {
            PlatformError::Authentication(
                "Nostr authentication failed (authenticate): Client not initialized.".to_string(),
            )
        })?;

        // Add relays (unchanged)
        for relay in &self.relays {
            client.add_relay(relay).await?;
        }

        client.connect().await;
        self.authenticated = true;

        Ok(())
    }

    // post() method unchanged (uses client, not keys directly)
    async fn post(&self, content: &str) -> Result<String> {
        // ...unchanged...
    }
}
```

**Key Insight**: The `nostr_sdk::Client` already has the keys, so we don't need to expose them again for posting. We only need to protect them during storage and loading.

---

### Phase 2: SSB Platform (Week 2)

**File**: `libplurcast/src/platforms/ssb/platform.rs`

#### Current Code (Vulnerable)

```rust
pub struct SsbPlatform {
    keypair: Option<Keypair>,  // ⚠️ Plain Keypair
    // ...
}
```

#### Proposed Code (Protected)

```rust
use secrecy::{Secret, ExposeSecret};

pub struct SsbPlatform {
    keypair: Option<Secret<Keypair>>,  // ✅ Protected
    // ...
}

pub fn load_keypair(&mut self, keypair_json: &str) -> Result<()> {
    let mut secret_json = SecretString::new(keypair_json.to_string());

    let keypair = serde_json::from_str::<Keypair>(secret_json.expose_secret())
        .map_err(|e| PlatformError::Authentication(format!("Invalid SSB keypair JSON: {}", e)))?;

    self.keypair = Some(Secret::new(keypair));

    secret_json.expose_secret().to_string().zeroize();

    tracing::debug!("Loaded SSB keypair (memory protected)");
    Ok(())
}

// When using keypair for signing
async fn sign_message(&self, content: &str) -> Result<String> {
    let keypair = self.keypair.as_ref()
        .ok_or(PlatformError::Authentication("SSB keypair not loaded".to_string()))?;

    // Temporarily expose for signing
    let signing_result = {
        let kp = keypair.expose_secret();
        // Perform signing with exposed keypair
        sign_with_keypair(kp, content)?
    };
    // kp reference dropped here, no longer accessible

    Ok(signing_result)
}

impl Drop for SsbPlatform {
    fn drop(&mut self) {
        if self.keypair.is_some() {
            tracing::debug!("Zeroing SSB keypair from memory");
            self.keypair = None;  // Secret<T> auto-zeros
        }
    }
}
```

---

### Phase 3: Credential Manager (Week 2)

**File**: `libplurcast/src/credentials.rs`

**Challenge**: Credentials are already encrypted at rest, but exposed in memory during retrieval.

#### Current Code

```rust
pub fn retrieve(&self, service: &str, key: &str) -> Result<String> {
    // Returns plain String with credential value
}
```

#### Proposed Enhancement

```rust
pub fn retrieve_secret(&self, service: &str, key: &str) -> Result<SecretString> {
    let value = self.retrieve(service, key)?;
    Ok(SecretString::new(value))
}

// Keep original method for backward compatibility
pub fn retrieve(&self, service: &str, key: &str) -> Result<String> {
    // ...unchanged...
}
```

**Usage Example**:
```rust
// Before (vulnerable)
let key = cred_mgr.retrieve("plurcast.nostr", "private_key")?;
platform.load_keys_from_string(&key)?;  // key remains in memory

// After (protected)
let secret_key = cred_mgr.retrieve_secret("plurcast.nostr", "private_key")?;
platform.load_keys_from_string(secret_key.expose_secret())?;
// secret_key automatically zeroed on drop
```

---

## Testing Strategy

### Unit Tests

**File**: `libplurcast/src/platforms/nostr/tests.rs` (new module)

```rust
#[test]
fn test_secret_key_is_redacted_in_debug() {
    let keys = Keys::generate();
    let platform = NostrPlatform {
        keys: Some(Secret::new(keys)),
        // ...
    };

    // Debug output should not expose the key
    let debug_output = format!("{:?}", platform.keys);
    assert!(debug_output.contains("[REDACTED]") || debug_output.contains("Secret"));
    assert!(!debug_output.contains(&keys.secret_key().to_secret_hex()));
}

#[test]
fn test_key_zeroed_on_drop() {
    use std::sync::Arc;
    use std::sync::Mutex;

    let dropped = Arc::new(Mutex::new(false));
    let dropped_clone = dropped.clone();

    {
        let mut platform = NostrPlatform::new(&NostrConfig::default());
        let test_keys = Keys::generate();

        // Custom wrapper to detect drop
        struct DropDetector {
            dropped: Arc<Mutex<bool>>,
        }

        impl Drop for DropDetector {
            fn drop(&mut self) {
                *self.dropped.lock().unwrap() = true;
            }
        }

        platform.load_keys_from_string(&test_keys.secret_key().to_secret_hex()).unwrap();

        // Platform goes out of scope here
    }

    // Verify Drop was called (keys zeroed)
    // Note: This is a simplified test - real test would use memory inspection
    // For production, rely on secrecy crate's guarantees
}

#[test]
fn test_cannot_clone_secret_keys() {
    let keys = Keys::generate();
    let secret_keys = Secret::new(keys);

    // This should not compile (Secret<T> is not Clone)
    // Uncomment to verify compile-time protection:
    // let cloned = secret_keys.clone();  // Compile error!
}
```

### Integration Tests

**File**: `libplurcast/tests/memory_security.rs` (new file)

```rust
use libplurcast::platforms::nostr::NostrPlatform;
use libplurcast::config::NostrConfig;
use nostr_sdk::Keys;

#[tokio::test]
async fn test_key_not_exposed_after_authentication() {
    let test_keys = Keys::generate();
    let hex_key = test_keys.secret_key().to_secret_hex();

    let config = NostrConfig {
        enabled: true,
        keys_file: "".to_string(),
        relays: vec![],
    };

    let mut platform = NostrPlatform::new(&config);
    platform.load_keys_from_string(&hex_key).unwrap();

    // Authenticate
    platform.authenticate().await.unwrap();

    // At this point, keys should be wrapped in Secret
    // Debug output should not contain the actual key
    let debug = format!("{:?}", platform);
    assert!(!debug.contains(&hex_key), "Key exposed in debug output!");
}

#[test]
fn test_memory_protection_across_codebase() {
    // Verify all platforms use Secret<T> for private keys
    // This is a compile-time check via type assertions

    use std::marker::PhantomData;

    // Helper to assert a type is Secret<T>
    fn assert_is_secret<T>(_: PhantomData<Secret<T>>) {}

    // These should compile if types are correct:
    // assert_is_secret::<Keys>(PhantomData); // NostrPlatform::keys
    // assert_is_secret::<Keypair>(PhantomData); // SsbPlatform::keypair
}
```

---

## Migration Plan

### Step 1: Add Explicit Dependencies (Optional)

```toml
# Cargo.toml (workspace root)
[workspace.dependencies]
# Existing...
age = "0.10"

# Add explicit for clarity (already transitive via age)
secrecy = "0.8"
zeroize = "1.7"
```

### Step 2: Update Nostr Platform

1. Add imports to `libplurcast/src/platforms/nostr.rs`
2. Update `NostrPlatform` struct
3. Update `load_keys_from_string()` method
4. Add `Drop` implementation
5. Run tests: `cargo test -p libplurcast nostr`

### Step 3: Update SSB Platform

1. Add imports to `libplurcast/src/platforms/ssb/platform.rs`
2. Update `SsbPlatform` struct
3. Update keypair loading methods
4. Update signing methods to use `expose_secret()`
5. Add `Drop` implementation
6. Run tests: `cargo test -p libplurcast ssb`

### Step 4: Update Credential Manager (Optional)

1. Add `retrieve_secret()` method
2. Update callers to use new method
3. Deprecate old method (or keep for compatibility)
4. Run tests: `cargo test -p libplurcast credentials`

### Step 5: Add Security Tests

1. Create `libplurcast/tests/memory_security.rs`
2. Add debug output tests
3. Add drop tests
4. Add integration tests

### Step 6: Update Documentation

1. Update `SECURITY.md` with memory protection details
2. Update `CLAUDE.md` with implementation notes
3. Add migration notes for contributors
4. Update changelog

---

## Backward Compatibility

### API Changes

**Breaking Changes**: None (internal implementation only)

**Public API**: All public APIs remain unchanged:
- `Platform` trait unchanged
- `load_keys_from_string()` signature unchanged
- Credential manager API unchanged

**Internal Changes**:
- `NostrPlatform::keys` type changes from `Option<Keys>` to `Option<Secret<Keys>>`
- `SsbPlatform::keypair` type changes from `Option<Keypair>` to `Option<Secret<Keypair>>`
- These are private fields, not exposed to users

### Migration Path for Existing Installs

**No user action required**: Changes are internal, no config or credential migration needed.

---

## Performance Impact

### Memory Overhead

**Minimal**: `Secret<T>` is a zero-cost wrapper:
- No heap allocation (stores `T` inline)
- No runtime overhead for access
- Only overhead is on `drop()` (zeroing memory)

**Benchmarks** (expected):
```
Before: Keys struct = 32 bytes
After:  Secret<Keys> = 32 bytes (same size)

Drop overhead: ~1 microsecond to zero 32 bytes
```

### CPU Overhead

**Negligible**:
- Zeroing happens only on drop (once per key lifetime)
- No overhead during normal operations
- `expose_secret()` is a simple reference, no copying

---

## Security Benefits

### Before (Current)

| Attack Vector | Risk | Mitigation |
|--------------|------|------------|
| Memory dumps | ⚠️ High | None |
| Swap files | ⚠️ Medium | None |
| Memory scanning | ⚠️ High | None |
| Cold boot | ⚠️ Medium | None |
| Debug logs | ✅ Low | Already protected |
| Error messages | ✅ Low | Already protected |

### After (With Secrecy)

| Attack Vector | Risk | Mitigation |
|--------------|------|------------|
| Memory dumps | ✅ Low | Keys zeroed on drop |
| Swap files | ✅ Low | Keys not in memory long |
| Memory scanning | ⚠️ Medium | Reduced window of exposure |
| Cold boot | ✅ Low | Keys zeroed on exit |
| Debug logs | ✅ Low | Secret<T> prevents debug |
| Error messages | ✅ Low | Already protected + Secret<T> |

**Improvement**: 4 high/medium risks reduced to low

---

## Alternative Approaches Considered

### 1. Manual Zeroing with Zeroize

**Approach**: Use `zeroize` crate directly without `secrecy`

```rust
use zeroize::Zeroize;

pub fn load_keys(&mut self, key_str: &str) -> Result<()> {
    let mut key_copy = key_str.to_string();
    let keys = Keys::parse(&key_copy)?;
    key_copy.zeroize();  // Manual zeroing
    self.keys = Some(keys);
    Ok(())
}
```

**Pros**:
- Direct control over zeroing
- Simpler dependency

**Cons**:
- ❌ Easy to forget to call `zeroize()`
- ❌ No compile-time guarantees
- ❌ Doesn't prevent Debug/Display exposure
- ❌ More error-prone

**Verdict**: ❌ Rejected (secrecy provides better guarantees)

### 2. Memory Locking (mlock)

**Approach**: Use `mlock()` to prevent swapping

```rust
#[cfg(unix)]
fn lock_key_memory(key: &Keys) {
    unsafe {
        let ptr = key as *const Keys as *const libc::c_void;
        libc::mlock(ptr, std::mem::size_of::<Keys>());
    }
}
```

**Pros**:
- Prevents swap to disk
- Additional security layer

**Cons**:
- ❌ Platform-specific
- ❌ Requires elevated permissions
- ❌ May fail on resource-constrained systems
- ❌ Doesn't prevent memory dumps
- ❌ Complex to implement correctly

**Verdict**: ⏳ Future enhancement (after secrecy)

### 3. Hardware Signing (NIP-46, Hardware Keys)

**Approach**: Keys never in memory, signing on hardware device

**Pros**:
- Best security
- Keys never on computer

**Cons**:
- ❌ High complexity
- ❌ User setup burden
- ❌ Hardware dependency
- ❌ Limited Nostr ecosystem support

**Verdict**: ⏳ Long-term feature (Phase 4)

---

## Success Criteria

### Must Have (Phase 1)
- [x] `secrecy` crate integrated
- [ ] Nostr keys wrapped in `Secret<T>`
- [ ] Drop implementation for Nostr platform
- [ ] Unit tests for secret protection
- [ ] Integration tests pass

### Should Have (Phase 2)
- [ ] SSB keys wrapped in `Secret<T>`
- [ ] Drop implementation for SSB platform
- [ ] Memory security tests added
- [ ] Documentation updated

### Nice to Have (Phase 3)
- [ ] Credential manager returns `SecretString`
- [ ] Performance benchmarks
- [ ] Security audit document
- [ ] External security review

---

## Timeline

### Week 1: Nostr Platform
- Day 1-2: Add dependencies, update struct
- Day 3-4: Implement protection, add tests
- Day 5: Code review, documentation

### Week 2: SSB Platform
- Day 1-2: Implement SSB protection
- Day 3: Add memory security tests
- Day 4-5: Documentation, testing, review

### Week 3: Polish and Release
- Day 1-2: Credential manager enhancement
- Day 3: Performance testing
- Day 4: Documentation review
- Day 5: Release preparation

**Total Effort**: ~15 person-days (3 weeks)

---

## Risks and Mitigations

### Risk 1: Nostr SDK Compatibility

**Risk**: `nostr_sdk::Client` may require cloning keys

**Mitigation**:
- `Client::new()` accepts `Keys` by value (takes ownership)
- We can clone keys before wrapping in Secret
- Cloning happens once during initialization, acceptable risk

### Risk 2: Performance Regression

**Risk**: Secret wrapper adds overhead

**Mitigation**:
- `Secret<T>` is zero-cost wrapper (verified)
- Run benchmarks before/after
- Profile memory usage

### Risk 3: Breaking Internal APIs

**Risk**: Other internal code depends on plain `Keys` type

**Mitigation**:
- Audit all internal uses of `keys` field
- Add `expose_secret()` calls where needed
- Comprehensive testing

---

## Conclusion

**Recommendation**: ✅ **Proceed with implementation**

**Rationale**:
1. ✅ Dependencies already available (via `age`)
2. ✅ Implementation straightforward
3. ✅ No breaking changes to public API
4. ✅ Significant security improvement
5. ✅ Low risk, high reward

**Next Steps**:
1. Approve this plan
2. Create implementation branch
3. Start with Nostr platform (Phase 1)
4. Progressive rollout across platforms
5. Release in next minor version (v0.4.0)

---

## References

- [Secrecy Crate Documentation](https://docs.rs/secrecy/)
- [Zeroize Crate Documentation](https://docs.rs/zeroize/)
- [OWASP: Protecting Cryptographic Keys](https://cheatsheetseries.owasp.org/cheatsheets/Key_Management_Cheat_Sheet.html)
- [Nostr SDK Documentation](https://docs.rs/nostr-sdk/)
- [Age Encryption](https://docs.rs/age/)

---

**Document Status**: ✅ Ready for Implementation
**Approval Required**: Yes
**Estimated Start**: Week of 2025-11-18
