//! Integration tests for SSB import functionality
//!
//! These tests verify the SSB import functionality by:
//! 1. Creating test SSB feeds with messages
//! 2. Running the import process
//! 3. Verifying posts are correctly imported into the database

use libplurcast::config::{Config, DatabaseConfig, SSBConfig};
use libplurcast::credentials::{CredentialConfig, CredentialManager};
use libplurcast::platforms::ssb::{SSBKeypair, SSBMessage, SSBPlatform};
use sqlx::SqlitePool;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Helper to create a test configuration
fn create_test_config(temp_dir: &TempDir) -> Config {
    let db_path = temp_dir.path().join("test.db");
    let feed_path = temp_dir.path().join("ssb-feed");
    let creds_path = temp_dir.path().join("credentials");
    
    Config {
        database: DatabaseConfig {
            path: db_path.to_string_lossy().to_string(),
        },
        ssb: Some(SSBConfig {
            enabled: true,
            feed_path: feed_path.to_string_lossy().to_string(),
            pubs: vec![],
        }),
        credentials: Some(CredentialConfig {
            storage: "plain".to_string(),
            path: creds_path.to_string_lossy().to_string(),
            master_password: None,
        }),
        nostr: None,
        mastodon: None,
        defaults: Default::default(),
    }
}

/// Helper to create a test SSB feed with messages
async fn create_test_feed(
    feed_path: &PathBuf,
    keypair: &SSBKeypair,
    num_posts: usize,
) -> anyhow::Result<Vec<SSBMessage>> {
    std::fs::create_dir_all(feed_path)?;
    let messages_dir = feed_path.join("messages");
    std::fs::create_dir_all(&messages_dir)?;
    
    let mut messages = Vec::new();
    let mut previous: Option<String> = None;
    
    for i in 1..=num_posts {
        let content = format!("Test post number {}", i);
        let mut message = SSBMessage::new_post(
            &keypair.id,
            i as u64,
            previous.clone(),
            &content,
        );
        
        message.sign(keypair)?;
        
        // Calculate hash for next message
        let hash = message.calculate_hash()?;
        previous = Some(hash);
        
        // Write message to file
        let message_file = messages_dir.join(format!("{:010}.json", i));
        let message_json = serde_json::to_string_pretty(&message)?;
        std::fs::write(&message_file, message_json)?;
        
        messages.push(message);
    }
    
    // Write feed state
    if let Some(last_message) = messages.last() {
        let hash = last_message.calculate_hash()?;
        let state = serde_json::json!({
            "sequence": last_message.sequence,
            "previous": hash,
            "author": last_message.author,
            "updated_at": chrono::Utc::now().to_rfc3339(),
        });
        
        let state_json = serde_json::to_string_pretty(&state)?;
        std::fs::write(feed_path.join("feed.json"), state_json)?;
    }
    
    Ok(messages)
}

#[tokio::test]
async fn test_import_from_empty_feed() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);
    
    // Initialize database
    let db_url = format!("sqlite://{}?mode=rwc", config.database.path);
    let pool = SqlitePool::connect(&db_url).await.unwrap();
    
    // Run migrations
    sqlx::migrate!("../libplurcast/migrations")
        .run(&pool)
        .await
        .unwrap();
    
    // Create empty feed directory
    let feed_path = PathBuf::from(&config.ssb.as_ref().unwrap().feed_path);
    std::fs::create_dir_all(&feed_path).unwrap();
    
    // Generate keypair
    let keypair = SSBKeypair::generate();
    
    // Store credentials
    let credentials = CredentialManager::new(config.credentials.clone().unwrap()).unwrap();
    SSBPlatform::store_keypair(&credentials, &keypair, "default", true).unwrap();
    
    // Save config
    config.save().unwrap();
    
    // Run import command
    let output = Command::new(env!("CARGO_BIN_EXE_plur-import"))
        .arg("ssb")
        .arg("--account")
        .arg("default")
        .output()
        .expect("Failed to execute plur-import");
    
    // Should succeed even with empty feed
    assert!(output.status.success(), "Import should succeed with empty feed");
    
    // Verify no posts were imported
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM posts")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count.0, 0, "Should have 0 posts");
}

#[tokio::test]
async fn test_import_posts_from_feed() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);
    
    // Initialize database
    let db = Database::new(&config.database.path).await.unwrap();
    
    // Generate keypair
    let keypair = SSBKeypair::generate().unwrap();
    
    // Create test feed with 3 posts
    let feed_path = PathBuf::from(&config.ssb.feed_path);
    let messages = create_test_feed(&feed_path, &keypair, 3).await.unwrap();
    
    // Store credentials
    let credentials = CredentialManager::new(&config.credentials).unwrap();
    SSBPlatform::store_keypair(&credentials, &keypair, "default", true).unwrap();
    
    // Run import
    let result = plur_import::ssb::import_ssb(&config, &db, "default").await;
    assert!(result.is_ok(), "Import should succeed: {:?}", result.err());
    
    // Verify posts were imported
    let posts = sqlx::query!(
        "SELECT id, content, status FROM posts ORDER BY created_at"
    )
    .fetch_all(db.pool())
    .await
    .unwrap();
    
    assert_eq!(posts.len(), 3, "Should import 3 posts");
    
    for (i, post) in posts.iter().enumerate() {
        assert_eq!(post.status, "imported");
        assert!(post.content.contains(&format!("Test post number {}", i + 1)));
    }
    
    // Verify post records
    let records = sqlx::query!(
        "SELECT platform, platform_post_id, success FROM post_records WHERE platform = 'ssb'"
    )
    .fetch_all(db.pool())
    .await
    .unwrap();
    
    assert_eq!(records.len(), 3, "Should have 3 post records");
    
    for record in records {
        assert_eq!(record.platform, "ssb");
        assert!(record.platform_post_id.is_some());
        assert_eq!(record.success, 1);
    }
}

#[tokio::test]
async fn test_import_skips_duplicates() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);
    
    // Initialize database
    let db = Database::new(&config.database.path).await.unwrap();
    
    // Generate keypair
    let keypair = SSBKeypair::generate().unwrap();
    
    // Create test feed with 2 posts
    let feed_path = PathBuf::from(&config.ssb.feed_path);
    create_test_feed(&feed_path, &keypair, 2).await.unwrap();
    
    // Store credentials
    let credentials = CredentialManager::new(&config.credentials).unwrap();
    SSBPlatform::store_keypair(&credentials, &keypair, "default", true).unwrap();
    
    // First import
    let result = plur_import::ssb::import_ssb(&config, &db, "default").await;
    assert!(result.is_ok(), "First import should succeed");
    
    let posts_count = sqlx::query!("SELECT COUNT(*) as count FROM posts")
        .fetch_one(db.pool())
        .await
        .unwrap();
    assert_eq!(posts_count.count, 2, "Should have 2 posts after first import");
    
    // Second import (should skip duplicates)
    let result = plur_import::ssb::import_ssb(&config, &db, "default").await;
    assert!(result.is_ok(), "Second import should succeed");
    
    let posts_count = sqlx::query!("SELECT COUNT(*) as count FROM posts")
        .fetch_one(db.pool())
        .await
        .unwrap();
    assert_eq!(posts_count.count, 2, "Should still have 2 posts (no duplicates)");
}

#[tokio::test]
async fn test_import_skips_non_post_messages() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);
    
    // Initialize database
    let db = Database::new(&config.database.path).await.unwrap();
    
    // Generate keypair
    let keypair = SSBKeypair::generate().unwrap();
    
    // Create feed with mixed message types
    let feed_path = PathBuf::from(&config.ssb.feed_path);
    std::fs::create_dir_all(&feed_path).unwrap();
    let messages_dir = feed_path.join("messages");
    std::fs::create_dir_all(&messages_dir).unwrap();
    
    // Create a post message
    let mut post_message = SSBMessage::new_post(
        &keypair.id,
        1,
        None,
        "This is a post",
    );
    post_message.sign(&keypair).unwrap();
    
    let post_file = messages_dir.join("0000000001.json");
    let post_json = serde_json::to_string_pretty(&post_message).unwrap();
    std::fs::write(&post_file, post_json).unwrap();
    
    // Create a non-post message (e.g., "about" type)
    let about_content = serde_json::json!({
        "type": "about",
        "about": keypair.id,
        "name": "Test User"
    });
    
    let mut about_message = SSBMessage {
        previous: Some(post_message.calculate_hash().unwrap()),
        author: keypair.id.clone(),
        sequence: 2,
        timestamp: chrono::Utc::now().timestamp_millis(),
        hash: "sha256".to_string(),
        content: about_content,
        signature: None,
    };
    about_message.sign(&keypair).unwrap();
    
    let about_file = messages_dir.join("0000000002.json");
    let about_json = serde_json::to_string_pretty(&about_message).unwrap();
    std::fs::write(&about_file, about_json).unwrap();
    
    // Store credentials
    let credentials = CredentialManager::new(&config.credentials).unwrap();
    SSBPlatform::store_keypair(&credentials, &keypair, "default", true).unwrap();
    
    // Run import
    let result = plur_import::ssb::import_ssb(&config, &db, "default").await;
    assert!(result.is_ok(), "Import should succeed");
    
    // Verify only post message was imported
    let posts = sqlx::query!("SELECT COUNT(*) as count FROM posts")
        .fetch_one(db.pool())
        .await
        .unwrap();
    
    assert_eq!(posts.count, 1, "Should import only 1 post (skip 'about' message)");
}

#[tokio::test]
async fn test_import_handles_missing_feed() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(&temp_dir);
    
    // Initialize database
    let db = Database::new(&config.database.path).await.unwrap();
    
    // Generate keypair
    let keypair = SSBKeypair::generate().unwrap();
    
    // Store credentials
    let credentials = CredentialManager::new(&config.credentials).unwrap();
    SSBPlatform::store_keypair(&credentials, &keypair, "default", true).unwrap();
    
    // Run import without creating feed directory
    let result = plur_import::ssb::import_ssb(&config, &db, "default").await;
    
    // Should fail with appropriate error
    assert!(result.is_err(), "Import should fail when feed doesn't exist");
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("not found") || error_msg.contains("does not exist"),
        "Error should mention missing feed: {}",
        error_msg
    );
}
