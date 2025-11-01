//! plur-creds - Credential management tool for Plurcast
//!
//! This tool provides commands for managing platform credentials securely.

use anyhow::Result;
use clap::{Parser, Subcommand};
use libplurcast::accounts::AccountManager;
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

        /// Account name (default: "default")
        #[arg(long, default_value = "default")]
        account: String,

        /// Read credential from stdin (for automation/agents)
        #[arg(long)]
        stdin: bool,
    },

    /// List stored credentials (without showing values)
    List {
        /// Filter by platform (optional)
        #[arg(long)]
        platform: Option<String>,
    },

    /// Delete credentials for a platform
    Delete {
        /// Platform name (nostr, mastodon, bluesky)
        platform: String,

        /// Account name (default: "default")
        #[arg(long, default_value = "default")]
        account: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },

    /// Set active account for a platform
    Use {
        /// Platform name (nostr, mastodon, bluesky)
        platform: String,

        /// Account name to set as active
        #[arg(long)]
        account: String,
    },

    /// Test credentials by authenticating with the platform
    Test {
        /// Platform name (nostr, mastodon, bluesky), or --all for all platforms
        platform: Option<String>,

        /// Account name (default: active account)
        #[arg(long, default_value = "default")]
        account: String,

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
        Commands::Set {
            platform,
            account,
            stdin,
        } => set_credentials(&platform, &account, stdin).await,
        Commands::List { platform } => list_credentials(platform.as_deref()).await,
        Commands::Delete {
            platform,
            account,
            force,
        } => delete_credentials(&platform, &account, force).await,
        Commands::Use { platform, account } => use_account(&platform, &account).await,
        Commands::Test {
            platform,
            account,
            all,
        } => {
            if all {
                test_all_credentials().await
            } else if let Some(platform) = platform {
                test_credentials(&platform, &account).await
            } else {
                anyhow::bail!("Either specify a platform or use --all flag");
            }
        }
        Commands::Migrate => migrate_credentials().await,
        Commands::Audit => audit_credentials().await,
    }
}

/// Set credentials for a platform
async fn set_credentials(platform: &str, account: &str, use_stdin: bool) -> Result<()> {
    // Validate account name
    AccountManager::validate_account_name(account)?;

    // Load config to get credential configuration
    let config = Config::load()?;

    // Get or create credential config
    let cred_config = config.credentials.unwrap_or_default();

    // Create credential manager and account manager
    let manager = CredentialManager::new(cred_config)?;
    let account_manager = AccountManager::new()?;

    // Determine service and key based on platform
    let (service, key, prompt) = match platform.to_lowercase().as_str() {
        "nostr" => (
            "plurcast.nostr",
            "private_key",
            format!("Enter Nostr private key for account '{}' (hex or nsec format): ", account),
        ),
        "mastodon" => (
            "plurcast.mastodon",
            "access_token",
            format!("Enter Mastodon OAuth access token for account '{}': ", account),
        ),
        "bluesky" => (
            "plurcast.bluesky",
            "app_password",
            format!("Enter Bluesky app password for account '{}': ", account),
        ),
        _ => anyhow::bail!(
            "Unknown platform: {}. Supported platforms: nostr, mastodon, bluesky",
            platform
        ),
    };

    // If a credential already exists, require explicit confirmation before overwriting
    if manager.exists_account(service, key, account)? {
        if use_stdin || !atty::is(atty::Stream::Stdin) {
            anyhow::bail!(
                "Credentials for '{}' account '{}' already exist. Refusing to overwrite in non-interactive mode. \
                 Run interactively or delete first with 'plur-creds delete {} --account {}'.",
                platform, account, platform, account
            );
        } else {
            use std::io::{self, Write};
            println!(
                "\n⚠️  A credential already exists for '{}' account '{}'. This will OVERWRITE the existing secret.",
                platform, account
            );
            print!("Type 'overwrite' to confirm (or anything else to cancel): ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            if input.trim() != "overwrite" {
                println!("Cancelled");
                return Ok(());
            }
        }
    }

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
        rpassword::prompt_password(&prompt)?
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
    manager.store_account(service, key, account, &value)?;

    // Register account with AccountManager
    account_manager.register_account(platform, account)?;

    println!(
        "✓ Stored {} credentials for account '{}' securely using {} backend",
        platform,
        account,
        manager.primary_backend().unwrap_or("unknown")
    );

    Ok(())
}

/// Set active account for a platform
async fn use_account(platform: &str, account: &str) -> Result<()> {
    // Validate account name
    AccountManager::validate_account_name(account)?;

    // Validate platform
    let platform_lower = platform.to_lowercase();
    if !["nostr", "mastodon", "bluesky"].contains(&platform_lower.as_str()) {
        anyhow::bail!(
            "Unknown platform: {}. Supported platforms: nostr, mastodon, bluesky",
            platform
        );
    }

    // Load config and credential manager to check if credentials exist
    let config = Config::load()?;
    let cred_config = config.credentials.unwrap_or_default();
    let manager = CredentialManager::new(cred_config)?;

    // Determine service and key based on platform
    let (service, key) = match platform_lower.as_str() {
        "nostr" => ("plurcast.nostr", "private_key"),
        "mastodon" => ("plurcast.mastodon", "access_token"),
        "bluesky" => ("plurcast.bluesky", "app_password"),
        _ => unreachable!(), // Already validated above
    };

    // Check if credentials exist for this account
    if !manager.exists_account(service, key, account)? {
        anyhow::bail!(
            "Account '{}' not found for platform '{}'. Use 'plur-creds set {} --account {}' to create it.",
            account, platform, platform, account
        );
    }

    // Load account manager and set as active account
    let account_manager = AccountManager::new()?;
    account_manager.set_active_account(&platform_lower, account)?;

    println!(
        "✓ Set '{}' as active account for {}",
        account, platform
    );

    Ok(())
}

/// List stored credentials
async fn list_credentials(platform_filter: Option<&str>) -> Result<()> {
    // Load config to get credential configuration
    let config = Config::load()?;

    // Get or create credential config
    let cred_config = config.credentials.unwrap_or_default();

    // Create credential manager and account manager
    let manager = CredentialManager::new(cred_config)?;
    let account_manager = AccountManager::new()?;

    println!("Stored credentials:");
    println!();

    // Define platforms to check
    let all_platforms = vec![
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

    // Filter platforms if requested
    let platforms: Vec<_> = if let Some(filter) = platform_filter {
        all_platforms
            .into_iter()
            .filter(|(name, _, _, _)| name.eq_ignore_ascii_case(filter))
            .collect()
    } else {
        all_platforms
    };

    if platforms.is_empty() {
        anyhow::bail!(
            "Unknown platform: {}. Supported platforms: nostr, mastodon, bluesky",
            platform_filter.unwrap_or("")
        );
    }

    let mut found_any = false;

    for (platform_name, service, key, credential_type) in platforms {
        // Get all accounts for this platform from AccountManager
        // (CredentialManager.list_accounts() returns empty for keyring since it can't enumerate)
        let accounts = account_manager.list_accounts(platform_name);

        if !accounts.is_empty() {
            // Get active account for this platform
            let active_account = account_manager.get_active_account(platform_name);

            for account in &accounts {
                // Verify the credential actually exists
                if !manager.exists_account(service, key, account)? {
                    continue; // Skip if credential doesn't exist (stale registry entry)
                }

                // Find which backend has it
                let backend = manager.primary_backend().unwrap_or("unknown");

                // Mark active account
                let active_marker = if account == &active_account {
                    " [active]"
                } else {
                    ""
                };

                println!(
                    "  ✓ {} ({}): {} (stored in {}){}",
                    platform_name, account, credential_type, backend, active_marker
                );
                found_any = true;
            }
        }
    }

    if !found_any {
        println!("  No credentials found.");
        println!();
        println!("Use 'plur-creds set <platform> --account <name>' to store credentials.");
    }

    Ok(())
}

/// Delete credentials for a platform
async fn delete_credentials(platform: &str, account: &str, force: bool) -> Result<()> {
    // Validate account name
    AccountManager::validate_account_name(account)?;

    // Load config to get credential configuration
    let config = Config::load()?;

    // Get or create credential config
    let cred_config = config.credentials.unwrap_or_default();

    // Create credential manager and account manager
    let manager = CredentialManager::new(cred_config)?;
    let account_manager = AccountManager::new()?;

    // Determine service and key based on platform
    let platform_lower = platform.to_lowercase();
    let (service, key) = match platform_lower.as_str() {
        "nostr" => ("plurcast.nostr", "private_key"),
        "mastodon" => ("plurcast.mastodon", "access_token"),
        "bluesky" => ("plurcast.bluesky", "app_password"),
        _ => anyhow::bail!(
            "Unknown platform: {}. Supported platforms: nostr, mastodon, bluesky",
            platform
        ),
    };

    // Check if credential exists
    if !manager.exists_account(service, key, account)? {
        println!("No credentials found for {} account '{}'", platform, account);
        return Ok(());
    }

    // Confirm deletion unless --force is used
    if !force && atty::is(atty::Stream::Stdin) {
        use std::io::{self, Write};
        print!(
            "Delete {} credentials for account '{}'? [y/N]: ",
            platform, account
        );
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled");
            return Ok(());
        }
    }

    // Check if deleting active account BEFORE deletion
    let active_account = account_manager.get_active_account(&platform_lower);
    let is_active = active_account == account;

    // Delete the credential
    manager.delete_account(service, key, account)?;

    // Unregister account with AccountManager
    account_manager.unregister_account(&platform_lower, account)?;

    println!("✓ Deleted {} credentials for account '{}'", platform, account);

    // If we deleted the active account, reset to "default"
    if is_active {
        account_manager.set_active_account(&platform_lower, "default")?;
        println!(
            "ℹ Active account was '{}', reset to 'default'",
            account
        );
    }

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

    println!("Migrating credentials to multi-account format...");
    println!();

    // Perform multi-account migration
    let report = manager.migrate_to_multi_account()?;

    // Display results
    println!("Migration complete:");
    println!("  ✓ Migrated: {}", report.migrated.len());
    println!("  ✗ Failed: {}", report.failed.len());
    println!("  ⊘ Skipped: {}", report.skipped.len());
    println!();

    // Show details
    if !report.migrated.is_empty() {
        println!("Successfully migrated to 'default' account:");
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
        println!("Skipped (already migrated):");
        for cred in &report.skipped {
            println!("  ⊘ {}", cred);
        }
        println!();
    }

    // Offer to delete old format credentials if migration was successful
    if report.is_success() && !report.migrated.is_empty() {
        if atty::is(atty::Stream::Stdin) {
            use std::io::{self, Write};
            print!("Delete old format credentials? [y/N]: ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if input.trim().eq_ignore_ascii_case("y") {
                println!("ℹ Old format credentials kept for backward compatibility.");
                println!("  They will not interfere with multi-account operations.");
            } else {
                println!("Old format credentials kept for backward compatibility.");
            }
        }
    } else if !report.is_success() {
        println!("⚠ Some migrations failed. Old format credentials were not deleted.");
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
async fn test_credentials(platform: &str, account: &str) -> Result<()> {
    // Validate account name
    AccountManager::validate_account_name(account)?;

    // Load account manager to determine which account to use
    let account_manager = AccountManager::new()?;

    // If account is "default" and not explicitly set, use active account
    let platform_lower = platform.to_lowercase();
    let account_to_use = if account == "default" {
        account_manager.get_active_account(&platform_lower)
    } else {
        account.to_string()
    };

    println!(
        "Testing {} credentials for account '{}'...",
        platform, account_to_use
    );

    // For now, just check if credentials exist
    // Full authentication testing would require platform client integration
    let config = Config::load()?;
    let cred_config = config.credentials.unwrap_or_default();
    let manager = CredentialManager::new(cred_config)?;

    let (service, key) = match platform_lower.as_str() {
        "nostr" => ("plurcast.nostr", "private_key"),
        "mastodon" => ("plurcast.mastodon", "access_token"),
        "bluesky" => ("plurcast.bluesky", "app_password"),
        _ => anyhow::bail!(
            "Unknown platform: {}. Supported platforms: nostr, mastodon, bluesky",
            platform
        ),
    };

    if manager.exists_account(service, key, &account_to_use)? {
        println!("✓ {} credentials found for account '{}'", platform, account_to_use);
        println!("  Note: Full authentication testing requires platform client integration");
        Ok(())
    } else {
        anyhow::bail!(
            "No credentials found for {} account '{}'",
            platform,
            account_to_use
        );
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
        // Use "default" account for testing all
        match test_credentials(platform, "default").await {
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
