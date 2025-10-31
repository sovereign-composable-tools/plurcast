//! plur-creds - Credential management tool for Plurcast
//!
//! This tool provides commands for managing platform credentials securely.

use anyhow::Result;
use clap::{Parser, Subcommand};
use libplurcast::config::Config;
use libplurcast::credentials::CredentialManager;
use tracing::error;

#[derive(Parser)]
#[command(name = "plur-creds")]
#[command(about = "Manage Plurcast platform credentials securely", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Store credentials for a platform
    Set {
        /// Platform name (nostr, mastodon, bluesky)
        platform: String,

        /// Read credential from stdin (for automation/agents)
        #[arg(long)]
        stdin: bool,
    },

    /// List stored credentials (without showing values)
    List,

    /// Delete credentials for a platform
    Delete {
        /// Platform name (nostr, mastodon, bluesky)
        platform: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },

    /// Test credentials by authenticating with the platform
    Test {
        /// Platform name (nostr, mastodon, bluesky), or --all for all platforms
        platform: Option<String>,

        /// Test all configured platforms
        #[arg(short, long)]
        all: bool,
    },

    /// Migrate credentials from plain text files to secure storage
    Migrate,

    /// Audit credential security
    Audit,
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
        .with_target(false)
        .with_writer(std::io::stderr)
        .init();

    // Execute command
    if let Err(e) = run_command(cli.command).await {
        error!("{}", e);
        std::process::exit(1);
    }

    Ok(())
}

async fn run_command(command: Commands) -> Result<()> {
    match command {
        Commands::Set { platform, stdin } => set_credentials(&platform, stdin).await,
        Commands::List => list_credentials().await,
        Commands::Delete { platform, force } => delete_credentials(&platform, force).await,
        Commands::Test { platform, all } => {
            if all {
                test_all_credentials().await
            } else if let Some(platform) = platform {
                test_credentials(&platform).await
            } else {
                anyhow::bail!("Either specify a platform or use --all flag");
            }
        }
        Commands::Migrate => migrate_credentials().await,
        Commands::Audit => audit_credentials().await,
    }
}

/// Set credentials for a platform
async fn set_credentials(platform: &str, use_stdin: bool) -> Result<()> {
    // Load config to get credential configuration
    let config = Config::load()?;

    // Get or create credential config
    let cred_config = config.credentials.unwrap_or_default();

    // Create credential manager
    let manager = CredentialManager::new(cred_config)?;

    // Determine service and key based on platform
    let (service, key, prompt) = match platform.to_lowercase().as_str() {
        "nostr" => (
            "plurcast.nostr",
            "private_key",
            "Enter Nostr private key (hex or nsec format): ",
        ),
        "mastodon" => (
            "plurcast.mastodon",
            "access_token",
            "Enter Mastodon OAuth access token: ",
        ),
        "bluesky" => (
            "plurcast.bluesky",
            "app_password",
            "Enter Bluesky app password: ",
        ),
        _ => anyhow::bail!(
            "Unknown platform: {}. Supported platforms: nostr, mastodon, bluesky",
            platform
        ),
    };

    // Get credential value: either from stdin or interactive prompt
    let value = if use_stdin {
        // Explicit stdin mode: for automation/agents
        use std::io::{self, Read};
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer.trim().to_string()
    } else {
        // Interactive mode: secure password prompt
        if !atty::is(atty::Stream::Stdin) {
            anyhow::bail!(
                "Not a TTY. Use --stdin flag to read credentials from stdin for automation."
            );
        }
        rpassword::prompt_password(prompt)?
    };

    if value.is_empty() {
        anyhow::bail!("Credential value cannot be empty");
    }

    // Validate credential format for Nostr
    if platform.to_lowercase() == "nostr" {
        let trimmed = value.trim();
        if trimmed.len() != 64 && !trimmed.starts_with("nsec") {
            anyhow::bail!(
                "Invalid Nostr key format. Must be 64-character hex or bech32 nsec format."
            );
        }
    }

    // Store the credential
    manager.store(service, key, &value)?;

    println!(
        "✓ Stored {} credentials securely using {} backend",
        platform,
        manager.primary_backend().unwrap_or("unknown")
    );

    Ok(())
}

/// List stored credentials
async fn list_credentials() -> Result<()> {
    // Load config to get credential configuration
    let config = Config::load()?;

    // Get or create credential config
    let cred_config = config.credentials.unwrap_or_default();

    // Create credential manager
    let manager = CredentialManager::new(cred_config)?;

    println!("Stored credentials:");
    println!();

    // Check for each known platform
    let platforms = vec![
        ("nostr", "plurcast.nostr", "private_key", "Private Key"),
        (
            "mastodon",
            "plurcast.mastodon",
            "access_token",
            "Access Token",
        ),
        (
            "bluesky",
            "plurcast.bluesky",
            "app_password",
            "App Password",
        ),
    ];

    let mut found_any = false;

    for (platform_name, service, key, credential_type) in platforms {
        if manager.exists(service, key)? {
            // Find which backend has it
            let backend = manager.primary_backend().unwrap_or("unknown");
            println!(
                "  ✓ {}: {} (stored in {})",
                platform_name, credential_type, backend
            );
            found_any = true;
        }
    }

    if !found_any {
        println!("  No credentials found.");
        println!();
        println!("Use 'plur-creds set <platform>' to store credentials.");
    }

    Ok(())
}

/// Delete credentials for a platform
async fn delete_credentials(platform: &str, force: bool) -> Result<()> {
    // Load config to get credential configuration
    let config = Config::load()?;

    // Get or create credential config
    let cred_config = config.credentials.unwrap_or_default();

    // Create credential manager
    let manager = CredentialManager::new(cred_config)?;

    // Determine service and key based on platform
    let (service, key) = match platform.to_lowercase().as_str() {
        "nostr" => ("plurcast.nostr", "private_key"),
        "mastodon" => ("plurcast.mastodon", "access_token"),
        "bluesky" => ("plurcast.bluesky", "app_password"),
        _ => anyhow::bail!(
            "Unknown platform: {}. Supported platforms: nostr, mastodon, bluesky",
            platform
        ),
    };

    // Check if credential exists
    if !manager.exists(service, key)? {
        println!("No credentials found for {}", platform);
        return Ok(());
    }

    // Confirm deletion unless --force is used
    if !force && atty::is(atty::Stream::Stdin) {
        use std::io::{self, Write};
        print!("Delete {} credentials? [y/N]: ", platform);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled");
            return Ok(());
        }
    }

    // Delete the credential
    manager.delete(service, key)?;

    println!("✓ Deleted {} credentials", platform);

    Ok(())
}

/// Migrate credentials from plain text files to secure storage
async fn migrate_credentials() -> Result<()> {
    // Load config to get credential configuration
    let config = Config::load()?;

    // Get or create credential config
    let cred_config = config.credentials.unwrap_or_default();

    // Create credential manager
    let manager = CredentialManager::new(cred_config)?;

    // Check if using insecure storage
    if manager.is_insecure() {
        anyhow::bail!(
            "Cannot migrate to plain text storage. \
            Configure keyring or encrypted storage in config.toml first."
        );
    }

    println!("Migrating credentials from plain text files to secure storage...");
    println!();

    // Detect plain text credentials
    let plain_creds = manager.detect_plain_credentials()?;

    if plain_creds.is_empty() {
        println!("No plain text credential files found.");
        return Ok(());
    }

    println!("Found {} plain text credential file(s):", plain_creds.len());
    for (service, key, path) in &plain_creds {
        println!("  - {}.{} at {}", service, key, path.display());
    }
    println!();

    // Perform migration
    let report = manager.migrate_from_plain()?;

    // Display results
    println!("Migration complete:");
    println!("  ✓ Migrated: {}", report.migrated.len());
    println!("  ✗ Failed: {}", report.failed.len());
    println!("  ⊘ Skipped: {}", report.skipped.len());
    println!();

    // Show details
    if !report.migrated.is_empty() {
        println!("Successfully migrated:");
        for cred in &report.migrated {
            println!("  ✓ {}", cred);
        }
        println!();
    }

    if !report.failed.is_empty() {
        println!("Failed to migrate:");
        for (cred, error) in &report.failed {
            println!("  ✗ {}: {}", cred, error);
        }
        println!();
    }

    if !report.skipped.is_empty() {
        println!("Skipped (already in secure storage):");
        for cred in &report.skipped {
            println!("  ⊘ {}", cred);
        }
        println!();
    }

    // Offer to delete plain text files if migration was successful
    if report.is_success() && !report.migrated.is_empty() {
        if atty::is(atty::Stream::Stdin) {
            use std::io::{self, Write};
            print!("Delete plain text files? [y/N]: ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if input.trim().eq_ignore_ascii_case("y") {
                let deleted = manager.cleanup_plain_files(&report.migrated)?;
                println!("✓ Deleted {} plain text file(s)", deleted.len());
            } else {
                println!("Plain text files kept. You can delete them manually later.");
            }
        }
    } else if !report.is_success() {
        println!("⚠ Some migrations failed. Plain text files were not deleted.");
        println!("Fix the errors and run migration again.");
    }

    Ok(())
}

/// Audit credential security
async fn audit_credentials() -> Result<()> {
    println!("Auditing credential security...");
    println!();

    let mut issues_found = false;

    // Load config to get credential configuration
    let config = Config::load()?;

    // Check credential storage configuration
    if let Some(cred_config) = &config.credentials {
        println!("Credential storage configuration:");
        println!("  Backend: {:?}", cred_config.storage);
        println!("  Path: {}", cred_config.path);
        println!();

        // Create credential manager
        let manager = CredentialManager::new(cred_config.clone())?;

        // Check if using insecure storage
        if manager.is_insecure() {
            println!("⚠ SECURITY ISSUE: Using plain text credential storage");
            println!("  Recommendation: Configure keyring or encrypted storage");
            println!("  Run: plur-creds migrate");
            println!();
            issues_found = true;
        } else {
            println!(
                "✓ Using secure credential storage: {}",
                manager.primary_backend().unwrap_or("unknown")
            );
            println!();
        }
    } else {
        println!("⚠ No credential storage configured (using legacy plain text files)");
        println!("  Recommendation: Add [credentials] section to config.toml");
        println!();
        issues_found = true;
    }

    // Check for plain text credential files
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Config directory not found"))?
        .join("plurcast");

    let known_files = vec![
        ("nostr.keys", "Nostr private key"),
        ("mastodon.token", "Mastodon access token"),
        ("bluesky.auth", "Bluesky app password"),
    ];

    let mut plain_files_found = Vec::new();

    for (filename, description) in &known_files {
        let file_path = config_dir.join(filename);
        if file_path.exists() {
            plain_files_found.push((file_path.clone(), description));

            // Check file permissions on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let metadata = std::fs::metadata(&file_path)?;
                let permissions = metadata.permissions();
                let mode = permissions.mode() & 0o777;

                if mode != 0o600 {
                    println!("⚠ SECURITY ISSUE: Incorrect file permissions");
                    println!("  File: {}", file_path.display());
                    println!("  Current: {:o}", mode);
                    println!("  Expected: 600 (owner read/write only)");
                    println!("  Fix: chmod 600 {}", file_path.display());
                    println!();
                    issues_found = true;
                }
            }
        }
    }

    if !plain_files_found.is_empty() {
        println!("⚠ SECURITY ISSUE: Plain text credential files found:");
        for (path, desc) in &plain_files_found {
            println!("  - {} ({})", path.display(), desc);
        }
        println!("  Recommendation: Run 'plur-creds migrate' to move to secure storage");
        println!();
        issues_found = true;
    }

    // Summary
    if issues_found {
        println!("Security audit complete: Issues found");
        println!("Follow the recommendations above to improve security.");
        std::process::exit(1);
    } else {
        println!("✓ Security audit complete: No issues found");
        Ok(())
    }
}

/// Test credentials for a specific platform
async fn test_credentials(platform: &str) -> Result<()> {
    println!("Testing {} credentials...", platform);

    // For now, just check if credentials exist
    // Full authentication testing would require platform client integration
    let config = Config::load()?;
    let cred_config = config.credentials.unwrap_or_default();
    let manager = CredentialManager::new(cred_config)?;

    let (service, key) = match platform.to_lowercase().as_str() {
        "nostr" => ("plurcast.nostr", "private_key"),
        "mastodon" => ("plurcast.mastodon", "access_token"),
        "bluesky" => ("plurcast.bluesky", "app_password"),
        _ => anyhow::bail!(
            "Unknown platform: {}. Supported platforms: nostr, mastodon, bluesky",
            platform
        ),
    };

    if manager.exists(service, key)? {
        println!("✓ {} credentials found", platform);
        println!("  Note: Full authentication testing requires platform client integration");
        Ok(())
    } else {
        anyhow::bail!("No credentials found for {}", platform);
    }
}

/// Test all platform credentials
async fn test_all_credentials() -> Result<()> {
    println!("Testing all platform credentials...");
    println!();

    let platforms = vec!["nostr", "mastodon", "bluesky"];
    let mut found = 0;
    let mut not_found = 0;

    for platform in platforms {
        match test_credentials(platform).await {
            Ok(_) => found += 1,
            Err(_) => {
                println!("✗ {} credentials not found", platform);
                not_found += 1;
            }
        }
    }

    println!();
    println!("Summary: {} found, {} not found", found, not_found);

    if not_found > 0 {
        std::process::exit(1);
    }

    Ok(())
}
