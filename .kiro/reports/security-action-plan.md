# Security Action Plan

**Created**: 2025-10-04  
**Status**: ACTIVE  
**Priority**: HIGH

This document provides a prioritized, actionable plan to address security findings from the 2025-10-04 security audit.

---

## Phase 1: Immediate Actions (Before Alpha Release)

**Timeline**: 1-2 days  
**Total Effort**: ~2 hours  
**Blocking**: Yes - must complete before alpha release

### 1.1 Document Security Risks (30 minutes)

**Issue**: C1 - Plaintext key storage  
**Action**: Update README.md with security warnings

```markdown
## Security Considerations

⚠️ **IMPORTANT**: Plurcast currently stores Nostr private keys in plaintext files. While file permissions are set to 600 (owner-only), this provides limited protection. 

**Best Practices**:
- Never commit key files to version control
- Use strong file system permissions (chmod 600)
- Store keys on encrypted volumes when possible
- Rotate keys if compromise is suspected
- Future releases will support OS keyring integration

**Key File Security**:
```bash
# Set proper permissions
chmod 600 ~/.config/plurcast/nostr.keys

# Verify permissions
ls -la ~/.config/plurcast/nostr.keys
# Should show: -rw------- (600)

# Store on encrypted volume (recommended)
# macOS: Use FileVault
# Linux: Use LUKS/dm-crypt
# Windows: Use BitLocker
```

### 1.2 Add Content Length Validation (1 hour)

**Issue**: H2 - Missing input validation  
**Action**: Implement MAX_CONTENT_LENGTH in plur-post/src/main.rs

**File**: `plur-post/src/main.rs`

```rust
// Add constant at top of file
const MAX_CONTENT_LENGTH: usize = 100_000; // 100KB

// Update get_content() function
fn get_content(cli: &Cli) -> Result<String> {
    let content = if let Some(content) = &cli.content {
        content.clone()
    } else {
        // Check if stdin is a TTY
        let stdin = io::stdin();
        if stdin.is_terminal() {
            return Err(PlurcastError::InvalidInput(
                "No content provided. Provide content as argument or pipe via stdin".to_string(),
            ));
        }

        // Read from stdin with size limit
        let mut buffer = String::new();
        stdin
            .lock()
            .take(MAX_CONTENT_LENGTH as u64)
            .read_to_string(&mut buffer)
            .map_err(|e| PlurcastError::InvalidInput(format!("Failed to read from stdin: {}", e)))?;
        
        buffer
    };

    // Validate length
    if content.len() > MAX_CONTENT_LENGTH {
        return Err(PlurcastError::InvalidInput(format!(
            "Content too large: {} bytes (maximum: {} bytes)",
            content.len(),
            MAX_CONTENT_LENGTH
        )));
    }

    // Validate not empty
    if content.trim().is_empty() {
        return Err(PlurcastError::InvalidInput(
            "Content cannot be empty".to_string(),
        ));
    }

    Ok(content)
}
```

**Test**:
```bash
# Should succeed
echo "Normal post" | cargo run --bin plur-post

# Should fail with clear error
dd if=/dev/zero bs=1M count=2 | cargo run --bin plur-post
```

### 1.3 Install and Run cargo-audit (30 minutes)

**Issue**: L1 - Missing dependency scanning  
**Action**: Install cargo-audit and document findings

```bash
# Install
cargo install cargo-audit

# Run audit
cargo audit

# Document results
cargo audit > .kiro/reports/dependency-audit-$(date +%Y%m%d).txt

# Add to documentation
echo "Last audit: $(date)" >> .kiro/reports/security-audit-2025-10-04.md
```

**Add to README.md**:
```markdown
## Security

### Dependency Auditing

We regularly audit dependencies for known vulnerabilities:

```bash
# Install cargo-audit
cargo install cargo-audit

# Run audit
cargo audit
```

Last audit: 2025-10-04 - No known vulnerabilities
```

---

## Phase 2: Beta Release Requirements

**Timeline**: 1-2 weeks  
**Total Effort**: ~9 hours  
**Blocking**: Yes - must complete before beta release

### 2.1 Implement Rate Limiting (3 hours)

**Issue**: C2 - No rate limiting  
**Action**: Add connection pool limits and rate limiting

**File**: `libplurcast/Cargo.toml`
```toml
[dependencies]
# Add for rate limiting
governor = "0.6"
```

**File**: `libplurcast/src/db.rs`
```rust
use sqlx::sqlite::SqlitePoolOptions;
use std::time::Duration;

impl Database {
    pub async fn new(db_path: &str) -> Result<Self> {
        let expanded_path = shellexpand::tilde(db_path).to_string();
        let path = Path::new(&expanded_path);

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(crate::error::DbError::IoError)?;
        }

        // Create connection pool with limits
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .min_connections(1)
            .acquire_timeout(Duration::from_secs(3))
            .idle_timeout(Duration::from_secs(600))
            .connect(&format!("sqlite:{}", expanded_path))
            .await
            .map_err(crate::error::DbError::SqlxError)?;

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(crate::error::DbError::MigrationError)?;

        Ok(Self { pool })
    }
}
```

**Test**: Create test that attempts rapid post creation

### 2.2 Add Network Timeouts (2 hours)

**Issue**: M2 - No timeouts  
**Action**: Add timeouts to all network operations

**File**: `libplurcast/src/platforms/nostr.rs`
```rust
use tokio::time::{timeout, Duration};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
const POST_TIMEOUT: Duration = Duration::from_secs(10);

impl Platform for NostrPlatform {
    async fn authenticate(&mut self) -> Result<()> {
        if self.keys.is_none() {
            return Err(PlatformError::Authentication("Keys not loaded".to_string()).into());
        }

        tracing::debug!("Adding {} Nostr relays", self.relays.len());
        for relay in &self.relays {
            tracing::debug!("  Adding relay: {}", relay);
            self.client.add_relay(relay).await
                .map_err(|e| PlatformError::Network(format!("Failed to add relay {}: {}", relay, e)))?;
        }

        tracing::debug!("Connecting to Nostr relays...");
        timeout(CONNECT_TIMEOUT, self.client.connect())
            .await
            .map_err(|_| PlatformError::Network(
                format!("Connection timeout after {}s", CONNECT_TIMEOUT.as_secs())
            ))?;

        self.authenticated = true;
        tracing::debug!("Nostr authentication complete");
        Ok(())
    }

    async fn post(&self, content: &str) -> Result<String> {
        if !self.authenticated {
            return Err(PlatformError::Authentication("Not authenticated".to_string()).into());
        }

        let event_id = timeout(POST_TIMEOUT, self.client.publish_text_note(content, []))
            .await
            .map_err(|_| PlatformError::Network(
                format!("Post timeout after {}s", POST_TIMEOUT.as_secs())
            ))?
            .map_err(|e| PlatformError::Posting(format!("Failed to publish: {}", e)))?;

        Ok(event_id.id().to_bech32().unwrap_or_else(|_| event_id.id().to_hex()))
    }
}
```

### 2.3 Sanitize Error Messages (2 hours)

**Issue**: H3 - Error message leakage  
**Action**: Implement user-friendly error messages

**File**: `libplurcast/src/error.rs`
```rust
impl PlurcastError {
    /// Get user-friendly error message (safe for display)
    pub fn user_message(&self) -> String {
        match self {
            PlurcastError::Config(ConfigError::ReadError(_)) => {
                "Configuration file not found or not readable. \
                 Check ~/.config/plurcast/config.toml exists and has proper permissions.".to_string()
            }
            PlurcastError::Config(ConfigError::ParseError(_)) => {
                "Configuration file has invalid format. \
                 Check TOML syntax in ~/.config/plurcast/config.toml".to_string()
            }
            PlurcastError::Config(ConfigError::MissingField(field)) => {
                format!("Configuration missing required field: {}", field)
            }
            PlurcastError::Platform(PlatformError::Authentication(_)) => {
                "Authentication failed. Check your credentials and try again.".to_string()
            }
            PlurcastError::Platform(PlatformError::Network(_)) => {
                "Network error. Check your internet connection and try again.".to_string()
            }
            PlurcastError::Platform(PlatformError::Posting(_)) => {
                "Failed to post content. Check platform status and try again.".to_string()
            }
            PlurcastError::Platform(PlatformError::Validation(msg)) => {
                format!("Content validation failed: {}", msg)
            }
            PlurcastError::Database(_) => {
                "Database error. Check permissions and disk space.".to_string()
            }
            PlurcastError::InvalidInput(msg) => msg.clone(),
        }
    }

    /// Get detailed error message (for debugging)
    pub fn debug_message(&self) -> String {
        format!("{:?}", self)
    }
}
```

**File**: `plur-post/src/main.rs`
```rust
#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Initialize logging
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_env_filter("debug")
            .with_writer(std::io::stderr)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter("error")
            .with_writer(std::io::stderr)
            .init();
    }

    // Run the main logic and handle errors
    if let Err(e) = run(cli.clone()).await {
        if cli.verbose {
            eprintln!("Error: {}", e.debug_message());
        } else {
            eprintln!("Error: {}", e.user_message());
            eprintln!("\nRun with --verbose for more details");
        }
        std::process::exit(e.exit_code());
    }
}
```

### 2.4 Add Security Event Logging (2 hours)

**Issue**: L2 - Insufficient logging  
**Action**: Add structured security logging

**File**: `libplurcast/src/platforms/nostr.rs`
```rust
pub fn load_keys(&mut self, keys_file: &str) -> Result<()> {
    tracing::info!(
        event = "security.key_load_attempt",
        platform = "nostr",
        "Attempting to load Nostr keys"
    );

    let expanded_path = shellexpand::tilde(keys_file).to_string();
    let result = std::fs::read_to_string(&expanded_path);

    match &result {
        Ok(_) => {
            tracing::info!(
                event = "security.key_load_success",
                platform = "nostr",
                "Successfully loaded Nostr keys"
            );
        }
        Err(e) => {
            tracing::warn!(
                event = "security.key_load_failure",
                platform = "nostr",
                error_kind = ?e.kind(),
                "Failed to load Nostr keys"
            );
        }
    }

    // ... rest of function
}
```

---

## Phase 3: Stable Release (1.0)

**Timeline**: 2-3 months  
**Total Effort**: ~21 hours  
**Blocking**: Yes - must complete before 1.0 release

### 3.1 Keyring Integration (8 hours)

**Issue**: C1 - Plaintext key storage  
**Action**: Implement OS keyring support

**Dependencies**:
```toml
keyring = "2.0"
```

**Implementation**: See detailed code in security-audit-2025-10-04.md section C1

### 3.2 Database Constraints (3 hours)

**Issue**: H1 - SQL injection (defense in depth)  
**Action**: Add platform enum and constraints

**Implementation**: See detailed code in security-audit-2025-10-04.md section H1

### 3.3 Windows ACL Support (4 hours)

**Issue**: M1 - Windows permissions  
**Action**: Implement Windows ACL restrictions

**Implementation**: See detailed code in security-audit-2025-10-04.md section M1

### 3.4 Path Validation (2 hours)

**Issue**: M3 - Path traversal  
**Action**: Implement path validation

**Implementation**: See detailed code in security-audit-2025-10-04.md section M3

### 3.5 Configuration Integrity (4 hours)

**Issue**: H4 - No integrity checks  
**Action**: Implement checksum verification

**Implementation**: See detailed code in security-audit-2025-10-04.md section H4

---

## Testing Requirements

### Phase 1 Tests
```bash
# Test content length validation
cargo test test_content_length_validation

# Test with oversized input
dd if=/dev/zero bs=1M count=2 | cargo run --bin plur-post 2>&1 | grep "too large"
```

### Phase 2 Tests
```bash
# Test rate limiting
cargo test test_rate_limiting

# Test network timeouts
cargo test test_network_timeouts

# Test error message sanitization
cargo test test_error_messages_no_sensitive_data
```

### Phase 3 Tests
```bash
# Test keyring integration
cargo test test_keyring_storage

# Test path validation
cargo test test_path_traversal_blocked

# Test Windows ACLs (on Windows)
cargo test test_windows_permissions
```

---

## Progress Tracking

Update this section as work progresses:

- [ ] Phase 1.1: Security documentation
- [ ] Phase 1.2: Content length validation
- [ ] Phase 1.3: cargo-audit setup
- [ ] Phase 2.1: Rate limiting
- [ ] Phase 2.2: Network timeouts
- [ ] Phase 2.3: Error sanitization
- [ ] Phase 2.4: Security logging
- [ ] Phase 3.1: Keyring integration
- [ ] Phase 3.2: Database constraints
- [ ] Phase 3.3: Windows ACL support
- [ ] Phase 3.4: Path validation
- [ ] Phase 3.5: Configuration integrity

---

## Success Criteria

### Alpha Release
- ✅ All Phase 1 items complete
- ✅ Security warnings documented
- ✅ No known CRITICAL vulnerabilities in dependencies
- ✅ Basic input validation working

### Beta Release
- ✅ All Phase 1 and 2 items complete
- ✅ Rate limiting implemented
- ✅ Network timeouts configured
- ✅ Error messages sanitized
- ✅ Security events logged

### Stable (1.0) Release
- ✅ All phases complete
- ✅ Keyring integration working
- ✅ All CRITICAL and HIGH issues resolved
- ✅ Security tests passing
- ✅ Security documentation complete
- ✅ External security review completed

---

**Next Review**: After Phase 1 completion  
**Owner**: Development Team  
**Status**: ACTIVE

