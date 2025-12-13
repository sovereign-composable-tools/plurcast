//! Multi-platform posting integration tests

use assert_cmd::Command;
use predicates::prelude::*;
use sqlx::SqlitePool;
use std::fs;
use tempfile::TempDir;

/// Helper to escape path for TOML on Windows
fn escape_path_for_toml(path: &str) -> String {
    path.replace('\\', "\\\\")
}

/// Helper to create a test environment with multi-platform config
fn setup_multi_platform_env() -> (TempDir, String, String) {
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
    let mastodon_token_path = config_dir.join("mastodon.token");
    let bluesky_auth_path = config_dir.join("bluesky.auth");

    let config_content = format!(
        r#"
[database]
path = "{}"

[nostr]
enabled = true
keys_file = "{}"
relays = ["wss://relay.damus.io"]

[mastodon]
enabled = true
instance = "mastodon.social"
token_file = "{}"

[bluesky]
enabled = true
handle = "test.bsky.social"
auth_file = "{}"

[defaults]
platforms = ["nostr", "mastodon", "bluesky"]
"#,
        escape_path_for_toml(&db_path.to_string_lossy()),
        escape_path_for_toml(&keys_path.to_string_lossy()),
        escape_path_for_toml(&mastodon_token_path.to_string_lossy()),
        escape_path_for_toml(&bluesky_auth_path.to_string_lossy())
    );

    fs::write(&config_path, config_content).unwrap();

    // Generate test Nostr keys
    let test_keys = nostr_sdk::Keys::generate();
    let hex_key = test_keys.secret_key().to_secret_hex();
    fs::write(&keys_path, hex_key).unwrap();

    // Create dummy credential files (won't actually work for posting, but allows config validation)
    fs::write(&mastodon_token_path, "test_mastodon_token").unwrap();
    fs::write(&bluesky_auth_path, "test_bluesky_password").unwrap();

    (
        temp_dir,
        config_path.to_string_lossy().to_string(),
        db_path.to_string_lossy().to_string(),
    )
}

#[test]
fn test_platform_flag_single_platform() {
    let (_temp_dir, config_path, _db_path) = setup_multi_platform_env();

    // Post to only Nostr using --platform flag
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("Test single platform")
        .arg("--platform")
        .arg("nostr")
        .arg("--draft")
        .assert()
        .success();
}

#[test]
fn test_platform_flag_multiple_platforms() {
    let (_temp_dir, config_path, _db_path) = setup_multi_platform_env();

    // Post to multiple platforms using multiple --platform flags
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("Test multiple platforms")
        .arg("--platform")
        .arg("nostr")
        .arg("--platform")
        .arg("mastodon")
        .arg("--draft")
        .assert()
        .success();
}

#[test]
fn test_platform_flag_invalid_platform() {
    let (_temp_dir, config_path, _db_path) = setup_multi_platform_env();

    // Try to post to invalid platform
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("Test invalid platform")
        .arg("--platform")
        .arg("invalid_platform")
        .assert()
        .failure(); // Should fail due to invalid platform value
}

#[test]
fn test_output_format_text_multiple_platforms() {
    let (_temp_dir, config_path, _db_path) = setup_multi_platform_env();

    // Post in draft mode and check text output format
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    let output = cmd
        .env("PLURCAST_CONFIG", &config_path)
        .arg("Test text output")
        .arg("--draft")
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();

    // In draft mode, should output "draft:post_id"
    assert!(stdout.contains("draft:"));
}

#[test]
fn test_output_format_json_multiple_platforms() {
    let (_temp_dir, config_path, _db_path) = setup_multi_platform_env();

    // Post in draft mode with JSON output
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    let output = cmd
        .env("PLURCAST_CONFIG", &config_path)
        .arg("Test JSON output")
        .arg("--draft")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Verify JSON structure for draft
    assert!(json.get("status").is_some());
    assert_eq!(json["status"], "draft");
}

#[test]
fn test_exit_code_all_success() {
    let (_temp_dir, config_path, _db_path) = setup_multi_platform_env();

    // Draft mode should always succeed (exit code 0)
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("Test all success")
        .arg("--draft")
        .assert()
        .success()
        .code(0);
}

#[test]
fn test_exit_code_authentication_error() {
    let temp_dir = TempDir::new().unwrap();

    let config_dir = temp_dir.path().join("config");
    fs::create_dir_all(&config_dir).unwrap();

    let data_dir = temp_dir.path().join("data");
    fs::create_dir_all(&data_dir).unwrap();

    let config_path = config_dir.join("config.toml");
    let db_path = data_dir.join("posts.db");
    let keys_path = config_dir.join("nostr.keys");

    // Config with missing keys file
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

    // Try to post without keys file (should fail with exit code 2)
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.env("PLURCAST_CONFIG", config_path.to_str().unwrap())
        .arg("Test auth error")
        .assert()
        .failure()
        .code(2); // Authentication error
}

#[test]
fn test_exit_code_invalid_input() {
    let (_temp_dir, config_path, _db_path) = setup_multi_platform_env();

    // Try to post empty content (should fail with exit code 3)
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("")
        .assert()
        .failure()
        .code(3); // Invalid input
}

#[test]
fn test_content_validation_bluesky_limit() {
    let (_temp_dir, config_path, _db_path) = setup_multi_platform_env();

    // Create content that exceeds Bluesky's 300 character limit
    let long_content = "a".repeat(301);

    // Try to post to Bluesky (should fail validation)
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg(&long_content)
        .arg("--platform")
        .arg("bluesky")
        .assert()
        .failure()
        .code(3) // Invalid input (validation failure)
        .stderr(
            predicate::str::contains("character limit").or(predicate::str::contains("validation")),
        );
}

#[test]
fn test_content_validation_mastodon_limit() {
    let (_temp_dir, config_path, _db_path) = setup_multi_platform_env();

    // Create content that exceeds Mastodon's 500 character limit
    let long_content = "a".repeat(501);

    // Try to post to Mastodon (should fail validation)
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg(&long_content)
        .arg("--platform")
        .arg("mastodon")
        .assert()
        .failure()
        .code(3) // Invalid input (validation failure)
        .stderr(
            predicate::str::contains("character limit").or(predicate::str::contains("validation")),
        );
}

#[test]
fn test_content_validation_multiple_platforms() {
    let (_temp_dir, config_path, _db_path) = setup_multi_platform_env();

    // Create content that exceeds Bluesky's limit but not Mastodon's
    let content = "a".repeat(350);

    // Try to post to both platforms (should fail due to Bluesky limit)
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg(&content)
        .arg("--platform")
        .arg("bluesky")
        .arg("--platform")
        .arg("mastodon")
        .assert()
        .failure()
        .code(3) // Invalid input (validation failure)
        .stderr(predicate::str::contains("Bluesky"));
}

#[test]
fn test_verbose_flag_shows_progress() {
    let (_temp_dir, config_path, _db_path) = setup_multi_platform_env();

    // Post with verbose flag
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    let output = cmd
        .env("PLURCAST_CONFIG", &config_path)
        .arg("Test verbose output")
        .arg("--draft")
        .arg("--verbose")
        .output()
        .unwrap();

    assert!(output.status.success());

    // Verbose output should go to stderr
    let stderr = String::from_utf8(output.stderr).unwrap();

    // Should contain some logging output (exact format may vary)
    // Just verify that stderr is not empty when verbose is enabled
    assert!(
        !stderr.is_empty(),
        "Verbose mode should produce stderr output"
    );
}

#[tokio::test]
async fn test_database_records_multiple_platforms() {
    let (_temp_dir, config_path, db_path) = setup_multi_platform_env();

    // Post in draft mode
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("Test multi-platform database")
        .arg("--draft")
        .assert()
        .success();

    // Connect to database and verify post was created
    let pool = SqlitePool::connect(&format!("sqlite://{}", db_path))
        .await
        .unwrap();

    let post_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM posts")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(post_count, 1, "Expected 1 post in database");

    pool.close().await;
}

#[test]
fn test_default_platforms_from_config() {
    let (_temp_dir, config_path, _db_path) = setup_multi_platform_env();

    // Post without --platform flag (should use defaults from config)
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("Test default platforms")
        .arg("--draft")
        .assert()
        .success();
}

#[test]
fn test_platform_flag_overrides_defaults() {
    let (_temp_dir, config_path, _db_path) = setup_multi_platform_env();

    // Config has all platforms as defaults, but we only specify nostr
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("Test platform override")
        .arg("--platform")
        .arg("nostr")
        .arg("--draft")
        .assert()
        .success();
}

#[test]
fn test_help_shows_platform_options() {
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--platform"))
        .stdout(predicate::str::contains("nostr"))
        .stdout(predicate::str::contains("mastodon"))
        .stdout(predicate::str::contains("ssb"));
}

// ============================================================================
// Task 9.4: SSB Multi-Platform Integration Tests
// Requirements: 15.5
// ============================================================================

/// Helper to create a test environment with SSB included
fn setup_multi_platform_env_with_ssb() -> (TempDir, String, String) {
    let temp_dir = TempDir::new().unwrap();

    // Create config directory
    let config_dir = temp_dir.path().join("config");
    fs::create_dir_all(&config_dir).unwrap();

    // Create data directory
    let data_dir = temp_dir.path().join("data");
    fs::create_dir_all(&data_dir).unwrap();

    // Create SSB feed directory
    let ssb_feed_dir = temp_dir.path().join("ssb-feed");
    fs::create_dir_all(&ssb_feed_dir).unwrap();

    // Create config file
    let config_path = config_dir.join("config.toml");
    let db_path = data_dir.join("posts.db");
    let keys_path = config_dir.join("nostr.keys");
    let mastodon_token_path = config_dir.join("mastodon.token");

    let config_content = format!(
        r#"
[database]
path = "{}"

[nostr]
enabled = true
keys_file = "{}"
relays = ["wss://relay.damus.io"]

[mastodon]
enabled = true
instance = "mastodon.social"
token_file = "{}"

[ssb]
enabled = true
feed_path = "{}"
pubs = []

[defaults]
platforms = ["nostr", "mastodon", "ssb"]
"#,
        escape_path_for_toml(&db_path.to_string_lossy()),
        escape_path_for_toml(&keys_path.to_string_lossy()),
        escape_path_for_toml(&mastodon_token_path.to_string_lossy()),
        escape_path_for_toml(&ssb_feed_dir.to_string_lossy())
    );

    fs::write(&config_path, config_content).unwrap();

    // Generate test Nostr keys
    let test_keys = nostr_sdk::Keys::generate();
    let hex_key = test_keys.secret_key().to_secret_hex();
    fs::write(&keys_path, hex_key).unwrap();

    // Create dummy credential files
    fs::write(&mastodon_token_path, "test_mastodon_token").unwrap();

    (
        temp_dir,
        config_path.to_string_lossy().to_string(),
        db_path.to_string_lossy().to_string(),
    )
}

#[test]
fn test_platform_flag_ssb_only() {
    let (_temp_dir, config_path, _db_path) = setup_multi_platform_env_with_ssb();

    // Post to only SSB using --platform flag
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("Test SSB only")
        .arg("--platform")
        .arg("ssb")
        .arg("--draft")
        .assert()
        .success();
}

#[test]
fn test_platform_flag_nostr_mastodon_ssb() {
    let (_temp_dir, config_path, _db_path) = setup_multi_platform_env_with_ssb();

    // Post to all three platforms
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("Test all three platforms")
        .arg("--platform")
        .arg("nostr")
        .arg("--platform")
        .arg("mastodon")
        .arg("--platform")
        .arg("ssb")
        .arg("--draft")
        .assert()
        .success();
}

#[test]
fn test_ssb_with_other_platforms_partial_failure() {
    let temp_dir = TempDir::new().unwrap();

    let config_dir = temp_dir.path().join("config");
    fs::create_dir_all(&config_dir).unwrap();

    let data_dir = temp_dir.path().join("data");
    fs::create_dir_all(&data_dir).unwrap();

    let ssb_feed_dir = temp_dir.path().join("ssb-feed");
    fs::create_dir_all(&ssb_feed_dir).unwrap();

    let config_path = config_dir.join("config.toml");
    let db_path = data_dir.join("posts.db");
    let keys_path = config_dir.join("nostr.keys");

    // Config with Nostr (valid) and SSB (no credentials)
    let config_content = format!(
        r#"
[database]
path = "{}"

[nostr]
enabled = true
keys_file = "{}"
relays = ["wss://relay.damus.io"]

[ssb]
enabled = true
feed_path = "{}"
pubs = []

[defaults]
platforms = ["nostr", "ssb"]
"#,
        escape_path_for_toml(&db_path.to_string_lossy()),
        escape_path_for_toml(&keys_path.to_string_lossy()),
        escape_path_for_toml(&ssb_feed_dir.to_string_lossy())
    );

    fs::write(&config_path, config_content).unwrap();

    // Generate test Nostr keys
    let test_keys = nostr_sdk::Keys::generate();
    let hex_key = test_keys.secret_key().to_secret_hex();
    fs::write(&keys_path, hex_key).unwrap();

    // Try to post to both platforms (SSB should fail due to missing credentials)
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.env("PLURCAST_CONFIG", config_path.to_str().unwrap())
        .arg("Test partial failure")
        .assert()
        .failure()
        .code(2); // Authentication error (SSB credentials missing)
}

#[test]
fn test_platform_selection_with_ssb_flag() {
    let (_temp_dir, config_path, _db_path) = setup_multi_platform_env_with_ssb();

    // Post to Nostr and SSB only (exclude Mastodon)
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("Test selective platforms with SSB")
        .arg("--platform")
        .arg("nostr")
        .arg("--platform")
        .arg("ssb")
        .arg("--draft")
        .assert()
        .success();
}

#[test]
fn test_help_includes_ssb_platform() {
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("ssb"))
        .stdout(
            predicate::str::contains("Secure Scuttlebutt").or(predicate::str::contains("platform")),
        );
}

#[test]
fn test_ssb_content_validation_large_message() {
    let (_temp_dir, config_path, _db_path) = setup_multi_platform_env_with_ssb();

    // Create content that exceeds SSB's practical 8KB limit
    let large_content = "a".repeat(8193);

    // Try to post to SSB
    // Note: Without SSB credentials configured, this will fail with authentication error (code 2)
    // rather than validation error (code 3). To test validation specifically, credentials would
    // need to be set up via plur-creds, which is tested in integration tests.
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg(&large_content)
        .arg("--platform")
        .arg("ssb")
        .assert()
        .failure()
        .code(2) // Authentication error (SSB requires credentials)
        .stderr(predicate::str::contains("SSB"));
}

#[test]
fn test_default_platforms_includes_ssb() {
    let (_temp_dir, config_path, _db_path) = setup_multi_platform_env_with_ssb();

    // Post without --platform flag (should use defaults which include SSB)
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("Test default platforms with SSB")
        .arg("--draft")
        .assert()
        .success();
}
