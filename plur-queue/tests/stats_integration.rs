//! Integration tests for plur-queue stats command (Phase 5.3 Task 15)

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

/// Helper to create scheduled posts with various times
async fn create_test_posts(db_path: &str) {
    use libplurcast::{Database, Post, PostStatus};

    let db = Database::new(db_path).await.unwrap();
    let now = chrono::Utc::now().timestamp();

    // Post in 30 minutes (next hour bucket)
    let post1 = Post {
        id: uuid::Uuid::new_v4().to_string(),
        content: "Post in 30 minutes".to_string(),
        created_at: now,
        scheduled_at: Some(now + 1800), // 30 minutes
        status: PostStatus::Scheduled,
        metadata: Some(r#"{"platforms":["nostr"]}"#.to_string()),
    };
    db.create_post(&post1).await.unwrap();

    // Post in 3 hours (today bucket)
    let post2 = Post {
        id: uuid::Uuid::new_v4().to_string(),
        content: "Post in 3 hours".to_string(),
        created_at: now,
        scheduled_at: Some(now + 10800), // 3 hours
        status: PostStatus::Scheduled,
        metadata: Some(r#"{"platforms":["nostr"]}"#.to_string()),
    };
    db.create_post(&post2).await.unwrap();

    // Post in 2 days (this week bucket)
    let post3 = Post {
        id: uuid::Uuid::new_v4().to_string(),
        content: "Post in 2 days".to_string(),
        created_at: now,
        scheduled_at: Some(now + 172800), // 2 days
        status: PostStatus::Scheduled,
        metadata: Some(r#"{"platforms":["nostr"]}"#.to_string()),
    };
    db.create_post(&post3).await.unwrap();

    // Post in 10 days (later bucket)
    let post4 = Post {
        id: uuid::Uuid::new_v4().to_string(),
        content: "Post in 10 days".to_string(),
        created_at: now,
        scheduled_at: Some(now + 864000), // 10 days
        status: PostStatus::Scheduled,
        metadata: Some(r#"{"platforms":["ssb"]}"#.to_string()),
    };
    db.create_post(&post4).await.unwrap();

    // Post in 15 days (later bucket)
    let post5 = Post {
        id: uuid::Uuid::new_v4().to_string(),
        content: "Post in 15 days".to_string(),
        created_at: now,
        scheduled_at: Some(now + 1296000), // 15 days
        status: PostStatus::Scheduled,
        metadata: Some(r#"{"platforms":["nostr"]}"#.to_string()),
    };
    db.create_post(&post5).await.unwrap();
}

// BASIC STATS TESTS

#[tokio::test]
async fn test_stats_shows_total_count() {
    let (_temp_dir, config_path, db_path) = setup_test_env().await;
    create_test_posts(&db_path).await;

    let mut cmd = Command::cargo_bin("plur-queue").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("stats")
        .assert()
        .success()
        .stdout(predicate::str::contains("Total: 5"));
}

#[tokio::test]
async fn test_stats_empty_queue() {
    let (_temp_dir, config_path, _db_path) = setup_test_env().await;

    let mut cmd = Command::cargo_bin("plur-queue").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("stats")
        .assert()
        .success()
        .stdout(predicate::str::contains("Total: 0"));
}

#[tokio::test]
async fn test_stats_shows_platform_counts() {
    let (_temp_dir, config_path, db_path) = setup_test_env().await;
    create_test_posts(&db_path).await;

    let mut cmd = Command::cargo_bin("plur-queue").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("stats")
        .assert()
        .success()
        .stdout(predicate::str::contains("nostr"))
        .stdout(predicate::str::contains("ssb"));
}

// TIME BUCKET TESTS

#[tokio::test]
async fn test_stats_shows_time_buckets() {
    let (_temp_dir, config_path, db_path) = setup_test_env().await;
    create_test_posts(&db_path).await;

    let mut cmd = Command::cargo_bin("plur-queue").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("stats")
        .assert()
        .success()
        .stdout(predicate::str::contains("Next hour"))
        .stdout(predicate::str::contains("Today"))
        .stdout(predicate::str::contains("This week"))
        .stdout(predicate::str::contains("Later"));
}

// UPCOMING POSTS TESTS

#[tokio::test]
async fn test_stats_shows_upcoming_posts() {
    let (_temp_dir, config_path, db_path) = setup_test_env().await;
    create_test_posts(&db_path).await;

    let mut cmd = Command::cargo_bin("plur-queue").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("stats")
        .assert()
        .success()
        .stdout(predicate::str::contains("Upcoming"));
}

#[tokio::test]
async fn test_stats_limits_upcoming_to_5() {
    let (_temp_dir, config_path, db_path) = setup_test_env().await;

    // Create 10 posts
    use libplurcast::{Database, Post, PostStatus};
    let db = Database::new(&db_path).await.unwrap();
    let now = chrono::Utc::now().timestamp();

    for i in 0..10 {
        let post = Post {
            id: uuid::Uuid::new_v4().to_string(),
            content: format!("Post {}", i),
            created_at: now,
            scheduled_at: Some(now + ((i + 1) * 3600)),
            status: PostStatus::Scheduled,
            metadata: None,
        };
        db.create_post(&post).await.unwrap();
    }

    let mut cmd = Command::cargo_bin("plur-queue").unwrap();

    let output = cmd
        .env("PLURCAST_CONFIG", &config_path)
        .arg("stats")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();

    // Count how many times "Post" appears in upcoming section
    // Should be at most 5
    let post_count = stdout.matches("Post ").count();
    assert!(
        post_count <= 5,
        "Should show at most 5 upcoming posts, found {}",
        post_count
    );
}

// JSON FORMAT TESTS

#[tokio::test]
async fn test_stats_json_format() {
    let (_temp_dir, config_path, db_path) = setup_test_env().await;
    create_test_posts(&db_path).await;

    let mut cmd = Command::cargo_bin("plur-queue").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("stats")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("{"))
        .stdout(predicate::str::contains("\"total\""))
        .stdout(predicate::str::contains("\"by_platform\""))
        .stdout(predicate::str::contains("\"by_time_bucket\""))
        .stdout(predicate::str::contains("\"upcoming\""));
}

#[tokio::test]
async fn test_stats_json_format_empty() {
    let (_temp_dir, config_path, _db_path) = setup_test_env().await;

    let mut cmd = Command::cargo_bin("plur-queue").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("stats")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"total\": 0"));
}

// ERROR HANDLING TESTS

#[tokio::test]
async fn test_stats_invalid_format() {
    let (_temp_dir, config_path, _db_path) = setup_test_env().await;

    let mut cmd = Command::cargo_bin("plur-queue").unwrap();

    cmd.env("PLURCAST_CONFIG", &config_path)
        .arg("stats")
        .arg("--format")
        .arg("invalid")
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("Invalid format"));
}
