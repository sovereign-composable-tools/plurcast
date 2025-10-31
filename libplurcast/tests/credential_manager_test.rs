use libplurcast::credentials::{CredentialConfig, CredentialManager, StorageBackend};
use tempfile::TempDir;

#[test]
fn test_credential_manager_basic_operations() {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_str().unwrap().to_string();

    // Configure to use plain text storage for testing
    let config = CredentialConfig {
        storage: StorageBackend::Plain,
        path: temp_path.clone(),
        master_password: None,
    };

    let manager = CredentialManager::new(config).unwrap();

    // Test store
    manager
        .store("plurcast.test", "test_key", "test_value")
        .unwrap();

    // Test exists
    assert!(manager.exists("plurcast.test", "test_key").unwrap());

    // Test retrieve
    let value = manager.retrieve("plurcast.test", "test_key").unwrap();
    assert_eq!(value, "test_value");

    // Test delete
    manager.delete("plurcast.test", "test_key").unwrap();

    // Verify deletion
    assert!(!manager.exists("plurcast.test", "test_key").unwrap());
}

#[test]
fn test_credential_manager_encrypted_storage() {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_str().unwrap().to_string();

    // Configure to use encrypted storage with a master password
    let config = CredentialConfig {
        storage: StorageBackend::Encrypted,
        path: temp_path.clone(),
        master_password: Some("test-password-12345".to_string()),
    };

    let manager = CredentialManager::new(config).unwrap();

    // Test store
    manager
        .store("plurcast.test", "encrypted_key", "secret_value")
        .unwrap();

    // Test retrieve
    let value = manager.retrieve("plurcast.test", "encrypted_key").unwrap();
    assert_eq!(value, "secret_value");

    // Test exists
    assert!(manager.exists("plurcast.test", "encrypted_key").unwrap());

    // Test delete
    manager.delete("plurcast.test", "encrypted_key").unwrap();
    assert!(!manager.exists("plurcast.test", "encrypted_key").unwrap());
}

#[test]
fn test_credential_manager_fallback_logic() {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_str().unwrap().to_string();

    // Configure to use keyring (which will likely fail in CI)
    // Should fall back to plain text
    let config = CredentialConfig {
        storage: StorageBackend::Keyring,
        path: temp_path.clone(),
        master_password: None,
    };

    let manager = CredentialManager::new(config).unwrap();

    // Should still work with fallback
    manager
        .store("plurcast.test", "fallback_key", "fallback_value")
        .unwrap();
    let value = manager.retrieve("plurcast.test", "fallback_key").unwrap();
    assert_eq!(value, "fallback_value");
}

#[test]
fn test_credential_manager_retrieve_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_str().unwrap().to_string();

    let config = CredentialConfig {
        storage: StorageBackend::Plain,
        path: temp_path,
        master_password: None,
    };

    let manager = CredentialManager::new(config).unwrap();

    // Try to retrieve non-existent credential
    let result = manager.retrieve("plurcast.test", "nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_credential_manager_backends() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_str().unwrap().to_string();

    let config = CredentialConfig {
        storage: StorageBackend::Plain,
        path: temp_path,
        master_password: None,
    };

    let manager = CredentialManager::new(config).unwrap();

    // Check that at least one backend is available
    let backends = manager.backends();
    assert!(!backends.is_empty());
    assert!(backends.contains(&"plain_file"));
}

#[test]
fn test_credential_manager_security_checks() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_str().unwrap().to_string();

    // Plain text storage should be marked as insecure
    let plain_config = CredentialConfig {
        storage: StorageBackend::Plain,
        path: temp_path.clone(),
        master_password: None,
    };

    let plain_manager = CredentialManager::new(plain_config).unwrap();
    assert!(plain_manager.is_insecure());
    assert_eq!(plain_manager.primary_backend(), Some("plain_file"));

    // Encrypted storage should NOT be marked as insecure
    let encrypted_config = CredentialConfig {
        storage: StorageBackend::Encrypted,
        path: temp_path.clone(),
        master_password: Some("secure-password-123".to_string()),
    };

    let encrypted_manager = CredentialManager::new(encrypted_config).unwrap();
    assert!(!encrypted_manager.is_insecure());
    assert_eq!(encrypted_manager.primary_backend(), Some("encrypted_file"));
}
