//! Integration tests for plur-queue reschedule command (Phase 5.3 Task 13)

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
        metadata: None,
    };
    db.create_post(&post).await.unwrap();

    post_id
}

// BASIC RESCHEDULE TESTS

#[tokio::test]
async fn test_reschedule_with_duration() {
    let (_temp_dir, config_path, db_path) = setup_test_env().await;
    let post_id = create_scheduled_post(&db_path).await;

    let mut cmd = Command::cargo_bin("plur-queue").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("reschedule")
        .arg(&post_id)
        .arg("2h")
        .assert()
        .success()
        .stdout(predicate::str::contains("Rescheduled post"));

    // Verify scheduled_at was updated
    let db = libplurcast::Database::new(&db_path).await.unwrap();
    let post = db.get_post(&post_id).await.unwrap().unwrap();
    assert!(post.scheduled_at.is_some(), "Post should have scheduled_at");
}

#[tokio::test]
async fn test_reschedule_with_natural_language() {
    let (_temp_dir, config_path, db_path) = setup_test_env().await;
    let post_id = create_scheduled_post(&db_path).await;

    let mut cmd = Command::cargo_bin("plur-queue").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("reschedule")
        .arg(&post_id)
        .arg("tomorrow")
        .assert()
        .success()
        .stdout(predicate::str::contains("Rescheduled post"));
}

// RELATIVE ADJUSTMENT TESTS

#[tokio::test]
async fn test_reschedule_relative_addition() {
    let (_temp_dir, config_path, db_path) = setup_test_env().await;
    let post_id = create_scheduled_post(&db_path).await;

    // Get original scheduled time
    let db = libplurcast::Database::new(&db_path).await.unwrap();
    let original_time = db
        .get_post(&post_id)
        .await
        .unwrap()
        .unwrap()
        .scheduled_at
        .unwrap();

    let mut cmd = Command::cargo_bin("plur-queue").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("reschedule")
        .arg(&post_id)
        .arg("+1h")
        .assert()
        .success();

    // Verify time increased by approximately 1 hour
    let new_time = db
        .get_post(&post_id)
        .await
        .unwrap()
        .unwrap()
        .scheduled_at
        .unwrap();
    let diff = new_time - original_time;
    assert!(
        diff >= 3590 && diff <= 3610,
        "Time should increase by ~1 hour (3600s), got {}s",
        diff
    );
}

#[tokio::test]
async fn test_reschedule_relative_subtraction() {
    let (_temp_dir, config_path, db_path) = setup_test_env().await;
    let post_id = create_scheduled_post(&db_path).await;

    // Get original scheduled time
    let db = libplurcast::Database::new(&db_path).await.unwrap();
    let original_time = db
        .get_post(&post_id)
        .await
        .unwrap()
        .unwrap()
        .scheduled_at
        .unwrap();

    let mut cmd = Command::cargo_bin("plur-queue").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("reschedule")
        .arg(&post_id)
        .arg("--")
        .arg("-30m")
        .assert()
        .success();

    // Verify time decreased by approximately 30 minutes
    let new_time = db
        .get_post(&post_id)
        .await
        .unwrap()
        .unwrap()
        .scheduled_at
        .unwrap();
    let diff = original_time - new_time;
    assert!(
        diff >= 1790 && diff <= 1810,
        "Time should decrease by ~30 minutes (1800s), got {}s",
        diff
    );
}

// ERROR HANDLING TESTS

#[tokio::test]
async fn test_reschedule_nonexistent_post() {
    let (_temp_dir, config_path, _db_path) = setup_test_env().await;

    let fake_id = uuid::Uuid::new_v4().to_string();
    let mut cmd = Command::cargo_bin("plur-queue").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("reschedule")
        .arg(&fake_id)
        .arg("2h")
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("Post not found"));
}

#[tokio::test]
async fn test_reschedule_invalid_post_id() {
    let (_temp_dir, config_path, _db_path) = setup_test_env().await;

    let mut cmd = Command::cargo_bin("plur-queue").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("reschedule")
        .arg("not-a-uuid")
        .arg("2h")
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("Invalid post ID format"));
}

#[tokio::test]
async fn test_reschedule_invalid_time_format() {
    let (_temp_dir, config_path, db_path) = setup_test_env().await;
    let post_id = create_scheduled_post(&db_path).await;

    let mut cmd = Command::cargo_bin("plur-queue").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("reschedule")
        .arg(&post_id)
        .arg("invalid-time")
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("Could not parse schedule"));
}

#[tokio::test]
async fn test_reschedule_to_past_time() {
    let (_temp_dir, config_path, db_path) = setup_test_env().await;
    let post_id = create_scheduled_post(&db_path).await;

    let mut cmd = Command::cargo_bin("plur-queue").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("reschedule")
        .arg(&post_id)
        .arg("--")
        .arg("-10h") // Move to past
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("Cannot schedule in the past"));
}

// OUTPUT TESTS

#[tokio::test]
async fn test_reschedule_shows_new_time() {
    let (_temp_dir, config_path, db_path) = setup_test_env().await;
    let post_id = create_scheduled_post(&db_path).await;

    let mut cmd = Command::cargo_bin("plur-queue").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("reschedule")
        .arg(&post_id)
        .arg("2h")
        .assert()
        .success()
        .stdout(predicate::str::contains("Rescheduled post"))
        .stdout(predicate::str::contains("for"));
}
