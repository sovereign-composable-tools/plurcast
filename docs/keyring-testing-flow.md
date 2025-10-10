# Keyring Testing Flow Diagram

Visual guide to the credential testing process.

## Testing Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Testing Layers                           │
└─────────────────────────────────────────────────────────────┘

Layer 1: Unit Tests (Automated)
├── KeyringStore tests
│   ├── test_keyring_store_operations
│   ├── test_keyring_multiple_platforms
│   └── test_keyring_service_naming
├── EncryptedFileStore tests
│   ├── test_encrypted_store_operations
│   ├── test_encrypted_store_weak_password
│   └── test_encrypted_store_file_permissions
└── CredentialManager tests
    ├── test_credential_manager_plain_backend
    └── test_credential_manager_fallback_logic

Layer 2: CLI Tool Tests (Manual/Scripted)
├── plur-creds set <platform>
├── plur-creds list
├── plur-creds test <platform>
├── plur-creds audit
├── plur-creds migrate
└── plur-creds delete <platform>

Layer 3: Integration Tests
├── plur-post with keyring credentials
├── Cross-process credential access
└── Credential rotation testing

Layer 4: OS-Level Verification
├── Windows Credential Manager
├── macOS Keychain Access
└── Linux Secret Service
```

## Test Flow Sequence

```
┌─────────────┐
│   START     │
└──────┬──────┘
       │
       ▼
┌─────────────────────────────────┐
│  Phase 0: Build Binaries        │
│  cargo build --release          │
└──────┬──────────────────────────┘
       │
       ▼
┌─────────────────────────────────┐
│  Phase 1: Unit Tests            │
│  cargo test --lib credentials   │
└──────┬──────────────────────────┘
       │
       ▼
┌─────────────────────────────────┐
│  Phase 2: Configuration Check   │
│  Verify config.toml settings    │
└──────┬──────────────────────────┘
       │
       ▼
┌─────────────────────────────────┐
│  Phase 3: Cleanup               │
│  Remove existing test creds     │
└──────┬──────────────────────────┘
       │
       ▼
┌─────────────────────────────────┐
│  Phase 4: Store Credentials     │
│  plur-creds set nostr           │
└──────┬──────────────────────────┘
       │
       ▼
┌─────────────────────────────────┐
│  Phase 5: List Credentials      │
│  plur-creds list                │
│  → Verify backend used          │
└──────┬──────────────────────────┘
       │
       ▼
┌─────────────────────────────────┐
│  Phase 6: Test Retrieval        │
│  plur-creds test nostr          │
└──────┬──────────────────────────┘
       │
       ▼
┌─────────────────────────────────┐
│  Phase 7: Security Audit        │
│  plur-creds audit               │
└──────┬──────────────────────────┘
       │
       ▼
┌─────────────────────────────────┐
│  Phase 8: OS Verification       │
│  Check Windows Cred Manager     │
└──────┬──────────────────────────┘
       │
       ▼
┌─────────────────────────────────┐
│  Phase 9: Integration Test      │
│  plur-post with credentials     │
└──────┬──────────────────────────┘
       │
       ▼
┌─────────────────────────────────┐
│  Phase 10: Cleanup              │
│  plur-creds delete nostr        │
└──────┬──────────────────────────┘
       │
       ▼
┌─────────────┐
│   RESULTS   │
│  ✓ Passed   │
│  ✗ Failed   │
│  ⊘ Skipped  │
└─────────────┘
```

## Credential Storage Decision Tree

```
                    ┌─────────────────┐
                    │  Store Request  │
                    └────────┬────────┘
                             │
                             ▼
                    ┌─────────────────┐
                    │ Check Config    │
                    │ storage = ?     │
                    └────────┬────────┘
                             │
                ┌────────────┼────────────┐
                │            │            │
                ▼            ▼            ▼
        ┌──────────┐  ┌──────────┐  ┌──────────┐
        │ keyring  │  │encrypted │  │  plain   │
        └─────┬────┘  └─────┬────┘  └─────┬────┘
              │             │             │
              ▼             ▼             ▼
        ┌──────────┐  ┌──────────┐  ┌──────────┐
        │ Keyring  │  │ Master   │  │  File    │
        │Available?│  │Password? │  │  Write   │
        └─────┬────┘  └─────┬────┘  └─────┬────┘
              │             │             │
         Yes  │  No    Yes  │  No         │
              │   │         │   │         │
              ▼   │         ▼   │         ▼
        ┌──────────┐  ┌──────────┐  ┌──────────┐
        │  Store   │  │ Encrypt  │  │  Store   │
        │in Keyring│  │ & Write  │  │Plain Text│
        └─────┬────┘  └─────┬────┘  └─────┬────┘
              │             │             │
              │   ┌─────────┘             │
              │   │   ┌───────────────────┘
              ▼   ▼   ▼
        ┌──────────────────┐
        │  Log Backend     │
        │  Used & Return   │
        └──────────────────┘
```

## Fallback Logic Flow

```
┌─────────────────────────────────────────────────────────┐
│              CredentialManager Fallback                 │
└─────────────────────────────────────────────────────────┘

Attempt 1: KeyringStore
├── Try: OS keyring access
├── Success? → Use KeyringStore
└── Failure? → Continue to Attempt 2

Attempt 2: EncryptedFileStore
├── Check: Master password set?
├── Yes? → Use EncryptedFileStore
├── No? → Prompt for password (if TTY)
│   ├── Provided? → Use EncryptedFileStore
│   └── Not provided? → Continue to Attempt 3
└── Failure? → Continue to Attempt 3

Attempt 3: PlainFileStore (Legacy)
├── Log: Deprecation warning
├── Use: PlainFileStore
└── Return: Success (with warning)

If all fail:
└── Error: NoStoreAvailable
```

## Migration Flow

```
┌─────────────────────────────────────────────────────────┐
│              Credential Migration Process               │
└─────────────────────────────────────────────────────────┘

Step 1: Detect Plain Text Files
├── Scan: ~/.config/plurcast/
├── Find: *.keys, *.token, *.auth
└── List: Found files

Step 2: Read Plain Text Credentials
├── For each file:
│   ├── Read content
│   ├── Parse credential
│   └── Validate format
└── Build: Migration list

Step 3: Store in Secure Storage
├── For each credential:
│   ├── Try: Store in keyring/encrypted
│   ├── Success? → Mark as migrated
│   └── Failure? → Mark as failed
└── Generate: Migration report

Step 4: Verify Migration
├── For each migrated:
│   ├── Retrieve from secure storage
│   ├── Compare with original
│   └── Verify match
└── Confirm: All verified

Step 5: Cleanup (Optional)
├── Prompt: Delete plain text files?
├── User confirms? → Delete files
└── Log: Cleanup complete

Step 6: Report Results
├── Display: Migrated count
├── Display: Failed count
├── Display: Skipped count
└── Exit: Success/Failure code
```

## Test Result States

```
┌─────────────────────────────────────────────────────────┐
│                  Test State Machine                     │
└─────────────────────────────────────────────────────────┘

                    ┌─────────┐
                    │  START  │
                    └────┬────┘
                         │
                         ▼
                  ┌──────────────┐
                  │  Run Test    │
                  └──────┬───────┘
                         │
            ┌────────────┼────────────┐
            │            │            │
            ▼            ▼            ▼
      ┌─────────┐  ┌─────────┐  ┌─────────┐
      │ PASSED  │  │ FAILED  │  │ SKIPPED │
      │  (✓)    │  │  (✗)    │  │  (⊘)    │
      └────┬────┘  └────┬────┘  └────┬────┘
           │            │            │
           └────────────┼────────────┘
                        │
                        ▼
                  ┌──────────────┐
                  │   Summary    │
                  │  Report      │
                  └──────┬───────┘
                         │
                         ▼
                  ┌──────────────┐
                  │  Exit Code   │
                  │  0 or 1      │
                  └──────────────┘

Exit Codes:
  0 = All tests passed (no failures)
  1 = One or more tests failed
```

## Platform-Specific Testing Paths

```
┌─────────────────────────────────────────────────────────┐
│            Platform-Specific Verification               │
└─────────────────────────────────────────────────────────┘

Windows Path:
  Store → Windows Credential Manager
       → Generic Credential
       → Target: plurcast.{platform}
       → Username: {key}
       → Password: {value}
  
  Verify: control /name Microsoft.CredentialManager
       → Search: "plurcast"
       → Confirm: Entries visible

macOS Path:
  Store → Keychain Access
       → Login Keychain
       → Generic Password
       → Service: plurcast.{platform}
       → Account: {key}
  
  Verify: security find-generic-password -s "plurcast.nostr"
       → Confirm: Entry found

Linux Path:
  Store → Secret Service (D-Bus)
       → Collection: Login
       → Item: plurcast.{platform}
       → Attribute: {key}
  
  Verify: secret-tool search service plurcast.nostr
       → Confirm: Entry found
```

## Error Handling Flow

```
┌─────────────────────────────────────────────────────────┐
│                  Error Handling                         │
└─────────────────────────────────────────────────────────┘

Error Type: KeyringUnavailable
├── Cause: OS keyring not accessible
├── Action: Fall back to encrypted storage
└── Log: Warning about fallback

Error Type: MasterPasswordNotSet
├── Cause: Encrypted storage needs password
├── Action: Prompt user (if TTY)
└── Fallback: Plain storage (with warning)

Error Type: DecryptionFailed
├── Cause: Wrong password or corrupted file
├── Action: Prompt for password again
└── Limit: 3 attempts, then fail

Error Type: NotFound
├── Cause: Credential doesn't exist
├── Action: Return error to caller
└── Suggest: Use 'plur-creds set' to store

Error Type: WeakPassword
├── Cause: Password < 8 characters
├── Action: Reject and prompt again
└── Suggest: Use 12+ characters

Error Type: NoStoreAvailable
├── Cause: All backends failed
├── Action: Return error
└── Suggest: Check configuration
```

## Performance Monitoring Points

```
┌─────────────────────────────────────────────────────────┐
│              Performance Checkpoints                    │
└─────────────────────────────────────────────────────────┘

Checkpoint 1: Store Operation
├── Start: Credential received
├── Process: Encrypt/Store
├── End: Confirmation returned
└── Target: < 100ms

Checkpoint 2: Retrieve Operation
├── Start: Retrieve request
├── Process: Fetch/Decrypt
├── End: Credential returned
└── Target: < 50ms

Checkpoint 3: List Operation
├── Start: List request
├── Process: Check all platforms
├── End: List returned
└── Target: < 100ms

Checkpoint 4: Full Post Operation
├── Start: Post command
├── Retrieve: Credentials (< 50ms)
├── Connect: Platform (< 500ms)
├── Post: Content (< 1000ms)
└── Target: < 2000ms total
```

---

**Version**: 0.2.0-alpha  
**Last Updated**: 2025-10-07  
**Status**: Documentation for testing infrastructure
