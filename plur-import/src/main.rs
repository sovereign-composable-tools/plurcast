//! plur-import - Import posts from platform exports
//!
//! This tool imports existing posts from various platform export formats
//! into the Plurcast database, preserving timestamps and metadata.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use libplurcast::config::Config;
use libplurcast::db::Database;
use libplurcast::logging::{LogFormat, LoggingConfig};
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

    /// Log format (text, json, pretty)
    #[arg(
        long,
        default_value = "text",
        value_name = "FORMAT",
        env = "PLURCAST_LOG_FORMAT",
        global = true
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
        env = "PLURCAST_LOG_LEVEL",
        global = true
    )]
    #[arg(help = "Minimum log level to display (error, warn, info, debug, trace)")]
    log_level: String,
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

    // Initialize logging with centralized configuration
    let log_format = cli.log_format.parse::<LogFormat>().unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(3);
    });

    let log_level = if cli.verbose {
        "debug".to_string()
    } else {
        cli.log_level.clone()
    };

    let logging_config = LoggingConfig::new(log_format, log_level, cli.verbose);
    logging_config.init();

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
