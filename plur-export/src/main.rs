//! plur-export - Export posts to various formats
//!
//! This tool exports Plurcast posts to platform-specific formats
//! for backup, migration, or sharing with other tools.

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use libplurcast::config::Config;
use libplurcast::db::Database;
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
}

#[derive(Debug, Clone, ValueEnum)]
enum ExportFormat {
    /// SSB message format (JSON lines)
    Ssb,
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
