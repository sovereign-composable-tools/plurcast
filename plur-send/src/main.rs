//! plur-send - Background daemon for scheduled posting
//!
//! Monitors the scheduled post queue and automatically posts content
//! at the scheduled time.

use clap::Parser;
use libplurcast::rate_limiter::RateLimiter;
use libplurcast::service::events::EventBus;
use libplurcast::service::posting::{PostRequest, PostingService};
use libplurcast::{Config, Database, Post, Result};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

#[derive(Parser, Debug)]
#[command(name = "plur-send")]
#[command(version)]
#[command(about = "Background daemon for scheduled posting")]
#[command(long_about = "\
plur-send - Background daemon for scheduled posting

DESCRIPTION:
    plur-send is a long-running daemon that monitors the Plurcast queue
    and automatically posts scheduled content at the right time.

    It polls the database at regular intervals, checks for posts that are
    due, handles rate limiting, implements retry logic, and updates post
    status after successful/failed posting.

USAGE:
    # Run in foreground (logs to stderr)
    plur-send

    # Run with custom poll interval
    plur-send --poll-interval 30

    # Enable verbose logging
    plur-send --verbose

SIGNALS:
    SIGTERM, SIGINT - Graceful shutdown (finishes current post)

CONFIGURATION:
    Configuration file: ~/.config/plurcast/config.toml
    Database location: ~/.local/share/plurcast/posts.db

    [scheduling]
    poll_interval = 60  # seconds between polls
    max_retries = 3     # retry failed posts
    retry_delay = 300   # seconds between retries

    [scheduling.rate_limits]
    nostr = { posts_per_hour = 100 }
    mastodon = { posts_per_hour = 300 }

EXIT CODES:
    0 - Clean shutdown
    1 - Runtime error
    2 - Configuration error

For more information, visit: https://github.com/plurcast/plurcast
")]
struct Cli {
    /// Poll interval in seconds (overrides config)
    #[arg(long, value_name = "SECONDS")]
    #[arg(help = "How often to check for scheduled posts (default: 60)")]
    poll_interval: Option<u64>,

    /// Enable verbose logging to stderr
    #[arg(short, long)]
    #[arg(help = "Enable verbose logging (useful for debugging)")]
    verbose: bool,

    /// Run once and exit (for testing)
    #[arg(long, hide = true)]
    #[arg(help = "Process due posts once and exit (for testing)")]
    once: bool,

    /// Startup delay in seconds before processing retries
    #[arg(long, value_name = "SECONDS")]
    #[arg(help = "Delay before processing retries on startup (prevents burst retries)")]
    startup_delay: Option<u64>,

    /// Disable retry processing (only process scheduled posts)
    #[arg(long)]
    #[arg(help = "Disable automatic retry of failed posts")]
    no_retry: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    init_logging(cli.verbose);

    // Load configuration
    let config = Config::load()?;
    let db = Database::new(&config.database.path).await?;

    info!("plur-send daemon starting");

    // Create posting service
    let event_bus = EventBus::new(100);
    let posting = PostingService::new(
        Arc::new(db.clone()),
        Arc::new(config.clone()),
        event_bus,
    );

    // Create rate limiter from config
    let rate_limits = create_rate_limits(&config);
    let rate_limiter = RateLimiter::new(rate_limits);

    // Set up graceful shutdown
    let shutdown = Arc::new(AtomicBool::new(false));
    setup_signal_handlers(shutdown.clone())?;

    // Determine poll interval (CLI overrides config)
    let poll_interval = cli
        .poll_interval
        .or_else(|| config.scheduling.as_ref().map(|s| s.poll_interval))
        .unwrap_or(60);
    info!("Poll interval: {}s", poll_interval);

    // Log scheduling configuration
    if let Some(ref sched_config) = config.scheduling {
        info!(
            "Scheduling config: max_retries={}, retry_delay={}s, rate_limits={} platforms",
            sched_config.max_retries,
            sched_config.retry_delay,
            sched_config.rate_limits.len()
        );
    } else {
        info!("No scheduling configuration found, using defaults");
    }

    // Determine startup delay
    let startup_delay = cli
        .startup_delay
        .or_else(|| config.scheduling.as_ref().and_then(|s| s.startup_delay))
        .unwrap_or(0);

    // Only log startup delay if retry processing is enabled
    if !cli.no_retry && startup_delay > 0 {
        info!("Waiting {}s before processing retries (startup delay)", startup_delay);
    }

    // Main daemon loop
    if cli.once {
        // Run once for testing
        process_due_posts(&db, &posting, &rate_limiter).await?;
        if !cli.no_retry {
            // Apply startup delay before retry processing in --once mode
            if startup_delay > 0 {
                sleep(Duration::from_secs(startup_delay)).await;
            }
            process_retry_posts(&db, &posting, &rate_limiter, &config).await?;
        } else {
            info!("Skipping retry processing (--no-retry flag set)");
        }
        info!("plur-send: processed posts once, exiting");
    } else {
        // Normal daemon mode
        run_daemon_loop(&db, &posting, &rate_limiter, &config, poll_interval, shutdown, startup_delay, cli.no_retry).await?;
    }

    info!("plur-send daemon stopped");
    Ok(())
}

/// Create rate limits map from config
fn create_rate_limits(config: &Config) -> HashMap<String, u32> {
    let mut limits = HashMap::new();

    if let Some(ref sched_config) = config.scheduling {
        for (platform, rate_config) in &sched_config.rate_limits {
            limits.insert(platform.clone(), rate_config.posts_per_hour);
        }
    }

    limits
}

/// Initialize logging based on verbosity level
fn init_logging(verbose: bool) {
    use tracing_subscriber::EnvFilter;

    let filter = if verbose {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug"))
    } else {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .init();
}

/// Set up signal handlers for graceful shutdown (Unix only)
#[cfg(unix)]
fn setup_signal_handlers(shutdown: Arc<AtomicBool>) -> Result<()> {
    use signal_hook::consts::{SIGINT, SIGTERM};
    use signal_hook::iterator::Signals;

    let mut signals = Signals::new([SIGINT, SIGTERM])
        .map_err(|e| libplurcast::PlurcastError::InvalidInput(format!("Signal setup failed: {}", e)))?;

    // Spawn thread to handle signals
    let shutdown_clone = shutdown.clone();
    std::thread::spawn(move || {
        for sig in signals.forever() {
            match sig {
                SIGTERM | SIGINT => {
                    info!("Received shutdown signal, stopping gracefully...");
                    shutdown_clone.store(true, Ordering::Relaxed);
                    break;
                }
                _ => {}
            }
        }
    });

    Ok(())
}

/// No-op signal handler for Windows
#[cfg(not(unix))]
fn setup_signal_handlers(_shutdown: Arc<AtomicBool>) -> Result<()> {
    // Windows doesn't support POSIX signals
    // Use Ctrl+C handler from tokio or ctrlc crate if needed
    Ok(())
}

/// Main daemon loop
async fn run_daemon_loop(
    db: &Database,
    posting: &PostingService,
    rate_limiter: &RateLimiter,
    config: &Config,
    poll_interval: u64,
    shutdown: Arc<AtomicBool>,
    startup_delay: u64,
    no_retry: bool,
) -> Result<()> {
    // Track if this is the first iteration (for startup delay)
    let mut first_iteration = true;

    loop {
        // Check for shutdown signal
        if shutdown.load(Ordering::Relaxed) {
            info!("Shutdown requested, stopping daemon loop");
            break;
        }

        // Always process due scheduled posts
        if let Err(e) = process_due_posts(db, posting, rate_limiter).await {
            error!("Error processing posts: {}", e);
        }

        // Process retry attempts (with startup delay and no_retry flag)
        if !no_retry {
            if first_iteration && startup_delay > 0 {
                // On first iteration, wait before processing retries
                info!("Applying startup delay of {}s before processing retries", startup_delay);
                for _ in 0..startup_delay {
                    if shutdown.load(Ordering::Relaxed) {
                        break;
                    }
                    sleep(Duration::from_secs(1)).await;
                }
                first_iteration = false;
            }

            // Skip retry processing if shutdown was requested during startup delay
            if !shutdown.load(Ordering::Relaxed) {
                if let Err(e) = process_retry_posts(db, posting, rate_limiter, &config).await {
                    error!("Error processing retries: {}", e);
                }
            }
        } else {
            first_iteration = false; // Clear flag even if retries disabled
        }

        // Sleep until next poll (check shutdown every second)
        for _ in 0..poll_interval {
            if shutdown.load(Ordering::Relaxed) {
                break;
            }
            sleep(Duration::from_secs(1)).await;
        }
    }

    Ok(())
}

/// Process all posts that are due for posting
async fn process_due_posts(
    db: &Database,
    posting: &PostingService,
    rate_limiter: &RateLimiter,
) -> Result<()> {
    // Get posts that are due
    let due_posts = db.get_scheduled_posts_due().await?;

    if due_posts.is_empty() {
        return Ok(());
    }

    info!("Found {} post(s) due for posting", due_posts.len());

    for post in due_posts {
        info!("Processing post: {}", post.id);

        // Extract platforms from metadata or use defaults
        let platforms = extract_platforms(&post);

        // Check rate limits for all platforms
        let now = chrono::Utc::now().timestamp();
        let allowed_platforms = check_rate_limits(rate_limiter, db, &platforms, now).await?;

        if allowed_platforms.is_empty() {
            warn!(
                "Post {} blocked by rate limits on all platforms, will retry later",
                post.id
            );
            continue;
        }

        if allowed_platforms.len() < platforms.len() {
            let blocked: Vec<_> = platforms
                .iter()
                .filter(|p| !allowed_platforms.contains(p))
                .collect();
            warn!(
                "Post {} partially blocked by rate limits on: {:?}",
                post.id, blocked
            );
        }

        // Create post request
        // Extract POW difficulty from post metadata if present
        let nostr_pow = post.metadata.as_ref().and_then(|metadata_str| {
            serde_json::from_str::<serde_json::Value>(metadata_str)
                .ok()
                .and_then(|metadata| {
                    metadata
                        .get("nostr")
                        .and_then(|nostr| nostr.get("pow_difficulty"))
                        .and_then(|diff| diff.as_u64())
                        .map(|d| d as u8)
                })
        });

        let request = PostRequest {
            content: post.content.clone(),
            platforms: allowed_platforms.clone(),
            draft: false,
            account: None,
            scheduled_at: None,
            nostr_pow,
        };

        // Post to platforms
        match posting.post(request).await {
            Ok(response) => {
                if response.overall_success {
                    info!(
                        "Successfully posted {} to {} platform(s)",
                        post.id,
                        response.results.iter().filter(|r| r.success).count()
                    );

                    // Record rate limit usage for successful platforms
                    for result in &response.results {
                        if result.success {
                            if let Err(e) = rate_limiter.record(db, &result.platform, now).await {
                                warn!(
                                    "Failed to record rate limit for {}: {}",
                                    result.platform, e
                                );
                            }
                        }
                    }
                } else {
                    warn!("Failed to post {} to all platforms", post.id);
                }
            }
            Err(e) => {
                error!("Error posting {}: {}", post.id, e);
            }
        }
    }

    Ok(())
}

/// Extract platforms from post metadata, or return empty list
fn extract_platforms(post: &Post) -> Vec<String> {
    // For now, return empty list - platforms should be determined by plur-queue
    // when scheduling the post. In the future, we could store platform info in metadata.
    post.metadata
        .as_ref()
        .and_then(|m| serde_json::from_str::<serde_json::Value>(m).ok())
        .and_then(|v| v.get("platforms").cloned())
        .and_then(|p| serde_json::from_value(p).ok())
        .unwrap_or_default()
}

/// Check rate limits for platforms and return allowed platforms
async fn check_rate_limits(
    rate_limiter: &RateLimiter,
    db: &Database,
    platforms: &[String],
    now: i64,
) -> Result<Vec<String>> {
    let mut allowed = Vec::new();

    for platform in platforms {
        match rate_limiter.check(db, platform, now).await {
            Ok(true) => allowed.push(platform.clone()),
            Ok(false) => {
                warn!("Rate limit exceeded for platform: {}", platform);
            }
            Err(e) => {
                warn!("Error checking rate limit for {}: {}", platform, e);
                // On error, allow the post (fail open)
                allowed.push(platform.clone());
            }
        }
    }

    Ok(allowed)
}

/// Process failed posts that are eligible for retry
async fn process_retry_posts(
    db: &Database,
    posting: &PostingService,
    rate_limiter: &RateLimiter,
    config: &Config,
) -> Result<()> {
    // Get retry configuration
    let max_retries = config
        .scheduling
        .as_ref()
        .map(|s| s.max_retries)
        .unwrap_or(3);
    let retry_delay = config
        .scheduling
        .as_ref()
        .map(|s| s.retry_delay)
        .unwrap_or(300);

    // Inter-retry delay (seconds to wait between processing different posts)
    // This prevents burst retries and respects relay rate limits
    let inter_retry_delay = config
        .scheduling
        .as_ref()
        .and_then(|s| s.inter_retry_delay)
        .unwrap_or(5); // Default: 5 seconds between retries

    // Max retries per iteration (prevents processing too many at once)
    let max_retries_per_iteration = config
        .scheduling
        .as_ref()
        .and_then(|s| s.max_retries_per_iteration)
        .unwrap_or(10); // Default: max 10 retries per poll

    // Get failed posts
    let failed_posts = db.get_failed_posts().await?;

    if failed_posts.is_empty() {
        return Ok(());
    }

    info!("Found {} failed post(s) to retry", failed_posts.len());

    let now = chrono::Utc::now().timestamp();
    let mut retries_processed = 0;

    for post in failed_posts {
        // Stop if we've hit the max retries per iteration
        if retries_processed >= max_retries_per_iteration {
            info!(
                "Reached max retries per iteration ({}), will process remaining posts in next poll",
                max_retries_per_iteration
            );
            break;
        }
        // Get post records to check retry attempts
        let records = db.get_post_records(&post.id).await?;

        // Group records by platform and count failures
        let platforms_to_retry = get_retry_platforms(&records, max_retries, retry_delay, now);

        if platforms_to_retry.is_empty() {
            continue;
        }

        info!(
            "Retrying post {} on {} platform(s)",
            post.id,
            platforms_to_retry.len()
        );

        // Check rate limits
        let allowed_platforms =
            check_rate_limits(rate_limiter, db, &platforms_to_retry, now).await?;

        if allowed_platforms.is_empty() {
            warn!("Retry for post {} blocked by rate limits", post.id);
            continue;
        }

        // Retry posting
        match posting
            .retry_post(&post.id, allowed_platforms.clone(), None)
            .await
        {
            Ok(response) => {
                if response.overall_success {
                    info!(
                        "Successfully retried post {} on {} platform(s)",
                        post.id,
                        response.results.iter().filter(|r| r.success).count()
                    );

                    // Record rate limit usage
                    for result in &response.results {
                        if result.success {
                            if let Err(e) = rate_limiter.record(db, &result.platform, now).await {
                                warn!(
                                    "Failed to record rate limit for {}: {}",
                                    result.platform, e
                                );
                            }
                        }
                    }
                } else {
                    warn!("Retry failed for post {}", post.id);
                }

                // Increment retry counter
                retries_processed += 1;

                // Add delay between retries to prevent bursting (skip on last retry)
                if retries_processed < max_retries_per_iteration && inter_retry_delay > 0 {
                    info!("Waiting {}s before next retry (inter-retry delay)", inter_retry_delay);
                    sleep(Duration::from_secs(inter_retry_delay)).await;
                }
            }
            Err(e) => {
                error!("Error retrying post {}: {}", post.id, e);
                retries_processed += 1; // Still count failed attempts

                // Add delay even on error
                if retries_processed < max_retries_per_iteration && inter_retry_delay > 0 {
                    sleep(Duration::from_secs(inter_retry_delay)).await;
                }
            }
        }
    }

    if retries_processed > 0 {
        info!("Processed {} retry attempt(s) this iteration", retries_processed);
    }

    Ok(())
}

/// Get platforms that should be retried based on retry count and delay
fn get_retry_platforms(
    records: &[libplurcast::PostRecord],
    max_retries: u32,
    retry_delay: u64,
    now: i64,
) -> Vec<String> {
    use std::collections::HashMap;

    // Group records by platform
    let mut platform_attempts: HashMap<String, Vec<&libplurcast::PostRecord>> = HashMap::new();

    for record in records {
        platform_attempts
            .entry(record.platform.clone())
            .or_default()
            .push(record);
    }

    let mut retry_platforms = Vec::new();

    for (platform, attempts) in platform_attempts {
        // Count failures
        let failure_count = attempts.iter().filter(|r| !r.success).count() as u32;

        // Check if we've exceeded max retries
        if failure_count >= max_retries {
            continue;
        }

        // Check if enough time has passed since last attempt
        if let Some(last_attempt) = attempts
            .iter()
            .filter_map(|r| r.posted_at)
            .max()
        {
            let time_since_last = now - last_attempt;
            if time_since_last < retry_delay as i64 {
                continue;
            }
        }

        retry_platforms.push(platform);
    }

    retry_platforms
}
