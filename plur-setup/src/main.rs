use anyhow::Result;
use clap::Parser;
use libplurcast::logging::{LogFormat, LoggingConfig};
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

    /// Log format (text, json, pretty)
    #[arg(long, default_value = "text", value_name = "FORMAT", env = "PLURCAST_LOG_FORMAT")]
    #[arg(help = "Log output format: 'text' (default), 'json' (machine-parseable), or 'pretty' (colored for development)")]
    log_format: String,

    /// Log level (error, warn, info, debug, trace)
    #[arg(long, default_value = "info", value_name = "LEVEL", env = "PLURCAST_LOG_LEVEL")]
    #[arg(help = "Minimum log level to display (error, warn, info, debug, trace)")]
    log_level: String,

    /// Set default Proof of Work difficulty for Nostr posts (0-64)
    /// This will be used for all Nostr posts unless overridden with --nostr-pow flag in plur-post
    #[arg(long, value_name = "DIFFICULTY")]
    nostr_pow: Option<u8>,
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
            Config::default_config()
        }
    };

    // Configure Nostr PoW if flag was provided
    if let Some(pow_difficulty) = cli.nostr_pow {
        configure_nostr_pow(&mut config, pow_difficulty)?;
    }

    // Step 1: Select storage backend
    select_storage_backend(&mut config, cli.non_interactive)?;

    // Step 2: Configure platform credentials
    configure_platforms(&mut config, cli.non_interactive).await?;

    // Step 3: Save configuration
    config.save()?;
    println!("\n✓ Configuration saved");

    // Step 4: Display completion message
    display_completion();

    Ok(())
}

fn configure_nostr_pow(config: &mut Config, pow_difficulty: u8) -> Result<()> {
    // Validate difficulty range (0-64)
    if pow_difficulty > 64 {
        return Err(anyhow::anyhow!(
            "Invalid PoW difficulty: {} (must be 0-64)",
            pow_difficulty
        ));
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Configuring Nostr Proof of Work");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Ensure nostr config exists
    if config.nostr.is_none() {
        config.nostr = Some(libplurcast::config::NostrConfig {
            enabled: true,
            keys_file: "~/.config/plurcast/nostr.keys".to_string(),
            relays: vec![
                "wss://relay.damus.io".to_string(),
                "wss://nos.lol".to_string(),
                "wss://relay.nostr.band".to_string(),
            ],
            default_pow_difficulty: None,
        });
    }

    // Set the default PoW difficulty
    if let Some(ref mut nostr_config) = config.nostr {
        nostr_config.default_pow_difficulty = Some(pow_difficulty);
    }

    println!("✓ Default Nostr PoW difficulty set to: {}", pow_difficulty);

    if pow_difficulty > 0 {
        println!("\n  Difficulty Guidelines:");
        println!("    0-10:  Very weak (< 1 second)");
        println!("   10-20:  Light (1-3 seconds)");
        println!("   20-25:  Recommended (1-5 seconds)");
        println!("   25-30:  Strong (5-15 seconds)");
        println!("   30+:    Very strong (15+ seconds)\n");

        println!("  This will be used for all Nostr posts unless you override");
        println!("  with the --nostr-pow flag when posting.\n");
    } else {
        println!("  PoW is disabled (difficulty set to 0)\n");
    }

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

    // Ensure credentials config exists
    if config.credentials.is_none() {
        config.credentials = Some(libplurcast::credentials::CredentialConfig::default());
    }

    if let Some(ref mut creds) = config.credentials {
        creds.storage = backend.clone();
    }

    println!("✓ Storage backend set to: {:?}\n", backend);

    Ok(())
}

fn prompt_storage_backend() -> Result<StorageBackend> {
    loop {
        print!("Select storage backend [1-2] (default: 1): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        match input {
            "" | "1" => return Ok(StorageBackend::Keyring),
            "2" => return Ok(StorageBackend::Encrypted),
            _ => println!("Invalid choice. Please enter 1 or 2.\n"),
        }
    }
}

async fn configure_platforms(config: &mut Config, non_interactive: bool) -> Result<()> {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Step 2: Configure Platform Credentials");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    if non_interactive {
        println!("Skipping platform configuration in non-interactive mode.\n");
        println!("Use 'plur-creds set <platform>' to configure credentials later.\n");
        return Ok(());
    }

    let cred_config = config.credentials.clone().unwrap_or_default();
    let credential_manager = CredentialManager::new(cred_config)?;

    // Configure Nostr
    if prompt_yes_no("Configure Nostr?", true)? {
        configure_nostr(&credential_manager).await?;
    }

    // Configure Mastodon
    if prompt_yes_no("\nConfigure Mastodon?", true)? {
        configure_mastodon(&credential_manager).await?;
    }

    // Configure SSB
    if prompt_yes_no("\nConfigure SSB (Secure Scuttlebutt)?", false)? {
        configure_ssb(&credential_manager, config).await?;
    }

    Ok(())
}

async fn configure_nostr(credential_manager: &CredentialManager) -> Result<()> {
    println!("\n📡 Nostr Configuration");
    println!("────────────────────────────────────────────────────────\n");

    // Check if credentials already exist for the default account
    let service = "plurcast.nostr";
    let key = "private_key";
    let account = "default"; // plur-setup uses the default account for backward compatibility

    if credential_manager.exists_account(service, key, account)? {
        println!("⚠️  WARNING: Nostr credentials already exist for the 'default' account!");
        println!("   Continuing will OVERWRITE your existing private key.");
        println!("   You will LOSE ACCESS to your current Nostr identity.\n");

        print!("Type 'overwrite' to confirm (or anything else to cancel): ");
        io::stdout().flush()?;

        let mut confirmation = String::new();
        io::stdin().read_line(&mut confirmation)?;

        if confirmation.trim() != "overwrite" {
            println!("✓ Cancelled - existing credentials preserved");
            return Ok(());
        }
        println!(); // Add blank line after confirmation
    }

    // Ask if user wants to generate a new key or use existing
    println!("Do you want to:");
    println!("  1. Generate a new Nostr key (recommended for testing)");
    println!("  2. Use an existing Nostr key\n");

    print!("Select option [1-2] (default: 1): ");
    io::stdout().flush()?;

    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    let choice = choice.trim();

    let private_key = if choice == "2" {
        // Use existing key
        println!("\nYou need a Nostr private key (hex or nsec format).");
        println!("If you don't have one, you can generate it using:");
        println!("  - Nostr clients like Damus, Amethyst, or Snort");
        println!("  - Command line tools like 'nak' or 'nostr-tool'\n");

        rpassword::prompt_password("Enter your Nostr private key: ")?
    } else {
        // Generate new key
        generate_nostr_key()?
    };

    // Validate format (basic check)
    let key_value = private_key.trim();
    if !key_value.starts_with("nsec1") && key_value.len() != 64 {
        println!("⚠️  Warning: Key doesn't look like a valid Nostr key");
        println!("   Expected: 64-character hex or nsec1... format");
        if !prompt_yes_no("Continue anyway?", false)? {
            return Ok(());
        }
    }

    // Store credential
    credential_manager.store("plurcast.nostr", "private_key", key_value)?;
    println!("✓ Nostr credentials stored");

    // Test authentication
    println!("\nTesting Nostr authentication...");
    match test_nostr_auth(key_value).await {
        Ok(_) => println!("✓ Nostr authentication successful"),
        Err(e) => {
            println!("⚠️  Authentication test failed: {}", e);
            println!("   Credentials were stored, but may not work correctly");
        }
    }

    Ok(())
}

fn generate_nostr_key() -> Result<String> {
    use nostr_sdk::{Keys, ToBech32};

    println!("\n🔑 Generating new Nostr key pair...\n");

    let keys = Keys::generate();

    let private_hex = keys.secret_key().to_secret_hex();
    let private_bech32 = keys.secret_key().to_bech32()?;
    let public_bech32 = keys.public_key().to_bech32()?;

    println!("✓ Key pair generated successfully!\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Your Nostr Identity");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    println!("Public Key (share this):");
    println!("  {}\n", public_bech32);

    println!("Private Key (keep this secret!):");
    println!("  nsec: {}", private_bech32);
    println!("  hex:  {}\n", private_hex);

    println!("⚠️  IMPORTANT: Save your private key securely!");
    println!("   - This key will be stored in your credential storage");
    println!("   - You may want to back it up separately");
    println!("   - Never share your private key with anyone\n");

    if !prompt_yes_no("Continue with this key?", true)? {
        println!("Cancelled. Generating a new key...\n");
        return generate_nostr_key();
    }

    Ok(private_hex)
}

async fn configure_mastodon(credential_manager: &CredentialManager) -> Result<()> {
    println!("\n🐘 Mastodon Configuration");
    println!("────────────────────────────────────────────────────────\n");

    // Check if credentials already exist for the default account
    let service = "plurcast.mastodon";
    let account = "default"; // plur-setup uses the default account for backward compatibility

    // Check for both access_token and instance
    let has_token = credential_manager.exists_account(service, "access_token", account)?;
    let has_instance = credential_manager.exists_account(service, "instance", account)?;

    if has_token || has_instance {
        println!("⚠️  WARNING: Mastodon credentials already exist for the 'default' account!");
        println!("   Continuing will OVERWRITE your existing access token and instance.");
        println!("   You will need to reconfigure your Mastodon connection.\n");

        print!("Type 'overwrite' to confirm (or anything else to cancel): ");
        io::stdout().flush()?;

        let mut confirmation = String::new();
        io::stdin().read_line(&mut confirmation)?;

        if confirmation.trim() != "overwrite" {
            println!("✓ Cancelled - existing credentials preserved");
            return Ok(());
        }
        println!(); // Add blank line after confirmation
    }

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
    use libplurcast::config::NostrConfig;
    use libplurcast::platforms::nostr::NostrPlatform;
    use libplurcast::platforms::Platform;

    let config = NostrConfig {
        enabled: true,
        keys_file: "".to_string(), // Not used when loading from string
        relays: vec![
            "wss://relay.damus.io".to_string(),
            "wss://nos.lol".to_string(),
        ],
        default_pow_difficulty: None,
    };

    let mut platform = NostrPlatform::new(&config);
    platform.load_keys_from_string(private_key)?;
    platform.authenticate().await?;
    Ok(())
}

async fn test_mastodon_auth(instance: &str, access_token: &str) -> Result<()> {
    use libplurcast::platforms::mastodon::MastodonClient;
    use libplurcast::platforms::Platform;

    // Ensure instance URL has https:// prefix
    let instance_url = if instance.starts_with("http://") || instance.starts_with("https://") {
        instance.to_string()
    } else {
        format!("https://{}", instance)
    };

    let mut client = MastodonClient::new(instance_url, access_token.to_string())?;
    client.authenticate().await?;
    Ok(())
}

async fn configure_ssb(credential_manager: &CredentialManager, config: &mut Config) -> Result<()> {
    println!("\n🔗 SSB (Secure Scuttlebutt) Configuration");
    println!("────────────────────────────────────────────────────────\n");

    // Check if credentials already exist for the default account
    let service = "plurcast.ssb";
    let key = "keypair";
    let account = "default"; // plur-setup uses the default account for backward compatibility

    if credential_manager.exists_account(service, key, account)? {
        println!("⚠️  WARNING: SSB keypair already exists for the 'default' account!");
        println!("   Continuing will OVERWRITE your existing keypair.");
        println!("   You will LOSE ACCESS to your current SSB identity and all posts.\n");

        print!("Type 'overwrite' to confirm (or anything else to cancel): ");
        io::stdout().flush()?;

        let mut confirmation = String::new();
        io::stdin().read_line(&mut confirmation)?;

        if confirmation.trim() != "overwrite" {
            println!("✓ Cancelled - existing credentials preserved");
            return Ok(());
        }
        println!(); // Add blank line after confirmation
    }

    println!("SSB is a peer-to-peer, offline-first social protocol.");
    println!("It uses Ed25519 keypairs and append-only logs.\n");

    // Check for existing ~/.ssb/secret file
    let ssb_secret_path = dirs::home_dir()
        .map(|h| h.join(".ssb").join("secret"))
        .unwrap_or_else(|| std::path::PathBuf::from("~/.ssb/secret"));

    let keypair = if ssb_secret_path.exists() {
        println!("✓ Found existing SSB secret at {}", ssb_secret_path.display());
        if prompt_yes_no("Import existing SSB keypair?", true)? {
            import_ssb_keypair(&ssb_secret_path)?
        } else {
            generate_ssb_keypair()?
        }
    } else {
        println!("No existing SSB secret found at {}", ssb_secret_path.display());
        if prompt_yes_no("Generate new SSB keypair?", true)? {
            generate_ssb_keypair()?
        } else {
            println!("⚠️  Skipped: No keypair configured");
            return Ok(());
        }
    };

    // Store keypair in credential manager
    credential_manager.store("plurcast.ssb", "keypair", &keypair)?;
    println!("✓ SSB keypair stored in credential manager");

    // Initialize feed database
    let feed_path = prompt_feed_path()?;
    initialize_feed_database(&feed_path)?;

    // Prompt for pub server addresses
    let pubs = prompt_pub_servers()?;

    // Test connection to pubs if any were configured
    if !pubs.is_empty() {
        println!("\nTesting pub server connections...");
        test_pub_connections(&pubs).await?;
    } else {
        println!("\n⚠️  No pub servers configured - SSB will run in local-only mode");
        println!("   You can add pub servers later in config.toml");
    }

    // Update config with SSB settings
    config.ssb = Some(libplurcast::config::SSBConfig {
        enabled: true,
        feed_path,
        pubs,
    });

    println!("\n✓ SSB configuration complete");

    Ok(())
}

fn import_ssb_keypair(path: &std::path::Path) -> Result<String> {
    println!("\n📥 Importing SSB keypair from {}", path.display());

    // Read the secret file
    let content = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Failed to read SSB secret file: {}", e))?;

    // Parse the JSON (SSB secret files are JSON with comments)
    // Remove comments (lines starting with #)
    let json_content: String = content
        .lines()
        .filter(|line| !line.trim().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");

    let secret: serde_json::Value = serde_json::from_str(&json_content)
        .map_err(|e| anyhow::anyhow!("Failed to parse SSB secret file: {}", e))?;

    // Extract the private key
    let private_key = secret
        .get("private")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("SSB secret file missing 'private' field"))?;

    // Validate format (should be base64 with .ed25519 suffix)
    if !private_key.ends_with(".ed25519") {
        return Err(anyhow::anyhow!(
            "Invalid SSB private key format (expected .ed25519 suffix)"
        ));
    }

    println!("✓ SSB keypair imported successfully");

    Ok(private_key.to_string())
}

fn generate_ssb_keypair() -> Result<String> {
    println!("\n🔑 Generating new SSB keypair...\n");

    // Generate Ed25519 keypair using kuska-ssb
    use kuska_ssb::crypto::{ed25519, ToSsbId};

    let keypair = ed25519::gen_keypair();
    let public_key = keypair.0.to_ssb_id();
    let private_key = keypair.1.to_ssb_id();

    println!("✓ Keypair generated successfully!\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Your SSB Identity");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    println!("Public Key (share this):");
    println!("  {}\n", public_key);

    println!("Private Key (keep this secret!):");
    println!("  {}\n", private_key);

    println!("⚠️  IMPORTANT: Save your private key securely!");
    println!("   - This key will be stored in your credential storage");
    println!("   - You may want to back it up separately");
    println!("   - Never share your private key with anyone\n");

    if !prompt_yes_no("Continue with this keypair?", true)? {
        println!("Cancelled. Generating a new keypair...\n");
        return generate_ssb_keypair();
    }

    Ok(private_key)
}

fn prompt_feed_path() -> Result<String> {
    println!("\n📁 Feed Database Path");
    println!("────────────────────────────────────────────────────────");
    println!("SSB stores your messages in a local feed database.");
    println!("Default: ~/.plurcast-ssb\n");

    print!("Enter feed path (or press Enter for default): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    let feed_path = if input.is_empty() {
        "~/.plurcast-ssb".to_string()
    } else {
        input.to_string()
    };

    println!("✓ Feed path set to: {}", feed_path);

    Ok(feed_path)
}

fn initialize_feed_database(feed_path: &str) -> Result<()> {
    println!("\n📦 Initializing feed database...");

    // Expand the path
    let expanded = shellexpand::full(feed_path)
        .map_err(|e| anyhow::anyhow!("Failed to expand feed path: {}", e))?;
    let path = std::path::PathBuf::from(expanded.as_ref());

    // Create directory if it doesn't exist
    if !path.exists() {
        std::fs::create_dir_all(&path)
            .map_err(|e| anyhow::anyhow!("Failed to create feed directory: {}", e))?;

        // Set permissions to 700 (owner read/write/execute only) on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = std::fs::Permissions::from_mode(0o700);
            std::fs::set_permissions(&path, permissions)
                .map_err(|e| anyhow::anyhow!("Failed to set directory permissions: {}", e))?;
        }

        println!("✓ Created feed database directory at {}", path.display());
    } else {
        println!("✓ Feed database directory already exists at {}", path.display());
    }

    Ok(())
}

fn prompt_pub_servers() -> Result<Vec<String>> {
    println!("\n🌐 Pub Server Configuration");
    println!("────────────────────────────────────────────────────────");
    println!("Pub servers help with peer discovery and replication.");
    println!("Format: net:host:port~shs:pubkey");
    println!("Example: net:hermies.club:8008~shs:base64key...\n");
    println!("You can add multiple pubs or skip this for local-only mode.\n");

    let mut pubs = Vec::new();

    loop {
        print!("Enter pub server address (or press Enter to finish): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            break;
        }

        // Basic validation of multiserver address format
        if !input.starts_with("net:") || !input.contains("~shs:") {
            println!("⚠️  Warning: Address doesn't match expected format");
            println!("   Expected: net:host:port~shs:pubkey");
            if !prompt_yes_no("Add anyway?", false)? {
                continue;
            }
        }

        pubs.push(input.to_string());
        println!("✓ Added pub server: {}", input);
    }

    if pubs.is_empty() {
        println!("No pub servers configured (local-only mode)");
    } else {
        println!("\n✓ Configured {} pub server(s)", pubs.len());
    }

    Ok(pubs)
}

async fn test_pub_connections(pubs: &[String]) -> Result<()> {
    println!("Testing connections to {} pub server(s)...", pubs.len());

    for (i, pub_addr) in pubs.iter().enumerate() {
        print!("  [{}/{}] Testing {}... ", i + 1, pubs.len(), pub_addr);
        io::stdout().flush()?;

        // For now, just validate the format
        // TODO: Implement actual connection test when kuska-ssb integration is complete
        if pub_addr.starts_with("net:") && pub_addr.contains("~shs:") {
            println!("✓ Format valid");
        } else {
            println!("⚠️  Format may be invalid");
        }
    }

    println!("\n⚠️  Note: Full connection testing will be available in a future update");
    println!("   For now, we've validated the address format only");

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
