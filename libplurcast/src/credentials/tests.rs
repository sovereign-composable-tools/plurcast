use super::*;
use crate::credentials::CredentialConfig;
use serial_test::serial;
use std::fs;
use tempfile::TempDir;

// Helper function to create a test config with encrypted storage
fn test_config(temp_dir: &TempDir) -> CredentialConfig {
    CredentialConfig {
        storage: StorageBackend::Encrypted,
        path: temp_dir.path().to_string_lossy().to_string(),
        master_password: Some("test_password_123".to_string()),
    }
}

#[cfg(test)]
mod keyring_store_tests {
    use super::*;

    #[test]
    #[serial] // Serialize keyring tests to avoid conflicts
    fn test_keyring_store_operations() {
        let store = KeyringStore::new().expect("Failed to create KeyringStore");
        let service = "plurcast.test";
        let key = "test_key";
        let value = "test_value_12345";

        // Clean up any existing test data
        let _ = store.delete(service, key);

        // Test store
        store.store(service, key, value).expect("Failed to store");

        // Test exists
        assert!(
            store.exists(service, key).expect("Failed to check exists"),
            "Credential should exist after storing"
        );

        // Test retrieve
        let retrieved = store.retrieve(service, key).expect("Failed to retrieve");
        assert_eq!(
            retrieved, value,
            "Retrieved value should match stored value"
        );

        // Test delete
        store.delete(service, key).expect("Failed to delete");

        // Test exists after delete
        assert!(
            !store
                .exists(service, key)
                .expect("Failed to check exists after delete"),
            "Credential should not exist after deletion"
        );
    }

    #[test]
    #[serial]
    fn test_keyring_retrieve_nonexistent() {
        let store = KeyringStore::new().expect("Failed to create KeyringStore");
        let result = store.retrieve("plurcast.test", "nonexistent_key");

        assert!(result.is_err(), "Should fail to retrieve nonexistent key");
        match result {
            Err(crate::error::PlurcastError::Credential(CredentialError::NotFound(_))) => {
                // Expected error
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_keyring_backend_name() {
        let store = KeyringStore::new().expect("Failed to create KeyringStore");
        assert_eq!(store.backend_name(), "keyring");
    }

    #[test]
    #[serial]
    fn test_keyring_service_naming() {
        let store = KeyringStore::new().expect("Failed to create KeyringStore");
        let service = "plurcast.test_service_naming";
        let key = "test_key";
        let value = "test_nostr_key";

        // Clean up
        let _ = store.delete(service, key);

        // Store and retrieve
        store.store(service, key, value).expect("Failed to store");
        let retrieved = store.retrieve(service, key).expect("Failed to retrieve");
        assert_eq!(retrieved, value);

        // Clean up
        store.delete(service, key).expect("Failed to delete");
    }

    #[test]
    #[serial]
    fn test_keyring_multiple_platforms() {
        let store = KeyringStore::new().expect("Failed to create KeyringStore");

        let platforms = vec![
            ("plurcast.nostr", "private_key", "nostr_key_123"),
            ("plurcast.mastodon", "access_token", "mastodon_token_456"),
            ("plurcast.bluesky", "app_password", "bluesky_pass_789"),
        ];

        // Clean up
        for (service, key, _) in &platforms {
            let _ = store.delete(service, key);
        }

        // Store all
        for (service, key, value) in &platforms {
            store
                .store(service, key, value)
                .expect("Failed to store credential");
        }

        // Verify all exist
        for (service, key, _) in &platforms {
            assert!(
                store.exists(service, key).expect("Failed to check exists"),
                "Credential should exist for {}.{}",
                service,
                key
            );
        }

        // Retrieve and verify all
        for (service, key, expected_value) in &platforms {
            let retrieved = store
                .retrieve(service, key)
                .expect("Failed to retrieve credential");
            assert_eq!(
                &retrieved, expected_value,
                "Value mismatch for {}.{}",
                service, key
            );
        }

        // Clean up
        for (service, key, _) in &platforms {
            store.delete(service, key).expect("Failed to delete");
        }
    }

    #[test]
    fn test_keyring_namespace_derivation() {
        // Test the keyring_key helper function
        let (service, key) = KeyringStore::keyring_key("plurcast.nostr", "test-account", "private_key");
        assert_eq!(service, "plurcast.nostr.test-account");
        assert_eq!(key, "private_key");

        let (service, key) = KeyringStore::keyring_key("plurcast.mastodon", "prod", "access_token");
        assert_eq!(service, "plurcast.mastodon.prod");
        assert_eq!(key, "access_token");
    }

    #[test]
    #[serial]
    fn test_keyring_multi_account_operations() {
        let store = KeyringStore::new().expect("Failed to create KeyringStore");
        let service = "plurcast.test";
        let key = "test_key";

        // Clean up
        let _ = store.delete_account(service, key, "default");
        let _ = store.delete_account(service, key, "test-account");
        let _ = store.delete_account(service, key, "prod-account");

        // Store credentials for multiple accounts
        store
            .store_account(service, key, "default", "default_value")
            .expect("Failed to store default account");
        store
            .store_account(service, key, "test-account", "test_value")
            .expect("Failed to store test account");
        store
            .store_account(service, key, "prod-account", "prod_value")
            .expect("Failed to store prod account");

        // Verify all exist
        assert!(
            store
                .exists_account(service, key, "default")
                .expect("Failed to check default exists"),
            "Default account should exist"
        );
        assert!(
            store
                .exists_account(service, key, "test-account")
                .expect("Failed to check test exists"),
            "Test account should exist"
        );
        assert!(
            store
                .exists_account(service, key, "prod-account")
                .expect("Failed to check prod exists"),
            "Prod account should exist"
        );

        // Retrieve and verify each account
        let default_val = store
            .retrieve_account(service, key, "default")
            .expect("Failed to retrieve default");
        assert_eq!(default_val, "default_value");

        let test_val = store
            .retrieve_account(service, key, "test-account")
            .expect("Failed to retrieve test");
        assert_eq!(test_val, "test_value");

        let prod_val = store
            .retrieve_account(service, key, "prod-account")
            .expect("Failed to retrieve prod");
        assert_eq!(prod_val, "prod_value");

        // Test account isolation - delete one account shouldn't affect others
        store
            .delete_account(service, key, "test-account")
            .expect("Failed to delete test account");

        assert!(
            !store
                .exists_account(service, key, "test-account")
                .expect("Failed to check test exists after delete"),
            "Test account should not exist after deletion"
        );
        assert!(
            store
                .exists_account(service, key, "default")
                .expect("Failed to check default exists after test delete"),
            "Default account should still exist"
        );
        assert!(
            store
                .exists_account(service, key, "prod-account")
                .expect("Failed to check prod exists after test delete"),
            "Prod account should still exist"
        );

        // Clean up
        let _ = store.delete_account(service, key, "default");
        let _ = store.delete_account(service, key, "prod-account");
    }

    #[test]
    #[serial]
    fn test_keyring_backward_compatibility() {
        let store = KeyringStore::new().expect("Failed to create KeyringStore");
        let service = "plurcast.test";
        let key = "compat_key";
        let value = "compat_value";

        // Clean up
        let _ = store.delete(service, key);

        // Store using old method (should delegate to default account)
        store.store(service, key, value).expect("Failed to store");

        // Retrieve using new method with "default" account
        let retrieved = store
            .retrieve_account(service, key, "default")
            .expect("Failed to retrieve with account");
        assert_eq!(retrieved, value, "Backward compatibility should work");

        // Retrieve using old method
        let retrieved_old = store.retrieve(service, key).expect("Failed to retrieve old way");
        assert_eq!(
            retrieved_old, value,
            "Old method should still work via delegation"
        );

        // Clean up
        let _ = store.delete(service, key);
    }
}

#[cfg(test)]
mod encrypted_file_store_tests {
    use super::*;

    #[test]
    fn test_encrypted_store_operations() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let base_path = temp_dir.path().to_path_buf();

        let store = EncryptedFileStore::new(base_path.clone());
        store
            .set_master_password("test_password_123".to_string())
            .expect("Failed to set master password");

        let service = "plurcast.test";
        let key = "test_key";
        let value = "test_value_encrypted";

        // Test store
        store
            .store(service, key, value)
            .expect("Failed to store encrypted");

        // Test exists
        assert!(
            store.exists(service, key).expect("Failed to check exists"),
            "Encrypted credential should exist"
        );

        // Test retrieve
        let retrieved = store
            .retrieve(service, key)
            .expect("Failed to retrieve encrypted");
        assert_eq!(
            retrieved, value,
            "Retrieved encrypted value should match stored value"
        );

        // Test delete
        store
            .delete(service, key)
            .expect("Failed to delete encrypted");

        // Test exists after delete
        assert!(
            !store
                .exists(service, key)
                .expect("Failed to check exists after delete"),
            "Encrypted credential should not exist after deletion"
        );
    }

    #[test]
    fn test_encrypted_store_weak_password() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let base_path = temp_dir.path().to_path_buf();

        let store = EncryptedFileStore::new(base_path);
        let result = store.set_master_password("short".to_string());

        assert!(
            result.is_err(),
            "Should reject password shorter than 8 characters"
        );
        match result {
            Err(crate::error::PlurcastError::Credential(CredentialError::WeakPassword)) => {
                // Expected error
            }
            _ => panic!("Expected WeakPassword error"),
        }
    }

    #[test]
    fn test_encrypted_store_no_password_set() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let base_path = temp_dir.path().to_path_buf();

        let store = EncryptedFileStore::new(base_path);
        let result = store.store("plurcast.test", "key", "value");

        assert!(
            result.is_err(),
            "Should fail to store without master password"
        );
        match result {
            Err(crate::error::PlurcastError::Credential(CredentialError::MasterPasswordNotSet)) => {
                // Expected error
            }
            _ => panic!("Expected MasterPasswordNotSet error"),
        }
    }

    #[test]
    fn test_encrypted_store_file_permissions() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let base_path = temp_dir.path().to_path_buf();

            let mut store = EncryptedFileStore::new(base_path.clone());
            store
                .set_master_password("test_password_123".to_string())
                .expect("Failed to set master password");

            let service = "plurcast.test";
            let key = "test_key";
            let value = "test_value";

            store
                .store(service, key, value)
                .expect("Failed to store encrypted");

            // Check file permissions
            let file_path = base_path.join(format!("{}.{}.age", service, key));
            let metadata = fs::metadata(&file_path).expect("Failed to get file metadata");
            let permissions = metadata.permissions();

            assert_eq!(
                permissions.mode() & 0o777,
                0o600,
                "Encrypted file should have 600 permissions"
            );
        }
    }

    #[test]
    fn test_encrypted_store_file_naming() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let base_path = temp_dir.path().to_path_buf();

        let store = EncryptedFileStore::new(base_path.clone());
        store
            .set_master_password("test_password_123".to_string())
            .expect("Failed to set master password");

        store
            .store("plurcast.nostr", "private_key", "test_key")
            .expect("Failed to store");

        // store() delegates to store_account() with "default" account
        let expected_file = base_path.join("plurcast.nostr.default.private_key.age");
        assert!(
            expected_file.exists(),
            "Encrypted file should exist with correct naming (default account)"
        );
    }

    #[test]
    fn test_encrypted_store_backend_name() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let store = EncryptedFileStore::new(temp_dir.path().to_path_buf());
        assert_eq!(store.backend_name(), "encrypted_file");
    }

    #[test]
    fn test_encrypted_store_corrupted_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let base_path = temp_dir.path().to_path_buf();

        let store = EncryptedFileStore::new(base_path.clone());
        store
            .set_master_password("test_password_123".to_string())
            .expect("Failed to set master password");

        let service = "plurcast.test";
        let key = "test_key";

        // Create a corrupted file
        let file_path = base_path.join(format!("{}.{}.age", service, key));
        fs::write(&file_path, b"corrupted data").expect("Failed to write corrupted file");

        // Try to retrieve
        let result = store.retrieve(service, key);
        assert!(
            result.is_err(),
            "Should fail to retrieve from corrupted file"
        );
    }

    #[test]
    fn test_encrypted_store_multi_account_operations() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let base_path = temp_dir.path().to_path_buf();

        let store = EncryptedFileStore::new(base_path.clone());
        store
            .set_master_password("test_password_123".to_string())
            .expect("Failed to set master password");

        let service = "plurcast.test";
        let key = "test_key";

        // Store credentials for multiple accounts
        store
            .store_account(service, key, "default", "default_value")
            .expect("Failed to store default account");
        store
            .store_account(service, key, "test-account", "test_value")
            .expect("Failed to store test account");
        store
            .store_account(service, key, "prod-account", "prod_value")
            .expect("Failed to store prod account");

        // Verify all exist
        assert!(
            store
                .exists_account(service, key, "default")
                .expect("Failed to check default exists"),
            "Default account should exist"
        );
        assert!(
            store
                .exists_account(service, key, "test-account")
                .expect("Failed to check test exists"),
            "Test account should exist"
        );
        assert!(
            store
                .exists_account(service, key, "prod-account")
                .expect("Failed to check prod exists"),
            "Prod account should exist"
        );

        // Retrieve and verify each account
        let default_val = store
            .retrieve_account(service, key, "default")
            .expect("Failed to retrieve default");
        assert_eq!(default_val, "default_value");

        let test_val = store
            .retrieve_account(service, key, "test-account")
            .expect("Failed to retrieve test");
        assert_eq!(test_val, "test_value");

        let prod_val = store
            .retrieve_account(service, key, "prod-account")
            .expect("Failed to retrieve prod");
        assert_eq!(prod_val, "prod_value");

        // Test account isolation
        store
            .delete_account(service, key, "test-account")
            .expect("Failed to delete test account");

        assert!(
            !store
                .exists_account(service, key, "test-account")
                .expect("Failed to check test exists after delete"),
            "Test account should not exist after deletion"
        );
        assert!(
            store
                .exists_account(service, key, "default")
                .expect("Failed to check default exists after test delete"),
            "Default account should still exist"
        );
    }

    #[test]
    fn test_encrypted_store_filename_generation() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let base_path = temp_dir.path().to_path_buf();

        let store = EncryptedFileStore::new(base_path.clone());
        store
            .set_master_password("test_password_123".to_string())
            .expect("Failed to set master password");

        store
            .store_account("plurcast.nostr", "private_key", "test-account", "test_key")
            .expect("Failed to store");

        let expected_file = base_path.join("plurcast.nostr.test-account.private_key.age");
        assert!(
            expected_file.exists(),
            "Encrypted file should exist with correct multi-account naming"
        );
    }

    #[test]
    fn test_encrypted_store_list_accounts() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let base_path = temp_dir.path().to_path_buf();

        let store = EncryptedFileStore::new(base_path.clone());
        store
            .set_master_password("test_password_123".to_string())
            .expect("Failed to set master password");

        let service = "plurcast.nostr";
        let key = "private_key";

        // Store multiple accounts
        store
            .store_account(service, key, "default", "default_value")
            .expect("Failed to store default");
        store
            .store_account(service, key, "test-account", "test_value")
            .expect("Failed to store test");
        store
            .store_account(service, key, "prod-account", "prod_value")
            .expect("Failed to store prod");

        // List accounts
        let accounts = store
            .list_accounts(service, key)
            .expect("Failed to list accounts");

        assert_eq!(accounts.len(), 3, "Should find 3 accounts");
        assert!(accounts.contains(&"default".to_string()));
        assert!(accounts.contains(&"test-account".to_string()));
        assert!(accounts.contains(&"prod-account".to_string()));
    }

    #[test]
    fn test_encrypted_store_backward_compatibility() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let base_path = temp_dir.path().to_path_buf();

        let store = EncryptedFileStore::new(base_path.clone());
        store
            .set_master_password("test_password_123".to_string())
            .expect("Failed to set master password");

        let service = "plurcast.test";
        let key = "compat_key";
        let value = "compat_value";

        // Store using old method (should delegate to default account)
        store.store(service, key, value).expect("Failed to store");

        // Retrieve using new method with "default" account
        let retrieved = store
            .retrieve_account(service, key, "default")
            .expect("Failed to retrieve with account");
        assert_eq!(retrieved, value, "Backward compatibility should work");

        // Retrieve using old method
        let retrieved_old = store.retrieve(service, key).expect("Failed to retrieve old way");
        assert_eq!(
            retrieved_old, value,
            "Old method should still work via delegation"
        );
    }
}

#[cfg(test)]
mod credential_manager_tests {
    use super::*;

    #[test]
    fn test_credential_manager_encrypted_backend() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let mut config = test_config(&temp_dir);
        config.storage = StorageBackend::Encrypted;
        config.master_password = Some("test_password_123".to_string());

        let manager = CredentialManager::new(config).expect("Failed to create manager");

        let service = "plurcast.test";
        let key = "test_key";
        let value = "test_value_encrypted";

        manager
            .store(service, key, value)
            .expect("Failed to store encrypted via manager");

        let retrieved = manager
            .retrieve(service, key)
            .expect("Failed to retrieve encrypted via manager");
        assert_eq!(
            retrieved, value,
            "Retrieved encrypted value should match via manager"
        );
    }

    #[test]
    fn test_credential_manager_fallback_logic() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let mut config = test_config(&temp_dir);

        // Try keyring first (will likely fail in test environment)
        config.storage = StorageBackend::Keyring;
        // Ensure we have encrypted storage as fallback
        config.master_password = Some("test_password_123".to_string());

        // Manager should fall back to encrypted storage
        let manager = CredentialManager::new(config).expect("Failed to create manager");

        let service = "plurcast.test";
        let key = "test_key";
        let value = "test_value_fallback";

        manager
            .store(service, key, value)
            .expect("Failed to store with fallback");

        let retrieved = manager
            .retrieve(service, key)
            .expect("Failed to retrieve with fallback");
        assert_eq!(
            retrieved, value,
            "Retrieved value should match with fallback"
        );
    }

    #[test]
    fn test_credential_manager_multiple_platforms() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config = test_config(&temp_dir);

        let manager = CredentialManager::new(config).expect("Failed to create manager");

        let credentials = vec![
            ("plurcast.nostr", "private_key", "nostr_key_123"),
            ("plurcast.mastodon", "access_token", "mastodon_token_456"),
            ("plurcast.bluesky", "app_password", "bluesky_pass_789"),
        ];

        // Store all
        for (service, key, value) in &credentials {
            manager
                .store(service, key, value)
                .expect("Failed to store credential");
        }

        // Retrieve and verify all
        for (service, key, expected_value) in &credentials {
            let retrieved = manager
                .retrieve(service, key)
                .expect("Failed to retrieve credential");
            assert_eq!(
                &retrieved, expected_value,
                "Value mismatch for {}.{}",
                service, key
            );
        }

        // Delete all
        for (service, key, _) in &credentials {
            manager
                .delete(service, key)
                .expect("Failed to delete credential");
        }

        // Verify all deleted
        for (service, key, _) in &credentials {
            assert!(
                !manager
                    .exists(service, key)
                    .expect("Failed to check exists after delete"),
                "Credential should not exist for {}.{}",
                service,
                key
            );
        }
    }

    #[test]
    fn test_credential_manager_multi_account_store_retrieve() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config = test_config(&temp_dir);

        let manager = CredentialManager::new(config).expect("Failed to create manager");

        let service = "plurcast.nostr";
        let key = "private_key";

        // Store credentials for multiple accounts
        manager
            .store_account(service, key, "default", "default_key_123")
            .expect("Failed to store default account");
        manager
            .store_account(service, key, "test-account", "test_key_456")
            .expect("Failed to store test account");
        manager
            .store_account(service, key, "prod-account", "prod_key_789")
            .expect("Failed to store prod account");

        // Retrieve and verify each account
        let default_val = manager
            .retrieve_account(service, key, "default")
            .expect("Failed to retrieve default");
        assert_eq!(default_val, "default_key_123");

        let test_val = manager
            .retrieve_account(service, key, "test-account")
            .expect("Failed to retrieve test");
        assert_eq!(test_val, "test_key_456");

        let prod_val = manager
            .retrieve_account(service, key, "prod-account")
            .expect("Failed to retrieve prod");
        assert_eq!(prod_val, "prod_key_789");
    }

    #[test]
    fn test_credential_manager_multi_account_isolation() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config = test_config(&temp_dir);

        let manager = CredentialManager::new(config).expect("Failed to create manager");

        let service = "plurcast.nostr";
        let key = "private_key";

        // Store credentials for test-account and prod-account
        manager
            .store_account(service, key, "test-account", "test_key")
            .expect("Failed to store test account");
        manager
            .store_account(service, key, "prod-account", "prod_key")
            .expect("Failed to store prod account");

        // Verify both exist
        assert!(
            manager
                .exists_account(service, key, "test-account")
                .expect("Failed to check test exists"),
            "Test account should exist"
        );
        assert!(
            manager
                .exists_account(service, key, "prod-account")
                .expect("Failed to check prod exists"),
            "Prod account should exist"
        );

        // Delete test-account
        manager
            .delete_account(service, key, "test-account")
            .expect("Failed to delete test account");

        // Verify test-account deleted but prod-account still exists
        assert!(
            !manager
                .exists_account(service, key, "test-account")
                .expect("Failed to check test exists after delete"),
            "Test account should not exist after deletion"
        );
        assert!(
            manager
                .exists_account(service, key, "prod-account")
                .expect("Failed to check prod exists after test delete"),
            "Prod account should still exist after test deletion"
        );

        // Verify prod-account value unchanged
        let prod_val = manager
            .retrieve_account(service, key, "prod-account")
            .expect("Failed to retrieve prod after test delete");
        assert_eq!(
            prod_val, "prod_key",
            "Prod account value should be unchanged"
        );
    }

    #[test]
    fn test_credential_manager_multi_account_fallback() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config = test_config(&temp_dir);

        let manager = CredentialManager::new(config).expect("Failed to create manager");

        let service = "plurcast.nostr";
        let key = "private_key";
        let account = "test-account";
        let value = "test_key_fallback";

        // Store credential
        manager
            .store_account(service, key, account, value)
            .expect("Failed to store");

        // Retrieve should work (fallback logic tries all backends)
        let retrieved = manager
            .retrieve_account(service, key, account)
            .expect("Failed to retrieve with fallback");
        assert_eq!(
            retrieved, value,
            "Fallback logic should retrieve credential"
        );
    }

    #[test]
    fn test_credential_manager_list_accounts() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config = test_config(&temp_dir);

        let manager = CredentialManager::new(config).expect("Failed to create manager");

        let service = "plurcast.nostr";
        let key = "private_key";

        // Initially no accounts
        let accounts = manager
            .list_accounts(service, key)
            .expect("Failed to list accounts");
        assert_eq!(accounts.len(), 0, "Should have no accounts initially");

        // Store multiple accounts
        manager
            .store_account(service, key, "default", "default_key")
            .expect("Failed to store default");
        manager
            .store_account(service, key, "test-account", "test_key")
            .expect("Failed to store test");
        manager
            .store_account(service, key, "prod-account", "prod_key")
            .expect("Failed to store prod");

        // List accounts
        let accounts = manager
            .list_accounts(service, key)
            .expect("Failed to list accounts");

        assert_eq!(accounts.len(), 3, "Should have 3 accounts");
        assert!(accounts.contains(&"default".to_string()));
        assert!(accounts.contains(&"test-account".to_string()));
        assert!(accounts.contains(&"prod-account".to_string()));
    }

    #[test]
    fn test_credential_manager_auto_migrate() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config = test_config(&temp_dir);

        let manager = CredentialManager::new(config).expect("Failed to create manager");

        let service = "plurcast.nostr";
        let key = "private_key";
        let old_value = "old_format_key";

        // Store credential in old format (using single-account method)
        manager
            .store(service, key, old_value)
            .expect("Failed to store old format");

        // Verify it exists in old format
        assert!(
            manager.exists(service, key).expect("Failed to check old format exists"),
            "Old format credential should exist"
        );

        // Run auto-migration
        manager
            .auto_migrate_if_needed()
            .expect("Failed to auto-migrate");

        // Verify credential now exists in new format (default account)
        assert!(
            manager
                .exists_account(service, key, "default")
                .expect("Failed to check new format exists"),
            "New format credential should exist after migration"
        );

        // Verify value is correct
        let migrated_value = manager
            .retrieve_account(service, key, "default")
            .expect("Failed to retrieve migrated credential");
        assert_eq!(
            migrated_value, old_value,
            "Migrated value should match original"
        );

        // Verify old format still exists (backward compatibility)
        assert!(
            manager.exists(service, key).expect("Failed to check old format after migration"),
            "Old format should still exist for backward compatibility"
        );
    }

    #[test]
    fn test_credential_manager_manual_migration() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config = test_config(&temp_dir);

        let manager = CredentialManager::new(config).expect("Failed to create manager");

        // Note: With the current implementation, store() delegates to store_account() with "default",
        // so there's no way to create "old format" credentials. This test verifies that the
        // migration method works correctly when credentials are already in the new format.

        // Store credentials using the current (new) format
        manager
            .store("plurcast.nostr", "private_key", "nostr_key")
            .expect("Failed to store nostr");
        manager
            .store("plurcast.mastodon", "access_token", "mastodon_token")
            .expect("Failed to store mastodon");

        // Run manual migration - should skip these since they're already in new format
        let report = manager
            .migrate_to_multi_account()
            .expect("Failed to run manual migration");

        // Since credentials are already in new format (default account), they should be skipped
        assert_eq!(
            report.skipped.len(),
            2,
            "Should have skipped 2 credentials (already in new format)"
        );
        assert!(report.skipped.contains(&"plurcast.nostr.private_key".to_string()));
        assert!(report.skipped.contains(&"plurcast.mastodon.access_token".to_string()));
        assert_eq!(report.migrated.len(), 0, "Should not have migrated anything");
        assert_eq!(report.failed.len(), 0, "Should have no failures");

        // Verify credentials still exist and are accessible
        let nostr_val = manager
            .retrieve_account("plurcast.nostr", "private_key", "default")
            .expect("Failed to retrieve nostr");
        assert_eq!(nostr_val, "nostr_key");

        let mastodon_val = manager
            .retrieve_account("plurcast.mastodon", "access_token", "default")
            .expect("Failed to retrieve mastodon");
        assert_eq!(mastodon_val, "mastodon_token");
    }

    #[test]
    fn test_credential_manager_migration_skip_existing() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config = test_config(&temp_dir);

        let manager = CredentialManager::new(config).expect("Failed to create manager");

        let service = "plurcast.nostr";
        let key = "private_key";

        // Store in both old and new format
        manager
            .store(service, key, "old_value")
            .expect("Failed to store old format");
        manager
            .store_account(service, key, "default", "new_value")
            .expect("Failed to store new format");

        // Run migration
        let report = manager
            .migrate_to_multi_account()
            .expect("Failed to run migration");

        // Should skip because new format already exists
        assert_eq!(
            report.skipped.len(),
            1,
            "Should have skipped 1 credential"
        );
        assert!(report.skipped.contains(&format!("{}.{}", service, key)));
        assert_eq!(report.migrated.len(), 0, "Should not have migrated anything");

        // Verify new format value is unchanged
        let value = manager
            .retrieve_account(service, key, "default")
            .expect("Failed to retrieve");
        assert_eq!(
            value, "new_value",
            "New format value should be unchanged"
        );
    }

    #[test]
    fn test_credential_manager_migration_no_old_credentials() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config = test_config(&temp_dir);

        let manager = CredentialManager::new(config).expect("Failed to create manager");

        // Run migration with no old credentials
        let report = manager
            .migrate_to_multi_account()
            .expect("Failed to run migration");

        // Should have nothing to migrate
        assert_eq!(report.migrated.len(), 0, "Should have migrated nothing");
        assert_eq!(report.failed.len(), 0, "Should have no failures");
        assert_eq!(report.skipped.len(), 0, "Should have skipped nothing");
        assert_eq!(report.total(), 0, "Total should be 0");
    }
}
