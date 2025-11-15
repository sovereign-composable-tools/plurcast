//! Integration tests for plur-queue list command (Phase 5.3 Task 11)

use assert_cmd::Command;
use predicates::prelude::*;
use predicates::ord::eq;
use std::fs;
use tempfile::TempDir;

/// Helper to escape path for TOML on Windows
fn escape_path_for_toml(path: &str) -> String {
    path.replace('\\', "\\\\")
}

/// Helper to create a test environment with config and database
async fn setup_test_env() -> (TempDir, String, String) {
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

/// Helper to create scheduled posts in database
async fn create_scheduled_posts(db_path: &str, count: usize) {
    use libplurcast::{Database, Post, PostStatus};

    let db = Database::new(db_path).await.unwrap();
    let now = chrono::Utc::now().timestamp();

    for i in 0..count {
        let post = Post {
            id: uuid::Uuid::new_v4().to_string(),
            content: format!("Scheduled post {}", i + 1),
            created_at: now,
            scheduled_at: Some(now + ((i as i64 + 1) * 3600)), // 1h, 2h, 3h...
            status: PostStatus::Scheduled,
            metadata: None,
        };
        db.create_post(&post).await.unwrap();
    }
}

// BASIC LIST TESTS

#[tokio::test]
async fn test_list_empty_queue() {
    let (_temp_dir, config_path, _db_path) = setup_test_env().await;

    let mut cmd = Command::cargo_bin("plur-queue").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[tokio::test]
async fn test_list_shows_scheduled_posts() {
    let (_temp_dir, config_path, db_path) = setup_test_env().await;

    // Create 3 scheduled posts
    create_scheduled_posts(&db_path, 3).await;

    let mut cmd = Command::cargo_bin("plur-queue").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("Scheduled post 1"))
        .stdout(predicate::str::contains("Scheduled post 2"))
        .stdout(predicate::str::contains("Scheduled post 3"));
}

#[tokio::test]
async fn test_list_shows_post_ids() {
    let (_temp_dir, config_path, db_path) = setup_test_env().await;
    create_scheduled_posts(&db_path, 1).await;

    let mut cmd = Command::cargo_bin("plur-queue").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("list")
        .assert()
        .success()
        // Should show UUID format (8-4-4-4-12)
        .stdout(predicate::str::is_match(r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}").unwrap());
}

#[tokio::test]
async fn test_list_ordered_by_scheduled_time() {
    let (_temp_dir, config_path, db_path) = setup_test_env().await;
    create_scheduled_posts(&db_path, 3).await;

    let mut cmd = Command::cargo_bin("plur-queue").unwrap();

    let output = cmd
        .env("PLURCAST_CONFIG", &config_path)
        .arg("list")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();

    // Verify "Scheduled post 1" appears before "Scheduled post 2"
    let pos1 = stdout.find("Scheduled post 1").unwrap();
    let pos2 = stdout.find("Scheduled post 2").unwrap();
    let pos3 = stdout.find("Scheduled post 3").unwrap();

    assert!(pos1 < pos2, "Posts should be ordered by scheduled time");
    assert!(pos2 < pos3, "Posts should be ordered by scheduled time");
}

// JSON FORMAT TESTS

#[tokio::test]
async fn test_list_json_format() {
    let (_temp_dir, config_path, db_path) = setup_test_env().await;
    create_scheduled_posts(&db_path, 2).await;

    let mut cmd = Command::cargo_bin("plur-queue").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("list")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("["))
        .stdout(predicate::str::ends_with("]\n"))
        .stdout(predicate::str::contains("\"id\""))
        .stdout(predicate::str::contains("\"content\""))
        .stdout(predicate::str::contains("\"scheduled_at\""));
}

#[tokio::test]
async fn test_list_json_format_empty() {
    let (_temp_dir, config_path, _db_path) = setup_test_env().await;

    let mut cmd = Command::cargo_bin("plur-queue").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("list")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(eq("[]\n"));
}

// PLATFORM FILTERING TESTS

#[tokio::test]
async fn test_list_filter_by_platform() {
    let (_temp_dir, config_path, db_path) = setup_test_env().await;

    // Create posts for different platforms
    use libplurcast::{Database, Post, PostStatus};
    let db = Database::new(&db_path).await.unwrap();
    let now = chrono::Utc::now().timestamp();

    // Nostr post
    let nostr_post = Post {
        id: uuid::Uuid::new_v4().to_string(),
        content: "Nostr post".to_string(),
        created_at: now,
        scheduled_at: Some(now + 3600),
        status: PostStatus::Scheduled,
        metadata: Some(r#"{"platforms":["nostr"]}"#.to_string()),
    };
    db.create_post(&nostr_post).await.unwrap();

    // SSB post
    let ssb_post = Post {
        id: uuid::Uuid::new_v4().to_string(),
        content: "SSB post".to_string(),
        created_at: now,
        scheduled_at: Some(now + 7200),
        status: PostStatus::Scheduled,
        metadata: Some(r#"{"platforms":["ssb"]}"#.to_string()),
    };
    db.create_post(&ssb_post).await.unwrap();

    let mut cmd = Command::cargo_bin("plur-queue").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("list")
        .arg("--platform")
        .arg("nostr")
        .assert()
        .success()
        .stdout(predicate::str::contains("Nostr post"))
        .stdout(predicate::str::contains("SSB post").not());
}

// TIME DISPLAY TESTS

#[tokio::test]
async fn test_list_shows_time_until() {
    let (_temp_dir, config_path, db_path) = setup_test_env().await;
    create_scheduled_posts(&db_path, 1).await;

    let mut cmd = Command::cargo_bin("plur-queue").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("list")
        .assert()
        .success()
        // Should show "in X" format
        .stdout(predicate::str::contains("in "));
}

// CONTENT PREVIEW TESTS

#[tokio::test]
async fn test_list_truncates_long_content() {
    let (_temp_dir, config_path, db_path) = setup_test_env().await;

    use libplurcast::{Database, Post, PostStatus};
    let db = Database::new(&db_path).await.unwrap();
    let now = chrono::Utc::now().timestamp();

    // Create post with long content
    let long_content = "a".repeat(200);
    let post = Post {
        id: uuid::Uuid::new_v4().to_string(),
        content: long_content,
        created_at: now,
        scheduled_at: Some(now + 3600),
        status: PostStatus::Scheduled,
        metadata: None,
    };
    db.create_post(&post).await.unwrap();

    let mut cmd = Command::cargo_bin("plur-queue").unwrap();

    let output = cmd
        .env("PLURCAST_CONFIG", &config_path)
        .arg("list")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();

    // Content should be truncated (not show full 200 chars)
    // Assuming we truncate at 50 chars
    assert!(
        !stdout.contains(&"a".repeat(100)),
        "Long content should be truncated"
    );
    assert!(stdout.contains("..."), "Should show ellipsis for truncated content");
}

// ERROR HANDLING TESTS

#[tokio::test]
async fn test_list_invalid_format() {
    let (_temp_dir, config_path, _db_path) = setup_test_env().await;

    let mut cmd = Command::cargo_bin("plur-queue").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("list")
        .arg("--format")
        .arg("invalid")
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("Invalid format"));
}
