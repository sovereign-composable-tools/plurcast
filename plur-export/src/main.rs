//! plur-export - Export posts to various formats
//!
//! This tool exports Plurcast posts to platform-specific formats
//! for backup, migration, or sharing with other tools.

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use libplurcast::config::Config;
use libplurcast::db::Database;
use libplurcast::logging::{LogFormat, LoggingConfig};
use tracing::{error, info};

pub mod ssb;

#[derive(Parser)]
#[command(name = "plur-export")]
#[command(about = "Export posts to various formats", long_about = None)]
struct Cli {
    /// Export format
    #[arg(short, long, value_enum)]
    format: ExportFormat,

    /// Output file (default: stdout)
    #[arg(short, long)]
    output: Option<String>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Log format (text, json, pretty)
    #[arg(long, default_value = "text", value_name = "FORMAT", env = "PLURCAST_LOG_FORMAT")]
    #[arg(help = "Log output format: 'text' (default), 'json' (machine-parseable), or 'pretty' (colored for development)")]
    log_format: String,

    /// Log level (error, warn, info, debug, trace)
    #[arg(long, default_value = "info", value_name = "LEVEL", env = "PLURCAST_LOG_LEVEL")]
    #[arg(help = "Minimum log level to display (error, warn, info, debug, trace)")]
    log_level: String,
}

#[derive(Debug, Clone, ValueEnum)]
enum ExportFormat {
    /// SSB message format (JSON lines)
    Ssb,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging with centralized configuration
    let log_format = cli
        .log_format
        .parse::<LogFormat>()
        .unwrap_or_else(|e| {
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

    // Execute export
    let result = match cli.format {
        ExportFormat::Ssb => ssb::export_ssb(&db, cli.output).await,
    };

    match result {
        Ok(()) => {
            info!("Export completed successfully");
            std::process::exit(0);
        }
        Err(e) => {
            error!("Export failed: {}", e);
            std::process::exit(1);
        }
    }
}
