//! SSB export functionality

use anyhow::{Context, Result};
use libplurcast::db::Database;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::io::Write;
use tracing::{debug, info};

/// SSB message structure for export
#[derive(Debug, Serialize, Deserialize)]
pub struct SsbExportMessage {
    /// Original SSB message ID (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    
    /// Post content
    pub content: String,
    
    /// Unix timestamp in milliseconds
    pub timestamp: i64,
    
    /// Sequence number (if available from metadata)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sequence: Option<i64>,
    
    /// Post ID in Plurcast database
    pub post_id: String,
}

/// Query SSB posts from database
pub async fn query_ssb_posts(db: &Database) -> Result<Vec<SsbExportMessage>> {
    debug!("Querying SSB posts from database");
    
    let query = r#"
        SELECT 
            p.id as post_id,
            p.content,
            p.created_at,
            p.metadata,
            pr.platform_post_id
        FROM posts p
        INNER JOIN post_records pr ON p.id = pr.post_id
        WHERE pr.platform = 'ssb' AND pr.success = 1
        ORDER BY p.created_at ASC
    "#;
    
    let rows = sqlx::query(query)
        .fetch_all(db.pool())
        .await
        .context("Failed to query SSB posts")?;
    
    let mut messages = Vec::new();
    
    for row in rows {
        let post_id: String = row.try_get("post_id")?;
        let content: String = row.try_get("content")?;
        let created_at: i64 = row.try_get("created_at")?;
        let platform_post_id: Option<String> = row.try_get("platform_post_id")?;
        let metadata: Option<String> = row.try_get("metadata")?;
        
        // Try to extract sequence from metadata if available
        let sequence = metadata
            .as_ref()
            .and_then(|m| serde_json::from_str::<serde_json::Value>(m).ok())
            .and_then(|v| v.get("sequence").and_then(|s| s.as_i64()));
        
        messages.push(SsbExportMessage {
            message_id: platform_post_id,
            content,
            timestamp: created_at,
            sequence,
            post_id,
        });
    }
    
    info!("Found {} SSB posts to export", messages.len());
    Ok(messages)
}

/// Export SSB posts to output
pub async fn export_ssb(
    db: &Database,
    output_file: Option<String>,
) -> Result<()> {
    // Query posts
    let messages = query_ssb_posts(db).await?;
    
    if messages.is_empty() {
        info!("No SSB posts found to export");
        return Ok(());
    }
    
    // Format as JSON lines (one message per line)
    let mut output: Box<dyn Write> = if let Some(path) = output_file {
        let expanded_path = shellexpand::tilde(&path).to_string();
        let file = std::fs::File::create(&expanded_path)
            .with_context(|| format!("Failed to create output file: {}", expanded_path))?;
        info!("Exporting to file: {}", expanded_path);
        Box::new(file)
    } else {
        debug!("Exporting to stdout");
        Box::new(std::io::stdout())
    };
    
    // Write each message as a JSON line
    for message in &messages {
        let json = serde_json::to_string(message)
            .context("Failed to serialize message to JSON")?;
        writeln!(output, "{}", json)
            .context("Failed to write message to output")?;
    }
    
    info!("Successfully exported {} SSB messages", messages.len());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    async fn setup_test_db() -> Result<(Database, TempDir)> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");
        let db = Database::new(db_path.to_str().unwrap()).await?;
        Ok((db, temp_dir))
    }
    
    #[tokio::test]
    async fn test_query_empty_database() -> Result<()> {
        let (db, _temp_dir) = setup_test_db().await?;
        let messages = query_ssb_posts(&db).await?;
        assert_eq!(messages.len(), 0);
        Ok(())
    }
    
    #[tokio::test]
    async fn test_query_ssb_posts() -> Result<()> {
        let (db, _temp_dir) = setup_test_db().await?;
        
        // Insert test post
        let post_id = uuid::Uuid::new_v4().to_string();
        let content = "Test SSB post";
        let created_at = chrono::Utc::now().timestamp_millis();
        
        sqlx::query(
            "INSERT INTO posts (id, content, created_at, status) VALUES (?, ?, ?, 'posted')"
        )
        .bind(&post_id)
        .bind(content)
        .bind(created_at)
        .execute(db.pool())
        .await?;
        
        // Insert post record
        sqlx::query(
            "INSERT INTO post_records (post_id, platform, platform_post_id, success) VALUES (?, 'ssb', ?, 1)"
        )
        .bind(&post_id)
        .bind("ssb:%abc123")
        .execute(db.pool())
        .await?;
        
        // Query posts
        let messages = query_ssb_posts(&db).await?;
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, content);
        assert_eq!(messages[0].message_id, Some("ssb:%abc123".to_string()));
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_export_to_stdout() -> Result<()> {
        let (db, _temp_dir) = setup_test_db().await?;
        
        // Insert test post
        let post_id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO posts (id, content, created_at, status) VALUES (?, ?, ?, 'posted')"
        )
        .bind(&post_id)
        .bind("Test content")
        .bind(chrono::Utc::now().timestamp_millis())
        .execute(db.pool())
        .await?;
        
        sqlx::query(
            "INSERT INTO post_records (post_id, platform, platform_post_id, success) VALUES (?, 'ssb', ?, 1)"
        )
        .bind(&post_id)
        .bind("ssb:%test123")
        .execute(db.pool())
        .await?;
        
        // Export (to stdout, which we can't easily capture in test)
        let result = export_ssb(&db, None).await;
        assert!(result.is_ok());
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_export_to_file() -> Result<()> {
        let (db, temp_dir) = setup_test_db().await?;
        
        // Insert test post
        let post_id = uuid::Uuid::new_v4().to_string();
        let content = "Test SSB export";
        sqlx::query(
            "INSERT INTO posts (id, content, created_at, status) VALUES (?, ?, ?, 'posted')"
        )
        .bind(&post_id)
        .bind(content)
        .bind(chrono::Utc::now().timestamp_millis())
        .execute(db.pool())
        .await?;
        
        sqlx::query(
            "INSERT INTO post_records (post_id, platform, platform_post_id, success) VALUES (?, 'ssb', ?, 1)"
        )
        .bind(&post_id)
        .bind("ssb:%file123")
        .execute(db.pool())
        .await?;
        
        // Export to file
        let output_path = temp_dir.path().join("export.jsonl");
        export_ssb(&db, Some(output_path.to_str().unwrap().to_string())).await?;
        
        // Verify file contents
        let contents = std::fs::read_to_string(&output_path)?;
        assert!(contents.contains(content));
        assert!(contents.contains("ssb:%file123"));
        
        Ok(())
    }
}
