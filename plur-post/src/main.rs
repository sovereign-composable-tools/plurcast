//! plur-post - Post content to decentralized social platforms

use clap::Parser;
use libplurcast::{PlurcastError, Result};

#[derive(Parser, Debug)]
#[command(name = "plur-post")]
#[command(about = "Post content to decentralized social platforms", long_about = None)]
struct Cli {
    /// Content to post (reads from stdin if not provided)
    content: Option<String>,

    /// Target specific platform(s) (comma-separated)
    #[arg(short, long)]
    platform: Option<String>,

    /// Save as draft without posting
    #[arg(short, long)]
    draft: bool,

    /// Output format (text or json)
    #[arg(short, long, default_value = "text")]
    format: String,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
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

async fn run(_cli: Cli) -> Result<()> {
    // TODO: Implement posting logic
    Err(PlurcastError::InvalidInput("Not yet implemented".to_string()))
}
