//! Integration tests for plur-send daemon

use assert_cmd::Command;
use libplurcast::{Database, Post, PostStatus};
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Setup test environment with config and database
async fn setup_test_env() -> (TempDir, String, String) {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");
    let db_path = temp_dir.path().join("test.db");

    // Create minimal config
    let config_content = format!(
        r#"
[database]
path = "{}"

[scheduling]
poll_interval = 1
max_retries = 3
retry_delay = 1

[scheduling.rate_limits.nostr]
posts_per_hour = 100

[scheduling.rate_limits.mastodon]
posts_per_hour = 300
"#,
        db_path.display().to_string().replace('\\', "/")
    );

    fs::write(&config_path, config_content).unwrap();

    // Initialize database
    let _db = Database::new(db_path.to_str().unwrap()).await.unwrap();

    (
        temp_dir,
        config_path.to_str().unwrap().to_string(),
        db_path.to_str().unwrap().to_string(),
    )
}

/// Create a scheduled post that is due for posting
async fn create_due_post(db_path: &str) -> String {
    let db = Database::new(db_path).await.unwrap();
    let now = chrono::Utc::now().timestamp();

    let post = Post {
        id: uuid::Uuid::new_v4().to_string(),
        content: "Test scheduled post".to_string(),
        created_at: now,
        scheduled_at: Some(now - 10), // 10 seconds in the past
        status: PostStatus::Scheduled,
        metadata: Some(r#"{"platforms":["nostr"]}"#.to_string()),
    };

    let post_id = post.id.clone();
    db.create_post(&post).await.unwrap();
    post_id
}

/// Create a failed post for retry testing
async fn create_failed_post(db_path: &str) -> String {
    let db = Database::new(db_path).await.unwrap();
    let now = chrono::Utc::now().timestamp();

    let post = Post {
        id: uuid::Uuid::new_v4().to_string(),
        content: "Test failed post".to_string(),
        created_at: now,
        scheduled_at: None,
        status: PostStatus::Failed,
        metadata: Some(r#"{"platforms":["nostr"]}"#.to_string()),
    };

    let post_id = post.id.clone();
    db.create_post(&post).await.unwrap();
    post_id
}

// BASIC FUNCTIONALITY TESTS

#[tokio::test]
async fn test_daemon_starts_with_config() {
    let (_temp_dir, config_path, _db_path) = setup_test_env().await;

    let mut cmd = Command::cargo_bin("plur-send").unwrap();

    // Run with --once flag to exit immediately
    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("--once")
        .assert()
        .success();
}

#[tokio::test]
async fn test_daemon_requires_valid_config() {
    let temp_dir = TempDir::new().unwrap();
    let invalid_config = temp_dir.path().join("invalid.toml");

    // Create invalid config
    fs::write(&invalid_config, "invalid toml content [[[").unwrap();

    let mut cmd = Command::cargo_bin("plur-send").unwrap();

    cmd.env("PLURCAST_CONFIG", invalid_config.to_str().unwrap())
        .arg("--once")
        .assert()
        .failure();
}

#[tokio::test]
async fn test_once_flag_exits_immediately() {
    let (_temp_dir, config_path, _db_path) = setup_test_env().await;

    let mut cmd = Command::cargo_bin("plur-send").unwrap();

    // Should exit successfully with --once even with no posts
    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("--once")
        .assert()
        .success()
        .stderr(predicate::str::contains("plur-send daemon starting"))
        .stderr(predicate::str::contains("processed posts once, exiting"));
}

#[tokio::test]
async fn test_verbose_logging() {
    let (_temp_dir, config_path, _db_path) = setup_test_env().await;

    let mut cmd = Command::cargo_bin("plur-send").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("--once")
        .arg("--verbose")
        .assert()
        .success();
}

#[tokio::test]
async fn test_custom_poll_interval() {
    let (_temp_dir, config_path, _db_path) = setup_test_env().await;

    let mut cmd = Command::cargo_bin("plur-send").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("--once")
        .arg("--poll-interval")
        .arg("30")
        .assert()
        .success()
        .stderr(predicate::str::contains("Poll interval: 30s"));
}

// POST PROCESSING TESTS

#[tokio::test]
async fn test_processes_due_posts() {
    let (_temp_dir, config_path, db_path) = setup_test_env().await;
    let _post_id = create_due_post(&db_path).await;

    let mut cmd = Command::cargo_bin("plur-send").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("--once")
        .assert()
        .success()
        .stderr(predicate::str::contains("Found 1 post(s) due for posting"));
}

#[tokio::test]
async fn test_no_posts_due() {
    let (_temp_dir, config_path, _db_path) = setup_test_env().await;

    let mut cmd = Command::cargo_bin("plur-send").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("--once")
        .assert()
        .success();
    // Should not log "Found X post(s)" if no posts are due
}

#[tokio::test]
async fn test_processes_multiple_due_posts() {
    let (_temp_dir, config_path, db_path) = setup_test_env().await;

    // Create 3 due posts
    for _ in 0..3 {
        create_due_post(&db_path).await;
    }

    let mut cmd = Command::cargo_bin("plur-send").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("--once")
        .assert()
        .success()
        .stderr(predicate::str::contains("Found 3 post(s) due for posting"));
}

// CONFIGURATION TESTS

#[tokio::test]
async fn test_loads_scheduling_config() {
    let (_temp_dir, config_path, _db_path) = setup_test_env().await;

    let mut cmd = Command::cargo_bin("plur-send").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("--once")
        .assert()
        .success()
        .stderr(predicate::str::contains("max_retries=3"))
        .stderr(predicate::str::contains("retry_delay=1s"))
        .stderr(predicate::str::contains("rate_limits=2 platforms"));
}

#[tokio::test]
async fn test_defaults_without_scheduling_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");
    let db_path = temp_dir.path().join("test.db");

    // Create minimal config without scheduling section
    let config_content = format!(
        r#"
[database]
path = "{}"
"#,
        db_path.display().to_string().replace('\\', "/")
    );

    fs::write(&config_path, config_content).unwrap();

    // Initialize database
    let _db = Database::new(db_path.to_str().unwrap()).await.unwrap();

    let mut cmd = Command::cargo_bin("plur-send").unwrap();

    cmd.env("PLURCAST_CONFIG", config_path.to_str().unwrap())
        .arg("--once")
        .assert()
        .success()
        .stderr(predicate::str::contains("Poll interval: 60s"))
        .stderr(predicate::str::contains("No scheduling configuration found"));
}

// METADATA EXTRACTION TESTS

#[tokio::test]
async fn test_post_without_metadata() {
    let (_temp_dir, config_path, db_path) = setup_test_env().await;
    let db = Database::new(&db_path).await.unwrap();
    let now = chrono::Utc::now().timestamp();

    // Create post without metadata
    let post = Post {
        id: uuid::Uuid::new_v4().to_string(),
        content: "Post without metadata".to_string(),
        created_at: now,
        scheduled_at: Some(now - 10),
        status: PostStatus::Scheduled,
        metadata: None, // No metadata
    };

    db.create_post(&post).await.unwrap();

    let mut cmd = Command::cargo_bin("plur-send").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("--once")
        .assert()
        .success()
        .stderr(predicate::str::contains("Found 1 post(s) due for posting"));
}

#[tokio::test]
async fn test_post_with_empty_platforms() {
    let (_temp_dir, config_path, db_path) = setup_test_env().await;
    let db = Database::new(&db_path).await.unwrap();
    let now = chrono::Utc::now().timestamp();

    // Create post with empty platforms array
    let post = Post {
        id: uuid::Uuid::new_v4().to_string(),
        content: "Post with empty platforms".to_string(),
        created_at: now,
        scheduled_at: Some(now - 10),
        status: PostStatus::Scheduled,
        metadata: Some(r#"{"platforms":[]}"#.to_string()),
    };

    db.create_post(&post).await.unwrap();

    let mut cmd = Command::cargo_bin("plur-send").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("--once")
        .assert()
        .success();
}

// ERROR HANDLING TESTS

#[tokio::test]
async fn test_handles_missing_config_gracefully() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent_config = temp_dir.path().join("nonexistent.toml");

    let mut cmd = Command::cargo_bin("plur-send").unwrap();

    // Should fail gracefully if config file doesn't exist
    cmd.env("PLURCAST_CONFIG", nonexistent_config.to_str().unwrap())
        .arg("--once")
        .assert()
        .failure();
}

#[tokio::test]
async fn test_continues_on_post_processing_error() {
    let (_temp_dir, config_path, db_path) = setup_test_env().await;

    // Create multiple due posts
    for _ in 0..3 {
        create_due_post(&db_path).await;
    }

    let mut cmd = Command::cargo_bin("plur-send").unwrap();

    // Even if some posts fail, daemon should continue
    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("--once")
        .assert()
        .success();
}

// OUTPUT TESTS

#[tokio::test]
async fn test_logs_startup_message() {
    let (_temp_dir, config_path, _db_path) = setup_test_env().await;

    let mut cmd = Command::cargo_bin("plur-send").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("--once")
        .assert()
        .success()
        .stderr(predicate::str::contains("plur-send daemon starting"));
}

#[tokio::test]
async fn test_logs_shutdown_message() {
    let (_temp_dir, config_path, _db_path) = setup_test_env().await;

    let mut cmd = Command::cargo_bin("plur-send").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("--once")
        .assert()
        .success()
        .stderr(predicate::str::contains("plur-send daemon stopped"));
}

#[tokio::test]
async fn test_logs_post_processing() {
    let (_temp_dir, config_path, db_path) = setup_test_env().await;
    let post_id = create_due_post(&db_path).await;

    let mut cmd = Command::cargo_bin("plur-send").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("--once")
        .assert()
        .success()
        .stderr(predicate::str::contains(&format!("Processing post: {}", post_id)));
}
