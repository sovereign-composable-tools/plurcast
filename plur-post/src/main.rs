//! plur-post - Post content to decentralized social platforms

use clap::Parser;
use libplurcast::{
    config::{resolve_db_path, Config},
    db::Database,
    platforms::{nostr::NostrPlatform, Platform},
    types::{Post, PostRecord, PostStatus},
    PlurcastError, Result,
};
use serde_json::json;
use std::io::{self, IsTerminal, Read};

/// Maximum content length in bytes (100KB)
///
/// This limit prevents memory exhaustion and DoS attacks while allowing
/// for long-form posts. Most social platforms have much lower limits:
/// - Nostr: ~32KB practical limit
/// - Mastodon: 500 characters default (configurable)
/// - Bluesky: 300 characters
///
/// 100KB provides headroom for future features while protecting against abuse.
/// This addresses security issue H2: Missing Input Validation on Content Length.
///
/// Rationale for 100,000 bytes:
/// - Sufficient for very long posts (≈50,000 words)
/// - Well above any platform's actual limits
/// - Small enough to prevent memory exhaustion attacks
/// - Easy to remember and document
/// - Prevents DoS via unbounded input streams (e.g., `cat /dev/zero | plur-post`)
const MAX_CONTENT_LENGTH: usize = 100_000;

#[derive(Parser, Debug)]
#[command(name = "plur-post")]
#[command(version)]
#[command(about = "Post content to decentralized social platforms")]
#[command(long_about = "\
plur-post - Post content to decentralized social platforms

DESCRIPTION:
    plur-post is a Unix-style tool for posting content to decentralized social
    media platforms like Nostr, Mastodon, and Bluesky. It follows Unix philosophy:
    reads from stdin or arguments, outputs to stdout, and uses meaningful exit codes.

USAGE EXAMPLES:
    # Post from command line argument
    plur-post \"Hello decentralized world!\"

    # Post from stdin (pipe)
    echo \"Hello from stdin\" | plur-post

    # Post to specific platform only
    echo \"Nostr-only post\" | plur-post --platform nostr

    # Post to multiple platforms
    plur-post \"Multi-platform post\" --platform nostr,mastodon

    # Save as draft without posting
    echo \"Draft content\" | plur-post --draft

    # Get machine-readable JSON output
    plur-post \"Test post\" --format json

    # Enable verbose logging for debugging
    plur-post \"Debug post\" --verbose

CONFIGURATION:
    Configuration file: ~/.config/plurcast/config.toml
    Database location: ~/.local/share/plurcast/posts.db
    
    Override with environment variables:
        PLURCAST_CONFIG    - Path to config file
        PLURCAST_DB_PATH   - Path to database file

EXIT CODES:
    0 - Success on all platforms
    1 - Posting failed on at least one platform
    2 - Authentication error (missing/invalid credentials)
    3 - Invalid input (empty content, malformed arguments)

OUTPUT FORMAT:
    Text format (default): platform:post_id (one per line)
        Example: nostr:note1abc123...
    
    JSON format (--format json): Machine-readable JSON array
        Example: [{\"platform\":\"nostr\",\"success\":true,\"post_id\":\"note1...\"}]

For more information, visit: https://github.com/plurcast/plurcast
")]
struct Cli {
    /// Content to post (reads from stdin if not provided)
    #[arg(value_name = "CONTENT")]
    content: Option<String>,

    /// Target specific platform(s) (comma-separated: nostr,mastodon,bluesky)
    #[arg(short, long, value_name = "PLATFORMS")]
    #[arg(help = "Target specific platform(s) (comma-separated: nostr,mastodon,bluesky)")]
    platform: Option<String>,

    /// Save as draft without posting
    #[arg(short, long)]
    #[arg(help = "Save as draft without posting to any platform")]
    draft: bool,

    /// Output format: text or json
    #[arg(short, long, default_value = "text", value_name = "FORMAT")]
    #[arg(help = "Output format: 'text' (default) or 'json' for machine-readable output")]
    format: String,

    /// Enable verbose logging to stderr
    #[arg(short, long)]
    #[arg(help = "Enable verbose logging to stderr (useful for debugging)")]
    verbose: bool,
}

#[derive(Debug)]
enum OutputFormat {
    Text,
    Json,
}

impl OutputFormat {
    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "text" => Ok(OutputFormat::Text),
            "json" => Ok(OutputFormat::Json),
            _ => Err(PlurcastError::InvalidInput(format!(
                "Invalid format '{}', must be 'text' or 'json'",
                s
            ))),
        }
    }
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
    // Task 7.2: Get content from args or stdin
    let content = get_content(&cli)?;

    // Task 7.3: Load configuration
    let config = Config::load()?;

    // Task 7.3: Initialize database
    let db_path = resolve_db_path(Some(&config.database.path))?;
    let db = Database::new(&db_path.to_string_lossy()).await?;

    // Task 7.3: Create post record in database
    let post = Post::new(content.clone());
    db.create_post(&post).await?;

    // If draft mode, just save and exit
    if cli.draft {
        let output_format = OutputFormat::from_str(&cli.format)?;
        output_draft_result(&post.id, &output_format);
        return Ok(());
    }

    // Task 7.3: Determine target platforms
    let target_platforms = determine_platforms(&cli, &config)?;

    // Task 8.2: Log which platforms are being targeted
    tracing::info!("Targeting platforms: {}", target_platforms.join(", "));

    // Task 7.4: Post to each platform
    let results = post_to_platforms(&target_platforms, &content, &post.id, &db, &config).await;

    // Update post status based on results
    let all_success = results.iter().all(|r| r.success);
    let any_success = results.iter().any(|r| r.success);

    if all_success {
        db.update_post_status(&post.id, PostStatus::Posted).await?;
    } else if any_success {
        // Partial success - mark as posted
        db.update_post_status(&post.id, PostStatus::Posted).await?;
    } else {
        // All failed
        db.update_post_status(&post.id, PostStatus::Failed).await?;
    }

    // Task 7.5: Output results
    let output_format = OutputFormat::from_str(&cli.format)?;
    output_results(&results, &output_format)?;

    // Task 7.6: Determine exit code
    let exit_code = determine_exit_code(&results);
    if exit_code != 0 {
        std::process::exit(exit_code);
    }

    Ok(())
}

/// Task 7.2: Get content from CLI argument or stdin
fn get_content(cli: &Cli) -> Result<String> {
    if let Some(content) = &cli.content {
        // Content provided as argument
        // Validate length BEFORE any processing (Security: Issue H2)
        if content.len() > MAX_CONTENT_LENGTH {
            return Err(PlurcastError::InvalidInput(format!(
                "Content too large: {} bytes (maximum: {} bytes)",
                content.len(),
                MAX_CONTENT_LENGTH
            )));
        }

        if content.trim().is_empty() {
            return Err(PlurcastError::InvalidInput(
                "Content cannot be empty".to_string(),
            ));
        }
        return Ok(content.clone());
    }

    // Check if stdin is a TTY
    let stdin = io::stdin();
    if stdin.is_terminal() {
        return Err(PlurcastError::InvalidInput(
            "No content provided. Provide content as argument or pipe via stdin".to_string(),
        ));
    }

    // Read from stdin with size limit (Security: Issue H2)
    // Use take() to limit bytes read - prevents reading infinite streams
    // Read MAX_CONTENT_LENGTH + 1 to detect if limit was exceeded
    let mut buffer = String::new();
    stdin
        .lock()
        .take((MAX_CONTENT_LENGTH + 1) as u64)
        .read_to_string(&mut buffer)
        .map_err(|e| PlurcastError::InvalidInput(format!("Failed to read from stdin: {}", e)))?;

    // Check if we hit the limit
    if buffer.len() > MAX_CONTENT_LENGTH {
        return Err(PlurcastError::InvalidInput(format!(
            "Content too large: exceeds {} bytes (maximum: {} bytes)",
            MAX_CONTENT_LENGTH,
            MAX_CONTENT_LENGTH
        )));
    }

    if buffer.trim().is_empty() {
        return Err(PlurcastError::InvalidInput(
            "Content cannot be empty".to_string(),
        ));
    }

    Ok(buffer)
}

/// Task 7.3: Determine which platforms to post to
fn determine_platforms(cli: &Cli, config: &Config) -> Result<Vec<String>> {
    if let Some(platform_str) = &cli.platform {
        // Use platforms from CLI flag
        let platforms: Vec<String> = platform_str
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .collect();

        if platforms.is_empty() {
            return Err(PlurcastError::InvalidInput(
                "No platforms specified".to_string(),
            ));
        }

        Ok(platforms)
    } else {
        // Use default platforms from config
        if config.defaults.platforms.is_empty() {
            return Err(PlurcastError::InvalidInput(
                "No default platforms configured".to_string(),
            ));
        }

        Ok(config.defaults.platforms.clone())
    }
}

/// Task 7.4: Post to all enabled platforms
async fn post_to_platforms(
    platforms: &[String],
    content: &str,
    post_id: &str,
    db: &Database,
    config: &Config,
) -> Vec<PostResult> {
    let mut results = Vec::new();

    for platform_name in platforms {
        let result = post_to_platform(platform_name, content, post_id, db, config).await;
        results.push(result);
    }

    results
}

#[derive(Debug)]
struct PostResult {
    platform: String,
    success: bool,
    post_id: Option<String>,
    error: Option<String>,
}

/// Post to a single platform
async fn post_to_platform(
    platform_name: &str,
    content: &str,
    post_id: &str,
    db: &Database,
    config: &Config,
) -> PostResult {
    // Task 8.2: Log posting attempt
    tracing::info!("Attempting to post to platform: {}", platform_name);

    let result = match platform_name {
        "nostr" => post_to_nostr(content, config).await,
        _ => Err(PlurcastError::InvalidInput(format!(
            "Unsupported platform: {}",
            platform_name
        ))),
    };

    match result {
        Ok(platform_post_id) => {
            // Task 8.2: Log posting success
            tracing::info!("✓ Successfully posted to {}: {}", platform_name, platform_post_id);

            // Create post record
            let record = PostRecord {
                id: None,
                post_id: post_id.to_string(),
                platform: platform_name.to_string(),
                platform_post_id: Some(platform_post_id.clone()),
                posted_at: Some(chrono::Utc::now().timestamp()),
                success: true,
                error_message: None,
            };

            if let Err(e) = db.create_post_record(&record).await {
                tracing::error!("Failed to create post record: {}", e);
            }

            PostResult {
                platform: platform_name.to_string(),
                success: true,
                post_id: Some(platform_post_id),
                error: None,
            }
        }
        Err(e) => {
            // Task 8.2: Log posting failure with clear platform indication
            tracing::error!("✗ Failed to post to platform '{}': {}", platform_name, e);

            // Create post record with error
            let record = PostRecord {
                id: None,
                post_id: post_id.to_string(),
                platform: platform_name.to_string(),
                platform_post_id: None,
                posted_at: None,
                success: false,
                error_message: Some(e.to_string()),
            };

            if let Err(e) = db.create_post_record(&record).await {
                tracing::error!("Failed to create post record: {}", e);
            }

            PostResult {
                platform: platform_name.to_string(),
                success: false,
                post_id: None,
                error: Some(e.to_string()),
            }
        }
    }
}

/// Task 7.4: Post to Nostr platform
async fn post_to_nostr(content: &str, config: &Config) -> Result<String> {
    let nostr_config = config
        .nostr
        .as_ref()
        .ok_or_else(|| PlurcastError::InvalidInput("Nostr not configured".to_string()))?;

    if !nostr_config.enabled {
        return Err(PlurcastError::InvalidInput(
            "Nostr is disabled in configuration".to_string(),
        ));
    }

    // Task 8.2: Log platform initialization
    tracing::debug!("Initializing Nostr platform");

    // Create platform instance
    let mut platform = NostrPlatform::new(nostr_config);

    // Load keys
    tracing::debug!("Loading Nostr keys from: {}", nostr_config.keys_file);
    platform.load_keys(&nostr_config.keys_file)?;

    // Validate content
    tracing::debug!("Validating content for Nostr");
    platform.validate_content(content)?;

    // Task 8.2: Log authentication attempt
    tracing::debug!("Authenticating with Nostr relays: {:?}", nostr_config.relays);
    platform.authenticate().await?;
    tracing::debug!("✓ Successfully authenticated with Nostr");

    // Post
    tracing::debug!("Publishing note to Nostr");
    let post_id = platform.post(content).await?;

    Ok(post_id)
}

/// Task 7.5: Output results in the specified format
fn output_results(results: &[PostResult], format: &OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Text => {
            for result in results {
                if result.success {
                    if let Some(post_id) = &result.post_id {
                        println!("{}:{}", result.platform, post_id);
                    }
                }
            }
        }
        OutputFormat::Json => {
            let json_results: Vec<_> = results
                .iter()
                .map(|r| {
                    json!({
                        "platform": r.platform,
                        "success": r.success,
                        "post_id": r.post_id,
                        "error": r.error,
                    })
                })
                .collect();

            println!("{}", serde_json::to_string_pretty(&json_results).unwrap());
        }
    }

    Ok(())
}

/// Output draft result
fn output_draft_result(post_id: &str, format: &OutputFormat) {
    match format {
        OutputFormat::Text => {
            println!("draft:{}", post_id);
        }
        OutputFormat::Json => {
            let result = json!({
                "status": "draft",
                "post_id": post_id,
            });
            println!("{}", serde_json::to_string_pretty(&result).unwrap());
        }
    }
}

/// Task 7.6: Determine exit code based on results
fn determine_exit_code(results: &[PostResult]) -> i32 {
    let all_success = results.iter().all(|r| r.success);
    let any_success = results.iter().any(|r| r.success);

    if all_success {
        0 // Success on all platforms
    } else if any_success {
        1 // Partial failure
    } else {
        // Check if any errors are authentication errors
        let has_auth_error = results.iter().any(|r| {
            r.error
                .as_ref()
                .map(|e| e.contains("Authentication") || e.contains("authentication"))
                .unwrap_or(false)
        });

        if has_auth_error {
            2 // Authentication error
        } else {
            1 // Posting failure
        }
    }
}
