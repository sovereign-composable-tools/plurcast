# Multi-Account Management: Manual Testing Guide

This document provides comprehensive manual testing procedures for the multi-account management feature across all supported platforms.

## Overview

The multi-account feature allows users to manage multiple credential sets per platform (e.g., test vs prod accounts). This guide covers manual testing on:

- **Windows** (Credential Manager)
- **macOS** (Keychain)
- **Linux** (Secret Service - GNOME Keyring/KWallet)

## Prerequisites

- Plurcast installed and built from source
- Access to the target operating system
- Terminal/command line access

## Test Scenarios

### Scenario 1: Basic Account Workflow (set → use → post → delete)

**Objective**: Verify the complete lifecycle of account management

**Steps**:

1. **Set credentials for default account**
   ```bash
   plur-creds set nostr --account default
   # Enter test private key when prompted
   ```
   
   **Expected**: Success message indicating credentials stored

2. **Set credentials for test account**
   ```bash
   plur-creds set nostr --account test
   # Enter different test private key
   ```
   
   **Expected**: Success message, no conflict with default account

3. **List accounts**
   ```bash
   plur-creds list --platform nostr
   ```
   
   **Expected**: Shows both "default" and "test" accounts with [active] marker on default

4. **Switch to test account**
   ```bash
   plur-creds use nostr --account test
   ```
   
   **Expected**: Success message confirming active account changed

5. **Verify active account**
   ```bash
   plur-creds list --platform nostr
   ```
   
   **Expected**: [active] marker now on "test" account

6. **Post using active account**
   ```bash
   echo "Test post from test account" | plur-post --platform nostr --verbose
   ```
   
   **Expected**: Post succeeds, verbose output shows "Using account: test"

7. **Post using explicit account override**
   ```bash
   echo "Test post from default account" | plur-post --platform nostr --account default --verbose
   ```
   
   **Expected**: Post succeeds, verbose output shows "Using account: default"

8. **Delete test account**
   ```bash
   plur-creds delete nostr --account test
   # Confirm deletion when prompted
   ```
   
   **Expected**: Success message, active account resets to "default"

9. **Verify deletion**
   ```bash
   plur-creds list --platform nostr
   ```
   
   **Expected**: Only "default" account shown

### Scenario 2: Multiple Accounts Per Platform

**Objective**: Verify isolation between multiple accounts

**Steps**:

1. **Create multiple accounts**
   ```bash
   plur-creds set nostr --account account1
   plur-creds set nostr --account account2
   plur-creds set nostr --account account3
   ```

2. **List all accounts**
   ```bash
   plur-creds list --platform nostr
   ```
   
   **Expected**: All three accounts listed

3. **Test account isolation**
   - Post with account1: `echo "From account1" | plur-post --platform nostr --account account1`
   - Post with account2: `echo "From account2" | plur-post --platform nostr --account account2`
   - Post with account3: `echo "From account3" | plur-post --platform nostr --account account3`
   
   **Expected**: Each post succeeds with correct account

4. **Verify credentials are isolated**
   ```bash
   plur-creds test nostr --account account1
   plur-creds test nostr --account account2
   plur-creds test nostr --account account3
   ```
   
   **Expected**: All tests pass with correct credentials

### Scenario 3: Account Switching

**Objective**: Verify active account switching works correctly

**Steps**:

1. **Create two accounts**
   ```bash
   plur-creds set nostr --account work
   plur-creds set nostr --account personal
   ```

2. **Set work as active**
   ```bash
   plur-creds use nostr --account work
   ```

3. **Post without explicit account**
   ```bash
   echo "Work post" | plur-post --platform nostr --verbose
   ```
   
   **Expected**: Uses work account

4. **Switch to personal**
   ```bash
   plur-creds use nostr --account personal
   ```

5. **Post without explicit account**
   ```bash
   echo "Personal post" | plur-post --platform nostr --verbose
   ```
   
   **Expected**: Uses personal account

6. **Verify active account persists across commands**
   ```bash
   plur-creds list --platform nostr
   ```
   
   **Expected**: [active] marker on personal account

### Scenario 4: Backward Compatibility

**Objective**: Verify existing single-account setups continue to work

**Steps**:

1. **Simulate old format** (if you have old credentials, test with them)
   - If starting fresh, set up default account
   ```bash
   plur-creds set nostr
   # This should create "default" account
   ```

2. **Verify default account behavior**
   ```bash
   plur-creds list --platform nostr
   ```
   
   **Expected**: Shows "default" account as active

3. **Post without account flag**
   ```bash
   echo "Backward compatible post" | plur-post --platform nostr
   ```
   
   **Expected**: Works as before, uses default account

4. **Add new account alongside default**
   ```bash
   plur-creds set nostr --account new-account
   ```
   
   **Expected**: Both accounts coexist

### Scenario 5: Error Handling

**Objective**: Verify proper error messages for invalid operations

**Steps**:

1. **Invalid account names**
   ```bash
   plur-creds set nostr --account "invalid name"  # space
   plur-creds set nostr --account "test@account"  # special char
   plur-creds set nostr --account ""              # empty
   ```
   
   **Expected**: Clear error messages explaining valid format

2. **Non-existent account**
   ```bash
   plur-creds use nostr --account nonexistent
   ```
   
   **Expected**: Error indicating account not found

3. **Delete non-existent account**
   ```bash
   plur-creds delete nostr --account nonexistent
   ```
   
   **Expected**: Error indicating account not found

4. **Post with non-existent account**
   ```bash
   echo "Test" | plur-post --platform nostr --account nonexistent
   ```
   
   **Expected**: Error indicating account not found

### Scenario 6: Multi-Platform Testing

**Objective**: Verify accounts work across different platforms

**Steps**:

1. **Set up accounts for multiple platforms**
   ```bash
   plur-creds set nostr --account test
   plur-creds set mastodon --account test
   plur-creds set bluesky --account test
   ```

2. **Verify platform isolation**
   ```bash
   plur-creds list
   ```
   
   **Expected**: Each platform shows its own "test" account

3. **Set different active accounts per platform**
   ```bash
   plur-creds use nostr --account test
   plur-creds use mastodon --account default
   ```

4. **Post to multiple platforms**
   ```bash
   echo "Multi-platform post" | plur-post --platform nostr,mastodon --verbose
   ```
   
   **Expected**: Uses correct active account for each platform

## Platform-Specific Testing

### Windows (Credential Manager)

**Storage Backend**: Windows Credential Manager

**Verification Steps**:

1. **Check Credential Manager**
   - Open Control Panel → Credential Manager → Windows Credentials
   - Look for entries starting with "plurcast.nostr."
   - Verify multiple accounts appear as separate entries

2. **Test persistence across reboots**
   - Set up accounts
   - Reboot system
   - Verify accounts still accessible: `plur-creds list`

3. **Test keyring-specific features**
   ```bash
   plur-creds audit
   ```
   
   **Expected**: Shows "keyring" as primary backend

### macOS (Keychain)

**Storage Backend**: macOS Keychain

**Verification Steps**:

1. **Check Keychain Access**
   - Open Keychain Access app
   - Search for "plurcast"
   - Verify multiple account entries exist

2. **Test Keychain permissions**
   - First access may prompt for Keychain password
   - Verify "Always Allow" option works

3. **Test persistence across reboots**
   - Set up accounts
   - Reboot system
   - Verify accounts still accessible: `plur-creds list`

4. **Test keyring-specific features**
   ```bash
   plur-creds audit
   ```
   
   **Expected**: Shows "keyring" as primary backend

### Linux (Secret Service)

**Storage Backend**: Secret Service (GNOME Keyring or KWallet)

**Verification Steps**:

1. **Check Secret Service**
   - GNOME: Use Seahorse (Passwords and Keys app)
   - KDE: Use KWalletManager
   - Search for "plurcast" entries
   - Verify multiple accounts appear

2. **Test D-Bus integration**
   ```bash
   # Verify Secret Service is running
   ps aux | grep -i "gnome-keyring\|kwalletd"
   ```

3. **Test persistence across sessions**
   - Set up accounts
   - Log out and log back in
   - Verify accounts still accessible: `plur-creds list`

4. **Test keyring-specific features**
   ```bash
   plur-creds audit
   ```
   
   **Expected**: Shows "keyring" as primary backend

5. **Test fallback to encrypted storage**
   - Stop Secret Service: `killall gnome-keyring-daemon` (GNOME)
   - Try to set credentials
   - Verify fallback to encrypted storage works

## Migration Testing

**Objective**: Verify migration from old format to new format

**Steps**:

1. **Simulate old format credentials** (if possible)
   - If you have old Plurcast installation, test upgrade path

2. **Run migration**
   ```bash
   plur-creds migrate --to-multi-account
   ```
   
   **Expected**: Shows migration plan and report

3. **Verify migrated credentials**
   ```bash
   plur-creds list
   ```
   
   **Expected**: Old credentials now under "default" account

4. **Test posting with migrated credentials**
   ```bash
   echo "Post after migration" | plur-post --platform nostr
   ```
   
   **Expected**: Works with default account

## Security Audit Testing

**Objective**: Verify security audit functionality

**Steps**:

1. **Run security audit**
   ```bash
   plur-creds audit
   ```
   
   **Expected**: Shows:
   - Primary storage backend
   - Security status
   - Any issues found
   - Recommendations

2. **Test with insecure storage** (if applicable)
   - Configure plain text storage
   - Run audit
   - Verify warnings appear

## Success Criteria

For each platform, verify:

- ✅ Credentials persist across process restarts
- ✅ Multiple accounts can coexist without conflicts
- ✅ Active account switching works correctly
- ✅ Account isolation is maintained
- ✅ Backward compatibility with single-account setup
- ✅ Error messages are clear and helpful
- ✅ Platform-specific storage backend works correctly
- ✅ Migration from old format succeeds
- ✅ Security audit provides useful information

## Reporting Issues

If any test fails, document:

1. **Platform**: Windows/macOS/Linux (distro)
2. **Test scenario**: Which scenario failed
3. **Expected behavior**: What should have happened
4. **Actual behavior**: What actually happened
5. **Error messages**: Full error output
6. **Steps to reproduce**: Exact commands used
7. **Environment**: Plurcast version, OS version, storage backend

## Notes

- **System Reboot Testing**: Some tests require system reboots to verify persistence. These should be performed manually.
- **Keyring Availability**: On some systems (especially CI environments), OS keyrings may not be available. The system should gracefully fall back to encrypted storage.
- **Permissions**: Ensure proper file permissions are set (600 for credential files, 644 for account state).

## Automated vs Manual Testing

**Automated** (covered by integration tests):
- Basic CRUD operations
- Account isolation
- State persistence across manager instances
- Backward compatibility
- Error handling

**Manual** (this guide):
- OS-specific keyring integration
- System reboot persistence
- Cross-platform verification
- User experience validation
- Real-world workflow testing
