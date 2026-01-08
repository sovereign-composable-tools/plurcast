//! Scheduling integration tests for plur-post (Phase 5.2 Task 9)
//!
//! Tests for --schedule flag functionality including:
//! - Duration formats ("30m", "2h", "1d")
//! - Natural language ("tomorrow")
//! - Random scheduling ("random:10m-20m")
//! - Invalid schedule formats
//! - Output format for scheduled posts

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper to escape path for TOML on Windows
fn escape_path_for_toml(path: &str) -> String {
    path.replace('\\', "\\\\")
}

/// Helper to create a test environment with config and database
fn setup_test_env() -> (TempDir, String, String) {
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

    let config_content = format!(
        r#"
[database]
path = "{}"

[nostr]
enabled = true
keys_file = "{}"
relays = ["wss://relay.damus.io"]

[defaults]
platforms = ["nostr"]
"#,
        escape_path_for_toml(&db_path.to_string_lossy()),
        escape_path_for_toml(&keys_path.to_string_lossy())
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

// SCHEDULE FLAG TESTS

#[test]
fn test_help_shows_schedule_flag() {
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--schedule"));
}

#[test]
fn test_schedule_with_duration_minutes() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();

    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("Test scheduled post")
        .arg("--schedule")
        .arg("30m")
        .assert()
        .success()
        .stdout(predicate::str::contains("scheduled:"));
}

#[test]
fn test_schedule_with_duration_hours() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();

    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("Test scheduled post")
        .arg("--schedule")
        .arg("2h")
        .assert()
        .success()
        .stdout(predicate::str::contains("scheduled:"));
}

#[test]
fn test_schedule_with_duration_days() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();

    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("Test scheduled post")
        .arg("--schedule")
        .arg("1d")
        .assert()
        .success()
        .stdout(predicate::str::contains("scheduled:"));
}

#[test]
fn test_schedule_with_natural_language() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();

    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("Test scheduled post")
        .arg("--schedule")
        .arg("tomorrow")
        .assert()
        .success()
        .stdout(predicate::str::contains("scheduled:"));
}

#[test]
fn test_schedule_with_random_interval() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();

    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("Test random scheduled post")
        .arg("--schedule")
        .arg("random:10m-20m")
        .assert()
        .success()
        .stdout(predicate::str::contains("scheduled:"));
}

#[test]
fn test_schedule_with_random_hours() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();

    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("Test random scheduled post")
        .arg("--schedule")
        .arg("random:1h-2h")
        .assert()
        .success()
        .stdout(predicate::str::contains("scheduled:"));
}

// ERROR HANDLING TESTS

#[test]
fn test_schedule_with_invalid_format() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();

    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("Test post")
        .arg("--schedule")
        .arg("invalid-time")
        .assert()
        .failure()
        .code(3) // Invalid input exit code
        .stderr(predicate::str::contains("Could not parse schedule"));
}

#[test]
fn test_schedule_with_empty_string() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();

    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("Test post")
        .arg("--schedule")
        .arg("")
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("cannot be empty"));
}

#[test]
fn test_schedule_random_min_greater_than_max() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();

    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("Test post")
        .arg("--schedule")
        .arg("random:2h-1h")
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains(
            "Minimum must be less than maximum",
        ));
}

#[test]
fn test_schedule_random_too_short() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();

    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("Test post")
        .arg("--schedule")
        .arg("random:1s-10s")
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("at least 30 seconds"));
}

// OUTPUT FORMAT TESTS

#[test]
fn test_schedule_output_format_text() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();

    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("Test post")
        .arg("--schedule")
        .arg("1h")
        .assert()
        .success()
        // Format: "scheduled:<post-id>:for:<human-readable time>"
        // Example: "scheduled:UUID:for:in 1 hour (Jan 8 05:56 UTC)"
        .stdout(predicate::str::is_match(r"scheduled:[0-9a-f-]+:for:.+UTC\)").unwrap());
}

#[test]
fn test_schedule_output_format_json() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();

    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("Test post")
        .arg("--schedule")
        .arg("1h")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"scheduled\""))
        .stdout(predicate::str::contains("\"post_id\""))
        .stdout(predicate::str::contains("\"scheduled_at\""));
}

// COMPATIBILITY TESTS

#[test]
fn test_schedule_cannot_be_used_with_draft() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();

    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("Test post")
        .arg("--schedule")
        .arg("1h")
        .arg("--draft")
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains(
            "cannot use --schedule with --draft",
        ));
}

#[test]
fn test_schedule_with_stdin() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();

    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("--schedule")
        .arg("30m")
        .write_stdin("Post content from stdin")
        .assert()
        .success()
        .stdout(predicate::str::contains("scheduled:"));
}

#[test]
fn test_schedule_with_platform_flag() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();

    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("Test post")
        .arg("--schedule")
        .arg("1h")
        .arg("--platform")
        .arg("nostr")
        .assert()
        .success()
        .stdout(predicate::str::contains("scheduled:"));
}
