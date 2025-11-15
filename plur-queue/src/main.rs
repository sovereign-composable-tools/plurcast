//! plur-queue - Manage scheduled posts
//!
//! Unix-style tool for managing the scheduled post queue.

use clap::{Parser, Subcommand};
use libplurcast::{Config, Database, Result};

#[derive(Parser, Debug)]
#[command(name = "plur-queue")]
#[command(version)]
#[command(about = "Manage scheduled posts")]
#[command(long_about = "\
plur-queue - Manage scheduled posts

DESCRIPTION:
    plur-queue is a Unix-style tool for managing scheduled posts in the Plurcast queue.
    Use it to list, cancel, reschedule, or view statistics about your scheduled posts.

COMMANDS:
    list        List all scheduled posts
    cancel      Cancel a scheduled post
    reschedule  Reschedule a post to a different time
    now         Post a scheduled post immediately
    stats       Show statistics about scheduled posts

USAGE EXAMPLES:
    # List all scheduled posts
    plur-queue list

    # List posts in JSON format
    plur-queue list --format json

    # Cancel a specific post
    plur-queue cancel <POST_ID>

    # Reschedule a post
    plur-queue reschedule <POST_ID> \"tomorrow 3pm\"

    # Post a scheduled post immediately
    plur-queue now <POST_ID>

    # View queue statistics
    plur-queue stats

CONFIGURATION:
    Configuration file: ~/.config/plurcast/config.toml
    Database location: ~/.local/share/plurcast/posts.db

    Override with environment variables:
        PLURCAST_CONFIG    - Path to config file
        PLURCAST_DB_PATH   - Path to database file

EXIT CODES:
    0 - Success
    1 - Operation failed
    2 - Database or configuration error
    3 - Invalid input (bad post ID, time format, etc.)

For more information, visit: https://github.com/plurcast/plurcast
")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging to stderr
    #[arg(short, long, global = true)]
    #[arg(help = "Enable verbose logging to stderr (useful for debugging)")]
    verbose: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// List scheduled posts
    List {
        /// Output format: text or json
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Filter by platform
        #[arg(short, long)]
        platform: Option<String>,
    },

    /// Cancel a scheduled post
    Cancel {
        /// Post ID to cancel
        post_id: Option<String>,

        /// Cancel all scheduled posts
        #[arg(long)]
        all: bool,

        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },

    /// Reschedule a post
    Reschedule {
        /// Post ID to reschedule
        post_id: String,

        /// New schedule time (e.g., "tomorrow 3pm", "+2h")
        time: String,
    },

    /// Post immediately
    Now {
        /// Post ID to post now
        post_id: String,
    },

    /// Show queue statistics
    Stats {
        /// Output format: text or json
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Manage failed posts
    Failed {
        #[command(subcommand)]
        action: FailedAction,
    },
}

#[derive(Subcommand, Debug)]
enum FailedAction {
    /// List failed posts
    List {
        /// Output format: text or json
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Delete all failed posts
    Clear {
        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },

    /// Delete a specific failed post
    Delete {
        /// Post ID to delete
        post_id: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Initialize logging
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_env_filter("debug")
            .with_writer(std::io::stderr)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter("error")
            .with_writer(std::io::stderr)
            .init();
    }

    // Run the main logic and handle errors
    if let Err(e) = run(cli).await {
        eprintln!("Error: {}", e);
        std::process::exit(e.exit_code());
    }
}

async fn run(cli: Cli) -> Result<()> {
    // Load configuration
    let config = Config::load()?;

    // Initialize database
    let db = Database::new(&config.database.path).await?;

    // Execute command
    match cli.command {
        Commands::List { format, platform } => {
            cmd_list(&db, &format, platform.as_deref()).await?;
        }
        Commands::Cancel { post_id, all, force } => {
            cmd_cancel(&db, post_id.as_deref(), all, force).await?;
        }
        Commands::Reschedule { post_id, time } => {
            cmd_reschedule(&db, &post_id, &time).await?;
        }
        Commands::Now { post_id } => {
            cmd_now(&db, &post_id).await?;
        }
        Commands::Stats { format } => {
            cmd_stats(&db, &format).await?;
        }
        Commands::Failed { action } => {
            match action {
                FailedAction::List { format } => {
                    cmd_failed_list(&db, &format).await?;
                }
                FailedAction::Clear { force } => {
                    cmd_failed_clear(&db, force).await?;
                }
                FailedAction::Delete { post_id, force } => {
                    cmd_failed_delete(&db, &post_id, force).await?;
                }
            }
        }
    }

    Ok(())
}

/// List scheduled posts
async fn cmd_list(db: &Database, format: &str, platform: Option<&str>) -> Result<()> {
    use libplurcast::PlurcastError;

    // Validate format
    if format != "text" && format != "json" {
        return Err(PlurcastError::InvalidInput(format!(
            "Invalid format '{}'. Must be 'text' or 'json'",
            format
        )));
    }

    // Get scheduled posts
    let mut posts = db.get_scheduled_posts().await?;

    // Filter by platform if specified
    if let Some(plat) = platform {
        posts.retain(|p| {
            if let Some(ref metadata) = p.metadata {
                metadata.contains(&format!("\"{}\"", plat))
            } else {
                false
            }
        });
    }

    // Output based on format
    if format == "json" {
        output_list_json(&posts);
    } else {
        output_list_text(&posts);
    }

    Ok(())
}

/// Output posts as JSON
fn output_list_json(posts: &[libplurcast::Post]) {
    let json: Vec<serde_json::Value> = posts
        .iter()
        .map(|p| {
            serde_json::json!({
                "id": p.id,
                "content": p.content,
                "scheduled_at": p.scheduled_at,
                "created_at": p.created_at,
                "status": format!("{:?}", p.status),
            })
        })
        .collect();

    println!("{}", serde_json::to_string_pretty(&json).unwrap());
}

/// Output posts as human-readable text
fn output_list_text(posts: &[libplurcast::Post]) {
    use chrono::{DateTime, Utc};

    if posts.is_empty() {
        return;
    }

    let now = Utc::now().timestamp();

    for post in posts {
        let content_preview = truncate_content(&post.content, 50);
        let time_until = post
            .scheduled_at
            .map(|ts| format_time_until(now, ts))
            .unwrap_or_else(|| "unknown".to_string());

        println!(
            "{} | {} | {}",
            post.id, content_preview, time_until
        );
    }
}

/// Truncate content to max length with ellipsis
fn truncate_content(content: &str, max_len: usize) -> String {
    if content.len() <= max_len {
        content.to_string()
    } else {
        format!("{}...", &content[..max_len])
    }
}

/// Format time until scheduled time in human-readable format
fn format_time_until(now: i64, scheduled_at: i64) -> String {
    let diff = scheduled_at - now;

    if diff < 0 {
        return "overdue".to_string();
    }

    let minutes = diff / 60;
    let hours = minutes / 60;
    let days = hours / 24;

    if days > 0 {
        format!("in {} day{}", days, if days == 1 { "" } else { "s" })
    } else if hours > 0 {
        format!("in {} hour{}", hours, if hours == 1 { "" } else { "s" })
    } else if minutes > 0 {
        format!("in {} minute{}", minutes, if minutes == 1 { "" } else { "s" })
    } else {
        "in <1 minute".to_string()
    }
}

/// Cancel scheduled post(s)
async fn cmd_cancel(db: &Database, post_id: Option<&str>, all: bool, force: bool) -> Result<()> {
    use libplurcast::PlurcastError;

    // Validate arguments
    validate_cancel_args(post_id, all)?;

    // Confirm if not forced
    if !force && !confirm_cancel(post_id, all)? {
        return Err(PlurcastError::InvalidInput("Cancelled by user".to_string()));
    }

    // Execute cancellation
    if let Some(id) = post_id {
        cancel_single_post(db, id).await?;
    } else {
        cancel_all_posts(db).await?;
    }

    Ok(())
}

/// Validate cancel command arguments
fn validate_cancel_args(post_id: Option<&str>, all: bool) -> Result<()> {
    use libplurcast::PlurcastError;

    if post_id.is_none() && !all {
        return Err(PlurcastError::InvalidInput(
            "Must provide either POST_ID or --all".to_string(),
        ));
    }

    if post_id.is_some() && all {
        return Err(PlurcastError::InvalidInput(
            "Cannot use both POST_ID and --all".to_string(),
        ));
    }

    // Validate UUID format if post_id provided
    if let Some(id) = post_id {
        if uuid::Uuid::parse_str(id).is_err() {
            return Err(PlurcastError::InvalidInput(
                "Invalid post ID format".to_string(),
            ));
        }
    }

    Ok(())
}

/// Prompt user for confirmation
fn confirm_cancel(post_id: Option<&str>, all: bool) -> Result<bool> {
    use libplurcast::PlurcastError;
    use std::io::{self, Write};

    let message = if all {
        "Cancel all scheduled posts? (y/N): "
    } else {
        "Cancel this post? (y/N): "
    };

    eprint!("{}", message);
    io::stderr().flush().map_err(|e| {
        PlurcastError::InvalidInput(format!("Failed to write confirmation prompt: {}", e))
    })?;

    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|e| {
        PlurcastError::InvalidInput(format!("Failed to read confirmation: {}", e))
    })?;

    Ok(input.trim().eq_ignore_ascii_case("y"))
}

/// Cancel a single post by ID
async fn cancel_single_post(db: &Database, post_id: &str) -> Result<()> {
    use libplurcast::PlurcastError;

    // Check if post exists
    let post = db.get_post(post_id).await?;
    if post.is_none() {
        return Err(PlurcastError::InvalidInput("Post not found".to_string()));
    }

    // Delete the post
    db.delete_post(post_id).await?;

    println!("Cancelled post {}", post_id);
    Ok(())
}

/// Cancel all scheduled posts
async fn cancel_all_posts(db: &Database) -> Result<()> {
    let posts = db.get_scheduled_posts().await?;

    if posts.is_empty() {
        println!("No scheduled posts to cancel");
        return Ok(());
    }

    let count = posts.len();
    for post in posts {
        db.delete_post(&post.id).await?;
    }

    println!(
        "Cancelled {} post{}",
        count,
        if count == 1 { "" } else { "s" }
    );
    Ok(())
}

/// Reschedule a post
async fn cmd_reschedule(db: &Database, post_id: &str, time: &str) -> Result<()> {
    use libplurcast::PlurcastError;

    // Validate post_id format
    validate_post_id(post_id)?;

    // Check if post exists and get current time
    let post = db.get_post(post_id).await?;
    let post = post.ok_or_else(|| PlurcastError::InvalidInput("Post not found".to_string()))?;

    // Parse new schedule time
    let new_time = parse_reschedule_time(time, post.scheduled_at)?;

    // Validate not in past
    let now = chrono::Utc::now().timestamp();
    if new_time <= now {
        return Err(PlurcastError::InvalidInput(
            "Cannot schedule in the past".to_string(),
        ));
    }

    // Update scheduled_at in database
    db.update_post_schedule(post_id, Some(new_time)).await?;

    println!("Rescheduled post {} for {}", post_id, new_time);
    Ok(())
}

/// Validate post ID format
fn validate_post_id(post_id: &str) -> Result<()> {
    use libplurcast::PlurcastError;

    if uuid::Uuid::parse_str(post_id).is_err() {
        return Err(PlurcastError::InvalidInput(
            "Invalid post ID format".to_string(),
        ));
    }
    Ok(())
}

/// Parse reschedule time, supporting absolute and relative formats
fn parse_reschedule_time(time: &str, current_scheduled: Option<i64>) -> Result<i64> {
    // Check for relative adjustment (+1h, -30m)
    if time.starts_with('+') || time.starts_with('-') {
        return parse_relative_adjustment(time, current_scheduled);
    }

    // Parse as absolute time (duration or natural language)
    let dt = libplurcast::scheduling::parse_schedule(time, None)?;
    Ok(dt.timestamp())
}

/// Parse relative time adjustment (+1h, -30m)
fn parse_relative_adjustment(time: &str, current_scheduled: Option<i64>) -> Result<i64> {
    use libplurcast::PlurcastError;

    let current = current_scheduled.ok_or_else(|| {
        PlurcastError::InvalidInput("Post has no scheduled time to adjust".to_string())
    })?;

    let is_addition = time.starts_with('+');
    let duration_str = &time[1..]; // Remove +/- prefix

    let duration = humantime::parse_duration(duration_str)
        .map_err(|e| PlurcastError::InvalidInput(format!("Invalid duration: {}", e)))?;

    let seconds = duration.as_secs() as i64;

    let new_time = if is_addition {
        current + seconds
    } else {
        current - seconds
    };

    Ok(new_time)
}

/// Post immediately
async fn cmd_now(db: &Database, post_id: &str) -> Result<()> {
    use libplurcast::PlurcastError;

    // Validate post_id format
    validate_post_id(post_id)?;

    // Check if post exists
    let post = db.get_post(post_id).await?;
    let post = post.ok_or_else(|| PlurcastError::InvalidInput("Post not found".to_string()))?;

    // Check if post is scheduled
    if post.status != libplurcast::PostStatus::Scheduled {
        return Err(PlurcastError::InvalidInput(
            "Post is not scheduled".to_string(),
        ));
    }

    // Clear scheduled_at and set status to pending
    db.update_post_schedule(post_id, None).await?;
    db.update_post_status(post_id, libplurcast::PostStatus::Pending)
        .await?;

    println!("Posting {} immediately", post_id);

    // TODO: In future, this should trigger posting via PostingService
    // For now, we just change status to pending, and plur-send daemon will pick it up

    Ok(())
}

/// Show queue statistics
async fn cmd_stats(db: &Database, format: &str) -> Result<()> {
    use libplurcast::PlurcastError;

    // Validate format
    if format != "text" && format != "json" {
        return Err(PlurcastError::InvalidInput(format!(
            "Invalid format '{}'. Must be 'text' or 'json'",
            format
        )));
    }

    // Get all scheduled posts
    let posts = db.get_scheduled_posts().await?;

    // Calculate stats
    let stats = calculate_stats(&posts);

    // Output based on format
    if format == "json" {
        output_stats_json(&stats);
    } else {
        output_stats_text(&stats);
    }

    Ok(())
}

/// Stats structure
struct QueueStats {
    total: usize,
    by_platform: std::collections::HashMap<String, usize>,
    by_time_bucket: TimeBuckets,
    upcoming: Vec<libplurcast::Post>,
}

/// Time bucket counts
struct TimeBuckets {
    next_hour: usize,
    today: usize,
    this_week: usize,
    later: usize,
}

/// Calculate queue statistics
fn calculate_stats(posts: &[libplurcast::Post]) -> QueueStats {
    use std::collections::HashMap;

    let total = posts.len();
    let mut by_platform: HashMap<String, usize> = HashMap::new();
    let now = chrono::Utc::now().timestamp();

    // Count by platform and time bucket
    let mut next_hour = 0;
    let mut today = 0;
    let mut this_week = 0;
    let mut later = 0;

    for post in posts {
        // Count platforms
        if let Some(ref metadata) = post.metadata {
            extract_platforms(metadata).iter().for_each(|p| {
                *by_platform.entry(p.clone()).or_insert(0) += 1;
            });
        }

        // Count time buckets
        if let Some(scheduled_at) = post.scheduled_at {
            let diff = scheduled_at - now;
            if diff < 3600 {
                // < 1 hour
                next_hour += 1;
            } else if diff < 86400 {
                // < 24 hours
                today += 1;
            } else if diff < 604800 {
                // < 7 days
                this_week += 1;
            } else {
                later += 1;
            }
        }
    }

    // Get next 5 upcoming posts
    let mut upcoming: Vec<_> = posts.to_vec();
    upcoming.sort_by_key(|p| p.scheduled_at);
    upcoming.truncate(5);

    QueueStats {
        total,
        by_platform,
        by_time_bucket: TimeBuckets {
            next_hour,
            today,
            this_week,
            later,
        },
        upcoming,
    }
}

/// Extract platform names from metadata JSON
fn extract_platforms(metadata: &str) -> Vec<String> {
    serde_json::from_str::<serde_json::Value>(metadata)
        .ok()
        .and_then(|v| v.get("platforms").cloned())
        .and_then(|v| serde_json::from_value::<Vec<String>>(v).ok())
        .unwrap_or_default()
}

/// Output stats as text
fn output_stats_text(stats: &QueueStats) {
    println!("Queue Statistics");
    println!("================");
    println!();
    println!("Total: {}", stats.total);
    println!();

    if !stats.by_platform.is_empty() {
        println!("By Platform:");
        for (platform, count) in &stats.by_platform {
            println!("  {}: {}", platform, count);
        }
        println!();
    }

    println!("By Time:");
    println!("  Next hour: {}", stats.by_time_bucket.next_hour);
    println!("  Today: {}", stats.by_time_bucket.today);
    println!("  This week: {}", stats.by_time_bucket.this_week);
    println!("  Later: {}", stats.by_time_bucket.later);
    println!();

    if !stats.upcoming.is_empty() {
        println!("Upcoming Posts:");
        for post in &stats.upcoming {
            let preview = truncate_content(&post.content, 40);
            let time_str = post
                .scheduled_at
                .map(|ts| {
                    let now = chrono::Utc::now().timestamp();
                    format_time_until(now, ts)
                })
                .unwrap_or_else(|| "unknown".to_string());
            println!("  {} | {}", time_str, preview);
        }
    }
}

/// Output stats as JSON
fn output_stats_json(stats: &QueueStats) {
    let upcoming: Vec<serde_json::Value> = stats
        .upcoming
        .iter()
        .map(|p| {
            serde_json::json!({
                "id": p.id,
                "content": p.content,
                "scheduled_at": p.scheduled_at,
            })
        })
        .collect();

    let output = serde_json::json!({
        "total": stats.total,
        "by_platform": stats.by_platform,
        "by_time_bucket": {
            "next_hour": stats.by_time_bucket.next_hour,
            "today": stats.by_time_bucket.today,
            "this_week": stats.by_time_bucket.this_week,
            "later": stats.by_time_bucket.later,
        },
        "upcoming": upcoming,
    });

    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

/// List failed posts
async fn cmd_failed_list(db: &Database, format: &str) -> Result<()> {
    use libplurcast::PlurcastError;

    // Validate format
    if format != "text" && format != "json" {
        return Err(PlurcastError::InvalidInput(format!(
            "Invalid format '{}'. Must be 'text' or 'json'",
            format
        )));
    }

    // Get failed posts
    let failed_posts = db.get_failed_posts().await?;

    if failed_posts.is_empty() {
        if format == "json" {
            println!("[]");
        } else {
            println!("No failed posts");
        }
        return Ok(());
    }

    // Output based on format
    if format == "json" {
        output_failed_posts_json(&failed_posts);
    } else {
        output_failed_posts_text(&failed_posts, db).await?;
    }

    Ok(())
}

/// Output failed posts as text
async fn output_failed_posts_text(posts: &[libplurcast::Post], db: &Database) -> Result<()> {
    println!("Failed Posts ({} total):", posts.len());
    println!();

    for post in posts {
        let content_preview = truncate_content(&post.content, 60);
        let created_at = chrono::DateTime::from_timestamp(post.created_at, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "unknown".to_string());

        println!("ID: {}", post.id);
        println!("Content: {}", content_preview);
        println!("Created: {}", created_at);

        // Fetch post records to show error information
        let records = db.get_post_records(&post.id).await?;
        let failed_records: Vec<_> = records.iter().filter(|r| !r.success).collect();

        if !failed_records.is_empty() {
            let errors: Vec<String> = failed_records
                .iter()
                .filter_map(|r| r.error_message.as_ref())
                .map(|e| e.to_string())
                .collect();
            if !errors.is_empty() {
                println!("Errors: {}", errors.join(", "));
            }
        }

        println!();
    }

    Ok(())
}

/// Output failed posts as JSON
fn output_failed_posts_json(posts: &[libplurcast::Post]) {
    let json: Vec<serde_json::Value> = posts
        .iter()
        .map(|p| {
            serde_json::json!({
                "id": p.id,
                "content": p.content,
                "created_at": p.created_at,
                "status": "failed",
            })
        })
        .collect();

    println!("{}", serde_json::to_string_pretty(&json).unwrap());
}

/// Clear all failed posts
async fn cmd_failed_clear(db: &Database, force: bool) -> Result<()> {
    use libplurcast::PlurcastError;

    // Get failed posts
    let failed_posts = db.get_failed_posts().await?;

    if failed_posts.is_empty() {
        println!("No failed posts to clear");
        return Ok(());
    }

    let count = failed_posts.len();

    // Confirm if not forced
    if !force && !confirm_clear_failed(count)? {
        return Err(PlurcastError::InvalidInput("Cancelled by user".to_string()));
    }

    // Delete all failed posts
    for post in &failed_posts {
        db.delete_post(&post.id).await?;
    }

    println!("Cleared {} failed post(s)", count);
    Ok(())
}

/// Prompt user for confirmation to clear failed posts
fn confirm_clear_failed(count: usize) -> Result<bool> {
    use libplurcast::PlurcastError;
    use std::io::{self, Write};

    eprint!("Clear {} failed post(s)? (y/N): ", count);
    io::stderr().flush().map_err(|e| {
        PlurcastError::InvalidInput(format!("Failed to write confirmation prompt: {}", e))
    })?;

    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|e| {
        PlurcastError::InvalidInput(format!("Failed to read confirmation: {}", e))
    })?;

    Ok(input.trim().eq_ignore_ascii_case("y"))
}

/// Delete a specific failed post
async fn cmd_failed_delete(db: &Database, post_id: &str, force: bool) -> Result<()> {
    use libplurcast::PlurcastError;

    // Validate post_id format
    validate_post_id(post_id)?;

    // Check if post exists and is failed
    let post = db.get_post(post_id).await?;
    let post = post.ok_or_else(|| PlurcastError::InvalidInput("Post not found".to_string()))?;

    if post.status != libplurcast::PostStatus::Failed {
        return Err(PlurcastError::InvalidInput(format!(
            "Post is not in failed status (current: {:?})",
            post.status
        )));
    }

    // Confirm if not forced
    if !force && !confirm_delete_failed(post_id)? {
        return Err(PlurcastError::InvalidInput("Cancelled by user".to_string()));
    }

    // Delete the post
    db.delete_post(post_id).await?;

    println!("Deleted failed post: {}", post_id);
    Ok(())
}

/// Prompt user for confirmation to delete a failed post
fn confirm_delete_failed(post_id: &str) -> Result<bool> {
    use libplurcast::PlurcastError;
    use std::io::{self, Write};

    eprint!("Delete failed post {}? (y/N): ", post_id);
    io::stderr().flush().map_err(|e| {
        PlurcastError::InvalidInput(format!("Failed to write confirmation prompt: {}", e))
    })?;

    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|e| {
        PlurcastError::InvalidInput(format!("Failed to read confirmation: {}", e))
    })?;

    Ok(input.trim().eq_ignore_ascii_case("y"))
}
