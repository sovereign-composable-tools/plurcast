//! plur-post - Post content to decentralized social platforms

use std::collections::HashMap;
use std::io::{self, IsTerminal, Read};

use clap::Parser;
use serde_json::json;

use libplurcast::{
    config::Config,
    db::Database,
    logging::{LogFormat, LoggingConfig},
    platforms::id_detection::detect_platform_from_id,
    service::{
        posting::{PostRequest, PostResponse},
        validation::ValidationRequest,
        PlatformResult, PlurcastService,
    },
    PlurcastError, Result,
};

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

/// Maximum length for a single thread part (in characters)
/// Set to 450 to leave room for potential link shortening and be safely under Mastodon's 500 limit
const MAX_THREAD_PART_LENGTH: usize = 450;

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

    /// Reply to a specific post (creates a thread)
    #[arg(long, value_name = "POST_ID")]
    #[arg(
        help = "Reply to a specific post to create a thread. Accepts either a plurcast UUID (automatically resolves to platform-specific IDs) or a platform-specific ID (note1... for Nostr, status ID for Mastodon)."
    )]
    reply_to: Option<String>,

    /// Automatically split long content into a thread
    #[arg(long)]
    #[arg(
        help = "Automatically split content that exceeds platform limits into a thread. Each part replies to the previous, creating a cohesive thread."
    )]
    auto_thread: bool,

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
            auto_thread: cli.auto_thread,
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

    // Handle auto-threading: split content if needed
    let thread_parts = if cli.auto_thread {
        let parts = split_into_thread_parts(&content, MAX_THREAD_PART_LENGTH);
        if parts.len() > 1 {
            tracing::info!(
                "Auto-thread: splitting content into {} parts",
                parts.len()
            );
            if cli.verbose {
                eprintln!(
                    "Auto-thread: splitting into {} parts ({} chars each max)",
                    parts.len(),
                    MAX_THREAD_PART_LENGTH
                );
            }
        }
        parts
    } else {
        vec![content]
    };

    // Track all responses for final output
    let mut all_responses: Vec<PostResponse> = Vec::new();

    // Resolve reply_to: if it's a UUID, look up platform-specific IDs from database
    // If it's a platform-specific ID, detect platform and try cross-platform lookup
    let (mut current_reply_to, target_platforms) = if let Some(ref id) = cli.reply_to {
        if is_uuid(id) {
            // Look up platform-specific IDs from database
            let platform_ids = service.database().get_platform_post_ids(id).await?;
            if platform_ids.is_empty() {
                return Err(PlurcastError::InvalidInput(format!(
                    "Post ID not found in database: {}",
                    id
                )));
            }
            tracing::debug!(
                "Resolved UUID {} to platform IDs: {:?}",
                id,
                platform_ids
            );
            (platform_ids, target_platforms)
        } else {
            // Platform-specific ID provided - detect platform and resolve
            resolve_platform_specific_reply_to(
                id,
                &target_platforms,
                service.database(),
                cli.verbose,
            )
            .await?
        }
    } else {
        (HashMap::new(), target_platforms)
    };

    // Time gap between scheduled thread parts (60 seconds)
    const THREAD_SCHEDULE_GAP_SECS: i64 = 60;

    // For scheduled threads: track the previous post's UUID so plur-send can resolve threading
    // For immediate threads: track platform IDs directly for reply_to
    let mut previous_post_uuid: Option<String> = None;

    for (part_index, part_content) in thread_parts.iter().enumerate() {
        // Calculate scheduled time for this part (stagger by 60s for threads)
        let part_scheduled_at = scheduled_at.map(|base_time| {
            base_time + (part_index as i64 * THREAD_SCHEDULE_GAP_SECS)
        });

        // For scheduled threads, we use UUID chain instead of platform IDs.
        // The thread_parent_uuid will be resolved to platform IDs at send time by plur-send.
        // For immediate threads, we use platform IDs directly in reply_to.
        let is_scheduled = scheduled_at.is_some();

        // Create post request for this part
        let request = PostRequest {
            content: part_content.clone(),
            platforms: target_platforms.clone(),
            draft: cli.draft,
            account: cli.account.clone(),
            scheduled_at: part_scheduled_at,
            nostr_pow: cli.nostr_pow,
            nostr_21e8: cli.nostr_21e8,
            // For immediate posts: use platform IDs. For scheduled: empty (resolved at send time)
            reply_to: if is_scheduled {
                HashMap::new()
            } else {
                current_reply_to.clone()
            },
            // For scheduled threads: store parent's UUID for later resolution
            thread_parent_uuid: if is_scheduled {
                previous_post_uuid.clone()
            } else {
                None
            },
            // Track position in thread for scheduled threads
            thread_sequence: if is_scheduled && thread_parts.len() > 1 {
                Some(part_index as u32)
            } else {
                None
            },
        };

        // Post using PostingService
        let response = if cli.verbose {
            if thread_parts.len() > 1 {
                eprintln!(
                    "\n--- Thread part {}/{} ---",
                    part_index + 1,
                    thread_parts.len()
                );
            }
            post_with_progress(&service, request).await?
        } else {
            service.posting().post(request).await?
        };

        // For threading: track info for next part
        if part_index < thread_parts.len() - 1 {
            if is_scheduled {
                // For scheduled threads: track this post's UUID for next part's parent reference
                // At send time, plur-send will resolve this UUID to platform-specific IDs
                previous_post_uuid = Some(response.post_id.clone());
            } else {
                // For immediate threads: collect platform IDs for reply_to in next part
                let mut next_reply_to: HashMap<String, String> = HashMap::new();
                for result in &response.results {
                    if result.success {
                        if let Some(ref post_id) = result.post_id {
                            next_reply_to.insert(result.platform.clone(), post_id.clone());
                        }
                    }
                }
                current_reply_to = next_reply_to;
            }
        }

        all_responses.push(response);
    }

    // If draft mode, output draft results and exit
    if cli.draft {
        for (i, response) in all_responses.iter().enumerate() {
            if thread_parts.len() > 1 {
                output_draft_result_with_part(&response.post_id, &output_format, i + 1);
            } else {
                output_draft_result(&response.post_id, &output_format);
            }
        }
        return Ok(());
    }

    // If scheduled, output schedule results and exit
    if scheduled_at.is_some() {
        for (i, response) in all_responses.iter().enumerate() {
            let part_time = scheduled_at.unwrap() + (i as i64 * THREAD_SCHEDULE_GAP_SECS);
            if thread_parts.len() > 1 {
                output_schedule_result_with_part(&response.post_id, part_time, &output_format, i + 1);
            } else {
                output_schedule_result(&response.post_id, part_time, &output_format);
            }
        }
        return Ok(());
    }

    // Output results for all parts
    for (i, response) in all_responses.iter().enumerate() {
        if thread_parts.len() > 1 && !matches!(output_format, OutputFormat::Json) {
            println!("--- Thread part {}/{} ---", i + 1, thread_parts.len());
        }
        output_results(&response.results, &output_format, cli.verbose)?;
    }

    // Determine exit code (fail if any part failed)
    let exit_code = all_responses
        .iter()
        .map(|r| determine_exit_code(&r.results))
        .find(|&code| code != 0)
        .unwrap_or(0);

    if exit_code != 0 {
        std::process::exit(exit_code);
    }

    Ok(())
}

/// Output draft result with thread part number
fn output_draft_result_with_part(post_id: &str, format: &OutputFormat, part_num: usize) {
    match format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "draft": true,
                "post_id": post_id,
                "thread_part": part_num
            });
            println!("{}", output);
        }
        OutputFormat::Text => {
            println!("draft[{}]:{}", part_num, post_id);
        }
    }
}

/// Output schedule result with thread part number
fn output_schedule_result_with_part(post_id: &str, scheduled_at: i64, format: &OutputFormat, part_num: usize) {
    let scheduled_time = chrono::DateTime::from_timestamp(scheduled_at, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| scheduled_at.to_string());

    match format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "scheduled": true,
                "post_id": post_id,
                "scheduled_at": scheduled_at,
                "scheduled_time": scheduled_time,
                "thread_part": part_num
            });
            println!("{}", output);
        }
        OutputFormat::Text => {
            println!("scheduled[{}]:{} ({})", part_num, post_id, scheduled_time);
        }
    }
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

/// Check if a string is a valid UUID (plurcast internal post ID)
///
/// Returns true for UUIDs like "550e8400-e29b-41d4-a716-446655440000"
/// Returns false for platform-specific IDs like "note1abc..." or "12345678"
fn is_uuid(s: &str) -> bool {
    uuid::Uuid::parse_str(s).is_ok()
}

/// Resolve a platform-specific reply-to ID to a HashMap for all target platforms
///
/// This function implements intelligent cross-platform reply-to resolution:
///
/// 1. Detect which platform the ID belongs to based on format
/// 2. Try database lookup to find cross-platform IDs if the post was made via plurcast
/// 3. If found in DB, return all platform IDs for that post (enables cross-platform replies)
/// 4. If not found, only apply reply-to to the matching platform (skip others)
///
/// # Arguments
///
/// * `id` - The platform-specific post ID (e.g., "note1abc...", "12345678")
/// * `target_platforms` - The platforms the user wants to post to
/// * `db` - Database for cross-platform lookup
/// * `verbose` - Whether to print verbose output
///
/// # Returns
///
/// A tuple of (reply_to HashMap, filtered target_platforms)
/// The target_platforms may be filtered if cross-platform mapping wasn't found
async fn resolve_platform_specific_reply_to(
    id: &str,
    target_platforms: &[String],
    db: &Database,
    verbose: bool,
) -> Result<(HashMap<String, String>, Vec<String>)> {
    let detected = detect_platform_from_id(id);

    // Get the platform name from the detected platform
    let detected_platform = match detected.as_platform_name() {
        Some(name) => name,
        None => {
            // Unknown format - we can't determine the platform
            tracing::warn!(
                "Could not detect platform for ID '{}'. Unknown format.",
                id
            );
            return Err(PlurcastError::InvalidInput(format!(
                "Could not detect platform for reply-to ID '{}'. \
                 Expected formats: note1... (Nostr), numeric ID (Mastodon), \
                 %...=.sha256 (SSB), or a plurcast UUID.",
                id
            )));
        }
    };

    tracing::debug!("Detected {} ID: {}", detected_platform, id);

    // Try to find cross-platform IDs from database
    if let Some(post_id) = db
        .get_post_id_by_platform_post_id(detected_platform, id)
        .await?
    {
        // Found in database - get all platform IDs
        let platform_ids = db.get_platform_post_ids(&post_id).await?;
        if !platform_ids.is_empty() {
            tracing::info!(
                "Found cross-platform IDs for {} {}: {:?}",
                detected_platform,
                id,
                platform_ids.keys().collect::<Vec<_>>()
            );
            if verbose {
                eprintln!(
                    "Found cross-platform reply-to mapping: {:?}",
                    platform_ids.keys().collect::<Vec<_>>()
                );
            }
            // Filter to only target platforms and return
            let filtered_ids: HashMap<String, String> = platform_ids
                .into_iter()
                .filter(|(p, _)| target_platforms.contains(p))
                .collect();
            return Ok((filtered_ids, target_platforms.to_vec()));
        }
    }

    // Not in database - only apply to the matching platform, skip others
    let mut reply_to = HashMap::new();
    let mut filtered_platforms = Vec::new();

    if target_platforms.contains(&detected_platform.to_string()) {
        reply_to.insert(detected_platform.to_string(), id.to_string());
        filtered_platforms.push(detected_platform.to_string());

        // Find platforms that will be skipped
        let skipped: Vec<_> = target_platforms
            .iter()
            .filter(|p| p.as_str() != detected_platform)
            .collect();

        if !skipped.is_empty() {
            tracing::warn!(
                "Reply-to ID '{}' is a {} ID. No cross-platform mapping found in database. \
                 Only posting to {} as a reply. Skipping: {:?}",
                id,
                detected_platform,
                detected_platform,
                skipped
            );
            if verbose {
                eprintln!(
                    "Warning: Reply-to ID '{}' is a {} ID.",
                    id, detected_platform
                );
                eprintln!("         No cross-platform mapping found in database.");
                eprintln!(
                    "         Only posting to {} as a reply. Skipping: {:?}",
                    detected_platform, skipped
                );
            }
        }
    } else {
        // The detected platform is not in the target list
        tracing::warn!(
            "Reply-to ID '{}' is for {} but that platform is not in target list {:?}. \
             Cannot post a reply without the target platform.",
            id,
            detected_platform,
            target_platforms
        );
        return Err(PlurcastError::InvalidInput(format!(
            "Reply-to ID '{}' is a {} ID, but {} is not in the target platforms ({:?}). \
             Cannot post a reply to a platform you're not posting to.",
            id,
            detected_platform,
            detected_platform,
            target_platforms
        )));
    }

    Ok((reply_to, filtered_platforms))
}

/// Split content into thread parts at word boundaries
///
/// Each part will be at most `max_len` characters, splitting at the last space
/// before the limit to avoid breaking words.
fn split_into_thread_parts(content: &str, max_len: usize) -> Vec<String> {
    let content = content.trim();

    // Handle empty content
    if content.is_empty() {
        return vec![];
    }

    // If content fits in one part, return as-is
    if content.chars().count() <= max_len {
        return vec![content.to_string()];
    }

    let mut parts = Vec::new();
    let mut remaining = content;

    while !remaining.is_empty() {
        let char_count = remaining.chars().count();

        if char_count <= max_len {
            parts.push(remaining.trim().to_string());
            break;
        }

        // Find the last space before max_len characters
        let chars: Vec<char> = remaining.chars().collect();
        let search_end = max_len.min(chars.len());

        // Look for last space in the allowed range
        let split_at = (0..search_end)
            .rev()
            .find(|&i| chars[i] == ' ')
            .unwrap_or(search_end); // If no space found, split at max_len

        // Extract the part (as string from chars)
        let part: String = chars[..split_at].iter().collect();
        parts.push(part.trim().to_string());

        // Move past the split point (skip the space)
        let skip = if split_at < chars.len() && chars[split_at] == ' ' {
            split_at + 1
        } else {
            split_at
        };
        remaining = &remaining[chars[..skip].iter().collect::<String>().len()..];
        remaining = remaining.trim_start();
    }

    // Filter out any empty parts
    parts.into_iter().filter(|p| !p.is_empty()).collect()
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
                        || e.contains("No Nostr credentials found")
                        || e.contains("No Mastodon credentials found")
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

#[cfg(test)]
mod tests {
    use super::*;

    // Tests for split_into_thread_parts function

    #[test]
    fn test_split_short_content_no_split() {
        let content = "This is a short message.";
        let parts = split_into_thread_parts(content, 450);
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0], "This is a short message.");
    }

    #[test]
    fn test_split_exactly_at_limit() {
        // Create content exactly at the limit
        let content = "a".repeat(450);
        let parts = split_into_thread_parts(&content, 450);
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].chars().count(), 450);
    }

    #[test]
    fn test_split_long_content_at_word_boundary() {
        // Create content that needs to be split
        let content = "word ".repeat(100); // 500 chars
        let parts = split_into_thread_parts(&content, 50);

        // Each part should be under the limit
        for part in &parts {
            assert!(part.chars().count() <= 50);
        }

        // All parts combined should contain all the words
        let rejoined = parts.join(" ");
        let original_words: Vec<&str> = content.split_whitespace().collect();
        let rejoined_words: Vec<&str> = rejoined.split_whitespace().collect();
        assert_eq!(original_words.len(), rejoined_words.len());
    }

    #[test]
    fn test_split_respects_word_boundaries() {
        let content = "Hello world this is a test message";
        let parts = split_into_thread_parts(content, 15);

        // Check that no part ends with a partial word (no cut mid-word)
        for part in &parts {
            assert!(!part.ends_with('-')); // No hyphenation
            assert!(part.chars().count() <= 15);
        }
    }

    #[test]
    fn test_split_empty_content() {
        let parts = split_into_thread_parts("", 450);
        assert!(parts.is_empty());
    }

    #[test]
    fn test_split_whitespace_only() {
        let parts = split_into_thread_parts("   ", 450);
        assert!(parts.is_empty());
    }

    #[test]
    fn test_split_unicode_characters() {
        // Unicode chars should be counted as single chars, not bytes
        // "🎉 " = 2 chars per repeat (emoji + space), 250 repeats = 500 chars
        let content = "🎉 ".repeat(250);
        let parts = split_into_thread_parts(&content, 450);

        // Should need at least 2 parts (500 chars > 450 limit)
        assert!(parts.len() >= 2, "Expected >= 2 parts, got {}", parts.len());

        // Each part should be under the char limit
        for part in &parts {
            assert!(part.chars().count() <= 450);
        }
    }

    #[test]
    fn test_split_very_long_word() {
        // A word longer than max_len should be split at max_len
        let content = "a".repeat(100);
        let parts = split_into_thread_parts(&content, 50);

        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].chars().count(), 50);
        assert_eq!(parts[1].chars().count(), 50);
    }

    #[test]
    fn test_split_preserves_content() {
        let content = "This is a test message that should be split into multiple parts. Each part should contain complete words and the content should be fully preserved.";
        let parts = split_into_thread_parts(content, 50);

        // Rejoin and compare (allowing for slight spacing differences)
        let rejoined = parts.join(" ");
        let original_words: Vec<&str> = content.split_whitespace().collect();
        let rejoined_words: Vec<&str> = rejoined.split_whitespace().collect();

        // All original words should be present
        assert_eq!(original_words, rejoined_words);
    }

    #[test]
    fn test_split_with_max_thread_part_length_constant() {
        // Test with the actual constant used in production
        let content = "a ".repeat(300); // 600 chars
        let parts = split_into_thread_parts(&content, MAX_THREAD_PART_LENGTH);

        assert!(parts.len() >= 2);
        for part in &parts {
            assert!(part.chars().count() <= MAX_THREAD_PART_LENGTH);
        }
    }

    #[test]
    fn test_split_trims_parts() {
        let content = "  hello   world  ";
        let parts = split_into_thread_parts(content, 450);

        assert_eq!(parts.len(), 1);
        // Should be trimmed
        assert!(!parts[0].starts_with(' '));
        assert!(!parts[0].ends_with(' '));
    }

    // Tests for is_uuid function (cross-platform reply-to detection)

    #[test]
    fn test_is_uuid_valid() {
        // Standard UUID format
        assert!(is_uuid("550e8400-e29b-41d4-a716-446655440000"));
        // With uppercase
        assert!(is_uuid("550E8400-E29B-41D4-A716-446655440000"));
    }

    #[test]
    fn test_is_uuid_invalid_nostr_id() {
        // Nostr note1... format should not be a UUID
        assert!(!is_uuid("note1abc123def456"));
        // Nostr hex event ID
        assert!(!is_uuid("abc123def456abc123def456abc123def456abc123def456abc123def456abcd"));
    }

    #[test]
    fn test_is_uuid_invalid_mastodon_id() {
        // Mastodon numeric ID
        assert!(!is_uuid("12345678"));
        // Longer numeric ID
        assert!(!is_uuid("109876543210987654"));
    }

    #[test]
    fn test_is_uuid_invalid_empty() {
        assert!(!is_uuid(""));
    }

    #[test]
    fn test_is_uuid_invalid_random_string() {
        assert!(!is_uuid("not-a-uuid"));
        assert!(!is_uuid("hello world"));
    }
}
