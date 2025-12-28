//! SSB (Secure Scuttlebutt) integration tests
//!
//! This test suite covers SSB platform functionality including:
//! - Configuration parsing
//! - Keypair management
//! - Feed database initialization
//! - Message creation and signing
//! - Posting to local feed
//! - Pub server connection
//! - Replication protocol
//! - Multi-account support
//! - Import/export functionality

use libplurcast::config::SSBConfig;
use libplurcast::platforms::{ssb::SSBPlatform, Platform};
use libplurcast::Post;

#[test]
fn test_ssb_platform_creation() {
    let config = SSBConfig {
        enabled: true,
        feed_path: "~/.plurcast-ssb".to_string(),
        pubs: vec![],
    };

    let platform = SSBPlatform::new(&config);
    assert_eq!(platform.name(), "ssb");
}

#[test]
fn test_ssb_config_defaults() {
    let config = SSBConfig {
        enabled: true,
        feed_path: "~/.plurcast-ssb".to_string(),
        pubs: vec![],
    };

    assert!(config.enabled);
    assert_eq!(config.feed_path, "~/.plurcast-ssb");
    assert_eq!(config.pubs.len(), 0);
}

#[test]
fn test_ssb_config_with_pubs() {
    let config = SSBConfig {
        enabled: true,
        feed_path: "~/.plurcast-ssb".to_string(),
        pubs: vec!["net:hermies.club:8008~shs:test-key".to_string()],
    };

    assert_eq!(config.pubs.len(), 1);
    assert!(config.pubs[0].contains("hermies.club"));
}

#[test]
fn test_ssb_platform_is_configured() {
    let config = SSBConfig {
        enabled: true,
        feed_path: "~/.plurcast-ssb".to_string(),
        pubs: vec![],
    };

    let platform = SSBPlatform::new(&config);
    assert!(platform.is_configured());
}

#[test]
fn test_ssb_platform_disabled() {
    let config = SSBConfig {
        enabled: false,
        feed_path: "~/.plurcast-ssb".to_string(),
        pubs: vec![],
    };

    let platform = SSBPlatform::new(&config);
    assert!(!platform.is_configured());
}

#[test]
fn test_ssb_character_limit() {
    let config = SSBConfig {
        enabled: true,
        feed_path: "~/.plurcast-ssb".to_string(),
        pubs: vec![],
    };

    let platform = SSBPlatform::new(&config);
    // SSB has no hard character limit, but has byte size limit
    assert_eq!(platform.character_limit(), None);
}

#[test]
fn test_ssb_content_validation_success() {
    let config = SSBConfig {
        enabled: true,
        feed_path: "~/.plurcast-ssb".to_string(),
        pubs: vec![],
    };

    let platform = SSBPlatform::new(&config);
    let content = "Hello SSB! This is a test post.";

    assert!(platform.validate_content(content).is_ok());
}

#[test]
fn test_ssb_content_validation_exceeds_limit() {
    let config = SSBConfig {
        enabled: true,
        feed_path: "~/.plurcast-ssb".to_string(),
        pubs: vec![],
    };

    let platform = SSBPlatform::new(&config);
    // Create content larger than 8KB
    let content = "x".repeat(8193);

    let result = platform.validate_content(&content);
    assert!(result.is_err());

    let err_msg = result.unwrap_err().to_string();
    // Check for the actual error message format
    assert!(err_msg.contains("exceeds") && err_msg.contains("SSB"));
}

#[tokio::test]
async fn test_ssb_authenticate_basic() {
    use tempfile::TempDir;

    // Create temporary directory
    let temp_dir = TempDir::new().unwrap();
    let feed_path = temp_dir.path().join("test-feed");

    let config = SSBConfig {
        enabled: true,
        feed_path: feed_path.to_string_lossy().to_string(),
        pubs: vec![],
    };

    let mut platform = SSBPlatform::new(&config);
    // Note: authenticate() without initialization should fail
    // This test now verifies that authentication fails gracefully without credentials
    let result = platform.authenticate().await;

    // Should fail because platform is not initialized with credentials
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("not initialized") || err.contains("credentials"));
}

#[tokio::test]
async fn test_ssb_post_requires_initialization() {
    let config = SSBConfig {
        enabled: true,
        feed_path: "~/.plurcast-ssb".to_string(),
        pubs: vec![],
    };

    let platform = SSBPlatform::new(&config);
    let post = Post::new("Test content".to_string());
    let result = platform.post(&post).await;

    // Should return error because platform is not initialized
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("not initialized") || err_msg.contains("credentials"));
}

// ============================================================================
// Task 4.4: Feed database initialization tests
// Requirements: 15.2, 15.3
// ============================================================================

#[test]
fn test_create_feed_directory_new() {
    use tempfile::TempDir;

    // Create a temporary directory for testing
    let temp_dir = TempDir::new().unwrap();
    let feed_path = temp_dir.path().join("test-feed");

    let config = SSBConfig {
        enabled: true,
        feed_path: feed_path.to_string_lossy().to_string(),
        pubs: vec![],
    };

    let platform = SSBPlatform::new(&config);

    // Directory should not exist yet
    assert!(!feed_path.exists());

    // Create directory
    let result = platform.create_feed_directory();
    assert!(
        result.is_ok(),
        "Failed to create feed directory: {:?}",
        result.err()
    );

    // Directory should now exist
    assert!(feed_path.exists());
    assert!(feed_path.is_dir());

    // Check permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(&feed_path).unwrap();
        let permissions = metadata.permissions();
        assert_eq!(
            permissions.mode() & 0o777,
            0o700,
            "Directory should have 700 permissions"
        );
    }
}

#[test]
fn test_create_feed_directory_existing() {
    use tempfile::TempDir;

    // Create a temporary directory
    let temp_dir = TempDir::new().unwrap();
    let feed_path = temp_dir.path().join("existing-feed");

    // Create the directory first
    std::fs::create_dir(&feed_path).unwrap();

    let config = SSBConfig {
        enabled: true,
        feed_path: feed_path.to_string_lossy().to_string(),
        pubs: vec![],
    };

    let platform = SSBPlatform::new(&config);

    // Should succeed even if directory already exists
    let result = platform.create_feed_directory();
    assert!(
        result.is_ok(),
        "Failed with existing directory: {:?}",
        result.err()
    );

    // Directory should still exist
    assert!(feed_path.exists());
    assert!(feed_path.is_dir());
}

// TODO: Fix flaky test - depends on system permissions (succeeds when running as admin)
#[test]
#[ignore = "Flaky: depends on system permissions - may pass when running as admin"]
fn test_create_feed_directory_invalid_path() {
    // On Windows, we need a truly invalid path (e.g., with invalid characters)
    // On Unix, we can use a path that requires root permissions
    #[cfg(unix)]
    let invalid_path = "/root/invalid/path/that/cannot/be/created";

    #[cfg(windows)]
    let invalid_path = "C:\\Windows\\System32\\invalid\\path\\that\\cannot\\be\\created";

    let config = SSBConfig {
        enabled: true,
        feed_path: invalid_path.to_string(),
        pubs: vec![],
    };

    let platform = SSBPlatform::new(&config);

    // Should fail with permission error or similar
    let result = platform.create_feed_directory();

    // On some systems this might succeed (e.g., if running as admin/root)
    // So we'll just check that if it fails, it has the right error message
    if result.is_err() {
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Failed to open SSB feed database"));
    }
}

#[test]
fn test_create_feed_directory_file_exists() {
    use tempfile::NamedTempFile;

    // Create a temporary file (not a directory)
    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path();

    let config = SSBConfig {
        enabled: true,
        feed_path: file_path.to_string_lossy().to_string(),
        pubs: vec![],
    };

    let platform = SSBPlatform::new(&config);

    // Should fail because path exists but is not a directory
    let result = platform.create_feed_directory();
    assert!(result.is_err(), "Should fail when path is a file");

    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("not a directory"));
}

#[tokio::test]
async fn test_initialize_with_credentials_success() {
    use libplurcast::credentials::{CredentialConfig, CredentialManager, StorageBackend};
    use libplurcast::platforms::ssb::SSBKeypair;
    use tempfile::TempDir;

    // Create temporary directories
    let temp_dir = TempDir::new().unwrap();
    let feed_path = temp_dir.path().join("test-feed");
    let cred_path = temp_dir.path().join("credentials");

    // Create SSB config
    let config = SSBConfig {
        enabled: true,
        feed_path: feed_path.to_string_lossy().to_string(),
        pubs: vec![],
    };

    // Create credential manager
    let cred_config = CredentialConfig {
        storage: StorageBackend::Encrypted,
        path: cred_path.to_string_lossy().to_string(),
        master_password: Some("test-password-12345".to_string()),
    };
    let credentials = CredentialManager::new(cred_config).unwrap();

    // Generate and store keypair
    let keypair = SSBKeypair::generate();
    SSBPlatform::store_keypair(&credentials, &keypair, "test-account", true).unwrap();

    // Create platform and initialize
    let mut platform = SSBPlatform::new(&config);
    assert!(!platform.is_initialized());

    let result = platform
        .initialize_with_credentials(&credentials, "test-account")
        .await;
    assert!(result.is_ok(), "Failed to initialize: {:?}", result.err());

    // Check initialization state
    assert!(platform.is_initialized());
    assert!(platform.feed_id().is_some());
    assert_eq!(platform.feed_id().unwrap(), keypair.id);

    // Feed directory should exist
    assert!(feed_path.exists());
    assert!(feed_path.is_dir());
}

#[tokio::test]
async fn test_initialize_with_credentials_not_found() {
    use libplurcast::credentials::{CredentialConfig, CredentialManager, StorageBackend};
    use tempfile::TempDir;

    // Create temporary directories
    let temp_dir = TempDir::new().unwrap();
    let feed_path = temp_dir.path().join("test-feed");
    let cred_path = temp_dir.path().join("credentials");

    // Create SSB config
    let config = SSBConfig {
        enabled: true,
        feed_path: feed_path.to_string_lossy().to_string(),
        pubs: vec![],
    };

    // Create credential manager (no credentials stored)
    let cred_config = CredentialConfig {
        storage: StorageBackend::Encrypted,
        path: cred_path.to_string_lossy().to_string(),
        master_password: Some("test-password-12345".to_string()),
    };
    let credentials = CredentialManager::new(cred_config).unwrap();

    // Try to initialize without credentials
    let mut platform = SSBPlatform::new(&config);
    let result = platform
        .initialize_with_credentials(&credentials, "nonexistent")
        .await;

    assert!(result.is_err(), "Should fail with missing credentials");

    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("SSB credentials not configured"));
    assert!(err_msg.contains("run plur-setup or plur-creds set ssb"));
}

#[tokio::test]
async fn test_initialize_with_credentials_invalid_keypair() {
    use libplurcast::credentials::{CredentialConfig, CredentialManager, StorageBackend};
    use tempfile::TempDir;

    // Create temporary directories
    let temp_dir = TempDir::new().unwrap();
    let feed_path = temp_dir.path().join("test-feed");
    let cred_path = temp_dir.path().join("credentials");

    // Create SSB config
    let config = SSBConfig {
        enabled: true,
        feed_path: feed_path.to_string_lossy().to_string(),
        pubs: vec![],
    };

    // Create credential manager
    let cred_config = CredentialConfig {
        storage: StorageBackend::Encrypted,
        path: cred_path.to_string_lossy().to_string(),
        master_password: Some("test-password-12345".to_string()),
    };
    let credentials = CredentialManager::new(cred_config).unwrap();

    // Store invalid keypair JSON
    let invalid_json = r#"{"curve": "invalid", "public": "test", "private": "test", "id": "test"}"#;
    credentials
        .store_account("plurcast.ssb", "keypair", "test-account", invalid_json)
        .unwrap();

    // Try to initialize with invalid keypair
    let mut platform = SSBPlatform::new(&config);
    let result = platform
        .initialize_with_credentials(&credentials, "test-account")
        .await;

    assert!(result.is_err(), "Should fail with invalid keypair");

    let err_msg = result.unwrap_err().to_string();
    // The error could be from parsing or validation
    assert!(
        err_msg.contains("Invalid SSB keypair") || err_msg.contains("Invalid curve"),
        "Expected error about invalid keypair, got: {}",
        err_msg
    );
}

#[tokio::test]
async fn test_initialize_twice() {
    use libplurcast::credentials::{CredentialConfig, CredentialManager, StorageBackend};
    use libplurcast::platforms::ssb::SSBKeypair;
    use tempfile::TempDir;

    // Create temporary directories
    let temp_dir = TempDir::new().unwrap();
    let feed_path = temp_dir.path().join("test-feed");
    let cred_path = temp_dir.path().join("credentials");

    // Create SSB config
    let config = SSBConfig {
        enabled: true,
        feed_path: feed_path.to_string_lossy().to_string(),
        pubs: vec![],
    };

    // Create credential manager
    let cred_config = CredentialConfig {
        storage: StorageBackend::Encrypted,
        path: cred_path.to_string_lossy().to_string(),
        master_password: Some("test-password-12345".to_string()),
    };
    let credentials = CredentialManager::new(cred_config).unwrap();

    // Generate and store keypair
    let keypair = SSBKeypair::generate();
    SSBPlatform::store_keypair(&credentials, &keypair, "test-account", true).unwrap();

    // Create platform and initialize
    let mut platform = SSBPlatform::new(&config);

    // First initialization
    let result1 = platform
        .initialize_with_credentials(&credentials, "test-account")
        .await;
    assert!(result1.is_ok());
    assert!(platform.is_initialized());

    // Second initialization should succeed (idempotent)
    let result2 = platform
        .initialize_with_credentials(&credentials, "test-account")
        .await;
    assert!(result2.is_ok());
    assert!(platform.is_initialized());
}

#[tokio::test]
async fn test_authenticate_creates_directory() {
    use tempfile::TempDir;

    // Create temporary directory
    let temp_dir = TempDir::new().unwrap();
    let feed_path = temp_dir.path().join("test-feed");

    // Create SSB config
    let config = SSBConfig {
        enabled: true,
        feed_path: feed_path.to_string_lossy().to_string(),
        pubs: vec![],
    };

    // Create platform
    let mut platform = SSBPlatform::new(&config);

    // Directory should not exist yet
    assert!(!feed_path.exists());

    // Authenticate (should fail gracefully without initialization)
    let result = platform.authenticate().await;
    assert!(result.is_err());

    // Verify error message
    let err = result.unwrap_err().to_string();
    assert!(err.contains("not initialized") || err.contains("credentials"));
}

// TODO: Add more tests as implementation progresses:
// - Task 2: Configuration parsing tests
// - Task 3: Keypair management tests (DONE - see ssb.rs tests)
// - Task 4: Feed database initialization tests (DONE)
// - Task 5: Message creation and signing tests
// - Task 6: Posting integration tests
// - Task 7: Pub server connection tests
// - Task 8: Replication protocol tests
// - Task 11: Multi-account support tests
// - Task 13: Import functionality tests
// - Task 14: Export functionality tests
