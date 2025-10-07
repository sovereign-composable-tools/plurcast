use anyhow::Result;
use clap::Parser;
use libplurcast::{
    config::Config,
    credentials::{CredentialManager, StorageBackend},
};
use std::io::{self, Write};
use tracing::{error, info};

#[derive(Parser)]
#[command(name = "plur-setup")]
#[command(about = "Interactive setup wizard for Plurcast", long_about = None)]
struct Cli {
    /// Skip interactive prompts and use defaults where possible
    #[arg(long)]
    non_interactive: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(log_level)
        .with_writer(std::io::stderr)
        .init();

    info!("Starting Plurcast setup wizard");

    // Run the setup wizard
    if let Err(e) = run_setup(&cli).await {
        error!("Setup failed: {}", e);
        eprintln!("\n❌ Setup failed: {}", e);
        std::process::exit(1);
    }

    Ok(())
}

async fn run_setup(cli: &Cli) -> Result<()> {
    println!("\n🌟 Welcome to Plurcast Setup!\n");
    println!("This wizard will help you configure Plurcast for posting to");
    println!("decentralized social media platforms.\n");

    // Load or create configuration
    let mut config = match Config::load() {
        Ok(config) => {
            println!("✓ Found existing configuration\n");
            config
        }
        Err(_) => {
            println!("Creating new configuration...\n");
            Config::default()
        }
    };

    // Step 1: Select storage backend
    select_storage_backend(&mut config, cli.non_interactive)?;

    // Step 2: Configure platform credentials
    configure_platforms(&config, cli.non_interactive).await?;

    // Step 3: Save configuration
    config.save()?;
    println!("\n✓ Configuration saved");

    // Step 4: Display completion message
    display_completion();

    Ok(())
}

fn select_storage_backend(config: &mut Config, non_interactive: bool) -> Result<()> {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Step 1: Choose Credential Storage Backend");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    println!("Plurcast supports three storage backends for your credentials:\n");
    println!("  1. OS Keyring (recommended)");
    println!("     - macOS: Keychain");
    println!("     - Windows: Credential Manager");
    println!("     - Linux: Secret Service (GNOME Keyring/KWallet)");
    println!("     - Most secure, integrated with your OS\n");
    println!("  2. Encrypted Files");
    println!("     - Password-protected files using age encryption");
    println!("     - Good for systems without keyring support");
    println!("     - Requires master password\n");
    println!("  3. Plain Text (not recommended)");
    println!("     - Credentials stored in plain text files");
    println!("     - Only for testing or legacy compatibility");
    println!("     - Security risk\n");

    let backend = if non_interactive {
        println!("Using default: OS Keyring\n");
        StorageBackend::Keyring
    } else {
        prompt_storage_backend()?
    };

    config.credentials.storage = backend;
    println!("✓ Storage backend set to: {:?}\n", backend);

    Ok(())
}

fn prompt_storage_backend() -> Result<StorageBackend> {
    loop {
        print!("Select storage backend [1-3] (default: 1): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        match input {
            "" | "1" => return Ok(StorageBackend::Keyring),
            "2" => return Ok(StorageBackend::Encrypted),
            "3" => {
                println!("\n⚠️  Warning: Plain text storage is not secure!");
                print!("Are you sure? [y/N]: ");
                io::stdout().flush()?;

                let mut confirm = String::new();
                io::stdin().read_line(&mut confirm)?;
                if confirm.trim().to_lowercase() == "y" {
                    return Ok(StorageBackend::Plain);
                }
                println!("Cancelled. Please choose again.\n");
            }
            _ => println!("Invalid choice. Please enter 1, 2, or 3.\n"),
        }
    }
}

async fn configure_platforms(config: &Config, non_interactive: bool) -> Result<()> {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Step 2: Configure Platform Credentials");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    if non_interactive {
        println!("Skipping platform configuration in non-interactive mode.\n");
        println!("Use 'plur-creds set <platform>' to configure credentials later.\n");
        return Ok(());
    }

    let credential_manager = CredentialManager::new(config)?;

    // Configure Nostr
    if prompt_yes_no("Configure Nostr?", true)? {
        configure_nostr(&credential_manager).await?;
    }

    // Configure Mastodon
    if prompt_yes_no("\nConfigure Mastodon?", true)? {
        configure_mastodon(&credential_manager).await?;
    }

    // Configure Bluesky
    if prompt_yes_no("\nConfigure Bluesky?", true)? {
        configure_bluesky(&credential_manager).await?;
    }

    Ok(())
}

async fn configure_nostr(credential_manager: &CredentialManager) -> Result<()> {
    println!("\n📡 Nostr Configuration");
    println!("────────────────────────────────────────────────────────\n");
    println!("You need a Nostr private key (hex or nsec format).");
    println!("If you don't have one, you can generate it using:");
    println!("  - Nostr clients like Damus, Amethyst, or Snort");
    println!("  - Command line tools like 'nak' or 'nostr-tool'\n");

    let private_key = rpassword::prompt_password("Enter your Nostr private key: ")?;

    if private_key.trim().is_empty() {
        println!("⚠️  Skipped: No private key provided");
        return Ok(());
    }

    // Validate format (basic check)
    let key = private_key.trim();
    if !key.starts_with("nsec1") && key.len() != 64 {
        println!("⚠️  Warning: Key doesn't look like a valid Nostr key");
        println!("   Expected: 64-character hex or nsec1... format");
        if !prompt_yes_no("Continue anyway?", false)? {
            return Ok(());
        }
    }

    // Store credential
    credential_manager.store("plurcast.nostr", "private_key", key)?;
    println!("✓ Nostr credentials stored");

    // Test authentication
    println!("Testing Nostr authentication...");
    match test_nostr_auth(key).await {
        Ok(_) => println!("✓ Nostr authentication successful"),
        Err(e) => {
            println!("⚠️  Authentication test failed: {}", e);
            println!("   Credentials were stored, but may not work correctly");
        }
    }

    Ok(())
}

async fn configure_mastodon(credential_manager: &CredentialManager) -> Result<()> {
    println!("\n🐘 Mastodon Configuration");
    println!("────────────────────────────────────────────────────────\n");
    println!("You need:");
    println!("  1. Your Mastodon instance URL (e.g., mastodon.social)");
    println!("  2. An access token from your instance\n");
    println!("To get an access token:");
    println!("  - Go to Settings → Development → New Application");
    println!("  - Create an app with 'write:statuses' permission");
    println!("  - Copy the access token\n");

    print!("Enter instance URL: ");
    io::stdout().flush()?;
    let mut instance = String::new();
    io::stdin().read_line(&mut instance)?;
    let instance = instance.trim();

    if instance.is_empty() {
        println!("⚠️  Skipped: No instance URL provided");
        return Ok(());
    }

    let access_token = rpassword::prompt_password("Enter access token: ")?;

    if access_token.trim().is_empty() {
        println!("⚠️  Skipped: No access token provided");
        return Ok(());
    }

    // Store credentials
    credential_manager.store("plurcast.mastodon", "access_token", access_token.trim())?;
    credential_manager.store("plurcast.mastodon", "instance", instance)?;
    println!("✓ Mastodon credentials stored");

    // Test authentication
    println!("Testing Mastodon authentication...");
    match test_mastodon_auth(instance, access_token.trim()).await {
        Ok(_) => println!("✓ Mastodon authentication successful"),
        Err(e) => {
            println!("⚠️  Authentication test failed: {}", e);
            println!("   Credentials were stored, but may not work correctly");
        }
    }

    Ok(())
}

async fn configure_bluesky(credential_manager: &CredentialManager) -> Result<()> {
    println!("\n🦋 Bluesky Configuration");
    println!("────────────────────────────────────────────────────────\n");
    println!("You need:");
    println!("  1. Your Bluesky handle (e.g., user.bsky.social)");
    println!("  2. An app password (not your main password!)\n");
    println!("To create an app password:");
    println!("  - Go to Settings → App Passwords");
    println!("  - Create a new app password");
    println!("  - Copy the generated password\n");

    print!("Enter your Bluesky handle: ");
    io::stdout().flush()?;
    let mut handle = String::new();
    io::stdin().read_line(&mut handle)?;
    let handle = handle.trim();

    if handle.is_empty() {
        println!("⚠️  Skipped: No handle provided");
        return Ok(());
    }

    let app_password = rpassword::prompt_password("Enter app password: ")?;

    if app_password.trim().is_empty() {
        println!("⚠️  Skipped: No app password provided");
        return Ok(());
    }

    // Store credentials
    credential_manager.store("plurcast.bluesky", "app_password", app_password.trim())?;
    credential_manager.store("plurcast.bluesky", "handle", handle)?;
    println!("✓ Bluesky credentials stored");

    // Test authentication
    println!("Testing Bluesky authentication...");
    match test_bluesky_auth(handle, app_password.trim()).await {
        Ok(_) => println!("✓ Bluesky authentication successful"),
        Err(e) => {
            println!("⚠️  Authentication test failed: {}", e);
            println!("   Credentials were stored, but may not work correctly");
        }
    }

    Ok(())
}

fn prompt_yes_no(prompt: &str, default: bool) -> Result<bool> {
    let default_str = if default { "Y/n" } else { "y/N" };
    print!("{} [{}]: ", prompt, default_str);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();

    Ok(match input.as_str() {
        "" => default,
        "y" | "yes" => true,
        "n" | "no" => false,
        _ => default,
    })
}

async fn test_nostr_auth(private_key: &str) -> Result<()> {
    use libplurcast::platforms::nostr::NostrClient;

    let client = NostrClient::from_key(private_key)?;
    client.connect().await?;
    Ok(())
}

async fn test_mastodon_auth(instance: &str, access_token: &str) -> Result<()> {
    use libplurcast::platforms::mastodon::MastodonClient;

    let client = MastodonClient::new(instance, access_token)?;
    client.verify_credentials().await?;
    Ok(())
}

async fn test_bluesky_auth(handle: &str, app_password: &str) -> Result<()> {
    use libplurcast::platforms::bluesky::BlueskyClient;

    let client = BlueskyClient::new(handle, app_password)?;
    client.authenticate().await?;
    Ok(())
}

fn display_completion() {
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🎉 Setup Complete!");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    println!("Next steps:\n");
    println!("  1. Test your configuration:");
    println!("     plur-creds test --all\n");
    println!("  2. Post your first message:");
    println!("     echo \"Hello decentralized world!\" | plur-post\n");
    println!("  3. View your posting history:");
    println!("     plur-history\n");
    println!("  4. Manage credentials:");
    println!("     plur-creds list\n");

    println!("For more information:");
    println!("  - Run 'plur-post --help' for posting options");
    println!("  - Run 'plur-creds --help' for credential management");
    println!("  - Visit https://github.com/plurcast/plurcast\n");
}
