//! Multi-account integration tests for plur-post
//!
//! Tests the --account flag functionality for posting with different accounts.

use assert_cmd::Command;
use libplurcast::{AccountManager, CredentialConfig, CredentialManager, StorageBackend};
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper to escape path for TOML on Windows
fn escape_path_for_toml(path: &str) -> String {
    path.replace('\\', "\\\\")
}

/// Helper to create a test environment with config, database, and multi-account support
fn setup_multi_account_test_env() -> (TempDir, String, String, CredentialManager, AccountManager) {
    let temp_dir = TempDir::new().unwrap();

    // Create config directory
    let config_dir = temp_dir.path().join("config");
    fs::create_dir_all(&config_dir).unwrap();

    // Create data directory
    let data_dir = temp_dir.path().join("data");
    fs::create_dir_all(&data_dir).unwrap();

    // Create credentials directory
    let creds_dir = temp_dir.path().join("credentials");
    fs::create_dir_all(&creds_dir).unwrap();

    // Create config file with credential storage
    let config_path = config_dir.join("config.toml");
    let db_path = data_dir.join("posts.db");
    let accounts_path = config_dir.join("accounts.toml");

    let config_content = format!(
        r#"
[database]
path = "{}"

[credentials]
storage = "encrypted"
path = "{}"
master_password = "test_password"

[nostr]
enabled = true
keys_file = "/dev/null"
relays = ["wss://relay.damus.io"]

[defaults]
platforms = ["nostr"]
"#,
        escape_path_for_toml(&db_path.to_string_lossy()),
        escape_path_for_toml(&creds_dir.to_string_lossy())
    );

    fs::write(&config_path, config_content).unwrap();

    // Create CredentialManager with encrypted storage
    let cred_config = CredentialConfig {
        storage: StorageBackend::Encrypted,
        path: creds_dir.to_string_lossy().to_string(),
        master_password: Some("test_password".to_string()),
    };
    let cred_manager = CredentialManager::new(cred_config).unwrap();

    // Create AccountManager with custom path
    let account_manager = AccountManager::with_path(accounts_path).unwrap();

    // Generate test Nostr keys for different accounts
    let test_account_key = nostr_sdk::Keys::generate();
    let test_hex = test_account_key.secret_key().to_secret_hex();

    let prod_account_key = nostr_sdk::Keys::generate();
    let prod_hex = prod_account_key.secret_key().to_secret_hex();

    // Store credentials for test account
    cred_manager
        .store_account("plurcast.nostr", "private_key", "test", &test_hex)
        .unwrap();

    // Store credentials for prod account
    cred_manager
        .store_account("plurcast.nostr", "private_key", "prod", &prod_hex)
        .unwrap();

    // Register accounts
    account_manager.register_account("nostr", "test").unwrap();
    account_manager.register_account("nostr", "prod").unwrap();

    // Set test as active account
    account_manager.set_active_account("nostr", "test").unwrap();

    (
        temp_dir,
        config_path.to_string_lossy().to_string(),
        db_path.to_string_lossy().to_string(),
        cred_manager,
        account_manager,
    )
}

#[test]
fn test_post_with_explicit_account_flag() {
    let (_temp_dir, config_path, _db_path, _cred_manager, _account_manager) =
        setup_multi_account_test_env();

    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("Test post with explicit account")
        .arg("--account")
        .arg("prod")
        .arg("--draft") // Use draft mode to avoid actual posting
        .assert()
        .success()
        .stdout(predicate::str::contains("draft:"));
}

#[test]
fn test_post_with_active_account_no_flag() {
    let (_temp_dir, config_path, _db_path, _cred_manager, _account_manager) =
        setup_multi_account_test_env();

    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    // Should use active account (test) when no --account flag provided
    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("Test post with active account")
        .arg("--draft") // Use draft mode to avoid actual posting
        .assert()
        .success()
        .stdout(predicate::str::contains("draft:"));
}

#[test]
fn test_post_with_nonexistent_account() {
    let (_temp_dir, config_path, _db_path, _cred_manager, _account_manager) =
        setup_multi_account_test_env();

    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    // Should fail when account doesn't exist
    // Note: May fail with credential error or authentication error depending on environment
    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("Test post with nonexistent account")
        .arg("--account")
        .arg("nonexistent")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("not found")
                .or(predicate::str::contains("Authentication"))
                .or(predicate::str::contains("Credential"))
                .or(predicate::str::contains("No credential store available")),
        );
}

#[test]
fn test_verbose_mode_shows_account_info() {
    let (_temp_dir, config_path, _db_path, _cred_manager, _account_manager) =
        setup_multi_account_test_env();

    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    // Verbose mode should show which account is being used
    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("Test post with verbose")
        .arg("--account")
        .arg("prod")
        .arg("--draft")
        .arg("--verbose")
        .assert()
        .success()
        .stderr(predicate::str::contains("Using account: prod"));
}

#[test]
fn test_verbose_mode_shows_active_account_message() {
    let (_temp_dir, config_path, _db_path, _cred_manager, _account_manager) =
        setup_multi_account_test_env();

    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    // Verbose mode should show active account message when no --account flag
    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("Test post with verbose active")
        .arg("--draft")
        .arg("--verbose")
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "Using active account for each platform",
        ));
}

#[test]
fn test_help_includes_account_flag() {
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--account"))
        .stdout(predicate::str::contains("Account to use for posting"));
}

#[test]
fn test_account_flag_with_platform_flag() {
    let (_temp_dir, config_path, _db_path, _cred_manager, _account_manager) =
        setup_multi_account_test_env();

    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    // Should work with both --account and --platform flags
    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("Test post with account and platform")
        .arg("--account")
        .arg("test")
        .arg("--platform")
        .arg("nostr")
        .arg("--draft")
        .assert()
        .success()
        .stdout(predicate::str::contains("draft:"));
}
