# Multi-Account Migration Guide

**Version**: 0.3.0-alpha2  
**Target Audience**: Existing Plurcast users upgrading from single-account versions

## Overview

Plurcast 0.3.0-alpha2 introduces multi-account support, allowing you to manage multiple credential sets per platform (e.g., test vs prod Nostr keys, personal vs work accounts). This guide explains how the migration works and how to use the new features.

## What's New

### Multi-Account Support

You can now store and manage multiple accounts per platform:

```bash
# Store credentials for different accounts
plur-creds set nostr --account test
plur-creds set nostr --account prod
plur-creds set nostr --account personal

# List all accounts
plur-creds list --platform nostr
# Output:
#   ✓ nostr (default): Private Key (stored in keyring) [active]
#   ✓ nostr (test): Private Key (stored in keyring)
#   ✓ nostr (prod): Private Key (stored in keyring)

# Switch active account
plur-creds use nostr --account prod

# Post using active account
plur-post "Hello from prod account"

# Or specify account explicitly
plur-post "Test message" --account test
```

### The "default" Account Concept

The **"default" account** is a special account name used for backward compatibility:

- When you omit the `--account` flag, Plurcast uses the "default" account
- Your existing credentials are automatically migrated to the "default" account
- This ensures existing workflows continue to work without changes

**Example**:
```bash
# These are equivalent:
plur-creds set nostr
plur-creds set nostr --account default

# These are equivalent:
plur-post "Hello"
plur-post "Hello" --account default
```

## Automatic Migration

### What Happens Automatically

When you upgrade to 0.3.0-alpha2, Plurcast automatically migrates your existing credentials:

**Old namespace format** (single account):
```
plurcast.nostr.private_key
plurcast.mastodon.access_token
plurcast.bluesky.app_password
```

**New namespace format** (multi-account):
```
plurcast.nostr.default.private_key
plurcast.mastodon.default.access_token
plurcast.bluesky.default.app_password
```

### Migration Process

1. **Detection**: On first run, Plurcast detects credentials in the old format
2. **Migration**: Credentials are copied to the new "default" account namespace
3. **Verification**: Plurcast verifies the migration by retrieving credentials
4. **Preservation**: Old credentials are kept for backward compatibility
5. **Logging**: Migration success/failure is logged

### No Action Required

For most users, **no action is required**. Your existing credentials will:
- Continue to work exactly as before
- Be accessible as the "default" account
- Allow you to add additional accounts when needed

## Adding Additional Accounts

Once migrated, you can add additional accounts:

### Example: Test and Production Accounts

```bash
# Your existing credentials are now the "default" account
plur-creds list --platform nostr
# Output: ✓ nostr (default): Private Key (stored in keyring) [active]

# Add a test account
plur-creds set nostr --account test
# Enter test private key...

# Add a production account
plur-creds set nostr --account prod
# Enter production private key...

# List all accounts
plur-creds list --platform nostr
# Output:
#   ✓ nostr (default): Private Key (stored in keyring) [active]
#   ✓ nostr (test): Private Key (stored in keyring)
#   ✓ nostr (prod): Private Key (stored in keyring)

# Switch to test account for development
plur-creds use nostr --account test

# Post to test account (uses active account)
plur-post "Testing new feature"

# Post to prod account explicitly
plur-post "Production announcement" --account prod
```

### Example: Personal and Work Accounts

```bash
# Rename your default account to "personal" by setting it explicitly
plur-creds set mastodon --account personal
# Enter personal access token...

# Add work account
plur-creds set mastodon --account work
# Enter work access token...

# Switch between accounts
plur-creds use mastodon --account work
plur-post "Team update"

plur-creds use mastodon --account personal
plur-post "Weekend plans"
```

## Checking Migration Status

### Verify Your Credentials

After upgrading, verify your credentials are accessible:

```bash
# Test all platforms
plur-creds test --all

# Test specific platform
plur-creds test nostr

# List all configured accounts
plur-creds list
```

### Check Active Accounts

View which account is active for each platform:

```bash
plur-creds list
# Output shows [active] marker next to active accounts:
#   ✓ nostr (default): Private Key (stored in keyring) [active]
#   ✓ mastodon (work): Access Token (stored in keyring) [active]
```

### Account State File

Active accounts are tracked in `~/.config/plurcast/accounts.toml`:

```toml
# Active account per platform
[active]
nostr = "default"
mastodon = "work"
bluesky = "default"

# Registered accounts per platform
[accounts.nostr]
names = ["default", "test", "prod"]

[accounts.mastodon]
names = ["default", "work"]
```

## Manual Migration (Optional)

If automatic migration doesn't work or you want explicit control:

```bash
# Run manual migration command
plur-creds migrate --to-multi-account

# This will:
# 1. Scan for old format credentials
# 2. Display migration plan
# 3. Ask for confirmation
# 4. Migrate to "default" account
# 5. Display migration report
```

**Example output**:
```
Migration Plan:
  nostr: private_key → default account
  mastodon: access_token → default account
  bluesky: app_password → default account

Proceed with migration? [y/N]: y

Migration Report:
  ✓ Migrated: nostr.private_key
  ✓ Migrated: mastodon.access_token
  ✓ Migrated: bluesky.app_password
  
Migration complete: 3 succeeded, 0 failed
```

## Backward Compatibility Guarantees

### Existing Workflows Continue to Work

All existing commands work without changes:

```bash
# These commands work exactly as before:
plur-creds set nostr
plur-creds test nostr
plur-post "Hello world"
plur-history --platform nostr
```

### No Breaking Changes

- **Configuration files**: No changes to `config.toml` required
- **Credential files**: Old format files still work (auto-migrated)
- **CLI commands**: All existing commands have same behavior
- **Exit codes**: Same exit codes as before
- **Output format**: Same output format as before

### Opt-In Multi-Account

Multi-account features are **opt-in**:
- Use `--account` flag only when you need multiple accounts
- Omit `--account` to use default account (existing behavior)
- No forced migration or configuration changes

## Account Naming Conventions

### Valid Account Names

Account names must follow these rules:
- **Alphanumeric characters**: a-z, A-Z, 0-9
- **Hyphens and underscores**: `-` and `_`
- **Maximum length**: 64 characters
- **Case-sensitive**: `Test` and `test` are different accounts

### Examples

**Valid names**:
- `default`
- `test`
- `prod`
- `test-account`
- `prod_2024`
- `work`
- `personal`
- `staging-env`

**Invalid names**:
- `test account` (space not allowed)
- `test@account` (special characters not allowed)
- `test.account` (period not allowed)
- `a` * 65 (exceeds 64 character limit)

### Best Practices

1. **Use descriptive names**: `test`, `prod`, `staging` instead of `a`, `b`, `c`
2. **Be consistent**: Use same naming scheme across platforms
3. **Avoid special characters**: Stick to alphanumeric, hyphens, underscores
4. **Keep it short**: Shorter names are easier to type and remember

## Common Workflows

### Developer Workflow: Test and Production

```bash
# Initial setup - store test credentials
plur-creds set nostr --account test
# Enter test private key...

# Store prod credentials
plur-creds set nostr --account prod
# Enter prod private key...

# Set test as active for development
plur-creds use nostr --account test

# Post to test account (uses active account)
plur-post "Testing new feature"

# Post to prod account explicitly
plur-post "Production announcement" --account prod

# List all accounts
plur-creds list --platform nostr
# Output:
#   ✓ nostr (test): Private Key (stored in keyring) [active]
#   ✓ nostr (prod): Private Key (stored in keyring)
```

### Multi-Platform Workflow

```bash
# Configure different accounts for different platforms
plur-creds set nostr --account personal
plur-creds set mastodon --account work
plur-creds set bluesky --account test

# Set active accounts
plur-creds use nostr --account personal
plur-creds use mastodon --account work
plur-creds use bluesky --account test

# Post to all platforms using their active accounts
plur-post "Cross-platform message"
# Uses: nostr (personal), mastodon (work), bluesky (test)

# Post to specific platform with specific account
plur-post "Nostr-only test" --platform nostr --account test
```

### Account Switching Workflow

```bash
# Morning: Switch to work accounts
plur-creds use mastodon --account work
plur-creds use bluesky --account work

# Post work updates
plur-post "Team standup notes"

# Evening: Switch to personal accounts
plur-creds use mastodon --account personal
plur-creds use bluesky --account personal

# Post personal content
plur-post "Weekend plans"
```

## Troubleshooting

### "Account not found"

**Error**: `Account 'test' not found for platform 'nostr'`

**Cause**: Account doesn't exist or hasn't been configured

**Solution**:
```bash
# List existing accounts
plur-creds list --platform nostr

# Create the account
plur-creds set nostr --account test
```

### "Migration failed"

**Error**: `Migration failed for nostr.private_key`

**Cause**: Old credential file is corrupted or inaccessible

**Solution**:
```bash
# Check credential file exists and has correct permissions
ls -la ~/.config/plurcast/nostr.keys
chmod 600 ~/.config/plurcast/nostr.keys

# Try manual migration
plur-creds set nostr --account default
# Enter credentials manually

# Verify it works
plur-creds test nostr
```

### "Credentials not persisting"

**Error**: Credentials work immediately but are lost after restart

**Cause**: OS keyring persistence issue (known issue)

**Solution**: Use encrypted file storage instead:
```toml
# In config.toml
[credentials]
storage = "encrypted"
path = "~/.config/plurcast/credentials"
```

Then reconfigure credentials:
```bash
export PLURCAST_MASTER_PASSWORD="your_secure_password"
plur-creds set nostr --account default
plur-creds set nostr --account test
```

### "Cannot delete active account"

**Error**: `Cannot delete active account 'test' for platform 'nostr'`

**Cause**: Trying to delete the currently active account

**Solution**: Switch to a different account first:
```bash
# Switch to default account
plur-creds use nostr --account default

# Now delete test account
plur-creds delete nostr --account test
```

## Getting Help

### Documentation

- **README.md**: General usage and setup
- **ARCHITECTURE.md**: Technical details and design
- **ADR 001**: Multi-account design decisions

### Commands

```bash
# Get help for any command
plur-creds --help
plur-creds set --help
plur-creds use --help

# Test authentication
plur-creds test --all

# Audit security
plur-creds audit
```

### Support

- **Issues**: https://github.com/plurcast/plurcast/issues
- **Discussions**: https://github.com/plurcast/plurcast/discussions

## Summary

- **Automatic migration**: Your existing credentials become the "default" account
- **No action required**: Existing workflows continue to work
- **Opt-in multi-account**: Use `--account` flag when you need multiple accounts
- **Backward compatible**: No breaking changes to commands or configuration
- **Account naming**: Alphanumeric, hyphens, underscores, max 64 characters
- **Active accounts**: Tracked in `~/.config/plurcast/accounts.toml`
- **Migration command**: `plur-creds migrate --to-multi-account` for manual migration

Welcome to multi-account Plurcast! 🎉
