//! Process persistence tests for multi-account credentials
//!
//! These tests verify that credentials stored for multiple accounts persist
//! across process boundaries and restarts.

use anyhow::Result;
use libplurcast::accounts::AccountManager;
use libplurcast::credentials::{CredentialConfig, CredentialManager, StorageBackend};
use tempfile::TempDir;

/// Test helper to create a test environment
struct TestEnv {
    _temp_dir: TempDir,
    state_file: std::path::PathBuf,
    cred_path: String,
}

impl TestEnv {
    fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let state_file = temp_dir.path().join("accounts.toml");
        let cred_path = temp_dir.path().join("credentials").to_string_lossy().to_string();

        Ok(Self {
            _temp_dir: temp_dir,
            state_file,
            cred_path,
        })
    }

    fn create_account_manager(&self) -> Result<AccountManager> {
        Ok(AccountManager::with_path(self.state_file.clone())?)
    }

    fn create_credential_manager(&self) -> Result<CredentialManager> {
        let config = CredentialConfig {
            storage: StorageBackend::Encrypted,
            path: self.cred_path.clone(),
            master_password: Some("test_password_123".to_string()),
        };
        Ok(CredentialManager::new(config)?)
    }
}

#[test]
fn test_credentials_persist_across_manager_instances() -> Result<()> {
    let env = TestEnv::new()?;

    // First instance: store credentials for multiple accounts
    {
        let manager = env.create_credential_manager()?;
        
        manager.store_account("plurcast.nostr", "private_key", "default", "default_key")?;
        manager.store_account("plurcast.nostr", "private_key", "test", "test_key")?;
        manager.store_account("plurcast.nostr", "private_key", "prod", "prod_key")?;
    }

    // Second instance: verify credentials are still accessible
    {
        let manager = env.create_credential_manager()?;
        
        let default_key = manager.retrieve_account("plurcast.nostr", "private_key", "default")?;
        assert_eq!(default_key, "default_key");
        
        let test_key = manager.retrieve_account("plurcast.nostr", "private_key", "test")?;
        assert_eq!(test_key, "test_key");
        
        let prod_key = manager.retrieve_account("plurcast.nostr", "private_key", "prod")?;
        assert_eq!(prod_key, "prod_key");
    }

    Ok(())
}

#[test]
fn test_account_state_persists_across_manager_instances() -> Result<()> {
    let env = TestEnv::new()?;

    // First instance: register accounts and set active
    {
        let manager = env.create_account_manager()?;
        
        manager.register_account("nostr", "default")?;
        manager.register_account("nostr", "test")?;
        manager.register_account("nostr", "prod")?;
        manager.set_active_account("nostr", "test")?;
    }

    // Second instance: verify state persisted
    {
        let manager = env.create_account_manager()?;
        
        assert!(manager.account_exists("nostr", "default"));
        assert!(manager.account_exists("nostr", "test"));
        assert!(manager.account_exists("nostr", "prod"));
        assert_eq!(manager.get_active_account("nostr"), "test");
        
        let accounts = manager.list_accounts("nostr");
        assert_eq!(accounts.len(), 3);
    }

    Ok(())
}

#[test]
fn test_multi_platform_credentials_persist() -> Result<()> {
    let env = TestEnv::new()?;

    // First instance: store credentials for multiple platforms
    {
        let cred_manager = env.create_credential_manager()?;
        let acct_manager = env.create_account_manager()?;
        
        // Nostr accounts
        cred_manager.store_account("plurcast.nostr", "private_key", "test", "nostr_test_key")?;
        acct_manager.register_account("nostr", "test")?;
        
        // Mastodon accounts
        cred_manager.store_account("plurcast.mastodon", "access_token", "test", "mastodon_test_token")?;
        acct_manager.register_account("mastodon", "test")?;
        
        // Bluesky accounts
        cred_manager.store_account("plurcast.bluesky", "app_password", "test", "bluesky_test_pass")?;
        acct_manager.register_account("bluesky", "test")?;
        
        // Set active accounts
        acct_manager.set_active_account("nostr", "test")?;
        acct_manager.set_active_account("mastodon", "test")?;
        acct_manager.set_active_account("bluesky", "test")?;
    }

    // Second instance: verify all platforms persisted
    {
        let cred_manager = env.create_credential_manager()?;
        let acct_manager = env.create_account_manager()?;
        
        // Verify Nostr
        let nostr_key = cred_manager.retrieve_account("plurcast.nostr", "private_key", "test")?;
        assert_eq!(nostr_key, "nostr_test_key");
        assert_eq!(acct_manager.get_active_account("nostr"), "test");
        
        // Verify Mastodon
        let mastodon_token = cred_manager.retrieve_account("plurcast.mastodon", "access_token", "test")?;
        assert_eq!(mastodon_token, "mastodon_test_token");
        assert_eq!(acct_manager.get_active_account("mastodon"), "test");
        
        // Verify Bluesky
        let bluesky_pass = cred_manager.retrieve_account("plurcast.bluesky", "app_password", "test")?;
        assert_eq!(bluesky_pass, "bluesky_test_pass");
        assert_eq!(acct_manager.get_active_account("bluesky"), "test");
    }

    Ok(())
}

#[test]
fn test_account_deletion_persists() -> Result<()> {
    let env = TestEnv::new()?;

    // First instance: create and delete account
    {
        let cred_manager = env.create_credential_manager()?;
        let acct_manager = env.create_account_manager()?;
        
        // Create accounts
        cred_manager.store_account("plurcast.nostr", "private_key", "test", "test_key")?;
        acct_manager.register_account("nostr", "test")?;
        
        cred_manager.store_account("plurcast.nostr", "private_key", "prod", "prod_key")?;
        acct_manager.register_account("nostr", "prod")?;
        
        // Delete test account
        cred_manager.delete_account("plurcast.nostr", "private_key", "test")?;
        acct_manager.unregister_account("nostr", "test")?;
    }

    // Second instance: verify deletion persisted
    {
        let cred_manager = env.create_credential_manager()?;
        let acct_manager = env.create_account_manager()?;
        
        // Test account should not exist
        assert!(!acct_manager.account_exists("nostr", "test"));
        assert!(!cred_manager.exists_account("plurcast.nostr", "private_key", "test")?);
        
        // Prod account should still exist
        assert!(acct_manager.account_exists("nostr", "prod"));
        let prod_key = cred_manager.retrieve_account("plurcast.nostr", "private_key", "prod")?;
        assert_eq!(prod_key, "prod_key");
    }

    Ok(())
}

#[test]
fn test_active_account_changes_persist() -> Result<()> {
    let env = TestEnv::new()?;

    // First instance: create accounts and set active
    {
        let acct_manager = env.create_account_manager()?;
        
        acct_manager.register_account("nostr", "account1")?;
        acct_manager.register_account("nostr", "account2")?;
        acct_manager.set_active_account("nostr", "account1")?;
    }

    // Second instance: verify and change active
    {
        let acct_manager = env.create_account_manager()?;
        assert_eq!(acct_manager.get_active_account("nostr"), "account1");
        
        acct_manager.set_active_account("nostr", "account2")?;
    }

    // Third instance: verify change persisted
    {
        let acct_manager = env.create_account_manager()?;
        assert_eq!(acct_manager.get_active_account("nostr"), "account2");
    }

    Ok(())
}

#[test]
#[cfg(target_os = "windows")]
fn test_keyring_persistence_windows() -> Result<()> {
    let env_vars = TestEnv::new()?;

    // First instance: store credentials using keyring
    {
        let config = CredentialConfig {
            storage: StorageBackend::Keyring,
            path: env_vars.cred_path.clone(),
            master_password: Some("test_password_123".to_string()),
        };
        
        // Try to create keyring manager (may fall back to encrypted)
        if let Ok(manager) = CredentialManager::new(config) {
            manager.store_account("plurcast.nostr", "private_key", "test", "keyring_test_key")?;
            
            // Verify it was stored
            let retrieved = manager.retrieve_account("plurcast.nostr", "private_key", "test")?;
            assert_eq!(retrieved, "keyring_test_key");
        } else {
            // Keyring not available in test environment, skip test
            println!("Keyring not available, skipping Windows keyring test");
            return Ok(());
        }
    }

    // Second instance: verify persistence
    {
        let config = CredentialConfig {
            storage: StorageBackend::Keyring,
            path: env_vars.cred_path.clone(),
            master_password: Some("test_password_123".to_string()),
        };
        
        if let Ok(manager) = CredentialManager::new(config) {
            let retrieved = manager.retrieve_account("plurcast.nostr", "private_key", "test")?;
            assert_eq!(retrieved, "keyring_test_key");
            
            // Clean up
            manager.delete_account("plurcast.nostr", "private_key", "test")?;
        }
    }

    Ok(())
}

#[test]
fn test_concurrent_read_access() -> Result<()> {
    let env = TestEnv::new()?;

    // Store initial credentials
    {
        let cred_manager = env.create_credential_manager()?;
        let acct_manager = env.create_account_manager()?;
        
        cred_manager.store_account("plurcast.nostr", "private_key", "shared", "shared_key")?;
        acct_manager.register_account("nostr", "shared")?;
    }

    // Simulate concurrent read access by creating multiple manager instances
    let handles: Vec<_> = (0..5)
        .map(|_i| {
            let state_file = env.state_file.clone();
            let cred_path = env.cred_path.clone();
            
            std::thread::spawn(move || -> Result<()> {
                let acct_manager = AccountManager::with_path(state_file)?;
                let config = CredentialConfig {
                    storage: StorageBackend::Encrypted,
                    path: cred_path,
                    master_password: Some("test_password_123".to_string()),
                };
                let cred_manager = CredentialManager::new(config)?;
                
                // Each thread reads the shared credential
                let key = cred_manager.retrieve_account("plurcast.nostr", "private_key", "shared")?;
                assert_eq!(key, "shared_key");
                
                // Each thread checks account exists
                assert!(acct_manager.account_exists("nostr", "shared"));
                
                Ok(())
            })
        })
        .collect();

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap()?;
    }

    // Verify shared account still exists
    {
        let acct_manager = env.create_account_manager()?;
        assert!(acct_manager.account_exists("nostr", "shared"));
    }

    Ok(())
}

#[test]
fn test_credential_update_persists() -> Result<()> {
    let env = TestEnv::new()?;

    // First instance: store initial credential
    {
        let manager = env.create_credential_manager()?;
        manager.store_account("plurcast.nostr", "private_key", "test", "initial_key")?;
    }

    // Second instance: update credential
    {
        let manager = env.create_credential_manager()?;
        let initial = manager.retrieve_account("plurcast.nostr", "private_key", "test")?;
        assert_eq!(initial, "initial_key");
        
        manager.store_account("plurcast.nostr", "private_key", "test", "updated_key")?;
    }

    // Third instance: verify update persisted
    {
        let manager = env.create_credential_manager()?;
        let updated = manager.retrieve_account("plurcast.nostr", "private_key", "test")?;
        assert_eq!(updated, "updated_key");
    }

    Ok(())
}

#[test]
fn test_empty_state_initialization() -> Result<()> {
    let env = TestEnv::new()?;

    // Create manager with no existing state
    let acct_manager = env.create_account_manager()?;
    
    // Should have default state
    assert_eq!(acct_manager.get_active_account("nostr"), "default");
    assert!(acct_manager.list_accounts("nostr").is_empty());
    
    // State file should be created on first write
    acct_manager.register_account("nostr", "test")?;
    assert!(env.state_file.exists());

    Ok(())
}

#[test]
fn test_large_number_of_accounts_persist() -> Result<()> {
    let env = TestEnv::new()?;

    // First instance: create many accounts
    {
        let cred_manager = env.create_credential_manager()?;
        let acct_manager = env.create_account_manager()?;
        
        for i in 0..50 {
            let account_name = format!("account-{}", i);
            let key_value = format!("key-{}", i);
            
            cred_manager.store_account("plurcast.nostr", "private_key", &account_name, &key_value)?;
            acct_manager.register_account("nostr", &account_name)?;
        }
    }

    // Second instance: verify all accounts persisted
    {
        let cred_manager = env.create_credential_manager()?;
        let acct_manager = env.create_account_manager()?;
        
        let accounts = acct_manager.list_accounts("nostr");
        assert_eq!(accounts.len(), 50);
        
        // Spot check a few accounts
        for i in [0, 10, 25, 49] {
            let account_name = format!("account-{}", i);
            let expected_key = format!("key-{}", i);
            
            assert!(acct_manager.account_exists("nostr", &account_name));
            let key = cred_manager.retrieve_account("plurcast.nostr", "private_key", &account_name)?;
            assert_eq!(key, expected_key);
        }
    }

    Ok(())
}
