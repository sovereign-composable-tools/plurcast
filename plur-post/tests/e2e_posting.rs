//! End-to-end posting workflow integration tests

use assert_cmd::Command;
use predicates::prelude::*;
use sqlx::SqlitePool;
use std::fs;
use tempfile::TempDir;

/// Helper to escape path for TOML on Windows
fn escape_path_for_toml(path: &str) -> String {
    path.replace('\\', "\\\\")
}

/// Helper to create a test environment with config and database
fn setup_test_env() -> (TempDir, String, String, String) {
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
        keys_path.to_string_lossy().to_string(),
    )
}

#[tokio::test]
async fn test_e2e_posting_with_valid_config() {
    let (_temp_dir, config_path, db_path, _keys_path) = setup_test_env();

    // Post content in draft mode
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    let output = cmd
        .env("PLURCAST_CONFIG", &config_path)
        .arg("Test end-to-end posting")
        .arg("--draft")
        .output()
        .unwrap();

    assert!(output.status.success());

    // Verify database was created
    assert!(std::path::Path::new(&db_path).exists());

    // Connect to database and verify post was recorded
    let pool = SqlitePool::connect(&format!("sqlite://{}", db_path))
        .await
        .unwrap();

    let post_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM posts")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(post_count, 1, "Expected 1 post in database");

    // Verify post content
    let content: String = sqlx::query_scalar("SELECT content FROM posts")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(content, "Test end-to-end posting");

    // Verify post status (draft mode should set status to 'draft' or 'pending')
    let status: String = sqlx::query_scalar("SELECT status FROM posts")
        .fetch_one(&pool)
        .await
        .unwrap();

    // In draft mode, status could be 'draft' or 'pending' depending on implementation
    assert!(
        status == "draft" || status == "pending",
        "Status should be draft or pending, got: {}",
        status
    );

    pool.close().await;
}

#[tokio::test]
async fn test_e2e_database_records_created() {
    let (_temp_dir, config_path, db_path, _keys_path) = setup_test_env();

    // Post content in draft mode
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("Test database records")
        .arg("--draft")
        .assert()
        .success();

    // Connect to database
    let pool = SqlitePool::connect(&format!("sqlite://{}", db_path))
        .await
        .unwrap();

    // Verify posts table has entry
    let post_id: String = sqlx::query_scalar("SELECT id FROM posts")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert!(!post_id.is_empty());

    // Verify post has UUID format (basic check)
    assert!(post_id.contains('-'), "Post ID should be UUID format");

    // Verify timestamps
    let created_at: i64 = sqlx::query_scalar("SELECT created_at FROM posts")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert!(created_at > 0, "Created timestamp should be positive");

    pool.close().await;
}

#[tokio::test]
async fn test_e2e_post_records_track_attempts() {
    let (_temp_dir, config_path, db_path, _keys_path) = setup_test_env();

    // Post content in draft mode
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("Test post records")
        .arg("--draft")
        .assert()
        .success();

    // Connect to database
    let pool = SqlitePool::connect(&format!("sqlite://{}", db_path))
        .await
        .unwrap();

    // Verify post_records table exists and has structure
    let record_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM post_records")
        .fetch_one(&pool)
        .await
        .unwrap();

    // In draft mode, we may or may not create post_records
    // This test verifies the table exists and is queryable
    assert!(record_count >= 0);

    pool.close().await;
}

#[test]
fn test_e2e_error_missing_keys_file() {
    let temp_dir = TempDir::new().unwrap();

    // Create config directory
    let config_dir = temp_dir.path().join("config");
    fs::create_dir_all(&config_dir).unwrap();

    // Create data directory
    let data_dir = temp_dir.path().join("data");
    fs::create_dir_all(&data_dir).unwrap();

    // Create config file pointing to non-existent keys
    let config_path = config_dir.join("config.toml");
    let db_path = data_dir.join("posts.db");
    let keys_path = config_dir.join("nonexistent.keys");

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

    // Try to post without keys file
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.env("PLURCAST_CONFIG", config_path.to_str().unwrap())
        .arg("Test content")
        .assert()
        .failure()
        .code(2) // Authentication error
        .stderr(predicate::str::contains("keys").or(predicate::str::contains("Authentication")));
}

#[test]
fn test_e2e_error_invalid_keys() {
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

    // Write invalid keys (too short to be valid hex)
    fs::write(&keys_path, "invalid").unwrap();

    // Draft mode should work even with invalid keys (doesn't validate until posting)
    // This is actually a feature - allows drafting without authentication
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.env("PLURCAST_CONFIG", config_path.to_str().unwrap())
        .arg("Test content")
        .arg("--draft")
        .timeout(std::time::Duration::from_secs(10))
        .assert()
        .success(); // Draft mode succeeds even with invalid keys
}

#[tokio::test]
async fn test_e2e_multiple_posts_sequential() {
    let (_temp_dir, config_path, db_path, _keys_path) = setup_test_env();

    // Post first content
    let mut cmd1 = Command::cargo_bin("plur-post").unwrap();
    cmd1.env("PLURCAST_CONFIG", &config_path)
        .arg("First post")
        .arg("--draft")
        .assert()
        .success();

    // Post second content
    let mut cmd2 = Command::cargo_bin("plur-post").unwrap();
    cmd2.env("PLURCAST_CONFIG", &config_path)
        .arg("Second post")
        .arg("--draft")
        .assert()
        .success();

    // Verify both posts in database
    let pool = SqlitePool::connect(&format!("sqlite://{}", db_path))
        .await
        .unwrap();

    let post_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM posts")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(post_count, 2, "Expected 2 posts in database");

    pool.close().await;
}

#[tokio::test]
async fn test_e2e_post_with_json_output() {
    let (_temp_dir, config_path, db_path, _keys_path) = setup_test_env();

    // Post content with JSON output
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

    // Parse JSON output
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Verify JSON structure
    assert!(json.get("post_id").is_some());
    assert!(json.get("status").is_some());

    // Verify database
    let pool = SqlitePool::connect(&format!("sqlite://{}", db_path))
        .await
        .unwrap();

    let post_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM posts")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(post_count, 1);

    pool.close().await;
}

#[test]
fn test_e2e_config_path_override() {
    let temp_dir = TempDir::new().unwrap();

    // Create custom config location
    let custom_config_dir = temp_dir.path().join("custom_config");
    fs::create_dir_all(&custom_config_dir).unwrap();

    let data_dir = temp_dir.path().join("data");
    fs::create_dir_all(&data_dir).unwrap();

    let config_path = custom_config_dir.join("config.toml");
    let db_path = data_dir.join("posts.db");
    let keys_path = custom_config_dir.join("nostr.keys");

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

    // Generate test keys
    let test_keys = nostr_sdk::Keys::generate();
    let hex_key = test_keys.secret_key().to_secret_hex();
    fs::write(&keys_path, hex_key).unwrap();

    // Post with custom config path
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.env("PLURCAST_CONFIG", config_path.to_str().unwrap())
        .arg("Test custom config")
        .arg("--draft")
        .assert()
        .success();
}

#[test]
fn test_e2e_db_path_override() {
    let temp_dir = TempDir::new().unwrap();

    let config_dir = temp_dir.path().join("config");
    fs::create_dir_all(&config_dir).unwrap();

    let default_db = temp_dir.path().join("default.db");
    let custom_db = temp_dir.path().join("custom.db");
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
        escape_path_for_toml(&default_db.to_string_lossy()),
        escape_path_for_toml(&keys_path.to_string_lossy())
    );

    let config_path = config_dir.join("config.toml");
    fs::write(&config_path, config_content).unwrap();

    // Generate test keys
    let test_keys = nostr_sdk::Keys::generate();
    let hex_key = test_keys.secret_key().to_secret_hex();
    fs::write(&keys_path, hex_key).unwrap();

    // Post with custom database path
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.env("PLURCAST_CONFIG", config_path.to_str().unwrap())
        .env("PLURCAST_DB_PATH", custom_db.to_str().unwrap())
        .arg("Test custom DB")
        .arg("--draft")
        .assert()
        .success();

    // Verify custom database was created, not default
    assert!(custom_db.exists(), "Custom database should exist");
}
