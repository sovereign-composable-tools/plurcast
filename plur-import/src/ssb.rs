//! SSB feed import functionality
//!
//! This module handles importing posts from a local SSB feed database
//! into the Plurcast database.

use anyhow::{Context, Result};
use libplurcast::config::Config;
use libplurcast::credentials::CredentialManager;
use libplurcast::db::Database;
use libplurcast::platforms::ssb::{SSBMessage, SSBPlatform};
use std::collections::HashSet;
use std::path::PathBuf;
use tracing::{debug, info, warn};

/// Import summary statistics
#[derive(Debug, Default)]
struct ImportSummary {
    total_messages: usize,
    imported: usize,
    skipped_duplicates: usize,
    skipped_non_posts: usize,
    errors: Vec<String>,
}

impl ImportSummary {
    fn display(&self) {
        println!("\n=== Import Summary ===");
        println!("Total messages found: {}", self.total_messages);
        println!("Imported: {}", self.imported);
        println!("Skipped (duplicates): {}", self.skipped_duplicates);
        println!("Skipped (non-posts): {}", self.skipped_non_posts);
        
        if !self.errors.is_empty() {
            println!("\nErrors encountered: {}", self.errors.len());
            for (i, error) in self.errors.iter().enumerate() {
                println!("  {}. {}", i + 1, error);
            }
        }
        
        if self.imported > 0 {
            println!("\n✓ Successfully imported {} post(s)", self.imported);
        } else if self.total_messages == 0 {
            println!("\nNo messages found in SSB feed");
        } else {
            println!("\nNo new posts to import");
        }
    }
}

/// Import posts from SSB feed
pub async fn import_ssb(config: &Config, db: &Database, account: &str) -> Result<()> {
    info!("Starting SSB import for account '{}'", account);
    
    // Check if SSB is configured and enabled
    let ssb_config = config.ssb.as_ref()
        .ok_or_else(|| anyhow::anyhow!("SSB is not configured"))?;
    
    if !ssb_config.enabled {
        anyhow::bail!("SSB is not enabled in configuration");
    }
    
    // Initialize credential manager
    let cred_config = config.credentials.clone()
        .ok_or_else(|| anyhow::anyhow!("Credential configuration is missing"))?;
    let credentials = CredentialManager::new(cred_config)
        .context("Failed to initialize credential manager")?;
    
    // Initialize SSB platform
    let mut platform = SSBPlatform::new(ssb_config);
    platform
        .initialize_with_credentials(&credentials, account)
        .await
        .context("Failed to initialize SSB platform")?;
    
    info!("SSB platform initialized, querying feed database");
    
    // Query local SSB feed
    let feed_path = get_feed_path(&ssb_config.feed_path);
    let messages = query_feed_messages(&feed_path)
        .context("Failed to query SSB feed")?;
    
    info!("Found {} message(s) in SSB feed", messages.len());
    
    // Get existing SSB message IDs to avoid duplicates
    let existing_ids = get_existing_ssb_message_ids(db).await?;
    debug!("Found {} existing SSB message(s) in database", existing_ids.len());
    
    // Import messages
    let mut summary = ImportSummary {
        total_messages: messages.len(),
        ..Default::default()
    };
    
    for message in messages {
        match import_message(db, &message, &existing_ids, &mut summary).await {
            Ok(()) => {}
            Err(e) => {
                let error_msg = format!("Failed to import message {}: {}", message.sequence, e);
                warn!("{}", error_msg);
                summary.errors.push(error_msg);
            }
        }
    }
    
    // Display summary
    summary.display();
    
    if !summary.errors.is_empty() {
        anyhow::bail!("Import completed with {} error(s)", summary.errors.len());
    }
    
    Ok(())
}

/// Get the expanded feed path
fn get_feed_path(path: &str) -> PathBuf {
    let expanded = shellexpand::tilde(path).to_string();
    PathBuf::from(expanded)
}

/// Query all messages from the SSB feed database
fn query_feed_messages(feed_path: &PathBuf) -> Result<Vec<SSBMessage>> {
    if !feed_path.exists() {
        anyhow::bail!(
            "SSB feed database not found at: {}",
            feed_path.display()
        );
    }
    
    let messages_dir = feed_path.join("messages");
    if !messages_dir.exists() {
        info!("No messages directory found, feed is empty");
        return Ok(Vec::new());
    }
    
    let mut messages = Vec::new();
    
    // Read all message files
    let entries = std::fs::read_dir(&messages_dir)
        .context("Failed to read messages directory")?;
    
    for entry in entries {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        
        debug!("Reading message file: {}", path.display());
        
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read message file: {}", path.display()))?;
        
        let message: SSBMessage = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse message file: {}", path.display()))?;
        
        messages.push(message);
    }
    
    // Sort by sequence number
    messages.sort_by_key(|m| m.sequence);
    
    debug!("Loaded {} message(s) from feed", messages.len());
    
    Ok(messages)
}

/// Get existing SSB message IDs from the database
async fn get_existing_ssb_message_ids(db: &Database) -> Result<HashSet<String>> {
    // Use Database API to query existing posts
    // query_posts_with_records(platform, since, until, search_term, limit)
    let posts = db.query_posts_with_records(Some("ssb"), None, None, None, 10000).await
        .context("Failed to query existing SSB posts")?;
    
    let ids: HashSet<String> = posts
        .into_iter()
        .flat_map(|p| p.records)
        .filter_map(|r| r.platform_post_id)
        .collect();
    
    Ok(ids)
}

/// Import a single message into the database
async fn import_message(
    db: &Database,
    message: &SSBMessage,
    existing_ids: &HashSet<String>,
    summary: &mut ImportSummary,
) -> Result<()> {
    // Calculate message ID
    let message_id = message.calculate_hash()
        .context("Failed to calculate message hash")?;
    
    let ssb_message_id = if message_id.ends_with(".sha256") {
        format!("ssb:{}", &message_id[..message_id.len() - 7])
    } else {
        format!("ssb:{}", message_id)
    };
    
    // Check if already imported
    if existing_ids.contains(&ssb_message_id) {
        debug!("Skipping duplicate message: {}", ssb_message_id);
        summary.skipped_duplicates += 1;
        return Ok(());
    }
    
    // Check if it's a post message
    let content_obj = message.content.as_object()
        .context("Message content is not an object")?;
    
    let msg_type = content_obj.get("type")
        .and_then(|v| v.as_str())
        .context("Message has no type field")?;
    
    if msg_type != "post" {
        debug!("Skipping non-post message (type: {})", msg_type);
        summary.skipped_non_posts += 1;
        return Ok(());
    }
    
    // Extract post text
    let text = content_obj.get("text")
        .and_then(|v| v.as_str())
        .context("Post message has no text field")?;
    
    // Create post record
    let post_id = uuid::Uuid::new_v4().to_string();
    let created_at = message.timestamp / 1000; // Convert milliseconds to seconds
    
    debug!(
        "Importing post: sequence={}, id={}, length={}",
        message.sequence,
        post_id,
        text.len()
    );
    
    // Create post using Database API
    let post = libplurcast::types::Post {
        id: post_id.clone(),
        content: text.to_string(),
        created_at,
        scheduled_at: None,
        status: libplurcast::types::PostStatus::Posted, // Mark as "posted" since it was already posted to SSB
        metadata: None,
    };
    
    db.create_post(&post).await
        .context("Failed to insert post")?;
    
    // Create post record using Database API
    let record = libplurcast::types::PostRecord {
        id: None, // Will be auto-assigned
        post_id: post_id.clone(),
        platform: "ssb".to_string(),
        platform_post_id: Some(ssb_message_id.clone()),
        posted_at: Some(created_at),
        success: true,
        error_message: None,
        account_name: "default".to_string(),
    };

    db.create_post_record(&record).await
        .context("Failed to insert post record")?;
    
    info!(
        "Imported SSB post: sequence={}, message_id={}",
        message.sequence,
        ssb_message_id
    );
    
    summary.imported += 1;
    
    Ok(())
}
