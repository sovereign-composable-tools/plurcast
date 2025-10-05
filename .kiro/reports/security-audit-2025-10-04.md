# Plurcast Security Audit Report

**Date**: 2025-10-04  
**Auditor**: Security Review System  
**Project**: Plurcast v0.1.0-alpha  
**Scope**: Complete codebase security review

---

## Executive Summary

This security audit reviewed the Plurcast codebase for vulnerabilities in authentication, data storage, input validation, cryptographic operations, and dependency security. The project demonstrates **good security practices** overall, with proper use of parameterized queries, secure file permissions, and mature libraries. However, several **critical and high-severity issues** require immediate attention before production use.

**Overall Risk Level**: MEDIUM-HIGH (Alpha stage appropriate)

**Critical Issues**: 2  
**High Severity**: 4  
**Medium Severity**: 3  
**Low Severity**: 2  

---

## Critical Severity Issues

### C1: Nostr Private Keys Stored in Plaintext

**Severity**: CRITICAL  
**Location**: `libplurcast/src/platforms/nostr.rs:load_keys()`  
**CWE**: CWE-312 (Cleartext Storage of Sensitive Information)

**Description**:
Nostr private keys are stored in plaintext files (`~/.config/plurcast/nostr.keys`). While file permissions are set to 600, this provides no protection against:
- Memory dumps
- Backup systems that may not preserve permissions
- Malware with user-level access
- Accidental exposure via version control or cloud sync

**Current Code**:
```rust
let content = std::fs::read_to_string(&expanded_path)
    .map_err(|e| PlatformError::Authentication(format!("Failed to read keys file: {}", e)))?;
```

**Risk**:
Complete compromise of user's Nostr identity if key file is accessed. Attacker can impersonate user, post malicious content, and access private messages.

**Recommendation**:
1. **Immediate (Alpha)**: Add clear documentation warning about plaintext storage
2. **Short-term (Beta)**: Implement system keyring integration using `keyring` crate
3. **Long-term (1.0)**: Support hardware security keys (YubiKey, etc.)

**Suggested Implementation**:
```rust
// Add to Cargo.toml
keyring = "2.0"

// In nostr.rs
use keyring::Entry;

pub fn load_keys_from_keyring(&mut self, service: &str, username: &str) -> Result<()> {
    let entry = Entry::new(service, username)
        .map_err(|e| PlatformError::Authentication(format!("Keyring error: {}", e)))?;
    
    let key_str = entry.get_password()
        .map_err(|e| PlatformError::Authentication(format!("Failed to retrieve key: {}", e)))?;
    
    let keys = Keys::parse(&key_str)
        .map_err(|e| PlatformError::Authentication(format!("Invalid key: {}", e)))?;
    
    self.keys = Some(keys);
    Ok(())
}
```

**Status**: OPEN - Requires immediate documentation update

---

### C2: No Rate Limiting on Database Operations

**Severity**: CRITICAL  
**Location**: `libplurcast/src/db.rs` (all methods)  
**CWE**: CWE-770 (Allocation of Resources Without Limits)

**Description**:
Database operations have no rate limiting or resource constraints. A malicious or buggy script could:
- Create unlimited posts, exhausting disk space
- Perform rapid queries, causing CPU/memory exhaustion
- Create denial-of-service conditions

**Risk**:
Local denial of service, data corruption, system instability.

**Recommendation**:
1. Implement connection pool limits (already using SqlitePool, but verify configuration)
2. Add rate limiting for post creation (e.g., max 100 posts/minute)
3. Implement database size limits with warnings
4. Add query timeouts

**Suggested Implementation**:
```rust
// In db.rs
use std::sync::Arc;
use tokio::sync::Semaphore;

pub struct Database {
    pool: SqlitePool,
    rate_limiter: Arc<Semaphore>,
}

impl Database {
    pub async fn new(db_path: &str) -> Result<Self> {
        // ... existing code ...
        
        // Configure pool with limits
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(3))
            .connect(&format!("sqlite:{}", expanded_path))
            .await
            .map_err(crate::error::DbError::SqlxError)?;
        
        Ok(Self { 
            pool,
            rate_limiter: Arc::new(Semaphore::new(100)), // 100 concurrent ops
        })
    }
    
    pub async fn create_post(&self, post: &Post) -> Result<()> {
        let _permit = self.rate_limiter.acquire().await
            .map_err(|_| DbError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other, 
                "Rate limit exceeded"
            )))?;
        
        // ... existing code ...
    }
}
```

**Status**: OPEN - Required for production use

---

## High Severity Issues

### H1: SQL Injection via Platform Names (Theoretical)

**Severity**: HIGH  
**Location**: `libplurcast/src/db.rs:create_post_record()`  
**CWE**: CWE-89 (SQL Injection)

**Description**:
While the code correctly uses parameterized queries with `sqlx::query()` and `.bind()`, the platform name comes from user input via CLI flags. Although currently validated against a whitelist in `post_to_platform()`, there's no database-level constraint.

**Current Protection**:
```rust
// In main.rs
match platform_name {
    "nostr" => post_to_nostr(content, config).await,
    _ => Err(PlurcastError::InvalidInput(format!(
        "Unsupported platform: {}",
        platform_name
    ))),
}
```

**Risk**:
If validation is bypassed or removed in future code changes, arbitrary platform names could be inserted into the database.

**Recommendation**:
1. Add CHECK constraint in database schema
2. Create an enum for platform names at the type level
3. Add database-level validation

**Suggested Fix**:
```sql
-- In migrations/001_initial.sql
ALTER TABLE post_records ADD CONSTRAINT check_platform 
    CHECK (platform IN ('nostr', 'mastodon', 'bluesky'));

-- Or better, use a foreign key
CREATE TABLE IF NOT EXISTS supported_platforms (
    name TEXT PRIMARY KEY
);

INSERT INTO supported_platforms (name) VALUES ('nostr'), ('mastodon'), ('bluesky');

ALTER TABLE post_records ADD CONSTRAINT fk_platform 
    FOREIGN KEY (platform) REFERENCES supported_platforms(name);
```

```rust
// In types.rs
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
pub enum PlatformName {
    Nostr,
    Mastodon,
    Bluesky,
}

impl PlatformName {
    pub fn as_str(&self) -> &str {
        match self {
            PlatformName::Nostr => "nostr",
            PlatformName::Mastodon => "mastodon",
            PlatformName::Bluesky => "bluesky",
        }
    }
}
```

**Status**: OPEN - Defense in depth improvement

---

### H2: Missing Input Validation on Content Length

**Severity**: HIGH  
**Location**: `plur-post/src/main.rs:get_content()`  
**CWE**: CWE-20 (Improper Input Validation)

**Description**:
No maximum length validation on post content. Users can submit arbitrarily large content, potentially:
- Exhausting memory when reading from stdin
- Creating oversized database entries
- Causing relay rejections (Nostr relays may have size limits)
- Enabling DoS attacks

**Current Code**:
```rust
let mut buffer = String::new();
stdin
    .lock()
    .read_to_string(&mut buffer)  // No size limit!
    .map_err(|e| PlurcastError::InvalidInput(format!("Failed to read from stdin: {}", e)))?;
```

**Risk**:
Memory exhaustion, database bloat, relay failures.

**Recommendation**:
Implement maximum content length validation:

```rust
const MAX_CONTENT_LENGTH: usize = 100_000; // 100KB reasonable limit

fn get_content(cli: &Cli) -> Result<String> {
    if let Some(content) = &cli.content {
        if content.len() > MAX_CONTENT_LENGTH {
            return Err(PlurcastError::InvalidInput(format!(
                "Content too large: {} bytes (max: {})",
                content.len(),
                MAX_CONTENT_LENGTH
            )));
        }
        // ... rest of validation
    }

    // For stdin, use take() to limit reading
    let mut buffer = String::new();
    stdin
        .lock()
        .take(MAX_CONTENT_LENGTH as u64)
        .read_to_string(&mut buffer)
        .map_err(|e| PlurcastError::InvalidInput(format!("Failed to read from stdin: {}", e)))?;
    
    if buffer.len() >= MAX_CONTENT_LENGTH {
        return Err(PlurcastError::InvalidInput(format!(
            "Content exceeds maximum length of {} bytes",
            MAX_CONTENT_LENGTH
        )));
    }
    
    // ... rest of validation
}
```

**Status**: OPEN - Required before beta release

---

### H3: Insufficient Error Message Sanitization

**Severity**: HIGH  
**Location**: Multiple locations (error.rs, main.rs, nostr.rs)  
**CWE**: CWE-209 (Information Exposure Through Error Message)

**Description**:
Error messages may leak sensitive information:
- File paths (revealing username, directory structure)
- Relay URLs (network topology)
- Key format details (aiding brute force)
- Database schema details

**Examples**:
```rust
// In nostr.rs
PlatformError::Authentication(format!("Failed to read keys file: {}", e))
// Could expose: "Failed to read keys file: /home/alice/.config/plurcast/nostr.keys: Permission denied"

// In config.rs
ConfigError::ReadError(std::io::Error::new(
    e.kind(),
    format!("Failed to read config from {}: {}", path.display(), e)
))
// Exposes full file path
```

**Risk**:
Information disclosure aiding targeted attacks, privacy violations.

**Recommendation**:
Implement error sanitization:

```rust
// In error.rs
impl PlurcastError {
    pub fn user_message(&self) -> String {
        match self {
            PlurcastError::Config(ConfigError::ReadError(_)) => {
                "Configuration file not found or not readable. Check ~/.config/plurcast/config.toml".to_string()
            }
            PlurcastError::Platform(PlatformError::Authentication(_)) => {
                "Authentication failed. Check your credentials and try again.".to_string()
            }
            PlurcastError::Database(_) => {
                "Database error occurred. Check permissions and disk space.".to_string()
            }
            PlurcastError::InvalidInput(msg) => msg.clone(), // User input, safe to show
        }
    }
    
    pub fn debug_message(&self) -> String {
        format!("{:?}", self) // Full details for --verbose mode
    }
}

// In main.rs
if let Err(e) = run(cli).await {
    if cli.verbose {
        eprintln!("Error: {}", e.debug_message());
    } else {
        eprintln!("Error: {}", e.user_message());
    }
    std::process::exit(e.exit_code());
}
```

**Status**: OPEN - Implement before stable release

---

### H4: No Integrity Verification for Configuration Files

**Severity**: HIGH  
**Location**: `libplurcast/src/config.rs:load_from_path()`  
**CWE**: CWE-353 (Missing Support for Integrity Check)

**Description**:
Configuration files are loaded without integrity verification. An attacker with file system access could:
- Modify relay URLs to malicious relays (man-in-the-middle)
- Change database paths to exfiltrate data
- Disable security features

**Risk**:
Configuration tampering, data exfiltration, man-in-the-middle attacks.

**Recommendation**:
1. Implement configuration file signing/verification
2. Add checksum validation
3. Warn on configuration changes
4. Consider using OS-level file integrity monitoring

**Suggested Implementation**:
```rust
// Add to Cargo.toml
sha2 = "0.10"

// In config.rs
use sha2::{Sha256, Digest};

impl Config {
    pub fn load_with_verification() -> Result<Self> {
        let config_path = resolve_config_path()?;
        let checksum_path = config_path.with_extension("toml.sha256");
        
        // Load config
        let config = Self::load_from_path(&config_path)?;
        
        // Verify checksum if exists
        if checksum_path.exists() {
            let content = std::fs::read(&config_path)
                .map_err(ConfigError::ReadError)?;
            let mut hasher = Sha256::new();
            hasher.update(&content);
            let computed = format!("{:x}", hasher.finalize());
            
            let stored = std::fs::read_to_string(&checksum_path)
                .map_err(ConfigError::ReadError)?;
            
            if computed != stored.trim() {
                tracing::warn!("Configuration file checksum mismatch! File may have been modified.");
            }
        }
        
        Ok(config)
    }
    
    pub fn save_with_checksum(&self, path: &PathBuf) -> Result<()> {
        let toml_content = toml::to_string_pretty(self)
            .map_err(|e| ConfigError::MissingField(format!("Serialization failed: {}", e)))?;
        
        std::fs::write(path, &toml_content)
            .map_err(ConfigError::ReadError)?;
        
        // Save checksum
        let mut hasher = Sha256::new();
        hasher.update(toml_content.as_bytes());
        let checksum = format!("{:x}", hasher.finalize());
        
        let checksum_path = path.with_extension("toml.sha256");
        std::fs::write(checksum_path, checksum)
            .map_err(ConfigError::ReadError)?;
        
        Ok(())
    }
}
```

**Status**: OPEN - Consider for 1.0 release

---

## Medium Severity Issues

### M1: Weak File Permission Handling on Windows

**Severity**: MEDIUM  
**Location**: `libplurcast/src/config.rs:create_default_config()`  
**CWE**: CWE-732 (Incorrect Permission Assignment)

**Description**:
File permissions are only set on Unix systems. Windows files use default permissions, which may be too permissive.

**Current Code**:
```rust
#[cfg(unix)]
{
    use std::os::unix::fs::PermissionsExt;
    let permissions = std::fs::Permissions::from_mode(0o600);
    std::fs::set_permissions(path, permissions)
        .map_err(ConfigError::ReadError)?;
}
```

**Risk**:
On Windows, configuration and key files may be readable by other users or processes.

**Recommendation**:
Implement Windows ACL restrictions:

```rust
// Add to Cargo.toml
[target.'cfg(windows)'.dependencies]
windows-acl = "0.3"

// In config.rs
#[cfg(windows)]
{
    use windows_acl::acl::{AceType, ACL};
    use windows_acl::helper;
    
    // Remove all permissions except for current user
    let path_str = path.to_str().ok_or_else(|| 
        ConfigError::MissingField("Invalid path".to_string()))?;
    
    helper::set_file_permissions(
        path_str,
        &[AceType::AccessAllow],
        false
    ).map_err(|e| ConfigError::ReadError(std::io::Error::new(
        std::io::ErrorKind::PermissionDenied,
        format!("Failed to set Windows ACL: {}", e)
    )))?;
}
```

**Status**: OPEN - Required for Windows production use

---

### M2: No Timeout on Network Operations

**Severity**: MEDIUM  
**Location**: `libplurcast/src/platforms/nostr.rs:authenticate()`, `post()`  
**CWE**: CWE-400 (Uncontrolled Resource Consumption)

**Description**:
Network operations (relay connections, posting) have no explicit timeouts. Slow or unresponsive relays can cause indefinite hangs.

**Risk**:
Application hangs, poor user experience, potential DoS.

**Recommendation**:
Add timeouts to all network operations:

```rust
use tokio::time::{timeout, Duration};

impl NostrPlatform {
    async fn authenticate(&mut self) -> Result<()> {
        // ... existing code ...
        
        // Add timeout to connection
        timeout(Duration::from_secs(30), self.client.connect())
            .await
            .map_err(|_| PlatformError::Network("Connection timeout after 30s".to_string()))?;
        
        self.authenticated = true;
        Ok(())
    }
    
    async fn post(&self, content: &str) -> Result<String> {
        // ... existing code ...
        
        // Add timeout to posting
        let event_id = timeout(
            Duration::from_secs(10),
            self.client.publish_text_note(content, [])
        )
        .await
        .map_err(|_| PlatformError::Network("Post timeout after 10s".to_string()))?
        .map_err(|e| PlatformError::Posting(format!("Failed to publish: {}", e)))?;
        
        Ok(event_id.id().to_bech32().unwrap_or_else(|_| event_id.id().to_hex()))
    }
}
```

**Status**: OPEN - Recommended for beta release

---

### M3: Potential Path Traversal in Configuration Paths

**Severity**: MEDIUM  
**Location**: `libplurcast/src/config.rs:resolve_db_path()`, `resolve_config_path()`  
**CWE**: CWE-22 (Path Traversal)

**Description**:
Environment variables `PLURCAST_CONFIG` and `PLURCAST_DB_PATH` accept arbitrary paths without validation. Malicious values could:
- Write to system directories
- Read sensitive files
- Bypass intended directory structure

**Example Attack**:
```bash
export PLURCAST_DB_PATH="/etc/passwd"
plur-post "test"  # Attempts to write to /etc/passwd
```

**Risk**:
Unauthorized file access, privilege escalation (if run with elevated privileges).

**Recommendation**:
Validate and sanitize paths:

```rust
use std::path::Component;

fn validate_path(path: &PathBuf) -> Result<()> {
    // Check for path traversal attempts
    for component in path.components() {
        match component {
            Component::ParentDir => {
                return Err(ConfigError::MissingField(
                    "Path traversal detected: '..' not allowed".to_string()
                ).into());
            }
            Component::RootDir if cfg!(unix) => {
                // Allow root dir on Unix (absolute paths are OK)
            }
            Component::Prefix(_) if cfg!(windows) => {
                // Allow drive letters on Windows
            }
            _ => {}
        }
    }
    
    // Ensure path is within user's home directory or standard XDG paths
    let home = dirs::home_dir()
        .ok_or_else(|| ConfigError::MissingField("Home directory not found".to_string()))?;
    
    let canonical = path.canonicalize()
        .map_err(|_| ConfigError::MissingField("Invalid path".to_string()))?;
    
    if !canonical.starts_with(&home) {
        tracing::warn!("Path outside home directory: {}", canonical.display());
        // Consider making this an error in production
    }
    
    Ok(())
}

pub fn resolve_db_path(config_path: Option<&str>) -> Result<PathBuf> {
    // ... existing code ...
    
    let path = /* ... path resolution ... */;
    validate_path(&path)?;
    Ok(path)
}
```

**Status**: OPEN - Recommended for stable release

---

## Low Severity Issues

### L1: Missing Dependency Vulnerability Scanning

**Severity**: LOW  
**Location**: CI/CD pipeline (not yet implemented)  
**CWE**: CWE-1104 (Use of Unmaintained Third Party Components)

**Description**:
No automated dependency vulnerability scanning. The project uses multiple dependencies that could have security vulnerabilities.

**Current Dependencies**:
- nostr-sdk 0.35
- sqlx 0.8
- tokio 1.x
- clap 4.5
- (and others)

**Recommendation**:
1. Install and run `cargo-audit` regularly
2. Add to CI/CD pipeline
3. Set up automated dependency updates (Dependabot)

**Implementation**:
```bash
# Install cargo-audit
cargo install cargo-audit

# Run audit
cargo audit

# Add to CI (GitHub Actions example)
# .github/workflows/security.yml
name: Security Audit
on:
  push:
    branches: [main]
  pull_request:
  schedule:
    - cron: '0 0 * * *'  # Daily

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/audit-check@v1
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
```

**Status**: OPEN - Implement before first release

---

### L2: No Logging of Security Events

**Severity**: LOW  
**Location**: Multiple locations  
**CWE**: CWE-778 (Insufficient Logging)

**Description**:
Security-relevant events are not consistently logged:
- Authentication attempts (success/failure)
- Configuration file modifications
- Failed input validation
- Rate limit violations

**Risk**:
Difficult to detect attacks, troubleshoot security issues, or perform forensics.

**Recommendation**:
Add structured security logging:

```rust
// In nostr.rs
pub fn load_keys(&mut self, keys_file: &str) -> Result<()> {
    tracing::info!(
        event = "key_load_attempt",
        keys_file = %keys_file,
        "Attempting to load Nostr keys"
    );
    
    let result = /* ... existing code ... */;
    
    match &result {
        Ok(_) => tracing::info!(
            event = "key_load_success",
            keys_file = %keys_file,
            "Successfully loaded Nostr keys"
        ),
        Err(e) => tracing::warn!(
            event = "key_load_failure",
            keys_file = %keys_file,
            error = %e,
            "Failed to load Nostr keys"
        ),
    }
    
    result
}

// In main.rs
async fn post_to_platform(...) -> PostResult {
    tracing::info!(
        event = "post_attempt",
        platform = %platform_name,
        post_id = %post_id,
        "Attempting to post to platform"
    );
    
    // ... existing code ...
}
```

**Status**: OPEN - Recommended for production use

---

## Dependency Security Analysis

### Current Dependencies Review

**Mature, Well-Maintained Libraries** ✅:
- `nostr-sdk` 0.35 - Active development, good security track record
- `sqlx` 0.8 - Widely used, compile-time query verification prevents SQL injection
- `tokio` 1.x - Industry standard, excellent security record
- `clap` 4.5 - Mature, widely audited
- `serde` 1.0 - Core Rust ecosystem, heavily audited

**Recommendations**:
1. Run `cargo audit` to check for known vulnerabilities
2. Keep dependencies updated (especially security patches)
3. Monitor security advisories for key dependencies
4. Consider using `cargo-deny` for policy enforcement

**Action Items**:
```bash
# Install tools
cargo install cargo-audit cargo-deny

# Create deny.toml configuration
cargo deny init

# Run checks
cargo audit
cargo deny check
```

---

## Positive Security Findings

The following security practices are **correctly implemented**:

✅ **Parameterized SQL Queries**: All database operations use `sqlx::query()` with `.bind()`, preventing SQL injection

✅ **No Unsafe Code**: No `unsafe` blocks found in the codebase

✅ **Type-Safe Error Handling**: Using `thiserror` and `Result<T>` throughout

✅ **File Permissions on Unix**: Config files set to 600 (owner read/write only)

✅ **Input Validation**: Empty content is rejected, basic validation exists

✅ **Mature Libraries**: Using well-maintained, security-focused libraries

✅ **No Hardcoded Credentials**: Credentials stored in separate files, not in code

✅ **Proper .gitignore**: Sensitive files (*.key, *.keys, *.token, *.auth) excluded from version control

✅ **Foreign Key Constraints**: Database enforces referential integrity

✅ **Structured Logging**: Using `tracing` framework for observability

---

## Priority Action Items

### Immediate (Block Alpha Release)

1. **Document plaintext key storage risk** (C1)
   - Add security warning to README
   - Document key file permissions
   - Estimated time: 30 minutes

2. **Add content length validation** (H2)
   - Implement MAX_CONTENT_LENGTH constant
   - Validate in get_content()
   - Estimated time: 1 hour

3. **Install cargo-audit** (L1)
   - Run initial audit
   - Document findings
   - Estimated time: 30 minutes

### Before Beta Release

4. **Implement rate limiting** (C2)
   - Add semaphore-based rate limiting
   - Configure connection pool limits
   - Estimated time: 3 hours

5. **Add network timeouts** (M2)
   - Timeout on relay connections
   - Timeout on posting operations
   - Estimated time: 2 hours

6. **Sanitize error messages** (H3)
   - Implement user_message() method
   - Update error display logic
   - Estimated time: 2 hours

### Before Stable Release (1.0)

7. **Implement keyring integration** (C1)
   - Add keyring crate dependency
   - Implement secure key storage
   - Maintain backward compatibility
   - Estimated time: 8 hours

8. **Add database constraints** (H1)
   - Create platform enum
   - Add CHECK constraints
   - Migration for existing data
   - Estimated time: 3 hours

9. **Windows ACL support** (M1)
   - Add windows-acl dependency
   - Implement permission setting
   - Test on Windows
   - Estimated time: 4 hours

10. **Path validation** (M3)
    - Implement validate_path()
    - Add to all path resolution functions
    - Estimated time: 2 hours

11. **Security event logging** (L2)
    - Add structured logging for security events
    - Document log format
    - Estimated time: 3 hours

12. **Configuration integrity** (H4)
    - Implement checksum verification
    - Add signing support
    - Estimated time: 4 hours

---

## Testing Recommendations

### Security Test Coverage Needed

1. **Authentication Tests**
   - Test with invalid key formats
   - Test with missing key files
   - Test with corrupted keys
   - Test permission denied scenarios

2. **Input Validation Tests**
   - Test with oversized content
   - Test with special characters
   - Test with null bytes
   - Test with Unicode edge cases

3. **Path Traversal Tests**
   - Test with ../ in environment variables
   - Test with absolute paths
   - Test with symlinks

4. **Rate Limiting Tests**
   - Test rapid post creation
   - Test concurrent operations
   - Test resource exhaustion

5. **Error Handling Tests**
   - Verify no sensitive data in error messages
   - Test error message sanitization
   - Test verbose vs normal mode

**Suggested Test Framework**:
```rust
#[cfg(test)]
mod security_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_oversized_content_rejected() {
        let huge_content = "x".repeat(1_000_000);
        let result = validate_content_length(&huge_content);
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_path_traversal_blocked() {
        std::env::set_var("PLURCAST_DB_PATH", "../../../etc/passwd");
        let result = resolve_db_path(None);
        assert!(result.is_err() || !result.unwrap().to_str().unwrap().contains("etc/passwd"));
    }
    
    #[test]
    fn test_error_messages_no_sensitive_data() {
        let error = PlurcastError::Config(ConfigError::ReadError(
            std::io::Error::new(std::io::ErrorKind::NotFound, "/home/alice/.config/plurcast/config.toml")
        ));
        
        let user_msg = error.user_message();
        assert!(!user_msg.contains("/home/alice"));
        assert!(!user_msg.contains("alice"));
    }
}
```

---

## Compliance Considerations

### OWASP Top 10 2021 Mapping

- **A01:2021 – Broken Access Control**: M3 (Path Traversal)
- **A02:2021 – Cryptographic Failures**: C1 (Plaintext Keys)
- **A03:2021 – Injection**: H1 (SQL Injection - mitigated)
- **A04:2021 – Insecure Design**: C2 (No Rate Limiting)
- **A05:2021 – Security Misconfiguration**: M1 (Windows Permissions)
- **A06:2021 – Vulnerable Components**: L1 (No Dependency Scanning)
- **A09:2021 – Security Logging Failures**: L2 (Insufficient Logging)

### CWE Coverage

- CWE-89: SQL Injection (MITIGATED - using parameterized queries)
- CWE-312: Cleartext Storage (C1)
- CWE-770: Resource Allocation (C2)
- CWE-20: Input Validation (H2)
- CWE-209: Information Exposure (H3)
- CWE-353: Integrity Check (H4)
- CWE-732: Permissions (M1)
- CWE-400: Resource Consumption (M2)
- CWE-22: Path Traversal (M3)

---

## Conclusion

Plurcast demonstrates **solid security fundamentals** for an alpha-stage project. The use of parameterized queries, type-safe error handling, and mature libraries provides a strong foundation. However, several critical issues must be addressed before production use:

**Must Fix Before Production**:
1. Implement secure key storage (keyring integration)
2. Add rate limiting and resource constraints
3. Validate and limit input sizes
4. Sanitize error messages
5. Add network timeouts

**Recommended Security Roadmap**:
- **Alpha**: Document risks, add basic validation
- **Beta**: Implement rate limiting, timeouts, error sanitization
- **1.0**: Keyring integration, full Windows support, comprehensive security logging

The project's Unix philosophy and local-first architecture inherently reduce attack surface compared to cloud-based alternatives. With the recommended fixes, Plurcast can achieve a strong security posture appropriate for handling sensitive cryptographic keys and user data.

---

**Next Review**: After implementing P0 security fixes  
**Reviewer**: Security team  
**Status**: ACTIVE - Tracking in .kiro/reports/security-audit-2025-10-04.md

