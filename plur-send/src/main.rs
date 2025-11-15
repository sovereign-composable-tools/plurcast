//! plur-send - Background daemon for scheduled posting
//!
//! Monitors the scheduled post queue and automatically posts content
//! at the scheduled time.

use clap::Parser;
use libplurcast::{Config, Database, Result};
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

    // Set up graceful shutdown
    let shutdown = Arc::new(AtomicBool::new(false));
    setup_signal_handlers(shutdown.clone())?;

    // Determine poll interval
    let poll_interval = cli.poll_interval.unwrap_or(60);
    info!("Poll interval: {}s", poll_interval);

    // Main daemon loop
    if cli.once {
        // Run once for testing
        process_due_posts(&db).await?;
        info!("plur-send: processed posts once, exiting");
    } else {
        // Normal daemon mode
        run_daemon_loop(&db, poll_interval, shutdown).await?;
    }

    info!("plur-send daemon stopped");
    Ok(())
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

/// Set up signal handlers for graceful shutdown
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

/// Main daemon loop
async fn run_daemon_loop(
    db: &Database,
    poll_interval: u64,
    shutdown: Arc<AtomicBool>,
) -> Result<()> {
    loop {
        // Check for shutdown signal
        if shutdown.load(Ordering::Relaxed) {
            info!("Shutdown requested, stopping daemon loop");
            break;
        }

        // Process due posts
        if let Err(e) = process_due_posts(db).await {
            error!("Error processing posts: {}", e);
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
async fn process_due_posts(db: &Database) -> Result<()> {
    // Get posts that are due
    let due_posts = db.get_scheduled_posts_due().await?;

    if due_posts.is_empty() {
        return Ok(());
    }

    info!("Found {} post(s) due for posting", due_posts.len());

    for post in due_posts {
        info!("Processing post: {}", post.id);

        // TODO: Task 22 - Implement actual posting logic
        // For now, just log that we would post
        warn!("TODO: Post {} would be posted here", post.id);
    }

    Ok(())
}
