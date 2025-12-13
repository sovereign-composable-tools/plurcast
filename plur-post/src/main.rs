//! plur-post - Post content to decentralized social platforms

use clap::Parser;
use libplurcast::{
    config::Config,
    logging::{LogFormat, LoggingConfig},
    service::{
        posting::{PostRequest, PostResponse},
        validation::ValidationRequest,
        PlatformResult, PlurcastService,
    },
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
/// - SSB: No hard limit (practical limit ~8KB per message)
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
    media platforms like Nostr, Mastodon, and SSB. It follows Unix philosophy:
    reads from stdin or arguments, outputs to stdout, and uses meaningful exit codes.

USAGE EXAMPLES:
    # Post from command line argument
    plur-post \"Hello decentralized world!\"

    # Post from stdin (pipe)
    echo \"Hello from stdin\" | plur-post
    cat message.txt | plur-post

    # Post to all enabled platforms (from config defaults)
    plur-post \"Multi-platform post\"

    # Post to specific platform only
    plur-post \"Nostr-only post\" --platform nostr

    # Post to multiple specific platforms
    plur-post \"Selective post\" --platform nostr --platform mastodon

    # Save as draft without posting
    echo \"Draft content\" | plur-post --draft

    # Get machine-readable JSON output
    plur-post \"Test post\" --format json

    # Enable verbose logging for debugging
    plur-post \"Debug post\" --verbose

    # Unix composability examples
    fortune | plur-post --platform nostr
    echo \"Status: $(date)\" | plur-post
    cat draft.txt | sed 's/foo/bar/g' | plur-post

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

    /// Target specific platform(s) (can be specified multiple times)
    #[arg(short, long, value_name = "PLATFORM")]
    #[arg(
        help = "Target specific platform (nostr, mastodon, or ssb). Can be specified multiple times. If not specified, uses default platforms from config."
    )]
    #[arg(value_parser = ["nostr", "mastodon", "ssb"])]
    platform: Vec<String>,

    /// Account to use for posting (uses active account if not specified)
    #[arg(short, long, value_name = "ACCOUNT")]
    #[arg(
        help = "Account to use for posting. If not specified, uses the active account for each platform."
    )]
    account: Option<String>,

    /// Proof of Work difficulty for Nostr events (NIP-13)
    #[arg(long, value_name = "DIFFICULTY")]
    #[arg(
        help = "Proof of Work difficulty for Nostr events (NIP-13). Higher values require more computation but provide better spam protection. Recommended: 20-25 (takes 1-5 seconds), maximum: 64. Only applies when posting to Nostr platform."
    )]
    nostr_pow: Option<u8>,

    /// Easter egg: require 21e8 pattern in PoW hash (hidden flag)
    #[arg(long = "21e8", hide = true)]
    nostr_21e8: bool,

    /// Save as draft without posting
    #[arg(short, long)]
    #[arg(help = "Save as draft without posting to any platform")]
    draft: bool,

    /// Schedule post for later (e.g., "30m", "2h", "tomorrow", "random:10m-20m")
    #[arg(short, long, value_name = "TIME")]
    #[arg(
        help = "Schedule post for later. Supports duration (\"30m\", \"2h\", \"1d\"), natural language (\"tomorrow\"), or random (\"random:10m-20m\")"
    )]
    schedule: Option<String>,

    /// Output format: text or json
    #[arg(short = 'f', long, default_value = "text", value_name = "FORMAT")]
    #[arg(
        help = "Output format: 'text' (default, one line per platform) or 'json' (machine-readable array)"
    )]
    format: String,

    /// Enable verbose logging to stderr
    #[arg(short, long)]
    #[arg(help = "Enable verbose logging to stderr (useful for debugging)")]
    verbose: bool,

    /// Log format (text, json, pretty)
    #[arg(
        long,
        default_value = "text",
        value_name = "FORMAT",
        env = "PLURCAST_LOG_FORMAT"
    )]
    #[arg(
        help = "Log output format: 'text' (default), 'json' (machine-parseable), or 'pretty' (colored for development)"
    )]
    log_format: String,

    /// Log level (error, warn, info, debug, trace)
    #[arg(
        long,
        default_value = "info",
        value_name = "LEVEL",
        env = "PLURCAST_LOG_LEVEL"
    )]
    #[arg(help = "Minimum log level to display (error, warn, info, debug, trace)")]
    log_level: String,
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

    // Initialize logging with centralized configuration
    let log_format = cli.log_format.parse::<LogFormat>().unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(3); // Exit code 3 for invalid input
    });

    let log_level = if cli.verbose {
        "debug".to_string()
    } else {
        cli.log_level.clone()
    };

    let logging_config = LoggingConfig::new(log_format, log_level, cli.verbose);
    logging_config.init();

    // Run the main logic and handle errors
    if let Err(e) = run(cli).await {
        eprintln!("Error: {}", e);
        std::process::exit(e.exit_code());
    }
}

async fn run(cli: Cli) -> Result<()> {
    // Validate format parameter first (fail fast on invalid input)
    let output_format = OutputFormat::from_str(&cli.format)?;

    // Validate --schedule and --draft cannot be used together
    if cli.schedule.is_some() && cli.draft {
        return Err(PlurcastError::InvalidInput(
            "cannot use --schedule with --draft".to_string(),
        ));
    }

    // Validate --21e8 requires --nostr-pow
    if cli.nostr_21e8 && cli.nostr_pow.is_none() {
        return Err(PlurcastError::InvalidInput(
            "--21e8 requires --nostr-pow to be specified".to_string(),
        ));
    }

    // Get content from args or stdin (fail fast on invalid input)
    let content = get_content(&cli)?;

    // Parse schedule time if provided
    let scheduled_at = if let Some(schedule_str) = &cli.schedule {
        // Query last scheduled timestamp for random scheduling
        let config = Config::load()?;
        let db = libplurcast::Database::new(&config.database.path).await?;
        let last_scheduled = db.get_last_scheduled_timestamp().await?;

        let scheduled_time = libplurcast::scheduling::parse_schedule(schedule_str, last_scheduled)?;
        Some(scheduled_time.timestamp())
    } else {
        None
    };

    // Load configuration (only after input is validated)
    let config = Config::load()?;

    // Initialize service layer
    let service = PlurcastService::from_config(config.clone()).await?;

    // Determine target platforms
    let target_platforms = determine_platforms(&cli, &config)?;
    tracing::info!("Targeting platforms: {}", target_platforms.join(", "));

    // Validate content using ValidationService (skip for draft mode)
    if !cli.draft {
        let validation_request = ValidationRequest {
            content: content.clone(),
            platforms: target_platforms.clone(),
        };
        let validation_response = service.validation().validate(validation_request);

        if !validation_response.valid {
            let errors: Vec<String> = validation_response
                .results
                .iter()
                .flat_map(|r| r.errors.iter().cloned())
                .collect();
            return Err(PlurcastError::InvalidInput(format!(
                "Content validation failed:\n{}",
                errors.join("\n")
            )));
        }
    }

    // Create post request
    let request = PostRequest {
        content,
        platforms: target_platforms,
        draft: cli.draft,
        account: cli.account.clone(),
        scheduled_at,
        nostr_pow: cli.nostr_pow,
        nostr_21e8: cli.nostr_21e8,
    };

    // Post using PostingService
    let response = if cli.verbose {
        post_with_progress(&service, request).await?
    } else {
        service.posting().post(request).await?
    };

    // If draft mode, output draft result and exit
    if cli.draft {
        output_draft_result(&response.post_id, &output_format);
        return Ok(());
    }

    // If scheduled, output schedule result and exit
    if scheduled_at.is_some() {
        output_schedule_result(&response.post_id, scheduled_at.unwrap(), &output_format);
        return Ok(());
    }

    // Output results
    output_results(&response.results, &output_format, cli.verbose)?;

    // Determine exit code
    let exit_code = determine_exit_code(&response.results);
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
            MAX_CONTENT_LENGTH, MAX_CONTENT_LENGTH
        )));
    }

    if buffer.trim().is_empty() {
        return Err(PlurcastError::InvalidInput(
            "Content cannot be empty".to_string(),
        ));
    }

    Ok(buffer)
}

/// Task 7.1: Determine which platforms to post to
fn determine_platforms(cli: &Cli, config: &Config) -> Result<Vec<String>> {
    if !cli.platform.is_empty() {
        // Use platforms from CLI flags
        let platforms: Vec<String> = cli.platform.iter().map(|s| s.to_lowercase()).collect();

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

/// Post with progress output for verbose mode
async fn post_with_progress(
    service: &PlurcastService,
    request: PostRequest,
) -> Result<PostResponse> {
    // Display which account is being used
    if let Some(ref account) = request.account {
        eprintln!("Using account: {}", account);
    } else {
        eprintln!("Using active account for each platform");
    }

    eprintln!("Posting to {} platform(s)...", request.platforms.len());

    let response = service.posting().post(request).await?;

    for result in &response.results {
        if result.success {
            eprintln!(
                "✓ {}: {}",
                result.platform,
                result.post_id.as_ref().unwrap()
            );
        } else {
            eprintln!("✗ {}: {}", result.platform, result.error.as_ref().unwrap());
        }
    }

    Ok(response)
}

/// Task 7.2: Output results in the specified format
/// Successful posts go to stdout, errors go to stderr
fn output_results(results: &[PlatformResult], format: &OutputFormat, verbose: bool) -> Result<()> {
    match format {
        OutputFormat::Text => {
            // Output successful posts to stdout
            for result in results {
                if result.success {
                    if let Some(post_id) = &result.post_id {
                        println!("{}:{}", result.platform, post_id);
                    }
                }
            }

            // Output errors to stderr (unless already shown in verbose mode)
            if !verbose {
                for result in results {
                    if !result.success {
                        if let Some(error) = &result.error {
                            eprintln!("Error [{}]: {}", result.platform, error);
                        }
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

/// Output scheduled post result
fn output_schedule_result(post_id: &str, scheduled_at: i64, format: &OutputFormat) {
    match format {
        OutputFormat::Text => {
            println!("scheduled:{}:for:{}", post_id, scheduled_at);
        }
        OutputFormat::Json => {
            let result = json!({
                "scheduled": true,
                "post_id": post_id,
                "scheduled_at": scheduled_at,
            });
            println!("{}", serde_json::to_string_pretty(&result).unwrap());
        }
    }
}

/// Task 7.3: Determine exit code based on results
/// Exit 0 if all platforms succeed
/// Exit 1 if at least one platform fails (non-auth)
/// Exit 2 if any platform has authentication error
/// Exit 3 for invalid input (handled elsewhere)
fn determine_exit_code(results: &[PlatformResult]) -> i32 {
    let all_success = results.iter().all(|r| r.success);

    if all_success {
        0 // Success on all platforms
    } else {
        // Check if any errors are authentication errors
        let has_auth_error = results.iter().any(|r| {
            r.error
                .as_ref()
                .map(|e| {
                    e.contains("Authentication")
                        || e.contains("authentication")
                        || e.contains("Invalid token")
                        || e.contains("Invalid credentials")
                        || e.contains("keys file not found")
                        || e.contains("token file not found")
                        || e.contains("auth file not found")
                })
                .unwrap_or(false)
        });

        if has_auth_error {
            2 // Authentication error
        } else {
            1 // Posting failure (non-auth)
        }
    }
}
