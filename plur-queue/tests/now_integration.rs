//! Integration tests for plur-queue now command (Phase 5.3 Task 14)

use assert_cmd::Command;
use predicates::prelude::*;
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

/// Helper to create a scheduled post in database
async fn create_scheduled_post(db_path: &str) -> String {
    use libplurcast::{Database, Post, PostStatus};

    let db = Database::new(db_path).await.unwrap();
    let now = chrono::Utc::now().timestamp();
    let post_id = uuid::Uuid::new_v4().to_string();

    let post = Post {
        id: post_id.clone(),
        content: "Test scheduled post".to_string(),
        created_at: now,
        scheduled_at: Some(now + 3600), // 1 hour from now
        status: PostStatus::Scheduled,
        metadata: Some(r#"{"platforms":["nostr"]}"#.to_string()),
    };
    db.create_post(&post).await.unwrap();

    post_id
}

// BASIC NOW TESTS

#[tokio::test]
async fn test_now_posts_scheduled_post() {
    let (_temp_dir, config_path, db_path) = setup_test_env().await;
    let post_id = create_scheduled_post(&db_path).await;

    let mut cmd = Command::cargo_bin("plur-queue").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("now")
        .arg(&post_id)
        .assert()
        .success()
        .stdout(predicate::str::contains("Posting"));

    // Verify post status changed from Scheduled to Posted/Pending
    let db = libplurcast::Database::new(&db_path).await.unwrap();
    let post = db.get_post(&post_id).await.unwrap().unwrap();
    assert_ne!(
        post.status,
        libplurcast::PostStatus::Scheduled,
        "Post should no longer be scheduled"
    );
}

#[tokio::test]
async fn test_now_clears_scheduled_at() {
    let (_temp_dir, config_path, db_path) = setup_test_env().await;
    let post_id = create_scheduled_post(&db_path).await;

    let mut cmd = Command::cargo_bin("plur-queue").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("now")
        .arg(&post_id)
        .assert()
        .success();

    // Verify scheduled_at was cleared
    let db = libplurcast::Database::new(&db_path).await.unwrap();
    let post = db.get_post(&post_id).await.unwrap().unwrap();
    assert_eq!(
        post.scheduled_at, None,
        "scheduled_at should be cleared after posting now"
    );
}

// ERROR HANDLING TESTS

#[tokio::test]
async fn test_now_nonexistent_post() {
    let (_temp_dir, config_path, _db_path) = setup_test_env().await;

    let fake_id = uuid::Uuid::new_v4().to_string();
    let mut cmd = Command::cargo_bin("plur-queue").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("now")
        .arg(&fake_id)
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("Post not found"));
}

#[tokio::test]
async fn test_now_invalid_post_id() {
    let (_temp_dir, config_path, _db_path) = setup_test_env().await;

    let mut cmd = Command::cargo_bin("plur-queue").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("now")
        .arg("not-a-uuid")
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("Invalid post ID format"));
}

#[tokio::test]
async fn test_now_already_posted() {
    let (_temp_dir, config_path, db_path) = setup_test_env().await;

    // Create an already posted post
    use libplurcast::{Database, Post, PostStatus};
    let db = Database::new(&db_path).await.unwrap();
    let now = chrono::Utc::now().timestamp();
    let post_id = uuid::Uuid::new_v4().to_string();

    let post = Post {
        id: post_id.clone(),
        content: "Already posted".to_string(),
        created_at: now,
        scheduled_at: None,
        status: PostStatus::Posted,
        metadata: None,
    };
    db.create_post(&post).await.unwrap();

    let mut cmd = Command::cargo_bin("plur-queue").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("now")
        .arg(&post_id)
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("not scheduled"));
}

// OUTPUT TESTS

#[tokio::test]
async fn test_now_shows_posting_message() {
    let (_temp_dir, config_path, db_path) = setup_test_env().await;
    let post_id = create_scheduled_post(&db_path).await;

    let mut cmd = Command::cargo_bin("plur-queue").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("now")
        .arg(&post_id)
        .assert()
        .success()
        .stdout(predicate::str::contains("Posting"));
}
