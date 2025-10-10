# Testing Documentation Overview

Complete guide to testing Plurcast's OS-level credential security system.

## 📚 Documentation Structure

We've created a comprehensive testing suite with multiple entry points for different needs:

### Quick Start (5 minutes)
**[KEYRING_QUICKSTART.md](./KEYRING_QUICKSTART.md)**
- Quick reference card
- Essential commands
- Common scenarios
- Troubleshooting tips
- Perfect for: Developers who want to test quickly

### Automated Testing (2-5 minutes)
**[test-keyring.ps1](./test-keyring.ps1)**
- PowerShell automation script
- Runs all test phases automatically
- Generates test summary report
- Perfect for: CI/CD and quick validation

### Comprehensive Guide (30-60 minutes)
**[TESTING_KEYRING.md](./TESTING_KEYRING.md)**
- Complete testing methodology
- All test scenarios
- Platform-specific instructions
- Performance benchmarking
- Security verification
- Perfect for: Thorough testing and validation

### Testing Checklist (15-30 minutes)
**[TESTING_CHECKLIST.md](./TESTING_CHECKLIST.md)**
- Step-by-step checklist
- 16 testing phases
- Sign-off template
- Issue tracking format
- Perfect for: QA and formal testing

### Summary & Navigation
**[TESTING_SUMMARY.md](./TESTING_SUMMARY.md)**
- Overview of all testing approaches
- Test coverage details
- Success criteria
- Quick reference commands
- Perfect for: Understanding the big picture

### Visual Diagrams
**[docs/keyring-testing-flow.md](./docs/keyring-testing-flow.md)**
- Testing architecture diagrams
- Flow charts
- Decision trees
- State machines
- Perfect for: Visual learners and documentation

## 🎯 Choose Your Path

### Path 1: "Just Run It" (Fastest)
```powershell
# 1. Build
cargo build --release

# 2. Run automated tests
.\test-keyring.ps1

# 3. Done!
```
**Time:** 2-5 minutes  
**Best for:** Quick validation, CI/CD

---

### Path 2: "Quick Manual Test"
```powershell
# Follow KEYRING_QUICKSTART.md
# 1. Store credential
.\target\release\plur-creds.exe set nostr

# 2. List credentials
.\target\release\plur-creds.exe list

# 3. Test retrieval
.\target\release\plur-creds.exe test nostr

# 4. Security audit
.\target\release\plur-creds.exe audit

# 5. Cleanup
.\target\release\plur-creds.exe delete nostr --force
```
**Time:** 5-10 minutes  
**Best for:** Understanding the basics

---

### Path 3: "Comprehensive Testing"
```
Follow TESTING_KEYRING.md:
1. Unit tests (all backends)
2. CLI tool tests (all commands)
3. Integration tests (plur-post)
4. OS-level verification
5. Migration testing
6. Performance benchmarking
7. Security audit
```
**Time:** 30-60 minutes  
**Best for:** Thorough validation, release testing

---

### Path 4: "Formal QA Process"
```
Follow TESTING_CHECKLIST.md:
- 16 phases with checkboxes
- Document all issues
- Sign-off template
- Complete verification
```
**Time:** 30-60 minutes  
**Best for:** QA teams, formal releases

## 🔍 What's Being Tested

### Core Functionality
- **KeyringStore** - OS-native secure storage (Windows/macOS/Linux)
- **EncryptedFileStore** - Password-protected file storage
- **PlainFileStore** - Legacy plain text (deprecated)
- **CredentialManager** - Facade with automatic fallback

### CLI Tools
- **plur-creds** - Credential management tool
  - `set` - Store credentials
  - `list` - Display credentials
  - `test` - Verify credentials
  - `delete` - Remove credentials
  - `migrate` - Migrate from plain text
  - `audit` - Security audit

### Integration
- **plur-post** - Posting tool that retrieves credentials
- Cross-process credential access
- Credential rotation
- Multi-platform support

### Security
- OS-level encryption
- File permissions (Unix)
- No plain text storage
- Credential isolation
- Audit capabilities

## 📊 Test Coverage

### Unit Tests
**Location:** `libplurcast/src/credentials/tests.rs`

**Coverage:**
- ✓ KeyringStore operations (15 tests)
- ✓ EncryptedFileStore operations (8 tests)
- ✓ PlainFileStore operations (6 tests)
- ✓ CredentialManager fallback (5 tests)
- ✓ Error handling (10 tests)
- ✓ Multi-platform scenarios (5 tests)

**Total:** 49 unit tests

### CLI Tests
**Tool:** `plur-creds` binary

**Coverage:**
- ✓ Store credentials (all platforms)
- ✓ List credentials
- ✓ Test credentials
- ✓ Delete credentials
- ✓ Migrate from plain text
- ✓ Security audit

**Total:** 6 command groups, 20+ scenarios

### Integration Tests
**Coverage:**
- ✓ plur-post credential retrieval
- ✓ Cross-process access
- ✓ Credential rotation
- ✓ Multi-platform simultaneous use

**Total:** 5 integration scenarios

### OS-Level Tests
**Coverage:**
- ✓ Windows Credential Manager
- ✓ macOS Keychain
- ✓ Linux Secret Service
- ✓ Persistence across reboots

**Total:** 4 platform verifications

## ✅ Success Criteria

After testing, you should have:

### Functional ✓
- [ ] All unit tests pass
- [ ] All CLI commands work
- [ ] Integration with plur-post works
- [ ] OS-level storage verified

### Security ✓
- [ ] Credentials in OS keyring (not plain text)
- [ ] Security audit passes
- [ ] File permissions correct (Unix)
- [ ] No credential leaks in logs

### Performance ✓
- [ ] Store: < 100ms
- [ ] Retrieve: < 50ms
- [ ] List: < 100ms
- [ ] Post: < 2s

### Documentation ✓
- [ ] All issues documented
- [ ] Platform quirks noted
- [ ] Workarounds recorded

## 🚀 Quick Commands Reference

### Build
```powershell
cargo build --release
```

### Unit Tests
```powershell
cargo test --lib credentials                # Standard tests
cargo test --lib credentials -- --ignored   # Keyring tests
```

### Automated Testing
```powershell
.\test-keyring.ps1                          # Full suite
.\test-keyring.ps1 -Verbose                 # With details
.\test-keyring.ps1 -SkipBuild              # Skip build
```

### Manual Testing
```powershell
.\target\release\plur-creds.exe set nostr   # Store
.\target\release\plur-creds.exe list        # List
.\target\release\plur-creds.exe test nostr  # Test
.\target\release\plur-creds.exe audit       # Audit
.\target\release\plur-creds.exe migrate     # Migrate
.\target\release\plur-creds.exe delete nostr --force  # Delete
```

### OS Verification
```powershell
# Windows
control /name Microsoft.CredentialManager

# macOS
security find-generic-password -s "plurcast.nostr"

# Linux
secret-tool search service plurcast.nostr
```

## 🐛 Common Issues

### Issue: "OS keyring not accessible"
**Solution:** See [KEYRING_QUICKSTART.md](./KEYRING_QUICKSTART.md#common-issues)

### Issue: Tests failing in CI/CD
**Solution:** Configure encrypted storage fallback

### Issue: Credentials not persisting
**Solution:** Verify `storage = "keyring"` in config.toml

### Issue: Performance slow
**Solution:** Check OS keyring service status

## 📖 Related Documentation

### Architecture
- [ARCHITECTURE.md](./ARCHITECTURE.md) - Credential system design
- [SECURITY.md](./SECURITY.md) - Security considerations

### Development
- [ROADMAP.md](./ROADMAP.md) - Development phases
- [README.md](./README.md) - Project overview

### Implementation
- [libplurcast/src/credentials.rs](./libplurcast/src/credentials.rs) - Core code
- [plur-creds/src/main.rs](./plur-creds/src/main.rs) - CLI tool

## 🎓 Learning Path

### Beginner
1. Read [KEYRING_QUICKSTART.md](./KEYRING_QUICKSTART.md)
2. Run `.\test-keyring.ps1`
3. Try manual commands from quick start

### Intermediate
1. Read [TESTING_SUMMARY.md](./TESTING_SUMMARY.md)
2. Follow [TESTING_KEYRING.md](./TESTING_KEYRING.md)
3. Test all scenarios manually

### Advanced
1. Read [ARCHITECTURE.md](./ARCHITECTURE.md)
2. Review [libplurcast/src/credentials.rs](./libplurcast/src/credentials.rs)
3. Use [TESTING_CHECKLIST.md](./TESTING_CHECKLIST.md) for formal QA
4. Study [docs/keyring-testing-flow.md](./docs/keyring-testing-flow.md)

## 📝 Feedback & Improvements

After testing, please document:

### What Worked Well
- Which testing approach was most helpful?
- Which documentation was clearest?
- What made testing easy?

### What Could Improve
- Missing test scenarios?
- Unclear instructions?
- Platform-specific issues?
- Performance concerns?

### Suggestions
- Additional test cases?
- Better automation?
- Documentation improvements?

## 🔄 Next Steps

After completing testing:

1. **Document findings** - Use issue tracking format
2. **Update documentation** - Based on discoveries
3. **Report bugs** - If any found
4. **Suggest improvements** - For testing process
5. **Proceed to next phase** - Continue development

## 📞 Support

If you encounter issues:

1. Check [KEYRING_QUICKSTART.md](./KEYRING_QUICKSTART.md) troubleshooting
2. Review [TESTING_KEYRING.md](./TESTING_KEYRING.md) comprehensive guide
3. Search existing issues in repository
4. Document and report new issues

## 🎯 Testing Goals

The testing documentation aims to:

- ✓ Make testing accessible to all skill levels
- ✓ Provide multiple testing approaches
- ✓ Ensure comprehensive coverage
- ✓ Enable automated testing
- ✓ Document platform-specific behavior
- ✓ Verify security properties
- ✓ Measure performance
- ✓ Support CI/CD integration

## 📦 Deliverables

After testing, you should have:

1. **Test Results** - Pass/fail for all scenarios
2. **Performance Metrics** - Timing for all operations
3. **Security Verification** - Audit results
4. **Issue Documentation** - Any problems found
5. **Platform Notes** - Platform-specific observations
6. **Recommendations** - Improvements for next phase

---

**Version**: 0.2.0-alpha  
**Last Updated**: 2025-10-07  
**Status**: Complete testing documentation suite

**Quick Start:** Begin with [KEYRING_QUICKSTART.md](./KEYRING_QUICKSTART.md) or run `.\test-keyring.ps1`

**Questions?** Review [TESTING_SUMMARY.md](./TESTING_SUMMARY.md) for comprehensive overview
