//! Comprehensive integration tests for multi-account management
//!
//! Tests the complete workflow: set → use → post → delete
//! Covers multiple accounts per platform, account switching, backward compatibility,
//! migration, and error scenarios.

use anyhow::Result;
use libplurcast::accounts::AccountManager;
use libplurcast::credentials::{CredentialConfig, CredentialManager, StorageBackend};
use libplurcast::db::Database;
use libplurcast::platforms::mock::MockPlatform;
use libplurcast::platforms::Platform;
use libplurcast::poster::MultiPlatformPoster;
use libplurcast::types::Post;
use std::sync::Arc;
use tempfile::TempDir;

/// Test helper to create a test environment with temporary directories
struct TestEnv {
    _temp_dir: TempDir,
    account_manager: AccountManager,
    credential_manager: CredentialManager,
    db: Database,
}

impl TestEnv {
    async fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();

        // Create account manager with temp state file
        let state_file = temp_path.join("accounts.toml");
        let account_manager = AccountManager::with_path(state_file)?;

        // Create credential manager with encrypted storage
        let cred_config = CredentialConfig {
            storage: StorageBackend::Encrypted,
            path: temp_path.join("credentials").to_string_lossy().to_string(),
            master_password: Some("test_password_123".to_string()),
        };
        let credential_manager = CredentialManager::new(cred_config)?;

        // Create database
        let db_path = temp_path.join("test.db");
        let db = Database::new(&db_path.to_string_lossy()).await?;

        Ok(Self {
            _temp_dir: temp_dir,
            account_manager,
            credential_manager,
            db,
        })
    }
}

#[tokio::test]
async fn test_complete_workflow_set_use_post_delete() -> Result<()> {
    let env = TestEnv::new().await?;

    // Step 1: Set credentials for default account
    env.credential_manager
        .store_account("plurcast.nostr", "private_key", "default", "test_key_default")?;
    env.account_manager
        .register_account("nostr", "default")?;

    // Step 2: Set credentials for test account
    env.credential_manager
        .store_account("plurcast.nostr", "private_key", "test", "test_key_test")?;
    env.account_manager.register_account("nostr", "test")?;

    // Step 3: Use test account (set as active)
    env.account_manager
        .set_active_account("nostr", "test")?;
    assert_eq!(env.account_manager.get_active_account("nostr"), "test");

    // Step 4: Post using active account
    let mut platform = MockPlatform::success("nostr");
    platform.authenticate().await?;

    let platforms: Vec<Box<dyn Platform>> = vec![Box::new(platform)];
    let poster = MultiPlatformPoster::new(platforms, env.db.clone());

    let post = Post::new("Test post with test account".to_string());
    let results = poster.post_to_all(&post).await;

    assert_eq!(results.len(), 1);
    assert!(results[0].success);

    // Step 5: Verify post was saved
    let saved_post = env.db.get_post(&post.id).await?;
    assert!(saved_post.is_some());

    // Step 6: Delete test account credentials
    env.credential_manager
        .delete_account("plurcast.nostr", "private_key", "test")?;
    env.account_manager.unregister_account("nostr", "test")?;

    // Step 7: Verify account was deleted and active reset to default
    assert!(!env.account_manager.account_exists("nostr", "test"));
    assert_eq!(env.account_manager.get_active_account("nostr"), "default");

    // Step 8: Verify credentials were deleted
    assert!(!env
        .credential_manager
        .exists_account("plurcast.nostr", "private_key", "test")?);

    Ok(())
}

#[tokio::test]
async fn test_multiple_accounts_per_platform() -> Result<()> {
    let env = TestEnv::new().await?;

    // Create multiple accounts for nostr
    let accounts = vec!["default", "test", "prod", "staging"];

    for account in &accounts {
        let key_value = format!("test_key_{}", account);
        env.credential_manager.store_account(
            "plurcast.nostr",
            "private_key",
            account,
            &key_value,
        )?;
        env.account_manager.register_account("nostr", account)?;
    }

    // Verify all accounts exist
    let registered_accounts = env.account_manager.list_accounts("nostr");
    assert_eq!(registered_accounts.len(), 4);

    for account in &accounts {
        assert!(env.account_manager.account_exists("nostr", account));
        assert!(env
            .credential_manager
            .exists_account("plurcast.nostr", "private_key", account)?);
    }

    // Verify credentials are isolated
    for account in &accounts {
        let retrieved = env
            .credential_manager
            .retrieve_account("plurcast.nostr", "private_key", account)?;
        assert_eq!(retrieved, format!("test_key_{}", account));
    }

    Ok(())
}

#[tokio::test]
async fn test_account_switching() -> Result<()> {
    let env = TestEnv::new().await?;

    // Create two accounts
    env.credential_manager.store_account(
        "plurcast.nostr",
        "private_key",
        "account1",
        "key1",
    )?;
    env.account_manager.register_account("nostr", "account1")?;

    env.credential_manager.store_account(
        "plurcast.nostr",
        "private_key",
        "account2",
        "key2",
    )?;
    env.account_manager.register_account("nostr", "account2")?;

    // Initially should be default
    assert_eq!(env.account_manager.get_active_account("nostr"), "default");

    // Switch to account1
    env.account_manager
        .set_active_account("nostr", "account1")?;
    assert_eq!(env.account_manager.get_active_account("nostr"), "account1");

    // Switch to account2
    env.account_manager
        .set_active_account("nostr", "account2")?;
    assert_eq!(env.account_manager.get_active_account("nostr"), "account2");

    // Switch back to account1
    env.account_manager
        .set_active_account("nostr", "account1")?;
    assert_eq!(env.account_manager.get_active_account("nostr"), "account1");

    Ok(())
}

#[tokio::test]
async fn test_backward_compatibility_with_existing_credentials() -> Result<()> {
    let env = TestEnv::new().await?;

    // Simulate old format credentials (no account namespace)
    // In the old format, credentials were stored without account parameter
    env.credential_manager
        .store("plurcast.nostr", "private_key", "old_format_key")?;

    // The old format should still be retrievable via default account
    // (backward compatibility layer in CredentialManager)
    let retrieved = env
        .credential_manager
        .retrieve("plurcast.nostr", "private_key")?;
    assert_eq!(retrieved, "old_format_key");

    // New format should work alongside old format
    env.credential_manager.store_account(
        "plurcast.nostr",
        "private_key",
        "test",
        "new_format_key",
    )?;

    let new_retrieved = env
        .credential_manager
        .retrieve_account("plurcast.nostr", "private_key", "test")?;
    assert_eq!(new_retrieved, "new_format_key");

    Ok(())
}

#[tokio::test]
async fn test_migration_from_old_format_to_new_format() -> Result<()> {
    let env = TestEnv::new().await?;

    // Store credentials in old format
    env.credential_manager
        .store("plurcast.nostr", "private_key", "old_key")?;
    env.credential_manager
        .store("plurcast.mastodon", "access_token", "old_token")?;

    // Migrate to new format (default account)
    // In real implementation, this would be done by CredentialManager::migrate_to_multi_account()
    // For this test, we'll manually migrate
    let old_nostr_key = env
        .credential_manager
        .retrieve("plurcast.nostr", "private_key")?;
    env.credential_manager.store_account(
        "plurcast.nostr",
        "private_key",
        "default",
        &old_nostr_key,
    )?;

    let old_mastodon_token = env
        .credential_manager
        .retrieve("plurcast.mastodon", "access_token")?;
    env.credential_manager.store_account(
        "plurcast.mastodon",
        "access_token",
        "default",
        &old_mastodon_token,
    )?;

    // Verify migration
    let migrated_nostr = env
        .credential_manager
        .retrieve_account("plurcast.nostr", "private_key", "default")?;
    assert_eq!(migrated_nostr, "old_key");

    let migrated_mastodon = env
        .credential_manager
        .retrieve_account("plurcast.mastodon", "access_token", "default")?;
    assert_eq!(migrated_mastodon, "old_token");

    Ok(())
}

#[tokio::test]
async fn test_error_invalid_account_names() -> Result<()> {
    let env = TestEnv::new().await?;

    // Test invalid account names
    let too_long = "a".repeat(65);
    let invalid_names = vec![
        "",                  // empty
        "test account",      // space
        "test@account",      // special char
        "test.account",      // dot
        "test/account",      // slash
        too_long.as_str(),   // too long
        "all",               // reserved
        "none",              // reserved
        "list",              // reserved
    ];

    for name in invalid_names {
        let result = env.account_manager.register_account("nostr", name);
        assert!(result.is_err(), "Should reject invalid name: {}", name);
    }

    Ok(())
}

#[tokio::test]
async fn test_error_missing_account() -> Result<()> {
    let env = TestEnv::new().await?;

    // Try to set non-existent account as active
    let result = env
        .account_manager
        .set_active_account("nostr", "nonexistent");
    assert!(result.is_err());

    // Try to retrieve credentials for non-existent account
    let result = env
        .credential_manager
        .retrieve_account("plurcast.nostr", "private_key", "nonexistent");
    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_error_account_not_found_for_platform() -> Result<()> {
    let env = TestEnv::new().await?;

    // Create account for nostr
    env.credential_manager.store_account(
        "plurcast.nostr",
        "private_key",
        "test",
        "test_key",
    )?;
    env.account_manager.register_account("nostr", "test")?;

    // Try to use same account name for different platform (should fail)
    let result = env
        .account_manager
        .set_active_account("mastodon", "test");
    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_account_isolation() -> Result<()> {
    let env = TestEnv::new().await?;

    // Create accounts with same name for different platforms
    env.credential_manager.store_account(
        "plurcast.nostr",
        "private_key",
        "test",
        "nostr_key",
    )?;
    env.account_manager.register_account("nostr", "test")?;

    env.credential_manager.store_account(
        "plurcast.mastodon",
        "access_token",
        "test",
        "mastodon_token",
    )?;
    env.account_manager
        .register_account("mastodon", "test")?;

    // Verify credentials are isolated by platform
    let nostr_cred = env
        .credential_manager
        .retrieve_account("plurcast.nostr", "private_key", "test")?;
    assert_eq!(nostr_cred, "nostr_key");

    let mastodon_cred = env
        .credential_manager
        .retrieve_account("plurcast.mastodon", "access_token", "test")?;
    assert_eq!(mastodon_cred, "mastodon_token");

    // Verify account managers are isolated
    env.account_manager
        .set_active_account("nostr", "test")?;
    env.account_manager
        .set_active_account("mastodon", "test")?;

    assert_eq!(env.account_manager.get_active_account("nostr"), "test");
    assert_eq!(env.account_manager.get_active_account("mastodon"), "test");

    Ok(())
}

#[tokio::test]
async fn test_multi_platform_posting_with_different_accounts() -> Result<()> {
    let env = TestEnv::new().await?;

    // Setup accounts for different platforms
    env.credential_manager.store_account(
        "plurcast.nostr",
        "private_key",
        "nostr-prod",
        "nostr_key",
    )?;
    env.account_manager
        .register_account("nostr", "nostr-prod")?;
    env.account_manager
        .set_active_account("nostr", "nostr-prod")?;

    env.credential_manager.store_account(
        "plurcast.mastodon",
        "access_token",
        "mastodon-test",
        "mastodon_token",
    )?;
    env.account_manager
        .register_account("mastodon", "mastodon-test")?;
    env.account_manager
        .set_active_account("mastodon", "mastodon-test")?;

    // Create platforms
    let mut platforms: Vec<Box<dyn Platform>> = vec![
        Box::new(MockPlatform::success("nostr")),
        Box::new(MockPlatform::success("mastodon")),
    ];

    for platform in &mut platforms {
        platform.authenticate().await?;
    }

    // Post to all platforms
    let poster = MultiPlatformPoster::new(platforms, env.db.clone());
    let post = Post::new("Multi-platform test with different accounts".to_string());
    let results = poster.post_to_all(&post).await;

    // Verify both platforms succeeded
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|r| r.success));

    // Verify active accounts are still correct
    assert_eq!(
        env.account_manager.get_active_account("nostr"),
        "nostr-prod"
    );
    assert_eq!(
        env.account_manager.get_active_account("mastodon"),
        "mastodon-test"
    );

    Ok(())
}

#[tokio::test]
async fn test_deleting_active_account_resets_to_default() -> Result<()> {
    let env = TestEnv::new().await?;

    // Create and activate account
    env.credential_manager.store_account(
        "plurcast.nostr",
        "private_key",
        "test",
        "test_key",
    )?;
    env.account_manager.register_account("nostr", "test")?;
    env.account_manager
        .set_active_account("nostr", "test")?;

    assert_eq!(env.account_manager.get_active_account("nostr"), "test");

    // Delete the active account
    env.credential_manager
        .delete_account("plurcast.nostr", "private_key", "test")?;
    env.account_manager.unregister_account("nostr", "test")?;

    // Should reset to default
    assert_eq!(env.account_manager.get_active_account("nostr"), "default");

    Ok(())
}

#[tokio::test]
async fn test_account_state_persistence_across_restarts() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let state_file = temp_dir.path().join("accounts.toml");

    // First session: create accounts
    {
        let manager = AccountManager::with_path(state_file.clone())?;
        manager.register_account("nostr", "test")?;
        manager.register_account("nostr", "prod")?;
        manager.set_active_account("nostr", "test")?;
    }

    // Second session: verify persistence
    {
        let manager = AccountManager::with_path(state_file)?;
        assert!(manager.account_exists("nostr", "test"));
        assert!(manager.account_exists("nostr", "prod"));
        assert_eq!(manager.get_active_account("nostr"), "test");

        let accounts = manager.list_accounts("nostr");
        assert_eq!(accounts.len(), 2);
        assert!(accounts.contains(&"test".to_string()));
        assert!(accounts.contains(&"prod".to_string()));
    }

    Ok(())
}

#[tokio::test]
async fn test_concurrent_account_operations() -> Result<()> {
    let env = Arc::new(TestEnv::new().await?);

    // Spawn multiple tasks that perform account operations
    let mut handles = vec![];

    for i in 0..10 {
        let env_clone = Arc::clone(&env);
        let handle = tokio::spawn(async move {
            let account_name = format!("account-{}", i);
            let key_value = format!("key-{}", i);

            // Register account
            env_clone
                .credential_manager
                .store_account("plurcast.nostr", "private_key", &account_name, &key_value)
                .unwrap();
            env_clone
                .account_manager
                .register_account("nostr", &account_name)
                .unwrap();

            // Verify it was stored
            let retrieved = env_clone
                .credential_manager
                .retrieve_account("plurcast.nostr", "private_key", &account_name)
                .unwrap();
            assert_eq!(retrieved, key_value);
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await?;
    }

    // Verify all accounts were registered
    let accounts = env.account_manager.list_accounts("nostr");
    assert_eq!(accounts.len(), 10);

    Ok(())
}

#[tokio::test]
async fn test_list_accounts_empty() -> Result<()> {
    let env = TestEnv::new().await?;

    // List accounts for platform with no accounts
    let accounts = env.account_manager.list_accounts("nostr");
    assert!(accounts.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_credential_exists_check() -> Result<()> {
    let env = TestEnv::new().await?;

    // Initially doesn't exist
    assert!(!env
        .credential_manager
        .exists_account("plurcast.nostr", "private_key", "test")?);

    // Store credential
    env.credential_manager.store_account(
        "plurcast.nostr",
        "private_key",
        "test",
        "test_key",
    )?;

    // Now exists
    assert!(env
        .credential_manager
        .exists_account("plurcast.nostr", "private_key", "test")?);

    // Different account doesn't exist
    assert!(!env
        .credential_manager
        .exists_account("plurcast.nostr", "private_key", "other")?);

    Ok(())
}

#[tokio::test]
async fn test_posting_with_explicit_account_override() -> Result<()> {
    let env = TestEnv::new().await?;

    // Setup two accounts
    env.credential_manager.store_account(
        "plurcast.nostr",
        "private_key",
        "default",
        "default_key",
    )?;
    env.account_manager.register_account("nostr", "default")?;

    env.credential_manager.store_account(
        "plurcast.nostr",
        "private_key",
        "test",
        "test_key",
    )?;
    env.account_manager.register_account("nostr", "test")?;

    // Set default as active
    env.account_manager
        .set_active_account("nostr", "default")?;

    // Post using explicit account (should override active account)
    // Note: This would be handled at the CLI level, but we can verify
    // that credentials for different accounts are accessible
    let default_cred = env
        .credential_manager
        .retrieve_account("plurcast.nostr", "private_key", "default")?;
    assert_eq!(default_cred, "default_key");

    let test_cred = env
        .credential_manager
        .retrieve_account("plurcast.nostr", "private_key", "test")?;
    assert_eq!(test_cred, "test_key");

    Ok(())
}

#[tokio::test]
async fn test_account_validation_edge_cases() -> Result<()> {
    let env = TestEnv::new().await?;

    // Valid edge cases
    assert!(env.account_manager.register_account("nostr", "a").is_ok()); // single char
    assert!(env.account_manager.register_account("nostr", "A").is_ok()); // uppercase
    assert!(env.account_manager.register_account("nostr", "0").is_ok()); // digit
    assert!(env.account_manager.register_account("nostr", "_").is_ok()); // underscore
    assert!(env.account_manager.register_account("nostr", "-").is_ok()); // hyphen
    
    let max_length_name = "a".repeat(64);
    assert!(env
        .account_manager
        .register_account("nostr", &max_length_name)
        .is_ok()); // max length

    // Invalid edge cases
    let too_long_name = "a".repeat(65);
    assert!(env
        .account_manager
        .register_account("nostr", &too_long_name)
        .is_err()); // too long
    assert!(env.account_manager.register_account("nostr", "").is_err()); // empty

    Ok(())
}

#[tokio::test]
async fn test_multiple_credential_types_per_account() -> Result<()> {
    let env = TestEnv::new().await?;

    // Store multiple credential types for same account
    env.credential_manager.store_account(
        "plurcast.nostr",
        "private_key",
        "test",
        "test_private_key",
    )?;
    env.credential_manager.store_account(
        "plurcast.nostr",
        "public_key",
        "test",
        "test_public_key",
    )?;

    // Verify both are stored and isolated
    let private_key = env
        .credential_manager
        .retrieve_account("plurcast.nostr", "private_key", "test")?;
    assert_eq!(private_key, "test_private_key");

    let public_key = env
        .credential_manager
        .retrieve_account("plurcast.nostr", "public_key", "test")?;
    assert_eq!(public_key, "test_public_key");

    Ok(())
}
