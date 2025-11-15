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
    todo!("implement cmd_cancel")
}

/// Reschedule a post
async fn cmd_reschedule(db: &Database, post_id: &str, time: &str) -> Result<()> {
    todo!("implement cmd_reschedule")
}

/// Post immediately
async fn cmd_now(db: &Database, post_id: &str) -> Result<()> {
    todo!("implement cmd_now")
}

/// Show queue statistics
async fn cmd_stats(db: &Database, format: &str) -> Result<()> {
    todo!("implement cmd_stats")
}
