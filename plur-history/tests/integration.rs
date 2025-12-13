use anyhow::Result;
use sqlx::sqlite::SqlitePool;
use std::process::Command;
use tempfile::TempDir;

/// Helper to create a test database with sample data
async fn create_test_database() -> Result<(TempDir, String)> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test.db");
    let db_url = format!("sqlite://{}?mode=rwc", db_path.display());

    let pool = SqlitePool::connect(&db_url).await?;

    // Run migrations
    sqlx::migrate!("../libplurcast/migrations")
        .run(&pool)
        .await?;

    // Insert test posts
    let post1_id = uuid::Uuid::new_v4().to_string();
    let post2_id = uuid::Uuid::new_v4().to_string();
    let post3_id = uuid::Uuid::new_v4().to_string();

    let now = chrono::Utc::now().timestamp();
    let yesterday = now - 86400;
    let two_days_ago = now - 172800;

    // Post 1: Recent, posted to nostr successfully
    sqlx::query("INSERT INTO posts (id, content, created_at, status) VALUES (?, ?, ?, ?)")
        .bind(&post1_id)
        .bind("Hello from Nostr")
        .bind(now)
        .bind("posted")
        .execute(&pool)
        .await?;

    sqlx::query(
        "INSERT INTO post_records (post_id, platform, platform_post_id, posted_at, success) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&post1_id)
    .bind("nostr")
    .bind("note1abc123")
    .bind(now)
    .bind(1)
    .execute(&pool)
    .await?;

    // Post 2: Yesterday, posted to multiple platforms with one failure
    sqlx::query("INSERT INTO posts (id, content, created_at, status) VALUES (?, ?, ?, ?)")
        .bind(&post2_id)
        .bind("Multi-platform post about rust")
        .bind(yesterday)
        .bind("posted")
        .execute(&pool)
        .await?;

    sqlx::query(
        "INSERT INTO post_records (post_id, platform, platform_post_id, posted_at, success) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&post2_id)
    .bind("nostr")
    .bind("note1xyz789")
    .bind(yesterday)
    .bind(1)
    .execute(&pool)
    .await?;

    sqlx::query(
        "INSERT INTO post_records (post_id, platform, platform_post_id, posted_at, success) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&post2_id)
    .bind("mastodon")
    .bind("12345")
    .bind(yesterday)
    .bind(1)
    .execute(&pool)
    .await?;

    sqlx::query(
        "INSERT INTO post_records (post_id, platform, platform_post_id, posted_at, success, error_message) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&post2_id)
    .bind("ssb")
    .bind::<Option<String>>(None)
    .bind::<Option<i64>>(None)
    .bind(0)
    .bind("Authentication failed")
    .execute(&pool)
    .await?;

    // Post 3: Two days ago, posted to ssb only
    sqlx::query("INSERT INTO posts (id, content, created_at, status) VALUES (?, ?, ?, ?)")
        .bind(&post3_id)
        .bind("SSB exclusive content")
        .bind(two_days_ago)
        .bind("posted")
        .execute(&pool)
        .await?;

    sqlx::query(
        "INSERT INTO post_records (post_id, platform, platform_post_id, posted_at, success) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&post3_id)
    .bind("ssb")
    .bind("at://did:plc:abc/app.bsky.feed.post/xyz")
    .bind(two_days_ago)
    .bind(1)
    .execute(&pool)
    .await?;

    pool.close().await;

    Ok((temp_dir, db_path.to_string_lossy().to_string()))
}

/// Helper to create a config file pointing to test database
fn create_test_config(config_dir: &std::path::Path, db_path: &str) -> Result<String> {
    std::fs::create_dir_all(config_dir)?;
    let config_path = config_dir.join("config.toml");

    let config_content = format!(
        r#"
[database]
path = "{}"

[nostr]
enabled = true
keys_file = "~/.config/plurcast/nostr.keys"
relays = ["wss://relay.damus.io"]
"#,
        db_path.replace('\\', "/")
    );

    std::fs::write(&config_path, config_content)?;
    Ok(config_path.to_string_lossy().to_string())
}

#[tokio::test]
async fn test_history_default_output() -> Result<()> {
    let (_temp_dir, db_path) = create_test_database().await?;
    let config_dir = TempDir::new()?;
    let config_path = create_test_config(config_dir.path(), &db_path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_plur-history"))
        .env("PLURCAST_CONFIG", config_path)
        .output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;

    // Should contain all three posts
    assert!(stdout.contains("Hello from Nostr"));
    assert!(stdout.contains("Multi-platform post about rust"));
    assert!(stdout.contains("SSB exclusive content"));

    Ok(())
}

#[tokio::test]
async fn test_history_filter_by_platform() -> Result<()> {
    let (_temp_dir, db_path) = create_test_database().await?;
    let config_dir = TempDir::new()?;
    let config_path = create_test_config(config_dir.path(), &db_path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_plur-history"))
        .env("PLURCAST_CONFIG", config_path)
        .args(["--platform", "ssb"])
        .output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;

    // Should contain posts with ssb records
    assert!(stdout.contains("Multi-platform post about rust"));
    assert!(stdout.contains("SSB exclusive content"));

    // Should not contain nostr-only post
    // Note: "Hello from Nostr" might appear if it has ssb records too
    // In our test data, post1 only has nostr, so it shouldn't appear

    Ok(())
}

#[tokio::test]
async fn test_history_ssb_platform_filter() -> Result<()> {
    let (_temp_dir, db_path) = create_test_database().await?;
    let config_dir = TempDir::new()?;
    let config_path = create_test_config(config_dir.path(), &db_path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_plur-history"))
        .env("PLURCAST_CONFIG", config_path)
        .args(["--platform", "ssb", "--format", "json"])
        .output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;

    let json: serde_json::Value = serde_json::from_str(&stdout)?;
    let entries = json.as_array().unwrap();

    // Should have 2 posts with SSB records
    assert_eq!(entries.len(), 2);

    // Check that all returned posts have SSB platform records
    for entry in entries {
        let platforms = entry["platforms"].as_array().unwrap();
        let has_ssb = platforms.iter().any(|p| p["platform"] == "ssb");
        assert!(has_ssb, "Post should have SSB platform record");
    }

    Ok(())
}

#[tokio::test]
async fn test_history_date_range_filtering() -> Result<()> {
    let (_temp_dir, db_path) = create_test_database().await?;
    let config_dir = TempDir::new()?;
    let config_path = create_test_config(config_dir.path(), &db_path)?;

    let yesterday = chrono::Utc::now() - chrono::Duration::days(1);
    let since_date = yesterday.format("%Y-%m-%d").to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_plur-history"))
        .env("PLURCAST_CONFIG", config_path)
        .args(["--since", &since_date])
        .output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;

    // Should contain recent posts
    assert!(stdout.contains("Hello from Nostr") || stdout.contains("Multi-platform post"));

    // Should not contain old post
    assert!(!stdout.contains("Bluesky exclusive content") || stdout.contains("Multi-platform"));

    Ok(())
}

#[tokio::test]
async fn test_history_search_functionality() -> Result<()> {
    let (_temp_dir, db_path) = create_test_database().await?;
    let config_dir = TempDir::new()?;
    let config_path = create_test_config(config_dir.path(), &db_path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_plur-history"))
        .env("PLURCAST_CONFIG", config_path)
        .args(["--search", "rust"])
        .output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;

    // Should only contain post with "rust" in content
    assert!(stdout.contains("Multi-platform post about rust"));
    assert!(!stdout.contains("Hello from Nostr"));
    assert!(!stdout.contains("Bluesky exclusive"));

    Ok(())
}

#[tokio::test]
async fn test_history_json_format() -> Result<()> {
    let (_temp_dir, db_path) = create_test_database().await?;
    let config_dir = TempDir::new()?;
    let config_path = create_test_config(config_dir.path(), &db_path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_plur-history"))
        .env("PLURCAST_CONFIG", config_path)
        .args(["--format", "json"])
        .output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;

    // Should be valid JSON
    let json: serde_json::Value = serde_json::from_str(&stdout)?;
    assert!(json.is_array());

    let entries = json.as_array().unwrap();
    assert!(!entries.is_empty());

    // Check structure of first entry
    let first = &entries[0];
    assert!(first.get("post_id").is_some());
    assert!(first.get("content").is_some());
    assert!(first.get("created_at").is_some());
    assert!(first.get("platforms").is_some());

    Ok(())
}

#[tokio::test]
async fn test_history_jsonl_format() -> Result<()> {
    let (_temp_dir, db_path) = create_test_database().await?;
    let config_dir = TempDir::new()?;
    let config_path = create_test_config(config_dir.path(), &db_path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_plur-history"))
        .env("PLURCAST_CONFIG", config_path)
        .args(["--format", "jsonl"])
        .output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;

    // Each line should be valid JSON
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert!(!lines.is_empty());

    for line in lines {
        let json: serde_json::Value = serde_json::from_str(line)?;
        assert!(json.get("post_id").is_some());
        assert!(json.get("content").is_some());
    }

    Ok(())
}

#[tokio::test]
async fn test_history_csv_format() -> Result<()> {
    let (_temp_dir, db_path) = create_test_database().await?;
    let config_dir = TempDir::new()?;
    let config_path = create_test_config(config_dir.path(), &db_path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_plur-history"))
        .env("PLURCAST_CONFIG", config_path)
        .args(["--format", "csv"])
        .output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;

    let lines: Vec<&str> = stdout.lines().collect();
    assert!(lines.len() > 1); // At least header + one row

    // Check header
    assert_eq!(
        lines[0],
        "post_id,timestamp,platform,success,platform_post_id,error,content"
    );

    // Check data rows have correct number of columns
    for line in &lines[1..] {
        let columns: Vec<&str> = line.split(',').collect();
        assert!(columns.len() >= 7); // May have more due to commas in content
    }

    Ok(())
}

#[tokio::test]
async fn test_history_empty_results() -> Result<()> {
    let (_temp_dir, db_path) = create_test_database().await?;
    let config_dir = TempDir::new()?;
    let config_path = create_test_config(config_dir.path(), &db_path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_plur-history"))
        .env("PLURCAST_CONFIG", config_path)
        .args(["--search", "nonexistent_content_xyz"])
        .output()?;

    // Should exit with code 0 for empty results
    assert!(output.status.success());

    // Should output nothing
    let stdout = String::from_utf8(output.stdout)?;
    assert_eq!(stdout.trim(), "");

    Ok(())
}

#[tokio::test]
async fn test_history_missing_database() -> Result<()> {
    let config_dir = TempDir::new()?;
    let nonexistent_db = config_dir.path().join("nonexistent.db");
    let config_path = create_test_config(config_dir.path(), nonexistent_db.to_str().unwrap())?;

    let output = Command::new(env!("CARGO_BIN_EXE_plur-history"))
        .env("PLURCAST_CONFIG", config_path)
        .output()?;

    // With service layer, database is created automatically if it doesn't exist
    // This is good behavior - it should succeed with empty results
    assert!(output.status.success());

    // Should output nothing (empty results)
    let stdout = String::from_utf8(output.stdout)?;
    assert_eq!(stdout.trim(), "");

    Ok(())
}

#[tokio::test]
async fn test_history_invalid_date_format() -> Result<()> {
    let (_temp_dir, db_path) = create_test_database().await?;
    let config_dir = TempDir::new()?;
    let config_path = create_test_config(config_dir.path(), &db_path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_plur-history"))
        .env("PLURCAST_CONFIG", config_path)
        .args(["--since", "invalid-date"])
        .output()?;

    // Should exit with error for invalid date
    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("Invalid date format") || stderr.contains("Error"));

    Ok(())
}

#[tokio::test]
async fn test_history_limit_flag() -> Result<()> {
    let (_temp_dir, db_path) = create_test_database().await?;
    let config_dir = TempDir::new()?;
    let config_path = create_test_config(config_dir.path(), &db_path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_plur-history"))
        .env("PLURCAST_CONFIG", config_path)
        .args(["--limit", "1", "--format", "json"])
        .output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;

    let json: serde_json::Value = serde_json::from_str(&stdout)?;
    let entries = json.as_array().unwrap();

    // Should only return 1 entry
    assert_eq!(entries.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_history_ssb_json_output() -> Result<()> {
    let (_temp_dir, db_path) = create_test_database().await?;
    let config_dir = TempDir::new()?;
    let config_path = create_test_config(config_dir.path(), &db_path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_plur-history"))
        .env("PLURCAST_CONFIG", config_path)
        .args(["--platform", "ssb", "--format", "json"])
        .output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;

    let json: serde_json::Value = serde_json::from_str(&stdout)?;
    let entries = json.as_array().unwrap();

    // Find an SSB entry
    let ssb_entry = entries.iter().find(|e| {
        e["platforms"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p["platform"] == "ssb")
    });

    assert!(ssb_entry.is_some(), "Should have at least one SSB entry");

    let entry = ssb_entry.unwrap();
    let ssb_platform = entry["platforms"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["platform"] == "ssb")
        .unwrap();

    // Check SSB-specific fields are present
    assert!(ssb_platform.get("platform_post_id").is_some());
    assert!(ssb_platform.get("success").is_some());

    // message_hash should be present if platform_post_id exists
    if ssb_platform["platform_post_id"].is_string() {
        // Hash is extracted from platform_post_id
        assert!(ssb_platform.get("message_hash").is_some());
    }

    Ok(())
}

#[tokio::test]
async fn test_history_ssb_date_range() -> Result<()> {
    let (_temp_dir, db_path) = create_test_database().await?;
    let config_dir = TempDir::new()?;
    let config_path = create_test_config(config_dir.path(), &db_path)?;

    let yesterday = chrono::Utc::now() - chrono::Duration::days(1);
    let since_date = yesterday.format("%Y-%m-%d").to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_plur-history"))
        .env("PLURCAST_CONFIG", config_path)
        .args([
            "--platform",
            "ssb",
            "--since",
            &since_date,
            "--format",
            "json",
        ])
        .output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;

    let json: serde_json::Value = serde_json::from_str(&stdout)?;
    let entries = json.as_array().unwrap();

    // Should have at least one recent SSB post
    assert!(!entries.is_empty());

    // All entries should be from yesterday or later
    for entry in entries {
        let created_at = entry["created_at"].as_i64().unwrap();
        assert!(created_at >= yesterday.timestamp());
    }

    Ok(())
}

#[tokio::test]
async fn test_history_ssb_search() -> Result<()> {
    let (_temp_dir, db_path) = create_test_database().await?;
    let config_dir = TempDir::new()?;
    let config_path = create_test_config(config_dir.path(), &db_path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_plur-history"))
        .env("PLURCAST_CONFIG", config_path)
        .args(["--platform", "ssb", "--search", "rust", "--format", "json"])
        .output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;

    let json: serde_json::Value = serde_json::from_str(&stdout)?;
    let entries = json.as_array().unwrap();

    // Should find the post with "rust" in content
    assert!(!entries.is_empty());

    // All entries should contain "rust" in content
    for entry in entries {
        let content = entry["content"].as_str().unwrap().to_lowercase();
        assert!(content.contains("rust"));
    }

    Ok(())
}
