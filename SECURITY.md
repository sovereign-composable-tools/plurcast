# Security Policy

## Credential Storage Security Model

Plurcast provides multiple options for storing platform credentials securely. This document explains the security properties of each storage backend, the threat model, and best practices.

## Threat Model

### What We Protect Against

1. **Casual File System Access**
   - Credentials are not stored in plain text by default
   - File permissions restrict access to owner only (600)
   - Encrypted storage protects against unauthorized file reads

2. **Credential Theft via File System**
   - OS keyring integration uses system-level encryption
   - Encrypted file storage uses age encryption with user password
   - Plain text files are deprecated and warned against

3. **Accidental Credential Exposure**
   - Credentials never appear in logs
   - Error messages don't include credential values
   - Memory is cleared on exit (best effort)

### What We Don't Protect Against

1. **Root/Administrator Access**
   - Users with root/admin privileges can access any credential storage
   - This is a fundamental limitation of local credential storage

2. **Memory Dumps**
   - Credentials are decrypted in memory during use
   - Memory dumps (core dumps, swap) may contain credentials
   - Use encrypted swap and disable core dumps for maximum security

3. **Malware/Keyloggers**
   - Malware running as your user can access credentials
   - This is outside the scope of application-level security

4. **Physical Access**
   - Physical access to an unlocked system bypasses all protections
   - Use full-disk encryption and screen locks

## Storage Backend Security

### 1. OS Keyring (Recommended)

**Security Properties**:
- **Encryption**: System-level encryption managed by OS
- **Access Control**: OS-enforced access control
- **Integration**: Leverages existing OS security infrastructure
- **Audit**: OS-level audit trails (on some systems)

**Platform-Specific Details**:

#### macOS Keychain
- **Encryption**: AES-256 encryption
- **Protection**: Keychain locked when screen locks
- **Access**: Requires user authentication for first access
- **Location**: System keychain database
- **Audit**: Keychain Access.app shows access logs

#### Windows Credential Manager
- **Encryption**: DPAPI (Data Protection API)
- **Protection**: Tied to user account and machine
- **Access**: Requires user to be logged in
- **Location**: Windows Credential Store
- **Audit**: Event Viewer logs credential access

#### Linux Secret Service
- **Encryption**: Varies by implementation (GNOME Keyring, KWallet)
- **Protection**: Unlocked with user login or separate password
- **Access**: D-Bus access control
- **Location**: Varies by implementation
- **Audit**: Depends on implementation

**Limitations**:
- Not available in all environments (headless servers, containers)
- Requires user session (not suitable for system services)
- Implementation varies by OS and desktop environment

### 2. Encrypted Files

**Security Properties**:
- **Encryption**: age encryption (modern, secure)
- **Key Derivation**: Passphrase-based (scrypt)
- **File Format**: age armor format (.age files)
- **Permissions**: 600 (owner read/write only)

**Technical Details**:
- **Algorithm**: ChaCha20-Poly1305 (authenticated encryption)
- **Key Derivation**: scrypt (work factor: N=2^18, r=8, p=1)
- **File Format**: age v1 format with armor encoding
- **Location**: `~/.config/plurcast/credentials/*.age`

**Password Requirements**:
- Minimum 8 characters (enforced)
- Recommended: 12+ characters with mixed case, numbers, symbols
- Not stored anywhere (must be provided each session)

**Limitations**:
- Requires password entry (interactive or environment variable)
- Password strength depends on user choice
- No built-in password recovery (lost password = lost credentials)

### 3. Plain Text Files (Deprecated)

**Security Properties**:
- **Encryption**: None
- **Protection**: File permissions only (600)
- **Audit**: Deprecation warnings logged

**Why Deprecated**:
- No encryption at rest
- Vulnerable to file system access
- Easy to accidentally expose (backups, version control)
- No protection if file permissions are misconfigured

**When to Use**:
- Testing only
- Legacy compatibility
- Environments where other options unavailable

**Migration Path**:
```bash
# Migrate to secure storage
plur-creds migrate
```

## Best Practices

### For Maximum Security

1. **Use OS Keyring**
   ```toml
   [credentials]
   storage = "keyring"
   ```

2. **Enable Full-Disk Encryption**
   - macOS: FileVault
   - Windows: BitLocker
   - Linux: LUKS

3. **Use Strong Passwords**
   - If using encrypted storage, choose a strong master password
   - Use a password manager to generate and store it

4. **Set Correct File Permissions**
   ```bash
   chmod 600 ~/.config/plurcast/credentials/*
   chmod 600 ~/.config/plurcast/*.keys
   chmod 600 ~/.config/plurcast/*.token
   chmod 600 ~/.config/plurcast/*.auth
   ```

5. **Audit Regularly**
   ```bash
   plur-creds audit
   ```

6. **Migrate from Plain Text**
   ```bash
   plur-creds migrate
   ```

### For Headless/Server Environments

1. **Use Encrypted Storage with Environment Variable**
   ```bash
   export PLURCAST_MASTER_PASSWORD="your_secure_password"
   ```

2. **Restrict Environment Variable Access**
   - Don't log environment variables
   - Use systemd `EnvironmentFile` with 600 permissions
   - Consider using secrets management (Vault, etc.)

3. **Use Dedicated Service Account**
   - Run Plurcast as dedicated user
   - Restrict access to that user only

### For Development/Testing

1. **Use Separate Credentials**
   - Don't use production credentials for testing
   - Create test accounts on each platform

2. **Use Plain Text Storage (with caution)**
   ```toml
   [credentials]
   storage = "plain"
   path = "/tmp/plurcast-test-credentials"
   ```

3. **Clean Up After Testing**
   ```bash
   rm -rf /tmp/plurcast-test-credentials
   ```

## Security Audit Checklist

Run this checklist periodically:

```bash
# 1. Check for plain text credential files
plur-creds audit

# 2. Verify file permissions
ls -la ~/.config/plurcast/
# All credential files should be 600 (rw-------)

# 3. Check storage backend
grep "storage" ~/.config/plurcast/config.toml
# Should be "keyring" or "encrypted", not "plain"

# 4. Test authentication
plur-creds test --all
# All platforms should authenticate successfully

# 5. Check for credentials in logs
grep -r "nsec\|sk-" ~/.local/share/plurcast/
# Should return no results

# 6. Verify no credentials in version control
git status
# Ensure no .keys, .token, .auth files are tracked
```

## Credential Lifecycle

### Initial Setup

```bash
# Option 1: Interactive wizard (recommended)
plur-setup

# Option 2: Manual configuration
plur-creds set nostr
plur-creds set mastodon
plur-creds set bluesky
```

### Regular Use

```bash
# Credentials are automatically retrieved when posting
plur-post "Hello world"

# Test authentication periodically
plur-creds test --all
```

### Rotation

```bash
# Update credentials (overwrites existing)
plur-creds set nostr

# Verify new credentials work
plur-creds test nostr
```

### Deletion

```bash
# Delete credentials for a platform
plur-creds delete nostr

# Verify deletion
plur-creds list
```

## Platform-Specific Security Notes

### Nostr

**Credential Type**: Private key (hex or nsec format)

**Security Considerations**:
- Private key grants full control of your Nostr identity
- Cannot be revoked or rotated (tied to your public key)
- Compromise requires generating new identity

**Best Practices**:
- Never share your private key
- Use OS keyring for storage
- Consider using multiple keys for different purposes
- Back up your key securely (encrypted backup)

### Mastodon

**Credential Type**: OAuth access token

**Security Considerations**:
- Token grants access to your Mastodon account
- Can be revoked from Mastodon settings
- Scoped to specific permissions

**Best Practices**:
- Create app-specific tokens
- Use minimal required scopes (write:statuses)
- Revoke and regenerate if compromised
- Monitor authorized applications in Mastodon settings

**Token Rotation**:
```bash
# 1. Generate new token in Mastodon settings
# 2. Update Plurcast
plur-creds set mastodon
# 3. Test
plur-creds test mastodon
# 4. Revoke old token in Mastodon settings
```

### Bluesky

**Credential Type**: App password

**Security Considerations**:
- App password grants full account access
- Can be revoked from Bluesky settings
- Separate from main account password

**Best Practices**:
- Use app passwords, not main password
- Create separate app password for Plurcast
- Revoke and regenerate if compromised
- Monitor app passwords in Bluesky settings

**Password Rotation**:
```bash
# 1. Generate new app password in Bluesky settings
# 2. Update Plurcast
plur-creds set bluesky
# 3. Test
plur-creds test bluesky
# 4. Revoke old app password in Bluesky settings
```

## Incident Response

### If Credentials Are Compromised

1. **Immediate Actions**:
   ```bash
   # Delete compromised credentials
   plur-creds delete <platform>
   
   # Check for unauthorized posts
   plur-history --since "1 hour ago"
   ```

2. **Platform-Specific Actions**:
   - **Nostr**: Generate new key pair, announce new identity
   - **Mastodon**: Revoke token in settings, generate new one
   - **Bluesky**: Revoke app password in settings, generate new one

3. **System-Level Actions**:
   - Change OS user password (if keyring compromised)
   - Change master password (if encrypted storage compromised)
   - Audit system for malware
   - Review system logs for unauthorized access

4. **Recovery**:
   ```bash
   # Reconfigure with new credentials
   plur-setup
   
   # Verify authentication
   plur-creds test --all
   ```

### If Master Password Is Forgotten

```bash
# 1. Delete encrypted credential files
rm -rf ~/.config/plurcast/credentials/

# 2. Reconfigure credentials
plur-setup

# 3. Verify
plur-creds test --all
```

## Reporting Security Issues

If you discover a security vulnerability in Plurcast:

1. **Do not** open a public GitHub issue
2. Email security concerns to: [security contact - TBD]
3. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

We will respond within 48 hours and work with you to address the issue.

## Security Updates

- Security updates will be released as patch versions
- Critical security issues will be announced via GitHub Security Advisories
- Subscribe to repository releases for notifications

## Compliance and Standards

### Encryption Standards

- **age encryption**: Modern, secure file encryption
- **OS keyrings**: Platform-specific standards (Keychain, DPAPI, Secret Service)
- **File permissions**: Unix 600 (owner read/write only)

### Password Standards

- Minimum 8 characters (enforced)
- Recommended: NIST SP 800-63B guidelines
- No password complexity requirements (length is more important)

### Audit and Logging

- Credential access events logged (service/key, not values)
- No credential values in logs (enforced)
- Error messages sanitized (no credential leakage)

## References

- [age encryption](https://age-encryption.org/)
- [NIST Password Guidelines](https://pages.nist.gov/800-63-3/sp800-63b.html)
- [OWASP Credential Storage](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html)
- [macOS Keychain](https://developer.apple.com/documentation/security/keychain_services)
- [Windows DPAPI](https://docs.microsoft.com/en-us/windows/win32/api/dpapi/)
- [freedesktop.org Secret Service](https://specifications.freedesktop.org/secret-service/)

---

**Last Updated**: 2025-10-07
**Version**: 0.2.0-alpha
