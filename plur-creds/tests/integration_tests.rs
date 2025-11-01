//! Integration tests for plur-creds CLI
//!
//! These tests verify the multi-account functionality of plur-creds commands.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to create a test environment with isolated config and data directories
struct TestEnv {
    _temp_dir: TempDir,
    config_dir: PathBuf,
    data_dir: PathBuf,
}

impl TestEnv {
    fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().join("config");
        let data_dir = temp_dir.path().join("data");

        fs::create_dir_all(&config_dir).unwrap();
        fs::create_dir_all(&data_dir).unwrap();

        // Create a minimal config file with properly escaped paths
        let cred_path = config_dir.join("credentials").to_string_lossy().replace('\\', "\\\\");
        let db_path = data_dir.join("posts.db").to_string_lossy().replace('\\', "\\\\");

        let config_content = format!(
            r#"
[credentials]
storage = "encrypted"
path = "{}"

[database]
path = "{}"
"#,
            cred_path, db_path
        );

        fs::write(config_dir.join("config.toml"), config_content).unwrap();

        Self {
            _temp_dir: temp_dir,
            config_dir,
            data_dir,
        }
    }

    fn cmd(&self) -> Command {
        let mut cmd = Command::cargo_bin("plur-creds").unwrap();
        // Point to our test config file
        cmd.env("PLURCAST_CONFIG", self.config_dir.join("config.toml"));
        // Set master password for encrypted storage
        cmd.env("PLURCAST_MASTER_PASSWORD", "test-password-12345");
        cmd
    }
}

#[test]
fn test_set_with_account_flag() {
    let env = TestEnv::new();

    // Set credentials for test account
    env.cmd()
        .args(&["set", "nostr", "--account", "test", "--stdin"])
        .write_stdin("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
        .assert()
        .success()
        .stdout(predicate::str::contains("Stored nostr credentials for account 'test'"));
}

#[test]
fn test_set_with_default_account() {
    let env = TestEnv::new();

    // Set credentials without account flag (should use "default")
    env.cmd()
        .args(&["set", "nostr", "--stdin"])
        .write_stdin("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
        .assert()
        .success()
        .stdout(predicate::str::contains("Stored nostr credentials for account 'default'"));
}

#[test]
fn test_set_with_invalid_account_name() {
    let env = TestEnv::new();

    // Try to set credentials with invalid account name (contains spaces)
    env.cmd()
        .args(&["set", "nostr", "--account", "test account", "--stdin"])
        .write_stdin("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid account name"));
}

#[test]
fn test_list_with_platform_filter() {
    let env = TestEnv::new();

    // Set credentials for multiple accounts
    env.cmd()
        .args(&["set", "nostr", "--account", "test", "--stdin"])
        .write_stdin("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
        .assert()
        .success();

    env.cmd()
        .args(&["set", "nostr", "--account", "prod", "--stdin"])
        .write_stdin("fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210")
        .assert()
        .success();

    // List only nostr credentials
    env.cmd()
        .args(&["list", "--platform", "nostr"])
        .assert()
        .success()
        .stdout(predicate::str::contains("nostr (test)"))
        .stdout(predicate::str::contains("nostr (prod)"));
}

#[test]
fn test_list_shows_active_account() {
    let env = TestEnv::new();

    // Set credentials for test account
    env.cmd()
        .args(&["set", "nostr", "--account", "test", "--stdin"])
        .write_stdin("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
        .assert()
        .success();

    // Set as active
    env.cmd()
        .args(&["use", "nostr", "--account", "test"])
        .assert()
        .success();

    // List should show [active] marker
    env.cmd()
        .args(&["list", "--platform", "nostr"])
        .assert()
        .success()
        .stdout(predicate::str::contains("[active]"));
}

#[test]
fn test_use_command() {
    let env = TestEnv::new();

    // Set credentials for test account
    env.cmd()
        .args(&["set", "nostr", "--account", "test", "--stdin"])
        .write_stdin("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
        .assert()
        .success();

    // Set as active
    env.cmd()
        .args(&["use", "nostr", "--account", "test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Set 'test' as active account for nostr"));
}

#[test]
fn test_use_nonexistent_account() {
    let env = TestEnv::new();

    // Try to use account that doesn't exist
    env.cmd()
        .args(&["use", "nostr", "--account", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Account 'nonexistent' not found"));
}

#[test]
fn test_delete_with_account_flag() {
    let env = TestEnv::new();

    // Set credentials for test account
    env.cmd()
        .args(&["set", "nostr", "--account", "test", "--stdin"])
        .write_stdin("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
        .assert()
        .success();

    // Delete with force flag
    env.cmd()
        .args(&["delete", "nostr", "--account", "test", "--force"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleted nostr credentials for account 'test'"));
}

#[test]
fn test_delete_active_account_resets_to_default() {
    let env = TestEnv::new();

    // Set credentials for test account
    env.cmd()
        .args(&["set", "nostr", "--account", "test", "--stdin"])
        .write_stdin("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
        .assert()
        .success();

    // Set as active
    env.cmd()
        .args(&["use", "nostr", "--account", "test"])
        .assert()
        .success();

    // Delete active account
    env.cmd()
        .args(&["delete", "nostr", "--account", "test", "--force"])
        .assert()
        .success()
        .stdout(predicate::str::contains("reset to 'default'"));
}

#[test]
fn test_test_command_with_account() {
    let env = TestEnv::new();

    // Set credentials for test account
    env.cmd()
        .args(&["set", "nostr", "--account", "test", "--stdin"])
        .write_stdin("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
        .assert()
        .success();

    // Test credentials
    env.cmd()
        .args(&["test", "nostr", "--account", "test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("nostr credentials found for account 'test'"));
}

#[test]
fn test_test_nonexistent_account() {
    let env = TestEnv::new();

    // Test credentials for nonexistent account
    env.cmd()
        .args(&["test", "nostr", "--account", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No credentials found"));
}

#[test]
fn test_multiple_accounts_isolation() {
    let env = TestEnv::new();

    // Set credentials for test account
    env.cmd()
        .args(&["set", "nostr", "--account", "test", "--stdin"])
        .write_stdin("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
        .assert()
        .success();

    // Set credentials for prod account
    env.cmd()
        .args(&["set", "nostr", "--account", "prod", "--stdin"])
        .write_stdin("fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210")
        .assert()
        .success();

    // Both should exist independently
    env.cmd()
        .args(&["test", "nostr", "--account", "test"])
        .assert()
        .success();

    env.cmd()
        .args(&["test", "nostr", "--account", "prod"])
        .assert()
        .success();

    // Delete test account shouldn't affect prod
    env.cmd()
        .args(&["delete", "nostr", "--account", "test", "--force"])
        .assert()
        .success();

    env.cmd()
        .args(&["test", "nostr", "--account", "prod"])
        .assert()
        .success();
}
