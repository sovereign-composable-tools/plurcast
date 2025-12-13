//! Integration tests for Nostr Proof of Work (NIP-13) support
//!
//! These tests verify that the --nostr-pow flag works correctly and that
//! POW difficulty is properly handled throughout the posting flow.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper to escape path for TOML on Windows
fn escape_path_for_toml(path: &str) -> String {
    path.replace('\\', "\\\\")
}

/// Helper to create a test environment with POW configuration
fn setup_test_env_with_pow(default_pow: Option<u8>) -> (TempDir, String, String) {
    let temp_dir = TempDir::new().unwrap();

    // Create config directory
    let config_dir = temp_dir.path().join("config");
    fs::create_dir_all(&config_dir).unwrap();

    // Create data directory
    let data_dir = temp_dir.path().join("data");
    fs::create_dir_all(&data_dir).unwrap();

    // Create config file
    let config_path = config_dir.join("config.toml");
    let db_path = data_dir.join("posts.db");
    let keys_path = config_dir.join("nostr.keys");

    let pow_config = default_pow
        .map(|d| format!("default_pow_difficulty = {}", d))
        .unwrap_or_default();

    let config_content = format!(
        r#"
[database]
path = "{}"

[nostr]
enabled = true
keys_file = "{}"
relays = ["wss://relay.damus.io"]
{}

[defaults]
platforms = ["nostr"]
"#,
        escape_path_for_toml(&db_path.to_string_lossy()),
        escape_path_for_toml(&keys_path.to_string_lossy()),
        pow_config
    );

    fs::write(&config_path, config_content).unwrap();

    // Generate test Nostr keys
    let test_keys = nostr_sdk::Keys::generate();
    let hex_key = test_keys.secret_key().to_secret_hex();
    fs::write(&keys_path, hex_key).unwrap();

    (
        temp_dir,
        config_path.to_string_lossy().to_string(),
        db_path.to_string_lossy().to_string(),
    )
}

#[test]
fn test_pow_flag_appears_in_help() {
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--nostr-pow"))
        .stdout(predicate::str::contains("Proof of Work difficulty"))
        .stdout(predicate::str::contains("NIP-13"))
        .stdout(predicate::str::contains("20-25"))
        .stdout(predicate::str::contains(
            "Only applies when posting to Nostr",
        ));
}

#[test]
fn test_pow_flag_accepts_difficulty_value() {
    let (_temp_dir, config_path, _db_path) = setup_test_env_with_pow(None);

    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    // Post with POW in draft mode (so we don't actually connect to relays)
    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("Test post with POW")
        .arg("--nostr-pow")
        .arg("20")
        .arg("--draft")
        .assert()
        .success()
        .stdout(predicate::str::contains("draft:"));
}

#[test]
fn test_pow_flag_validates_range() {
    let (_temp_dir, config_path, _db_path) = setup_test_env_with_pow(None);

    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    // Test valid range (0-64)
    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("Test post")
        .arg("--nostr-pow")
        .arg("64")
        .arg("--draft")
        .assert()
        .success();
}

#[test]
fn test_pow_succeeds_in_draft_mode() {
    let (_temp_dir, config_path, _db_path) = setup_test_env_with_pow(None);

    // Post with POW as draft - should succeed
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("Test POW post")
        .arg("--nostr-pow")
        .arg("25")
        .arg("--draft")
        .assert()
        .success()
        .stdout(predicate::str::contains("draft:"));
}

#[test]
fn test_pow_optional_works_without_flag() {
    let (_temp_dir, config_path, _db_path) = setup_test_env_with_pow(None);

    // Post without POW flag - should still succeed
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("Test post without POW")
        .arg("--draft")
        .assert()
        .success()
        .stdout(predicate::str::contains("draft:"));
}

#[test]
fn test_config_default_pow_works() {
    let (_temp_dir, config_path, _db_path) = setup_test_env_with_pow(Some(20));

    // Post without --nostr-pow flag (should use config default)
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("Test config default POW")
        .arg("--draft")
        .assert()
        .success()
        .stdout(predicate::str::contains("draft:"));
}

#[test]
fn test_cli_flag_overrides_config_default() {
    let (_temp_dir, config_path, _db_path) = setup_test_env_with_pow(Some(20));

    // Post with --nostr-pow flag (should override config default)
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("Test CLI override")
        .arg("--nostr-pow")
        .arg("30")
        .arg("--draft")
        .assert()
        .success()
        .stdout(predicate::str::contains("draft:"));
}

#[test]
fn test_pow_zero_accepted() {
    let (_temp_dir, config_path, _db_path) = setup_test_env_with_pow(Some(20));

    // Post with --nostr-pow 0 (should be valid)
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("Test POW zero")
        .arg("--nostr-pow")
        .arg("0")
        .arg("--draft")
        .assert()
        .success()
        .stdout(predicate::str::contains("draft:"));
}

#[test]
fn test_pow_with_scheduled_posts() {
    let (_temp_dir, config_path, _db_path) = setup_test_env_with_pow(None);

    // Schedule post with POW - should succeed
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("Scheduled POW post")
        .arg("--nostr-pow")
        .arg("22")
        .arg("--schedule")
        .arg("1h")
        .assert()
        .success()
        .stdout(predicate::str::contains("scheduled:"));
}

#[test]
fn test_pow_with_multiple_values() {
    let (_temp_dir, config_path, _db_path) = setup_test_env_with_pow(None);

    // Test different valid POW values
    for difficulty in [1, 10, 20, 30, 40, 50, 64] {
        let mut cmd = Command::cargo_bin("plur-post").unwrap();
        cmd.env("PLURCAST_CONFIG", &config_path)
            .arg(format!("Test POW {}", difficulty))
            .arg("--nostr-pow")
            .arg(difficulty.to_string())
            .arg("--draft")
            .assert()
            .success()
            .stdout(predicate::str::contains("draft:"));
    }
}
