//! plur-import - Import posts from platform exports
//!
//! This tool imports existing posts from various platform export formats
//! into the Plurcast database, preserving timestamps and metadata.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use libplurcast::config::Config;
use libplurcast::db::Database;
use tracing::{error, info};

pub mod ssb;

#[derive(Parser)]
#[command(name = "plur-import")]
#[command(about = "Import posts from platform exports", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Import from SSB feed
    Ssb {
        /// Account name to use (default: "default")
        #[arg(long, default_value = "default")]
        account: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(log_level)),
        )
        .init();

    // Load configuration
    let config = Config::load().context("Failed to load configuration")?;

    // Initialize database
    let db = Database::new(&config.database.path)
        .await
        .context("Failed to initialize database")?;

    // Execute command
    let result = match cli.command {
        Commands::Ssb { account } => ssb::import_ssb(&config, &db, &account).await,
    };

    match result {
        Ok(()) => {
            info!("Import completed successfully");
            Ok(())
        }
        Err(e) => {
            error!("Import failed: {}", e);
            std::process::exit(2);
        }
    }
}
