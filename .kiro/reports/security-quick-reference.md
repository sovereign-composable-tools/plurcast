# Security Quick Reference Card

**Last Updated**: 2025-10-04  
**For**: Plurcast Developers

---

## 🚨 Critical Issues (Fix Immediately)

| ID | Issue | Quick Fix | Time |
|----|-------|-----------|------|
| C1 | Plaintext keys | Document risk in README | 30m |
| C2 | No rate limiting | Add SqlitePoolOptions limits | 3h |
| H2 | No input validation | Add MAX_CONTENT_LENGTH check | 1h |

---

## 📋 Quick Commands

```bash
# Security audit
cargo audit

# Find unsafe code
rg "unsafe" --type rust

# Check for secrets
rg "password|secret|api_key" --type rust -i

# Run security tests
cargo test security_ -- --nocapture

# Check file permissions
ls -la ~/.config/plurcast/
```

---

## ✅ Security Checklist (Before Commit)

- [ ] No hardcoded credentials
- [ ] All SQL queries use .bind()
- [ ] Input validated for length/type
- [ ] Error messages don't leak paths
- [ ] Network calls have timeouts
- [ ] Sensitive files in .gitignore

---

## 🔒 Secure Coding Patterns

### ✅ DO THIS

```rust
// Parameterized queries
sqlx::query("SELECT * FROM posts WHERE id = ?")
    .bind(user_input)

// Bounded input
stdin.lock().take(MAX_LENGTH).read_to_string(&mut buf)?

// Timeouts
timeout(Duration::from_secs(30), operation).await?

// Safe errors
Err("Authentication failed. Check credentials.")
```

### ❌ NOT THIS

```rust
// String concatenation
format!("SELECT * FROM posts WHERE id = '{}'", input)

// Unbounded input
stdin.read_to_string(&mut buffer)?

// No timeout
operation.await?

// Leaky errors
Err(format!("Failed to read {}", secret_path))
```

---

## 📊 Issue Severity Guide

| Level | Description | Action |
|-------|-------------|--------|
| CRITICAL | Data loss, key compromise | Fix immediately |
| HIGH | Security bypass, DoS | Fix before release |
| MEDIUM | Limited impact | Fix in next version |
| LOW | Best practice | Fix when convenient |

---

## 🎯 Phase 1 Actions (Do Now)

1. Add security warnings to README (30m)
2. Implement content length validation (1h)
3. Run cargo-audit (30m)

**Total**: 2 hours

---

## 📚 Key Documents

- **Full Audit**: [security-audit-2025-10-04.md](security-audit-2025-10-04.md)
- **Action Plan**: [security-action-plan.md](security-action-plan.md)
- **Checklist**: [security-checklist.md](security-checklist.md)
- **Tracker**: [security-issues-tracker.md](security-issues-tracker.md)

---

## 🔧 Quick Fixes

### Add Content Length Validation

```rust
const MAX_CONTENT_LENGTH: usize = 100_000;

fn get_content(cli: &Cli) -> Result<String> {
    let content = /* ... get content ... */;
    
    if content.len() > MAX_CONTENT_LENGTH {
        return Err(PlurcastError::InvalidInput(
            format!("Content too large: {} bytes", content.len())
        ));
    }
    
    Ok(content)
}
```

### Add Network Timeout

```rust
use tokio::time::{timeout, Duration};

timeout(Duration::from_secs(30), client.connect())
    .await
    .map_err(|_| PlatformError::Network("Timeout".to_string()))?
```

### Add Rate Limiting

```rust
let pool = SqlitePoolOptions::new()
    .max_connections(5)
    .acquire_timeout(Duration::from_secs(3))
    .connect(&db_url)
    .await?;
```

---

## 🚀 Release Checklist

### Alpha
- [ ] Phase 1 complete
- [ ] Security warnings documented
- [ ] cargo-audit clean

### Beta
- [ ] Phase 2 complete
- [ ] Rate limiting working
- [ ] Timeouts configured

### 1.0
- [ ] Phase 3 complete
- [ ] All CRITICAL/HIGH fixed
- [ ] Security tests passing

---

## 📞 Emergency Response

If you discover a security issue:

1. **Don't commit** the vulnerable code
2. **Document** in security-issues-tracker.md
3. **Notify** maintainers immediately
4. **Create** private issue (don't disclose publicly)
5. **Fix** following secure coding practices
6. **Test** thoroughly
7. **Release** with security advisory

---

## 💡 Remember

- **Security is not optional** - It's a feature
- **Test security fixes** - Don't assume they work
- **Document limitations** - Be honest with users
- **Keep dependencies updated** - Run cargo-audit weekly
- **Review code** - Use security-checklist.md

---

**Print this card** and keep it visible while coding!

