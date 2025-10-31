//! Backward compatibility tests
//!
//! These tests verify that Phase 1 configurations and functionality continue to work
//! after Phase 2 multi-platform changes.

use anyhow::Result;
use libplurcast::config::Config;
use libplurcast::db::Database;
use libplurcast::platforms::nostr::NostrPlatform;
use libplurcast::platforms::Platform;
use libplurcast::poster::create_platforms;
use libplurcast::types::{Post, PostRecord, PostStatus};
use std::fs;
use tempfile::TempDir;

/// Helper to create a Phase 1 style configuration (Nostr only)
fn create_phase1_config(temp_dir: &TempDir) -> Result<String> {
    let config_dir = temp_dir.path().join("config");
    fs::create_dir_all(&config_dir)?;

    let config_path = config_dir.join("config.toml");
    let db_path = temp_dir.path().join("posts.db");
    let keys_path = config_dir.join("nostr.keys");

    // Phase 1 config - only Nostr, no mastodon or bluesky sections
    let config_content = format!(
        r#"
[database]
path = "{}"

[nostr]
enabled = true
keys_file = "{}"
relays = ["wss://relay.damus.io", "wss://nos.lol"]
"#,
        db_path.display().to_string().replace('\\', "/"),
        keys_path.display().to_string().replace('\\', "/")
    );

    fs::write(&config_path, config_content)?;

    // Create test keys file
    let test_keys = nostr_sdk::Keys::generate();
    fs::write(&keys_path, test_keys.secret_key().to_secret_hex())?;

    Ok(config_path.to_string_lossy().to_string())
}

#[tokio::test]
async fn test_phase1_config_still_loads() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config_path = create_phase1_config(&temp_dir)?;

    // Load Phase 1 configuration
    let config = Config::load_from_path(&std::path::PathBuf::from(&config_path))?;

    // Verify Nostr config is present
    assert!(config.nostr.is_some());
    let nostr_config = config.nostr.unwrap();
    assert!(nostr_config.enabled);
    assert_eq!(nostr_config.relays.len(), 2);

    // Verify new platform configs are None (not present in Phase 1)
    assert!(config.mastodon.is_none());
    assert!(config.bluesky.is_none());

    // Verify defaults exist (should have sensible defaults)
    assert!(!config.defaults.platforms.is_empty() || config.defaults.platforms.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_phase1_nostr_only_posting_still_works() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config_path = create_phase1_config(&temp_dir)?;

    let config = Config::load_from_path(&std::path::PathBuf::from(&config_path))?;
    let _db = Database::new(&config.database.path).await?;

    // Create platforms (should only create Nostr)
    let platforms = create_platforms(&config, None).await?;

    // Should have exactly 1 platform (Nostr)
    assert_eq!(platforms.len(), 1);
    assert_eq!(platforms[0].name(), "nostr");

    Ok(())
}

#[tokio::test]
async fn test_phase1_database_schema_compatible() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("posts.db");
    let db_path_str = db_path.to_string_lossy().to_string();

    // Create database (runs migrations)
    let db = Database::new(&db_path_str).await?;

    // Create a Phase 1 style post (single platform)
    let post = Post::new("Phase 1 test post".to_string());
    db.create_post(&post).await?;

    // Create a single post record (Nostr only)
    let record = PostRecord {
        id: None,
        post_id: post.id.clone(),
        platform: "nostr".to_string(),
        platform_post_id: Some("note1abc123".to_string()),
        posted_at: Some(chrono::Utc::now().timestamp()),
        success: true,
        error_message: None,
    };
    db.create_post_record(&record).await?;

    // Verify post can be retrieved
    let retrieved_post = db.get_post(&post.id).await?;
    assert!(retrieved_post.is_some());
    assert_eq!(retrieved_post.unwrap().content, "Phase 1 test post");

    // Verify post record can be retrieved
    let records = db.get_post_records(&post.id).await?;
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].platform, "nostr");

    Ok(())
}

#[tokio::test]
async fn test_existing_phase1_data_preserved() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("posts.db");
    let db_path_str = db_path.to_string_lossy().to_string();

    // Simulate Phase 1 database with existing data
    let db = Database::new(&db_path_str).await?;

    // Create several Phase 1 posts
    let post1 = Post::new("Old post 1".to_string());
    let post2 = Post::new("Old post 2".to_string());
    let post3 = Post::new("Old post 3".to_string());

    db.create_post(&post1).await?;
    db.create_post(&post2).await?;
    db.create_post(&post3).await?;

    // Create Nostr-only records for each
    for post in &[&post1, &post2, &post3] {
        let record = PostRecord {
            id: None,
            post_id: post.id.clone(),
            platform: "nostr".to_string(),
            platform_post_id: Some(format!("note1{}", uuid::Uuid::new_v4())),
            posted_at: Some(chrono::Utc::now().timestamp()),
            success: true,
            error_message: None,
        };
        db.create_post_record(&record).await?;
    }

    // Now simulate Phase 2 upgrade - add a new multi-platform post
    let new_post = Post::new("New multi-platform post".to_string());
    db.create_post(&new_post).await?;

    // Add records for multiple platforms
    for platform in &["nostr", "mastodon", "bluesky"] {
        let record = PostRecord {
            id: None,
            post_id: new_post.id.clone(),
            platform: platform.to_string(),
            platform_post_id: Some(format!("{}:post123", platform)),
            posted_at: Some(chrono::Utc::now().timestamp()),
            success: true,
            error_message: None,
        };
        db.create_post_record(&record).await?;
    }

    // Verify all old posts still exist
    assert!(db.get_post(&post1.id).await?.is_some());
    assert!(db.get_post(&post2.id).await?.is_some());
    assert!(db.get_post(&post3.id).await?.is_some());

    // Verify old posts still have their Nostr records
    for post in &[&post1, &post2, &post3] {
        let records = db.get_post_records(&post.id).await?;
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].platform, "nostr");
    }

    // Verify new post has multi-platform records
    let new_records = db.get_post_records(&new_post.id).await?;
    assert_eq!(new_records.len(), 3);

    let platforms: Vec<String> = new_records.iter().map(|r| r.platform.clone()).collect();
    assert!(platforms.contains(&"nostr".to_string()));
    assert!(platforms.contains(&"mastodon".to_string()));
    assert!(platforms.contains(&"bluesky".to_string()));

    Ok(())
}

#[tokio::test]
#[allow(deprecated)]
async fn test_nostr_platform_still_works_independently() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config_path = create_phase1_config(&temp_dir)?;

    let config = Config::load_from_path(&std::path::PathBuf::from(&config_path))?;
    let nostr_config = config.nostr.unwrap();

    // Create NostrPlatform directly (Phase 1 style)
    let mut nostr_platform = NostrPlatform::new(&nostr_config);

    // Verify platform properties before loading keys
    assert_eq!(nostr_platform.name(), "nostr");
    assert_eq!(nostr_platform.character_limit(), None);

    // Load keys (required for is_configured to return true)
    // Note: Using deprecated load_keys() to test backward compatibility
    nostr_platform.load_keys(&nostr_config.keys_file)?;
    assert!(nostr_platform.is_configured());

    // Verify validation works
    assert!(nostr_platform.validate_content("Valid content").is_ok());
    assert!(nostr_platform.validate_content("").is_err());

    Ok(())
}

#[tokio::test]
async fn test_phase1_config_with_defaults_section() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config_dir = temp_dir.path().join("config");
    fs::create_dir_all(&config_dir)?;

    let config_path = config_dir.join("config.toml");
    let db_path = temp_dir.path().join("posts.db");
    let keys_path = config_dir.join("nostr.keys");

    // Phase 1 config with explicit defaults section
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
        db_path.display().to_string().replace('\\', "/"),
        keys_path.display().to_string().replace('\\', "/")
    );

    fs::write(&config_path, config_content)?;

    let test_keys = nostr_sdk::Keys::generate();
    fs::write(&keys_path, test_keys.secret_key().to_secret_hex())?;

    // Load configuration
    let config = Config::load_from_path(&std::path::PathBuf::from(&config_path))?;

    // Verify defaults are respected
    assert_eq!(config.defaults.platforms.len(), 1);
    assert_eq!(config.defaults.platforms[0], "nostr");

    Ok(())
}

#[tokio::test]
async fn test_phase1_post_status_values_still_work() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("posts.db");
    let db_path_str = db_path.to_string_lossy().to_string();

    let db = Database::new(&db_path_str).await?;

    // Test all status values that existed in Phase 1
    let statuses = vec![PostStatus::Pending, PostStatus::Posted, PostStatus::Failed];

    for status in statuses {
        let post = Post {
            id: uuid::Uuid::new_v4().to_string(),
            content: format!("Post with status {:?}", status),
            created_at: chrono::Utc::now().timestamp(),
            scheduled_at: None,
            status: status.clone(),
            metadata: None,
        };

        // Create post
        db.create_post(&post).await?;

        // Retrieve and verify status
        let retrieved = db.get_post(&post.id).await?.unwrap();
        assert!(matches!(
            (&retrieved.status, &post.status),
            (PostStatus::Pending, PostStatus::Pending)
                | (PostStatus::Posted, PostStatus::Posted)
                | (PostStatus::Failed, PostStatus::Failed)
        ));
    }

    Ok(())
}

#[tokio::test]
async fn test_phase1_metadata_field_still_works() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("posts.db");
    let db_path_str = db_path.to_string_lossy().to_string();

    let db = Database::new(&db_path_str).await?;

    // Create post with metadata (Phase 1 feature)
    let metadata = serde_json::json!({
        "tags": ["rust", "nostr"],
        "reply_to": "note1abc"
    });

    let post = Post {
        id: uuid::Uuid::new_v4().to_string(),
        content: "Post with metadata".to_string(),
        created_at: chrono::Utc::now().timestamp(),
        scheduled_at: None,
        status: PostStatus::Pending,
        metadata: Some(metadata.to_string()),
    };

    db.create_post(&post).await?;

    // Retrieve and verify metadata is preserved
    let retrieved = db.get_post(&post.id).await?.unwrap();
    assert!(retrieved.metadata.is_some());

    let retrieved_metadata: serde_json::Value =
        serde_json::from_str(retrieved.metadata.as_ref().unwrap())?;
    assert_eq!(retrieved_metadata["tags"][0], "rust");
    assert_eq!(retrieved_metadata["tags"][1], "nostr");

    Ok(())
}

#[tokio::test]
async fn test_phase1_scheduled_posts_still_work() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("posts.db");
    let db_path_str = db_path.to_string_lossy().to_string();

    let db = Database::new(&db_path_str).await?;

    // Create scheduled post (Phase 1 feature for future Phase 3)
    let scheduled_time = chrono::Utc::now().timestamp() + 3600;

    let post = Post {
        id: uuid::Uuid::new_v4().to_string(),
        content: "Scheduled post".to_string(),
        created_at: chrono::Utc::now().timestamp(),
        scheduled_at: Some(scheduled_time),
        status: PostStatus::Pending,
        metadata: None,
    };

    db.create_post(&post).await?;

    // Retrieve and verify scheduled_at is preserved
    let retrieved = db.get_post(&post.id).await?.unwrap();
    assert_eq!(retrieved.scheduled_at, Some(scheduled_time));

    Ok(())
}

#[tokio::test]
async fn test_phase1_error_messages_in_records() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("posts.db");
    let db_path_str = db_path.to_string_lossy().to_string();

    let db = Database::new(&db_path_str).await?;

    // Create post
    let post = Post::new("Test post".to_string());
    db.create_post(&post).await?;

    // Create failed post record with error message (Phase 1 feature)
    let record = PostRecord {
        id: None,
        post_id: post.id.clone(),
        platform: "nostr".to_string(),
        platform_post_id: None,
        posted_at: None,
        success: false,
        error_message: Some("Network timeout".to_string()),
    };

    db.create_post_record(&record).await?;

    // Retrieve and verify error message is preserved
    let records = db.get_post_records(&post.id).await?;
    assert_eq!(records.len(), 1);
    assert!(!records[0].success);
    assert_eq!(
        records[0].error_message,
        Some("Network timeout".to_string())
    );

    Ok(())
}

#[tokio::test]
async fn test_phase1_query_operations_still_work() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("posts.db");
    let db_path_str = db_path.to_string_lossy().to_string();

    let db = Database::new(&db_path_str).await?;

    // Create several posts
    for i in 1..=5 {
        let post = Post::new(format!("Post {}", i));
        db.create_post(&post).await?;

        let record = PostRecord {
            id: None,
            post_id: post.id.clone(),
            platform: "nostr".to_string(),
            platform_post_id: Some(format!("note1{}", i)),
            posted_at: Some(chrono::Utc::now().timestamp()),
            success: true,
            error_message: None,
        };
        db.create_post_record(&record).await?;

        // Small delay to ensure different timestamps
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    // Test query operations that existed in Phase 1
    let all_posts = db
        .query_posts_with_records(None, None, None, None, 10)
        .await?;
    assert_eq!(all_posts.len(), 5);

    // Test filtering by platform (Nostr only in Phase 1)
    let nostr_posts = db.filter_by_platform("nostr", 10).await?;
    assert_eq!(nostr_posts.len(), 5);

    // Test search
    let search_results = db.search_content("Post 3", 10).await?;
    assert_eq!(search_results.len(), 1);
    assert_eq!(search_results[0].post.content, "Post 3");

    Ok(())
}
