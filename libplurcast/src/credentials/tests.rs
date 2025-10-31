use super::*;
use crate::credentials::CredentialConfig;
use serial_test::serial;
use std::fs;
use tempfile::TempDir;

// Helper function to create a test config
fn test_config(temp_dir: &TempDir) -> CredentialConfig {
    CredentialConfig {
        storage: StorageBackend::Plain,
        path: temp_dir.path().to_string_lossy().to_string(),
        master_password: None,
    }
}

#[cfg(test)]
mod keyring_store_tests {
    use super::*;

    // Note: These tests may fail in CI environments without keyring support
    // They are marked with #[ignore] by default

    #[test]
    #[ignore] // Run with: cargo test -- --ignored
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
    #[ignore]
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
    #[ignore]
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
    #[ignore]
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

        let expected_file = base_path.join("plurcast.nostr.private_key.age");
        assert!(
            expected_file.exists(),
            "Encrypted file should exist with correct naming"
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
}

#[cfg(test)]
#[allow(deprecated)]
mod plain_file_store_tests {
    use super::*;

    #[test]
    #[serial]
    fn test_plain_store_operations() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let base_path = temp_dir.path().to_path_buf();

        let store = PlainFileStore::new(base_path.clone());

        let service = "plurcast.test";
        let key = "test_key";
        let value = "test_value_plain";

        // Test store
        store.store(service, key, value).expect("Failed to store");

        // Test exists
        assert!(
            store.exists(service, key).expect("Failed to check exists"),
            "Plain credential should exist"
        );

        // Test retrieve
        let retrieved = store.retrieve(service, key).expect("Failed to retrieve");
        assert_eq!(
            retrieved, value,
            "Retrieved plain value should match stored value"
        );

        // Test delete
        store.delete(service, key).expect("Failed to delete");

        // Test exists after delete
        assert!(
            !store
                .exists(service, key)
                .expect("Failed to check exists after delete"),
            "Plain credential should not exist after deletion"
        );
    }

    #[test]
    #[serial]
    fn test_plain_store_legacy_mapping() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let base_path = temp_dir.path().to_path_buf();

        let store = PlainFileStore::new(base_path.clone());

        // Test Nostr mapping
        store
            .store("plurcast.nostr", "private_key", "nostr_key")
            .expect("Failed to store nostr key");
        let nostr_file = base_path.join("nostr.keys");
        assert!(nostr_file.exists(), "nostr.keys file should exist");

        // Test Mastodon mapping
        store
            .store("plurcast.mastodon", "access_token", "mastodon_token")
            .expect("Failed to store mastodon token");
        let mastodon_file = base_path.join("mastodon.token");
        assert!(mastodon_file.exists(), "mastodon.token file should exist");

        // Test Bluesky mapping
        store
            .store("plurcast.bluesky", "app_password", "bluesky_pass")
            .expect("Failed to store bluesky password");
        let bluesky_file = base_path.join("bluesky.auth");
        assert!(bluesky_file.exists(), "bluesky.auth file should exist");
    }

    #[test]
    fn test_plain_store_file_permissions() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let base_path = temp_dir.path().to_path_buf();

            let store = PlainFileStore::new(base_path.clone());

            store
                .store("plurcast.nostr", "private_key", "test_key")
                .expect("Failed to store");

            let file_path = base_path.join("nostr.keys");
            let metadata = fs::metadata(&file_path).expect("Failed to get file metadata");
            let permissions = metadata.permissions();

            assert_eq!(
                permissions.mode() & 0o777,
                0o600,
                "Plain file should have 600 permissions"
            );
        }
    }

    #[test]
    fn test_plain_store_backend_name() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let store = PlainFileStore::new(temp_dir.path().to_path_buf());
        assert_eq!(store.backend_name(), "plain_file");
    }

    #[test]
    #[serial]
    fn test_plain_store_deprecation_warning() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let base_path = temp_dir.path().to_path_buf();

        let store = PlainFileStore::new(base_path);

        // First access should log warning (we can't easily test logging, but we can verify it works)
        store
            .store("plurcast.test", "test_key", "test_value")
            .expect("Failed to store");

        // Second access should not log warning again (internal state tracks this)
        store
            .retrieve("plurcast.test", "test_key")
            .expect("Failed to retrieve");
    }
}

#[cfg(test)]
mod credential_manager_tests {
    use super::*;

    #[test]
    fn test_credential_manager_plain_backend() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config = test_config(&temp_dir);

        let manager = CredentialManager::new(config).expect("Failed to create manager");

        let service = "plurcast.test";
        let key = "test_key";
        let value = "test_value";

        // Test store
        manager
            .store(service, key, value)
            .expect("Failed to store via manager");

        // Test exists
        assert!(
            manager
                .exists(service, key)
                .expect("Failed to check exists via manager"),
            "Credential should exist via manager"
        );

        // Test retrieve
        let retrieved = manager
            .retrieve(service, key)
            .expect("Failed to retrieve via manager");
        assert_eq!(retrieved, value, "Retrieved value should match via manager");

        // Test delete
        manager
            .delete(service, key)
            .expect("Failed to delete via manager");

        // Test exists after delete
        assert!(
            !manager
                .exists(service, key)
                .expect("Failed to check exists after delete via manager"),
            "Credential should not exist after deletion via manager"
        );
    }

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

        // Manager should fall back to plain storage
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
}
