//! Integration tests for plur-queue cancel command (Phase 5.3 Task 12)

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

/// Helper to create scheduled posts in database
async fn create_scheduled_posts(db_path: &str, count: usize) -> Vec<String> {
    use libplurcast::{Database, Post, PostStatus};

    let db = Database::new(db_path).await.unwrap();
    let now = chrono::Utc::now().timestamp();
    let mut post_ids = Vec::new();

    for i in 0..count {
        let post_id = uuid::Uuid::new_v4().to_string();
        let post = Post {
            id: post_id.clone(),
            content: format!("Scheduled post {}", i + 1),
            created_at: now,
            scheduled_at: Some(now + ((i as i64 + 1) * 3600)), // 1h, 2h, 3h...
            status: PostStatus::Scheduled,
            metadata: None,
        };
        db.create_post(&post).await.unwrap();
        post_ids.push(post_id);
    }

    post_ids
}

// CANCEL SINGLE POST TESTS

#[tokio::test]
async fn test_cancel_single_post_with_force() {
    let (_temp_dir, config_path, db_path) = setup_test_env().await;
    let post_ids = create_scheduled_posts(&db_path, 3).await;

    let mut cmd = Command::cargo_bin("plur-queue").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("cancel")
        .arg(&post_ids[0])
        .arg("--force")
        .assert()
        .success()
        .stdout(predicate::str::contains("Cancelled post"));

    // Verify post was deleted
    let db = libplurcast::Database::new(&db_path).await.unwrap();
    let posts = db.get_scheduled_posts().await.unwrap();
    assert_eq!(posts.len(), 2, "Should have 2 posts remaining");
    assert!(!posts.iter().any(|p| p.id == post_ids[0]), "Cancelled post should be removed");
}

#[tokio::test]
async fn test_cancel_deletes_post() {
    let (_temp_dir, config_path, db_path) = setup_test_env().await;
    let post_ids = create_scheduled_posts(&db_path, 1).await;

    let mut cmd = Command::cargo_bin("plur-queue").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("cancel")
        .arg(&post_ids[0])
        .arg("--force")
        .assert()
        .success();

    // Verify post was deleted (not just status change)
    let db = libplurcast::Database::new(&db_path).await.unwrap();
    let post = db.get_post(&post_ids[0]).await.unwrap();
    assert!(post.is_none(), "Post should be deleted from database");
}

#[tokio::test]
async fn test_cancel_nonexistent_post() {
    let (_temp_dir, config_path, _db_path) = setup_test_env().await;

    let fake_id = uuid::Uuid::new_v4().to_string();
    let mut cmd = Command::cargo_bin("plur-queue").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("cancel")
        .arg(&fake_id)
        .arg("--force")
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("Post not found"));
}

// CANCEL ALL TESTS

#[tokio::test]
async fn test_cancel_all_with_force() {
    let (_temp_dir, config_path, db_path) = setup_test_env().await;
    create_scheduled_posts(&db_path, 5).await;

    let mut cmd = Command::cargo_bin("plur-queue").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("cancel")
        .arg("--all")
        .arg("--force")
        .assert()
        .success()
        .stdout(predicate::str::contains("Cancelled 5 post"));

    // Verify all posts were cancelled
    let db = libplurcast::Database::new(&db_path).await.unwrap();
    let posts = db.get_scheduled_posts().await.unwrap();
    assert_eq!(posts.len(), 0, "All scheduled posts should be cancelled");
}

#[tokio::test]
async fn test_cancel_all_empty_queue() {
    let (_temp_dir, config_path, _db_path) = setup_test_env().await;

    let mut cmd = Command::cargo_bin("plur-queue").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("cancel")
        .arg("--all")
        .arg("--force")
        .assert()
        .success()
        .stdout(predicate::str::contains("No scheduled posts to cancel"));
}

// ERROR HANDLING TESTS

#[tokio::test]
async fn test_cancel_requires_post_id_or_all() {
    let (_temp_dir, config_path, _db_path) = setup_test_env().await;

    let mut cmd = Command::cargo_bin("plur-queue").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("cancel")
        .arg("--force")
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("Must provide either POST_ID or --all"));
}

#[tokio::test]
async fn test_cancel_rejects_both_post_id_and_all() {
    let (_temp_dir, config_path, db_path) = setup_test_env().await;
    let post_ids = create_scheduled_posts(&db_path, 1).await;

    let mut cmd = Command::cargo_bin("plur-queue").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("cancel")
        .arg(&post_ids[0])
        .arg("--all")
        .arg("--force")
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("Cannot use both POST_ID and --all"));
}

#[tokio::test]
async fn test_cancel_invalid_post_id_format() {
    let (_temp_dir, config_path, _db_path) = setup_test_env().await;

    let mut cmd = Command::cargo_bin("plur-queue").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("cancel")
        .arg("not-a-uuid")
        .arg("--force")
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("Invalid post ID format"));
}

// CONFIRMATION PROMPT TESTS (without --force)

#[tokio::test]
async fn test_cancel_without_force_prompts_confirmation() {
    let (_temp_dir, config_path, db_path) = setup_test_env().await;
    let post_ids = create_scheduled_posts(&db_path, 1).await;

    let mut cmd = Command::cargo_bin("plur-queue").unwrap();

    // Without --force, should prompt for confirmation
    // We can't easily test interactive prompts, so we expect failure due to no stdin
    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("cancel")
        .arg(&post_ids[0])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Cancelled by user").or(predicate::str::contains("confirmation")));
}
