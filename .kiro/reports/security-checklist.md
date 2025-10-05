# Security Development Checklist

**Purpose**: Quick reference for developers to ensure security best practices

---

## Before Committing Code

### Input Validation
- [ ] All user input is validated for type, length, and format
- [ ] Maximum content length enforced (100KB limit)
- [ ] Special characters and null bytes handled safely
- [ ] Path inputs validated against traversal attacks

### Authentication & Credentials
- [ ] No credentials hardcoded in source code
- [ ] Sensitive files added to .gitignore
- [ ] File permissions set to 600 for key/token files
- [ ] Keys loaded securely with proper error handling

### Database Operations
- [ ] All queries use parameterized statements (sqlx::query with .bind())
- [ ] No string concatenation in SQL queries
- [ ] Foreign key constraints enforced
- [ ] Transaction rollback on errors

### Error Handling
- [ ] Error messages don't leak sensitive information (paths, usernames, keys)
- [ ] Detailed errors only shown in --verbose mode
- [ ] User-friendly error messages for production
- [ ] All errors properly logged

### Network Operations
- [ ] Timeouts set on all network calls (30s for connections, 10s for operations)
- [ ] Retry logic with exponential backoff
- [ ] TLS/SSL verification enabled
- [ ] Relay URLs validated

### Resource Management
- [ ] Rate limiting implemented where needed
- [ ] Connection pools configured with limits
- [ ] File handles properly closed
- [ ] Memory usage bounded

---

## Before Each Release

### Alpha Release
- [ ] Run `cargo audit` for dependency vulnerabilities
- [ ] Document all known security limitations
- [ ] Add security warnings to README
- [ ] Review .gitignore for sensitive files
- [ ] Test with invalid/malicious inputs

### Beta Release
- [ ] All CRITICAL issues resolved
- [ ] Most HIGH issues resolved
- [ ] Rate limiting implemented
- [ ] Network timeouts added
- [ ] Error message sanitization complete
- [ ] Security event logging added

### Stable (1.0) Release
- [ ] All CRITICAL and HIGH issues resolved
- [ ] Keyring integration implemented
- [ ] Windows ACL support added
- [ ] Path validation complete
- [ ] Configuration integrity checks
- [ ] Comprehensive security tests
- [ ] Security documentation complete
- [ ] Penetration testing performed

---

## Code Review Checklist

### For Reviewers

#### Authentication Code
- [ ] Keys never logged or printed
- [ ] Secure key storage used
- [ ] Authentication failures logged
- [ ] No timing attacks possible

#### Database Code
- [ ] Parameterized queries only
- [ ] Proper error handling
- [ ] Transactions used appropriately
- [ ] Constraints enforced

#### Network Code
- [ ] Timeouts configured
- [ ] Error handling for network failures
- [ ] No sensitive data in URLs
- [ ] TLS/SSL properly configured

#### Configuration Code
- [ ] Path validation present
- [ ] Environment variables sanitized
- [ ] Default values secure
- [ ] File permissions set correctly

---

## Security Testing Commands

```bash
# Dependency audit
cargo audit

# Check for unsafe code
rg "unsafe" --type rust

# Check for TODO/FIXME security items
rg "TODO.*security|FIXME.*security" --type rust -i

# Check for hardcoded secrets (basic)
rg "password|secret|api_key|private_key" --type rust -i

# Run security-focused tests
cargo test security_ -- --nocapture

# Check file permissions (Unix)
find ~/.config/plurcast ~/.local/share/plurcast -type f -exec ls -la {} \;
```

---

## Common Vulnerabilities to Avoid

### ❌ DON'T

```rust
// DON'T: String concatenation in SQL
let query = format!("SELECT * FROM posts WHERE id = '{}'", user_input);

// DON'T: Unbounded input reading
let mut buffer = String::new();
stdin.read_to_string(&mut buffer)?;

// DON'T: Expose sensitive data in errors
Err(format!("Failed to read key from {}", key_path))

// DON'T: No timeout on network operations
client.connect().await?;

// DON'T: Hardcoded credentials
const API_KEY: &str = "sk_live_abc123";
```

### ✅ DO

```rust
// DO: Parameterized queries
sqlx::query("SELECT * FROM posts WHERE id = ?")
    .bind(user_input)
    .fetch_one(&pool).await?;

// DO: Bounded input reading
let mut buffer = String::new();
stdin.lock()
    .take(MAX_LENGTH)
    .read_to_string(&mut buffer)?;

// DO: Sanitized error messages
Err("Failed to read key file. Check path and permissions.")

// DO: Timeout on network operations
timeout(Duration::from_secs(30), client.connect()).await??;

// DO: Load credentials from files
let api_key = std::fs::read_to_string(key_path)?;
```

---

## Security Resources

### Documentation
- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [CWE Top 25](https://cwe.mitre.org/top25/)

### Tools
- `cargo-audit` - Dependency vulnerability scanner
- `cargo-deny` - Dependency policy enforcement
- `cargo-geiger` - Unsafe code detector
- `cargo-tarpaulin` - Code coverage

### Crates
- `secrecy` - Secure secret handling
- `keyring` - OS keyring integration
- `zeroize` - Secure memory clearing
- `ring` - Cryptographic operations

---

## Incident Response

If a security issue is discovered:

1. **Assess severity** using CVSS or internal criteria
2. **Document** in security-issues-tracker.md
3. **Notify** maintainers immediately for CRITICAL/HIGH
4. **Create private issue** (don't disclose publicly yet)
5. **Develop fix** following secure coding practices
6. **Test thoroughly** including security tests
7. **Coordinate disclosure** with security team
8. **Release patch** with security advisory
9. **Update documentation** and audit reports

---

**Last Updated**: 2025-10-04  
**Maintained By**: Security Team  
**Review Frequency**: Before each release

