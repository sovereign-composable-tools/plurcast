# Private Key Security Audit Report

**Date**: 2025-11-15
**Auditor**: Claude (Automated Security Audit)
**Scope**: Private key handling and exposure risks in Plurcast codebase

---

## Executive Summary

**Overall Security Rating**: ⭐⭐⭐⭐ (4/5) - Good

**Key Findings**:
- ✅ No private keys logged to console or files
- ✅ Secure storage backends implemented (OS keyring, encrypted files)
- ✅ Symlink attack protection in place
- ⚠️ Private keys stored in memory as plain text (not zeroed)
- ⚠️ Keys displayed intentionally during setup (acceptable for UX)

**Recommended Actions**:
1. Add memory protection with `secrecy` crate
2. Document when keys are intentionally displayed
3. Add memory zeroing on Drop

---

## Audit Methodology

### Search Patterns Used

1. **Logging Patterns**:
   ```
   (tracing::|log::).*(key|secret|private)
   (println!|eprintln!|debug!|info!|warn!|error!).*(key|secret|password|credential)
   ```

2. **Key Access Patterns**:
   ```
   to_secret_hex|secret_key|private_key
   ```

3. **Debug Value Patterns**:
   ```
   debug!\(.*(retrieved|value|content|data).*\{
   ```

### Files Audited

- `libplurcast/src/credentials.rs` - Credential management
- `libplurcast/src/platforms/nostr.rs` - Nostr key handling
- `libplurcast/src/platforms/ssb/platform.rs` - SSB key handling
- `plur-setup/src/main.rs` - Setup wizard (user key entry)
- `plur-creds/src/main.rs` - Credential CLI tool
- `libplurcast/examples/generate_nostr_key.rs` - Key generation example

---

## Findings

### 1. Intentional Key Display (Acceptable)

**Location**: Setup and key generation tools

**Files**:
- `plur-setup/src/main.rs:244-248` - Display private key during setup
- `plur-setup/src/main.rs:476` - Display SSB private key during setup
- `libplurcast/examples/generate_nostr_key.rs:20-22` - Key generation example

**Code Example**:
```rust
// plur-setup/src/main.rs:244
println!("Private Key (keep this secret!):");
println!("  Hex:    {}", private_hex);
println!("  Bech32: {}", private_bech32);

println!("⚠️  IMPORTANT: Save your private key securely!");
println!("   - This key will be stored in your credential storage");
println!("   - If you lose it, you lose access to your Nostr identity");
println!("   - Never share your private key with anyone\n");
```

**Assessment**: ✅ **Acceptable**

**Rationale**:
- Users need to see keys during initial setup to save them
- Clear warnings displayed about keeping keys secret
- Only happens during interactive setup (not in production code)
- Terminal output is ephemeral (not logged to files)

**Recommendation**: Add to documentation that setup tools display keys temporarily

---

### 2. Logging Patterns (Secure)

**Finding**: No private key values logged

**Evidence**:

✅ **Credential operations log only metadata**:
```rust
// libplurcast/src/credentials.rs:430
tracing::debug!(
    "Stored credential for {}.{}.{} in OS keyring",
    service,
    account,
    key
);
// Note: Does NOT log the credential value
```

✅ **Platform operations log only context**:
```rust
// libplurcast/src/poster.rs:440
tracing::debug!(
    "Retrieved Nostr credentials from secure storage for account '{}'",
    account_to_use
);
// Note: Does NOT log the actual key
```

✅ **SSB operations log only feed IDs**:
```rust
// libplurcast/src/platforms/ssb/platform.rs:398
tracing::debug!("Signing message with keypair for feed: {}", keypair.id);
// Note: keypair.id is public, not secret
```

**Assessment**: ✅ **Secure** - No secret material leaked in logs

---

### 3. Memory Storage (Needs Improvement)

**Finding**: Private keys stored in memory as plain text

**Evidence**:

**Nostr Platform** (`libplurcast/src/platforms/nostr.rs`):
```rust
pub struct NostrPlatform {
    client: Option<Client>,
    keys: Option<Keys>,  // ⚠️ Plain Keys object in memory
    relays: Vec<String>,
    authenticated: bool,
}

pub fn load_keys_from_string(&mut self, key_str: &str) -> Result<()> {
    let key_str = key_str.trim();  // ⚠️ Plain String in memory

    let keys = if key_str.len() == 64 {
        Keys::parse(key_str)?  // ⚠️ Not zeroed after parse
    } else if key_str.starts_with("nsec") {
        Keys::parse(key_str)?
    } else {
        // ...
    };

    self.client = Some(Client::new(keys.clone()));
    self.keys = Some(keys);  // ⚠️ Stored without protection
    Ok(())
}
```

**SSB Platform** (`libplurcast/src/platforms/ssb/platform.rs`):
```rust
pub struct SsbPlatform {
    keypair: Option<Keypair>,  // ⚠️ Plain Keypair in memory
    // ...
}
```

**Risk Assessment**: ⚠️ **Medium Risk**

**Risks**:
1. **Memory dumps**: Keys visible in crash dumps or core dumps
2. **Swap to disk**: OS may swap memory pages containing keys to disk
3. **Memory scanning**: Malware could scan process memory for keys
4. **Cold boot attacks**: Keys may remain in RAM after power cycle

**Impact**:
- **Low** for typical users (requires local access or malware)
- **Medium** for shared systems or compromised machines
- **High** for targeted attacks on high-value accounts

**Recommendation**: ✅ **Implement memory protection (Priority 1)**

---

### 4. Credential Storage (Secure)

**Finding**: Secure storage backends properly implemented

**Evidence**:

✅ **OS Keyring** (`libplurcast/src/credentials.rs:369-518`):
```rust
pub struct KeyringStore;

impl CredentialStore for KeyringStore {
    fn store_account(&self, service: &str, key: &str, account: &str, value: &str) -> Result<()> {
        let entry = keyring::Entry::new(&keyring_service, &keyring_key)?;
        entry.set_password(value)?;  // Stored in OS-native secure storage
        Ok(())
    }
}
```

✅ **Encrypted Files** (`libplurcast/src/credentials.rs:627-860`):
```rust
pub struct EncryptedFileStore {
    base_path: PathBuf,
    master_password: Arc<RwLock<Option<String>>>,
}

fn encrypt(&self, data: &str) -> Result<Vec<u8>> {
    let encryptor = age::Encryptor::with_user_passphrase(
        age::secrecy::Secret::new(password.clone())
    );
    // Uses age encryption (strong, modern encryption)
}
```

✅ **Symlink Protection** (`libplurcast/src/credentials.rs:558-587`):
```rust
pub fn validate_not_symlink(path: &Path) -> Result<()> {
    let metadata = std::fs::symlink_metadata(path)?;

    if metadata.is_symlink() {
        return Err(CredentialError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Security: Credential file '{}' is a symbolic link...", path.display())
        )).into());
    }

    Ok(())
}
```

**Assessment**: ✅ **Secure** - Storage at rest is properly protected

---

### 5. Key Format Validation (Secure)

**Finding**: Keys validated before use, errors don't expose keys

**Evidence**:

✅ **Format validation**:
```rust
pub fn load_keys_from_string(&mut self, key_str: &str) -> Result<()> {
    let key_str = key_str.trim();

    let keys = if key_str.len() == 64 {
        // Hex format
        Keys::parse(key_str).map_err(|e| {
            PlatformError::Authentication(format!(
                "Nostr authentication failed (load keys): Invalid hex key format: {}. \
                Suggestion: Ensure the key is a valid 64-character hexadecimal string.",
                e
            ))
        })?
    } else if key_str.starts_with("nsec") {
        // Bech32 format
        Keys::parse(key_str).map_err(|e| {
            PlatformError::Authentication(format!(
                "Nostr authentication failed (load keys): Invalid bech32 key format: {}. \
                Suggestion: Ensure the key is a valid nsec-prefixed bech32 string.",
                e
            ))
        })?
    } else {
        return Err(PlatformError::Authentication(
            "Nostr authentication failed (load keys): Key must be 64-character hex or bech32 nsec format. \
            Suggestion: Generate a new key or ensure your existing key is in the correct format.".to_string(),
        ).into());
    };

    // Note: Error messages do NOT include the actual key string
}
```

**Assessment**: ✅ **Secure** - Errors provide guidance without exposing keys

---

### 6. Shared Test Key (Acceptable)

**Finding**: Public test key hardcoded for demos

**Location**: `libplurcast/src/platforms/nostr.rs:22`

```rust
/// Shared test account private key (publicly known, for testing/demos only)
///
/// This is a well-known test key that anyone can use. It's intentionally public
/// and serves as:
/// - A quick way to test Plurcast without setting up credentials
/// - A community bulletin board for Plurcast users
/// - A demo account for documentation and tutorials
///
/// Public key (npub): npub1qyv34w2prnz66zxrgqsmy2emrg0uqtrnvarhrrfaktxk9vp2dgllsajv05m
/// Handle: satoshi@nakamoto.btc
///
/// ⚠️ WARNING: Never use this for real posts! Anyone can post to this account.
pub const SHARED_TEST_KEY: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
```

**Assessment**: ✅ **Acceptable** - Intentionally public for testing

**Rationale**:
- Clearly documented as public test key
- Warnings in code comments
- Useful for quick testing and demos
- No expectation of privacy

---

## Summary of Security Controls

| Control | Status | Notes |
|---------|--------|-------|
| **Credential Storage** | ✅ Secure | OS keyring + encrypted files |
| **Symlink Protection** | ✅ Implemented | Prevents symlink attacks |
| **Logging Security** | ✅ Secure | No keys in logs, only metadata |
| **Error Messages** | ✅ Secure | No key values in errors |
| **Format Validation** | ✅ Implemented | Validates before parsing |
| **Memory Protection** | ⚠️ Needs Work | Keys not zeroed from memory |
| **Key Display (Setup)** | ✅ Acceptable | Intentional UX, clearly warned |
| **Shared Test Key** | ✅ Acceptable | Intentionally public, documented |

---

## Vulnerabilities and Recommendations

### Vulnerability 1: Memory Exposure

**Severity**: Medium
**Likelihood**: Low (requires local access or malware)
**Impact**: High (full key compromise)

**Description**: Private keys stored in process memory as plain text, not zeroed after use

**Attack Vectors**:
1. Memory dump attacks (crash dump, debugger)
2. Swap file reading (keys swapped to disk)
3. Memory scanning malware
4. Cold boot attacks

**Recommendation**: Implement memory protection

**Remediation**: See `NOSTR_KEY_SECURITY_ANALYSIS.md` Priority 1

**Effort**: Medium (2-3 days)
**Impact**: High (significantly improves security)

### Vulnerability 2: No Memory Locking

**Severity**: Low
**Likelihood**: Low
**Impact**: Medium

**Description**: OS may swap memory pages containing keys to disk

**Recommendation**: Implement memory locking with `mlock()`

**Remediation**: See `NOSTR_KEY_SECURITY_ANALYSIS.md` Priority 4

**Effort**: Medium (3-4 days)
**Impact**: Low (beneficial but not critical)

---

## Compliance Assessment

### Industry Best Practices

| Practice | Compliance | Notes |
|----------|------------|-------|
| **OWASP Key Management** | ⚠️ Partial | Storage secure, memory needs work |
| **CWE-312** (Cleartext Storage) | ✅ Pass | Keys encrypted at rest |
| **CWE-313** (Cleartext Transmission) | ✅ Pass | Keys only in memory, not transmitted |
| **CWE-359** (Privacy Leak via Logs) | ✅ Pass | No keys in logs |
| **CWE-200** (Information Exposure) | ✅ Pass | Error messages safe |

### Nostr Security Standards

| Standard | Compliance | Notes |
|----------|------------|-------|
| **NIP-01** (Event Signing) | ✅ Implemented | Uses nostr-sdk correctly |
| **NIP-07** (Browser Extension) | ❌ N/A | Not applicable for CLI tools |
| **NIP-46** (Remote Signing) | ⏳ Future | Recommended for future enhancement |

---

## Test Coverage

### Security Tests Reviewed

✅ **Security Test Suite** (`libplurcast/tests/security.rs`):
- Verifies no credentials in database
- Checks error messages don't leak credentials
- Validates config stores paths, not credentials
- Tests concurrent access safety
- Confirms no hardcoded credentials

✅ **Attack Scenarios** (`plur-post/tests/attack_scenarios.rs`):
- Input validation attacks
- Content size limits
- Malicious input handling

✅ **Backward Compatibility** (`libplurcast/tests/backward_compatibility.rs`):
- Legacy credential file handling
- Migration security

### Missing Test Coverage

⚠️ **Memory Security Tests**:
- No tests for key zeroing
- No tests for memory locking
- No crash dump analysis

**Recommendation**: Add memory security tests when implementing memory protection

---

## Action Items

### Immediate (This Week)
- [x] Document current security posture
- [x] Audit code for key exposure
- [ ] Add memory protection with `secrecy` crate

### Short-term (Next Month)
- [ ] Implement Drop handler for key zeroing
- [ ] Add security tests for memory protection
- [ ] Update user documentation on security

### Long-term (Next Quarter)
- [ ] Evaluate NIP-46 support
- [ ] Research hardware key support
- [ ] Conduct external security audit

---

## Conclusion

**Current State**: The codebase demonstrates good security practices for credential storage and handling. Private keys are never logged, stored securely at rest, and protected against common file-based attacks (symlinks).

**Primary Gap**: Memory protection. Keys remain in process memory as plain text, creating risk from memory dump attacks and swap files.

**Recommendation**: Implement memory protection as Priority 1 improvement. This will elevate security rating from 4/5 to 5/5.

**Overall Assessment**: ✅ **Safe for production use** with recommended improvements planned

---

## Appendix: Key Storage Locations

### Secure Storage
- **OS Keyring**: Platform-native (Keychain/Credential Manager/Secret Service)
- **Encrypted Files**: `~/.config/plurcast/credentials/*.age` (600 permissions)

### Legacy Storage (Backward Compatibility)
- **Nostr**: `~/.config/plurcast/nostr.keys` (deprecated, 600 permissions)
- **Mastodon**: `~/.config/plurcast/mastodon.token` (deprecated, 600 permissions)

### Test/Demo Keys
- **Shared Test Key**: Hardcoded constant (intentionally public)
- **Generated Keys**: Shown in terminal during setup only

---

**Audit Complete**: 2025-11-15
