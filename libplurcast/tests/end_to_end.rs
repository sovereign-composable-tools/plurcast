//! End-to-end workflow tests for multi-platform posting
//!
//! These tests verify complete workflows including:
//! - Posting to all platforms
//! - Posting with partial failures
//! - Querying history after posting
//! - Configuration loading and validation

use anyhow::Result;
use libplurcast::config::Config;
use libplurcast::db::Database;
use libplurcast::platforms::mock::MockPlatform;
use libplurcast::platforms::Platform;
use libplurcast::poster::MultiPlatformPoster;
use libplurcast::types::{Post, PostRecord, PostStatus};
use std::time::Duration;
use tempfile::TempDir;

/// Helper to create a test database
async fn create_test_db() -> Result<(TempDir, Database)> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test.db");
    let db_path_str = db_path.to_string_lossy().to_string();

    let db = Database::new(&db_path_str).await?;
    Ok((temp_dir, db))
}

#[tokio::test]
async fn test_complete_posting_workflow_all_platforms() -> Result<()> {
    let (_temp_dir, db) = create_test_db().await?;

    // Create mock platforms
    let mut platforms: Vec<Box<dyn Platform>> = vec![
        Box::new(MockPlatform::success("nostr")),
        Box::new(MockPlatform::success("mastodon")),
        Box::new(MockPlatform::success("bluesky")),
    ];

    // Authenticate all platforms
    for platform in &mut platforms {
        platform.authenticate().await?;
    }

    // Create poster
    let poster = MultiPlatformPoster::new(platforms, db.clone());

    // Create a post
    let content = "Hello from all platforms!";
    let post = Post::new(content.to_string());

    // Post to all platforms
    let results = poster.post_to_all(&post).await;

    // Verify all succeeded
    assert_eq!(results.len(), 3);
    for result in &results {
        assert!(
            result.success,
            "Platform {} should succeed",
            result.platform
        );
        assert!(result.platform_post_id.is_some());
        assert!(result.error.is_none());
    }

    // Verify post was saved to database
    let saved_post = db.get_post(&post.id).await?;
    assert!(saved_post.is_some());
    let saved_post = saved_post.unwrap();
    assert_eq!(saved_post.content, content);
    assert!(matches!(saved_post.status, PostStatus::Posted));

    // Verify post records were created
    let records = db.get_post_records(&post.id).await?;
    assert_eq!(records.len(), 3);

    for record in records {
        assert!(record.success);
        assert!(record.platform_post_id.is_some());
        assert!(record.error_message.is_none());
    }

    Ok(())
}

#[tokio::test]
async fn test_posting_with_partial_failures() -> Result<()> {
    let (_temp_dir, db) = create_test_db().await?;

    // Create platforms with one failure
    let mut platforms: Vec<Box<dyn Platform>> = vec![
        Box::new(MockPlatform::success("nostr")),
        Box::new(MockPlatform::post_failure(
            "mastodon",
            "Rate limit exceeded",
        )),
        Box::new(MockPlatform::success("bluesky")),
    ];

    // Authenticate all platforms
    for platform in &mut platforms {
        platform.authenticate().await?;
    }

    // Create poster
    let poster = MultiPlatformPoster::new(platforms, db.clone());

    // Create a post
    let content = "Testing partial failure";
    let post = Post::new(content.to_string());

    // Post to all platforms
    let results = poster.post_to_all(&post).await;

    // Verify results
    assert_eq!(results.len(), 3);

    let nostr_result = results.iter().find(|r| r.platform == "nostr").unwrap();
    assert!(nostr_result.success);
    assert!(nostr_result.platform_post_id.is_some());

    let mastodon_result = results.iter().find(|r| r.platform == "mastodon").unwrap();
    assert!(!mastodon_result.success);
    assert!(mastodon_result.platform_post_id.is_none());
    assert!(mastodon_result
        .error
        .as_ref()
        .unwrap()
        .contains("Rate limit"));

    let bluesky_result = results.iter().find(|r| r.platform == "bluesky").unwrap();
    assert!(bluesky_result.success);
    assert!(bluesky_result.platform_post_id.is_some());

    // Verify database records
    let records = db.get_post_records(&post.id).await?;
    assert_eq!(records.len(), 3);

    let success_count = records.iter().filter(|r| r.success).count();
    let failure_count = records.iter().filter(|r| !r.success).count();
    assert_eq!(success_count, 2);
    assert_eq!(failure_count, 1);

    Ok(())
}

#[tokio::test]
async fn test_querying_history_after_posting() -> Result<()> {
    let (_temp_dir, db) = create_test_db().await?;

    // Create mock platforms
    let mut platforms: Vec<Box<dyn Platform>> = vec![
        Box::new(MockPlatform::success("nostr")),
        Box::new(MockPlatform::success("mastodon")),
    ];

    for platform in &mut platforms {
        platform.authenticate().await?;
    }

    let poster = MultiPlatformPoster::new(platforms, db.clone());

    // Post multiple messages
    let posts = vec![
        Post::new("First post".to_string()),
        Post::new("Second post".to_string()),
        Post::new("Third post".to_string()),
    ];

    for post in &posts {
        poster.post_to_all(post).await;
        // Small delay to ensure different timestamps
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    // Query all posts with records
    let all_posts = db
        .query_posts_with_records(None, None, None, None, 10)
        .await?;
    assert_eq!(all_posts.len(), 3);

    // Verify posts are in reverse chronological order
    assert_eq!(all_posts[0].post.content, "Third post");
    assert_eq!(all_posts[1].post.content, "Second post");
    assert_eq!(all_posts[2].post.content, "First post");

    // Query posts by platform
    for post in &posts {
        let records = db.get_post_records(&post.id).await?;
        assert_eq!(records.len(), 2); // nostr and mastodon

        let platforms: Vec<String> = records.iter().map(|r| r.platform.clone()).collect();
        assert!(platforms.contains(&"nostr".to_string()));
        assert!(platforms.contains(&"mastodon".to_string()));
    }

    Ok(())
}

#[tokio::test]
async fn test_configuration_loading_and_validation() -> Result<()> {
    use std::fs;

    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("config.toml");
    let db_path = temp_dir.path().join("posts.db");
    let keys_path = temp_dir.path().join("nostr.keys");

    // Create a valid configuration
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
token_file = "/tmp/mastodon.token"

[ssb]
enabled = false
feed_path = "/tmp/ssb.feed"

[defaults]
platforms = ["nostr", "mastodon"]
"#,
        db_path.display().to_string().replace('\\', "/"),
        keys_path.display().to_string().replace('\\', "/")
    );

    fs::write(&config_path, config_content)?;

    // Create dummy keys file
    let test_keys = nostr_sdk::Keys::generate();
    fs::write(&keys_path, test_keys.secret_key().to_secret_hex())?;

    // Load configuration
    let config = Config::load_from_path(&config_path)?;

    // Verify configuration
    assert!(config.nostr.is_some());
    assert!(config.mastodon.is_some());
    assert!(config.ssb.is_some());

    let nostr_config = config.nostr.unwrap();
    assert!(nostr_config.enabled);
    assert_eq!(nostr_config.relays.len(), 1);

    let mastodon_config = config.mastodon.unwrap();
    assert!(mastodon_config.enabled);
    assert_eq!(mastodon_config.instance, "mastodon.social");

    let ssb_config = config.ssb.unwrap();
    assert!(!ssb_config.enabled);

    let defaults = config.defaults;
    assert_eq!(defaults.platforms.len(), 2);
    assert!(defaults.platforms.contains(&"nostr".to_string()));
    assert!(defaults.platforms.contains(&"mastodon".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_concurrent_posting_performance() -> Result<()> {
    let (_temp_dir, db) = create_test_db().await?;

    // Create platforms with delays
    let mut platforms: Vec<Box<dyn Platform>> = vec![
        Box::new(MockPlatform::with_delay(
            "nostr",
            Duration::from_millis(100),
        )),
        Box::new(MockPlatform::with_delay(
            "mastodon",
            Duration::from_millis(100),
        )),
        Box::new(MockPlatform::with_delay(
            "bluesky",
            Duration::from_millis(100),
        )),
    ];

    for platform in &mut platforms {
        platform.authenticate().await?;
    }

    let poster = MultiPlatformPoster::new(platforms, db.clone());

    let post = Post::new("Testing concurrent performance".to_string());

    // Measure time for concurrent posting
    let start = std::time::Instant::now();
    let results = poster.post_to_all(&post).await;
    let duration = start.elapsed();

    // Verify all succeeded
    assert_eq!(results.len(), 3);
    for result in &results {
        assert!(result.success);
    }

    // Concurrent posting should take roughly the time of the slowest platform
    // (100ms) plus overhead, not the sum (300ms)
    assert!(
        duration < Duration::from_millis(250),
        "Concurrent posting took {:?}, expected < 250ms",
        duration
    );

    Ok(())
}

#[tokio::test]
async fn test_selective_platform_posting() -> Result<()> {
    let (_temp_dir, db) = create_test_db().await?;

    // Create three platforms
    let mut platforms: Vec<Box<dyn Platform>> = vec![
        Box::new(MockPlatform::success("nostr")),
        Box::new(MockPlatform::success("mastodon")),
        Box::new(MockPlatform::success("bluesky")),
    ];

    for platform in &mut platforms {
        platform.authenticate().await?;
    }

    let poster = MultiPlatformPoster::new(platforms, db.clone());

    let post = Post::new("Selective posting test".to_string());

    // Post to only nostr and bluesky
    let selected_platforms = vec!["nostr", "bluesky"];
    let results = poster.post_to_selected(&post, &selected_platforms).await;

    // Verify only selected platforms were used
    assert_eq!(results.len(), 2);

    let platform_names: Vec<String> = results.iter().map(|r| r.platform.clone()).collect();
    assert!(platform_names.contains(&"nostr".to_string()));
    assert!(platform_names.contains(&"bluesky".to_string()));
    assert!(!platform_names.contains(&"mastodon".to_string()));

    // Verify database records
    let records = db.get_post_records(&post.id).await?;
    assert_eq!(records.len(), 2);

    Ok(())
}

#[tokio::test]
async fn test_content_validation_across_platforms() -> Result<()> {
    let (_temp_dir, _db) = create_test_db().await?;

    // Create platforms with different character limits
    let mut platforms: Vec<Box<dyn Platform>> = vec![
        Box::new(MockPlatform::success("nostr")), // No limit
        Box::new(MockPlatform::with_limit("mastodon", 500)),
        Box::new(MockPlatform::with_limit("bluesky", 300)),
    ];

    for platform in &mut platforms {
        platform.authenticate().await?;
    }

    // Test content that exceeds bluesky's limit
    let long_content = "a".repeat(350);

    // Validate against all platforms
    let mut validation_errors = Vec::new();
    for platform in &platforms {
        if let Err(e) = platform.validate_content(&long_content) {
            validation_errors.push((platform.name().to_string(), e.to_string()));
        }
    }

    // Should have one validation error (bluesky)
    assert_eq!(validation_errors.len(), 1);
    assert_eq!(validation_errors[0].0, "bluesky");
    assert!(validation_errors[0].1.contains("character limit"));

    Ok(())
}

#[tokio::test]
async fn test_authentication_failure_handling() -> Result<()> {
    let (_temp_dir, _db) = create_test_db().await?;

    // Create platforms with one auth failure
    let mut platforms: Vec<Box<dyn Platform>> = vec![
        Box::new(MockPlatform::success("nostr")),
        Box::new(MockPlatform::auth_failure("mastodon", "Invalid token")),
        Box::new(MockPlatform::success("bluesky")),
    ];

    // Try to authenticate all platforms
    let mut auth_results = Vec::new();
    for platform in &mut platforms {
        let result = platform.authenticate().await;
        auth_results.push((platform.name().to_string(), result));
    }

    // Verify authentication results
    assert!(auth_results[0].1.is_ok()); // nostr succeeds
    assert!(auth_results[1].1.is_err()); // mastodon fails
    assert!(auth_results[2].1.is_ok()); // bluesky succeeds

    let mastodon_error = auth_results[1].1.as_ref().unwrap_err();
    assert!(mastodon_error.to_string().contains("Invalid token"));

    Ok(())
}

#[tokio::test]
async fn test_post_status_tracking() -> Result<()> {
    let (_temp_dir, db) = create_test_db().await?;

    // Create platforms
    let mut platforms: Vec<Box<dyn Platform>> = vec![
        Box::new(MockPlatform::success("nostr")),
        Box::new(MockPlatform::post_failure("mastodon", "Network error")),
    ];

    for platform in &mut platforms {
        platform.authenticate().await?;
    }

    let poster = MultiPlatformPoster::new(platforms, db.clone());

    let post = Post::new("Status tracking test".to_string());

    // Post to platforms
    let results = poster.post_to_all(&post).await;

    // Verify mixed results
    assert_eq!(results.len(), 2);
    let success_count = results.iter().filter(|r| r.success).count();
    let failure_count = results.iter().filter(|r| !r.success).count();
    assert_eq!(success_count, 1);
    assert_eq!(failure_count, 1);

    // Check post status in database
    let saved_post = db.get_post(&post.id).await?;
    assert!(saved_post.is_some());

    // Post should be marked as posted even with partial failure
    let saved_post = saved_post.unwrap();
    assert!(matches!(saved_post.status, PostStatus::Posted));

    Ok(())
}

#[tokio::test]
async fn test_empty_content_validation() -> Result<()> {
    let (_temp_dir, _db) = create_test_db().await?;

    let mut platforms: Vec<Box<dyn Platform>> = vec![Box::new(MockPlatform::success("nostr"))];

    for platform in &mut platforms {
        platform.authenticate().await?;
    }

    // Try to validate empty content
    let result = platforms[0].validate_content("");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("cannot be empty"));

    Ok(())
}

#[tokio::test]
async fn test_database_transaction_integrity() -> Result<()> {
    let (_temp_dir, db) = create_test_db().await?;

    // Create a post
    let post = Post::new("Transaction test".to_string());

    // Save post to database
    db.create_post(&post).await?;

    // Verify post exists
    let saved_post = db.get_post(&post.id).await?;
    assert!(saved_post.is_some());
    assert_eq!(saved_post.unwrap().id, post.id);

    // Create post records
    let record1 = PostRecord {
        id: None,
        post_id: post.id.clone(),
        platform: "nostr".to_string(),
        platform_post_id: Some("note1abc".to_string()),
        posted_at: Some(chrono::Utc::now().timestamp()),
        success: true,
        error_message: None,
        account_name: "default".to_string(),
    };
    db.create_post_record(&record1).await?;

    let record2 = PostRecord {
        id: None,
        post_id: post.id.clone(),
        platform: "mastodon".to_string(),
        platform_post_id: Some("12345".to_string()),
        posted_at: Some(chrono::Utc::now().timestamp()),
        success: true,
        error_message: None,
        account_name: "default".to_string(),
    };
    db.create_post_record(&record2).await?;

    // Verify records
    let records = db.get_post_records(&post.id).await?;
    assert_eq!(records.len(), 2);

    Ok(())
}
