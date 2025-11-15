//! Configuration management for Plurcast

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::credentials::CredentialConfig;
use crate::error::{ConfigError, Result};

/// Main configuration structure for Plurcast
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Database configuration
    pub database: DatabaseConfig,

    /// Credential storage configuration (optional)
    #[serde(default)]
    pub credentials: Option<CredentialConfig>,

    /// Nostr platform configuration (optional)
    #[serde(default)]
    pub nostr: Option<NostrConfig>,

    /// Mastodon platform configuration (optional)
    #[serde(default)]
    pub mastodon: Option<MastodonConfig>,

    /// SSB platform configuration (optional)
    #[serde(default)]
    pub ssb: Option<SSBConfig>,

    /// Default settings
    #[serde(default)]
    pub defaults: DefaultsConfig,

    /// Scheduling configuration (optional)
    #[serde(default)]
    pub scheduling: Option<SchedulingConfig>,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Path to the SQLite database file
    /// Supports ~ expansion and environment variable override via PLURCAST_DB_PATH
    pub path: String,
}

/// Nostr platform configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NostrConfig {
    /// Whether Nostr posting is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Path to the file containing Nostr private keys
    /// Supports both hex and bech32 (nsec) formats
    pub keys_file: String,

    /// List of Nostr relay URLs to connect to
    #[serde(default = "default_nostr_relays")]
    pub relays: Vec<String>,
}

impl NostrConfig {
    /// Expand shell variables in the keys_file path
    pub fn expand_keys_file_path(&self) -> Result<PathBuf> {
        let expanded = shellexpand::full(&self.keys_file).map_err(|e| {
            ConfigError::MissingField(format!("Failed to expand keys_file path: {}", e))
        })?;
        Ok(PathBuf::from(expanded.as_ref()))
    }
}

/// Mastodon platform configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MastodonConfig {
    /// Whether Mastodon posting is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Mastodon instance URL (e.g., "mastodon.social")
    pub instance: String,

    /// Path to the file containing the OAuth access token
    pub token_file: String,
}

impl MastodonConfig {
    /// Expand shell variables in the token_file path
    pub fn expand_token_file_path(&self) -> Result<PathBuf> {
        let expanded = shellexpand::full(&self.token_file).map_err(|e| {
            ConfigError::MissingField(format!("Failed to expand token_file path: {}", e))
        })?;
        Ok(PathBuf::from(expanded.as_ref()))
    }
}

/// SSB (Secure Scuttlebutt) platform configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SSBConfig {
    /// Whether SSB posting is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Path to the local SSB feed database directory
    /// This is where kuska-ssb will store the append-only log
    #[serde(default = "default_ssb_feed_path")]
    pub feed_path: String,

    /// List of pub server addresses for replication
    /// Format: "net:host:port~shs:pubkey" (multiserver address format)
    #[serde(default)]
    pub pubs: Vec<String>,
}

impl SSBConfig {
    /// Expand shell variables in the feed_path
    pub fn expand_feed_path(&self) -> Result<PathBuf> {
        let expanded = shellexpand::full(&self.feed_path).map_err(|e| {
            ConfigError::MissingField(format!("Failed to expand feed_path: {}", e))
        })?;
        Ok(PathBuf::from(expanded.as_ref()))
    }
}

/// Default SSB feed path
fn default_ssb_feed_path() -> String {
    "~/.plurcast-ssb".to_string()
}

/// Default configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultsConfig {
    /// Default platforms to post to when not specified
    #[serde(default = "default_platforms")]
    pub platforms: Vec<String>,
}

// Default value functions for serde
fn default_true() -> bool {
    true
}

fn default_nostr_relays() -> Vec<String> {
    vec![
        // Popular public relays with good connectivity
        "wss://relay.damus.io".to_string(),
        "wss://relay.primal.net".to_string(),
        "wss://relay.snort.social".to_string(),
        "wss://nos.lol".to_string(),
        "wss://relay.nostr.band".to_string(),
        // Additional well-connected relays
        "wss://purplepag.es".to_string(),
        "wss://relay.mostr.pub".to_string(), // Bridges Mastodon/Fediverse
    ]
}

fn default_platforms() -> Vec<String> {
    vec!["nostr".to_string()]
}

impl Default for DefaultsConfig {
    fn default() -> Self {
        Self {
            platforms: default_platforms(),
        }
    }
}

/// Scheduling daemon configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingConfig {
    /// How often (in seconds) to poll the database for scheduled posts
    #[serde(default = "default_poll_interval")]
    pub poll_interval: u64,

    /// Maximum number of retry attempts for failed posts
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Delay (in seconds) before retrying a failed post
    #[serde(default = "default_retry_delay")]
    pub retry_delay: u64,

    /// Platform-specific rate limits
    #[serde(default)]
    pub rate_limits: std::collections::HashMap<String, RateLimitConfig>,
}

/// Rate limit configuration for a platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum number of posts allowed per hour
    pub posts_per_hour: u32,
}

// Default values for scheduling configuration
fn default_poll_interval() -> u64 {
    60 // Poll every 60 seconds
}

fn default_max_retries() -> u32 {
    3 // Retry up to 3 times
}

fn default_retry_delay() -> u64 {
    300 // Wait 5 minutes (300 seconds) before retrying
}

impl Default for SchedulingConfig {
    fn default() -> Self {
        use std::collections::HashMap;

        let mut rate_limits = HashMap::new();
        // Default rate limits (conservative)
        rate_limits.insert(
            "nostr".to_string(),
            RateLimitConfig {
                posts_per_hour: 100,
            },
        );
        rate_limits.insert(
            "mastodon".to_string(),
            RateLimitConfig {
                posts_per_hour: 300,
            },
        );
        rate_limits.insert(
            "ssb".to_string(),
            RateLimitConfig {
                posts_per_hour: 1000, // SSB is local, higher limit
            },
        );

        Self {
            poll_interval: default_poll_interval(),
            max_retries: default_max_retries(),
            retry_delay: default_retry_delay(),
            rate_limits,
        }
    }
}

impl Config {
    /// Load configuration from the default location
    ///
    /// If the configuration file doesn't exist at the default location, creates a default one.
    /// If PLURCAST_CONFIG is set and the file doesn't exist, returns an error.
    pub fn load() -> Result<Self> {
        let config_path = resolve_config_path()?;
        let is_explicit_path = std::env::var("PLURCAST_CONFIG").is_ok();

        // If config doesn't exist
        if !config_path.exists() {
            if is_explicit_path {
                // User explicitly set PLURCAST_CONFIG - fail if file doesn't exist
                return Err(ConfigError::ReadError(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!(
                        "Config file not found: {} (set via PLURCAST_CONFIG)",
                        config_path.display()
                    ),
                ))
                .into());
            } else {
                // Using default path - create default config
                Self::create_default_config(&config_path)?;
            }
        }

        Self::load_from_path(&config_path)
    }

    /// Load configuration from a specific path
    ///
    /// Returns detailed error messages for parsing failures
    pub fn load_from_path(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            ConfigError::ReadError(std::io::Error::new(
                e.kind(),
                format!("Failed to read config from {}: {}", path.display(), e),
            ))
        })?;

        let mut config: Config = toml::from_str(&content).map_err(ConfigError::ParseError)?;

        // Load master password from environment variable if available
        if let Some(credentials) = &mut config.credentials {
            credentials.load_master_password_from_env();
        }

        // Validate the configuration
        config.validate()?;

        Ok(config)
    }

    /// Validate the configuration
    ///
    /// Checks that required fields are present for enabled platforms
    /// and expands shell variables in credential file paths
    pub fn validate(&self) -> Result<()> {
        // Validate Nostr configuration if present and enabled
        if let Some(nostr) = &self.nostr {
            if nostr.enabled {
                if nostr.keys_file.is_empty() {
                    return Err(ConfigError::MissingField(
                        "Nostr is enabled but keys_file is empty".to_string(),
                    )
                    .into());
                }
                if nostr.relays.is_empty() {
                    return Err(ConfigError::MissingField(
                        "Nostr is enabled but no relays are configured".to_string(),
                    )
                    .into());
                }
            }
        }

        // Validate Mastodon configuration if present and enabled
        if let Some(mastodon) = &self.mastodon {
            if mastodon.enabled {
                if mastodon.instance.is_empty() {
                    return Err(ConfigError::MissingField(
                        "Mastodon is enabled but instance is empty".to_string(),
                    )
                    .into());
                }
                if mastodon.token_file.is_empty() {
                    return Err(ConfigError::MissingField(
                        "Mastodon is enabled but token_file is empty".to_string(),
                    )
                    .into());
                }
            }
        }

        // Validate SSB configuration if present and enabled
        if let Some(ssb) = &self.ssb {
            if ssb.enabled {
                if ssb.feed_path.is_empty() {
                    return Err(ConfigError::MissingField(
                        "SSB is enabled but feed_path is empty".to_string(),
                    )
                    .into());
                }
                // Note: pubs list can be empty (local-only mode)
            }
        }

        // Validate credential configuration if present
        if let Some(credentials) = &self.credentials {
            credentials.validate()?;
        }

        Ok(())
    }

    /// Create a default configuration file at the specified path
    ///
    /// Creates parent directories if they don't exist
    /// Sets file permissions to 600 (owner read/write only) for security
    pub fn create_default_config(path: &PathBuf) -> Result<()> {
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(ConfigError::ReadError)?;
        }

        // Generate config with helpful comments
        let toml_content = Self::generate_default_config_with_comments();

        // Write to file
        std::fs::write(path, toml_content).map_err(ConfigError::ReadError)?;

        // Set file permissions to 600 (owner read/write only) on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(path, permissions).map_err(ConfigError::ReadError)?;
        }

        Ok(())
    }

    /// Generate a default configuration file with helpful comments
    fn generate_default_config_with_comments() -> String {
        r#"# Plurcast Configuration File
# This file configures multi-platform posting for Plurcast
# Supports: Nostr, Mastodon, and SSB (Secure Scuttlebutt)

# Database configuration
[database]
# Path to the SQLite database file
# Supports ~ expansion and environment variable override via PLURCAST_DB_PATH
path = "~/.local/share/plurcast/posts.db"

# Credential storage configuration
[credentials]
# Storage backend: "keyring" (OS native), "encrypted" (password-protected files), "plain" (not recommended)
# - keyring: Uses OS-native secure storage (macOS Keychain, Windows Credential Manager, Linux Secret Service)
# - encrypted: Stores credentials in password-protected files using age encryption
# - plain: Stores credentials in plain text files (INSECURE - only for backward compatibility)
storage = "keyring"

# Path for encrypted/plain file storage (keyring doesn't use files)
# This is where encrypted credential files will be stored if using "encrypted" backend
path = "~/.config/plurcast/credentials"

# Note: Master password for encrypted storage can be set via PLURCAST_MASTER_PASSWORD environment variable
# or will be prompted interactively when needed

# Nostr platform configuration
[nostr]
# Enable or disable Nostr posting
enabled = true

# Path to the file containing Nostr private keys
# Supports both hex and bech32 (nsec) formats
keys_file = "~/.config/plurcast/nostr.keys"

# List of Nostr relay URLs to connect to
relays = [
    "wss://relay.damus.io",
    "wss://nos.lol",
    "wss://relay.nostr.band"
]

# Mastodon platform configuration (disabled by default)
# Uncomment and configure to enable Mastodon posting
# [mastodon]
# enabled = true
# instance = "mastodon.social"
# token_file = "~/.config/plurcast/mastodon.token"

# SSB (Secure Scuttlebutt) platform configuration (disabled by default)
# Uncomment and configure to enable SSB posting
# [ssb]
# enabled = true
# feed_path = "~/.plurcast-ssb"
# pubs = [
#     "net:hermies.club:8008~shs:base64-pubkey-here",
# ]

# Default settings
[defaults]
# Default platforms to post to when not specified via --platform flag
platforms = ["nostr"]
"#.to_string()
    }

    /// Get a default configuration structure
    ///
    /// This is used for creating new config files
    pub fn default_config() -> Self {
        Self {
            database: DatabaseConfig {
                path: "~/.local/share/plurcast/posts.db".to_string(),
            },
            credentials: Some(CredentialConfig::default()),
            nostr: Some(NostrConfig {
                enabled: true,
                keys_file: "~/.config/plurcast/nostr.keys".to_string(),
                relays: default_nostr_relays(),
            }),
            mastodon: None,
            ssb: None,
            defaults: DefaultsConfig::default(),
            scheduling: Some(SchedulingConfig::default()),
        }
    }

    /// Save configuration to the default location
    ///
    /// Creates parent directories if they don't exist
    pub fn save(&self) -> Result<()> {
        let config_path = resolve_config_path()?;
        self.save_to_path(&config_path)
    }

    /// Save configuration to a specific path
    ///
    /// Creates parent directories if they don't exist
    /// Sets file permissions to 600 (owner read/write only) for security
    pub fn save_to_path(&self, path: &PathBuf) -> Result<()> {
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(ConfigError::ReadError)?;
        }

        // Serialize to TOML
        let toml_content = toml::to_string_pretty(self)
            .map_err(|e| ConfigError::MissingField(format!("Failed to serialize config: {}", e)))?;

        // Write to file
        std::fs::write(path, toml_content).map_err(ConfigError::ReadError)?;

        // Set file permissions to 600 (owner read/write only) on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(path, permissions).map_err(ConfigError::ReadError)?;
        }

        Ok(())
    }
}

/// Resolve the configuration file path following XDG Base Directory spec
///
/// Priority order:
/// 1. PLURCAST_CONFIG environment variable (if set)
/// 2. XDG_CONFIG_HOME/plurcast/config.toml (typically ~/.config/plurcast/config.toml)
///
/// Supports ~ expansion and relative paths
pub fn resolve_config_path() -> Result<PathBuf> {
    // Check for environment variable override first
    if let Ok(path) = std::env::var("PLURCAST_CONFIG") {
        let expanded = shellexpand::full(&path)
            .map_err(|e| ConfigError::MissingField(format!("Failed to expand path: {}", e)))?;
        return Ok(PathBuf::from(expanded.as_ref()));
    }

    // Fall back to XDG Base Directory standard
    let config_dir = dirs::config_dir()
        .ok_or_else(|| ConfigError::MissingField("XDG config directory not found".to_string()))?;

    Ok(config_dir.join("plurcast").join("config.toml"))
}

/// Resolve the data directory path following XDG Base Directory spec
///
/// Returns: XDG_DATA_HOME/plurcast (typically ~/.local/share/plurcast)
///
/// This is where the SQLite database and other data files are stored
pub fn resolve_data_path() -> Result<PathBuf> {
    let data_dir = dirs::data_dir()
        .ok_or_else(|| ConfigError::MissingField("XDG data directory not found".to_string()))?;

    Ok(data_dir.join("plurcast"))
}

/// Resolve the database path with environment variable override support
///
/// Priority order:
/// 1. PLURCAST_DB_PATH environment variable (if set)
/// 2. Path from configuration (with ~ expansion)
/// 3. Default XDG data directory path
pub fn resolve_db_path(config_path: Option<&str>) -> Result<PathBuf> {
    // Check for environment variable override first
    if let Ok(path) = std::env::var("PLURCAST_DB_PATH") {
        let expanded = shellexpand::full(&path)
            .map_err(|e| ConfigError::MissingField(format!("Failed to expand DB path: {}", e)))?;
        return Ok(PathBuf::from(expanded.as_ref()));
    }

    // Use config path if provided
    if let Some(path) = config_path {
        let expanded = shellexpand::full(path)
            .map_err(|e| ConfigError::MissingField(format!("Failed to expand DB path: {}", e)))?;
        return Ok(PathBuf::from(expanded.as_ref()));
    }

    // Fall back to default data directory
    let data_path = resolve_data_path()?;
    Ok(data_path.join("posts.db"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{ConfigError, PlurcastError};
    use std::env;
    use tempfile::TempDir;

    #[test]
    fn test_toml_parsing_valid_config() {
        let toml_content = r#"
[database]
path = "~/.local/share/plurcast/posts.db"

[nostr]
enabled = true
keys_file = "~/.config/plurcast/nostr.keys"
relays = ["wss://relay.damus.io", "wss://nos.lol"]

[defaults]
platforms = ["nostr"]
"#;

        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.database.path, "~/.local/share/plurcast/posts.db");
        assert!(config.nostr.is_some());

        let nostr = config.nostr.unwrap();
        assert!(nostr.enabled);
        assert_eq!(nostr.keys_file, "~/.config/plurcast/nostr.keys");
        assert_eq!(nostr.relays.len(), 2);
        assert_eq!(config.defaults.platforms, vec!["nostr"]);
    }

    #[test]
    fn test_toml_parsing_minimal_config() {
        let toml_content = r#"
[database]
path = "/tmp/test.db"
"#;

        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.database.path, "/tmp/test.db");
        assert!(config.nostr.is_none());
        assert!(config.mastodon.is_none());
        assert_eq!(config.defaults.platforms, vec!["nostr"]); // Default value
    }

    #[test]
    fn test_toml_parsing_invalid_config_missing_database() {
        let toml_content = r#"
[nostr]
enabled = true
"#;

        let result: std::result::Result<Config, toml::de::Error> = toml::from_str(toml_content);
        assert!(result.is_err());
    }

    #[test]
    fn test_toml_parsing_invalid_syntax() {
        let toml_content = r#"
[database
path = "test.db"
"#;

        let result: std::result::Result<Config, toml::de::Error> = toml::from_str(toml_content);
        assert!(result.is_err());
    }

    #[test]
    fn test_default_config_generation() {
        let config = Config::default_config();

        assert_eq!(config.database.path, "~/.local/share/plurcast/posts.db");
        assert!(config.nostr.is_some());

        let nostr = config.nostr.unwrap();
        assert!(nostr.enabled);
        assert_eq!(nostr.keys_file, "~/.config/plurcast/nostr.keys");
        assert_eq!(nostr.relays.len(), 7); // Updated to match default_nostr_relays()
        assert!(nostr.relays.contains(&"wss://relay.damus.io".to_string()));
        assert_eq!(config.defaults.platforms, vec!["nostr"]);
    }

    #[test]
    fn test_create_default_config_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        Config::create_default_config(&config_path).unwrap();

        assert!(config_path.exists());

        // Verify file can be parsed
        let config = Config::load_from_path(&config_path).unwrap();
        assert_eq!(config.database.path, "~/.local/share/plurcast/posts.db");
    }

    #[test]
    #[cfg(unix)]
    fn test_config_file_permissions_unix() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        Config::create_default_config(&config_path).unwrap();

        let metadata = std::fs::metadata(&config_path).unwrap();
        let permissions = metadata.permissions();

        // Check that permissions are 600 (owner read/write only)
        assert_eq!(permissions.mode() & 0o777, 0o600);
    }

    #[test]
    fn test_xdg_path_resolution() {
        // Test that resolve_config_path returns a valid path
        let config_path = resolve_config_path().unwrap();

        // Should end with plurcast/config.toml
        assert!(config_path.to_string_lossy().contains("plurcast"));
        assert!(config_path.to_string_lossy().ends_with("config.toml"));
    }

    #[test]
    fn test_xdg_data_path_resolution() {
        let data_path = resolve_data_path().unwrap();

        // Should end with plurcast
        assert!(data_path.to_string_lossy().ends_with("plurcast"));
    }

    #[test]
    fn test_env_var_override_plurcast_config() {
        let temp_dir = TempDir::new().unwrap();
        let custom_config_path = temp_dir.path().join("custom_config.toml");

        // Create a config file
        Config::create_default_config(&custom_config_path).unwrap();

        // Set environment variable
        env::set_var("PLURCAST_CONFIG", custom_config_path.to_str().unwrap());

        let resolved_path = resolve_config_path().unwrap();
        assert_eq!(resolved_path, custom_config_path);

        // Clean up
        env::remove_var("PLURCAST_CONFIG");
    }

    #[test]
    fn test_env_var_override_plurcast_db_path() {
        let temp_dir = TempDir::new().unwrap();
        let custom_db_path = temp_dir.path().join("custom.db");

        // Set environment variable
        env::set_var("PLURCAST_DB_PATH", custom_db_path.to_str().unwrap());

        let resolved_path = resolve_db_path(None).unwrap();
        assert_eq!(resolved_path, custom_db_path);

        // Clean up
        env::remove_var("PLURCAST_DB_PATH");
    }

    #[test]
    fn test_db_path_from_config() {
        let config_path = "/tmp/test.db";
        let resolved = resolve_db_path(Some(config_path)).unwrap();
        assert_eq!(resolved, PathBuf::from(config_path));
    }

    #[test]
    fn test_db_path_default() {
        // Ensure no env var is set
        env::remove_var("PLURCAST_DB_PATH");

        let resolved = resolve_db_path(None).unwrap();

        // Should end with plurcast/posts.db
        assert!(resolved.to_string_lossy().contains("plurcast"));
        assert!(resolved.to_string_lossy().ends_with("posts.db"));
    }

    #[test]
    fn test_path_expansion_tilde() {
        let path_with_tilde = "~/test.db";
        let resolved = resolve_db_path(Some(path_with_tilde)).unwrap();

        // Should not contain tilde after expansion
        assert!(!resolved.to_string_lossy().contains('~'));
    }

    #[test]
    fn test_path_expansion_in_env_var() {
        let path_with_tilde = "~/custom/test.db";

        env::set_var("PLURCAST_CONFIG", path_with_tilde);
        let resolved = resolve_config_path().unwrap();

        // Should not contain tilde after expansion
        assert!(!resolved.to_string_lossy().contains('~'));

        // Clean up
        env::remove_var("PLURCAST_CONFIG");
    }

    #[test]
    fn test_load_config_from_path() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");

        // Create a test config
        let toml_content = r#"
[database]
path = "/tmp/test.db"

[nostr]
enabled = false
keys_file = "/tmp/keys"
relays = ["wss://test.relay"]

[defaults]
platforms = ["nostr"]
"#;
        std::fs::write(&config_path, toml_content).unwrap();

        let config = Config::load_from_path(&config_path).unwrap();
        assert_eq!(config.database.path, "/tmp/test.db");
        assert!(config.nostr.is_some());
        assert!(!config.nostr.unwrap().enabled);
    }

    #[test]
    fn test_load_config_nonexistent_file() {
        let nonexistent_path = PathBuf::from("/nonexistent/path/config.toml");
        let result = Config::load_from_path(&nonexistent_path);

        assert!(result.is_err());
        match result {
            Err(PlurcastError::Config(ConfigError::ReadError(_))) => {
                // Expected error type
            }
            _ => panic!("Expected ConfigError::ReadError"),
        }
    }

    #[test]
    fn test_load_config_invalid_toml() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("invalid.toml");

        std::fs::write(&config_path, "invalid toml content {{{").unwrap();

        let result = Config::load_from_path(&config_path);
        assert!(result.is_err());
        match result {
            Err(PlurcastError::Config(ConfigError::ParseError(_))) => {
                // Expected error type
            }
            _ => panic!("Expected ConfigError::ParseError"),
        }
    }

    #[test]
    fn test_nostr_config_defaults() {
        let toml_content = r#"
[database]
path = "/tmp/test.db"

[nostr]
keys_file = "/tmp/keys"
"#;

        let config: Config = toml::from_str(toml_content).unwrap();
        let nostr = config.nostr.unwrap();

        // Should have default values
        assert!(nostr.enabled); // default_true()
        assert_eq!(nostr.relays.len(), 7); // default_nostr_relays() - updated to match current default
    }

    #[test]
    fn test_config_with_nested_paths() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir
            .path()
            .join("nested")
            .join("deep")
            .join("config.toml");

        // Should create parent directories
        Config::create_default_config(&nested_path).unwrap();

        assert!(nested_path.exists());
        assert!(nested_path.parent().unwrap().exists());
    }

    // Multi-platform configuration tests

    #[test]
    fn test_parse_multi_platform_config() {
        let toml_content = r#"
[database]
path = "/tmp/test.db"

[nostr]
enabled = true
keys_file = "/tmp/nostr.keys"
relays = ["wss://relay1.com"]

[mastodon]
enabled = true
instance = "mastodon.social"
token_file = "/tmp/mastodon.token"

[defaults]
platforms = ["nostr", "mastodon"]
"#;

        let config: Config = toml::from_str(toml_content).unwrap();

        // Verify all platforms are parsed
        assert!(config.nostr.is_some());
        assert!(config.mastodon.is_some());

        // Verify Mastodon config
        let mastodon = config.mastodon.unwrap();
        assert!(mastodon.enabled);
        assert_eq!(mastodon.instance, "mastodon.social");
        assert_eq!(mastodon.token_file, "/tmp/mastodon.token");

        // Verify defaults
        assert_eq!(config.defaults.platforms.len(), 2);
        assert!(config.defaults.platforms.contains(&"nostr".to_string()));
        assert!(config.defaults.platforms.contains(&"mastodon".to_string()));
    }

    #[test]
    fn test_validate_config_missing_nostr_keys_file() {
        let toml_content = r#"
[database]
path = "/tmp/test.db"

[nostr]
enabled = true
keys_file = ""
relays = ["wss://relay1.com"]
"#;

        let config: Config = toml::from_str(toml_content).unwrap();
        let result = config.validate();

        assert!(result.is_err());
        match result {
            Err(PlurcastError::Config(ConfigError::MissingField(msg))) => {
                assert!(msg.contains("keys_file"));
            }
            _ => panic!("Expected MissingField error for keys_file"),
        }
    }

    #[test]
    fn test_validate_config_missing_nostr_relays() {
        let toml_content = r#"
[database]
path = "/tmp/test.db"

[nostr]
enabled = true
keys_file = "/tmp/keys"
relays = []
"#;

        let config: Config = toml::from_str(toml_content).unwrap();
        let result = config.validate();

        assert!(result.is_err());
        match result {
            Err(PlurcastError::Config(ConfigError::MissingField(msg))) => {
                assert!(msg.contains("relays"));
            }
            _ => panic!("Expected MissingField error for relays"),
        }
    }

    #[test]
    fn test_validate_config_missing_mastodon_instance() {
        let toml_content = r#"
[database]
path = "/tmp/test.db"

[mastodon]
enabled = true
instance = ""
token_file = "/tmp/token"
"#;

        let config: Config = toml::from_str(toml_content).unwrap();
        let result = config.validate();

        assert!(result.is_err());
        match result {
            Err(PlurcastError::Config(ConfigError::MissingField(msg))) => {
                assert!(msg.contains("instance"));
            }
            _ => panic!("Expected MissingField error for instance"),
        }
    }

    #[test]
    fn test_validate_config_missing_mastodon_token_file() {
        let toml_content = r#"
[database]
path = "/tmp/test.db"

[mastodon]
enabled = true
instance = "mastodon.social"
token_file = ""
"#;

        let config: Config = toml::from_str(toml_content).unwrap();
        let result = config.validate();

        assert!(result.is_err());
        match result {
            Err(PlurcastError::Config(ConfigError::MissingField(msg))) => {
                assert!(msg.contains("token_file"));
            }
            _ => panic!("Expected MissingField error for token_file"),
        }
    }

    #[test]
    fn test_validate_config_disabled_platforms_skip_validation() {
        let toml_content = r#"
[database]
path = "/tmp/test.db"

[nostr]
enabled = false
keys_file = ""
relays = []

[mastodon]
enabled = false
instance = ""
token_file = ""
"#;

        let config: Config = toml::from_str(toml_content).unwrap();
        let result = config.validate();

        // Should pass validation because all platforms are disabled
        assert!(result.is_ok());
    }

    #[test]
    fn test_path_expansion_nostr_keys_file() {
        let toml_content = r#"
[database]
path = "/tmp/test.db"

[nostr]
enabled = true
keys_file = "~/test/nostr.keys"
relays = ["wss://relay1.com"]
"#;

        let config: Config = toml::from_str(toml_content).unwrap();
        let nostr = config.nostr.unwrap();

        let expanded_path = nostr.expand_keys_file_path().unwrap();

        // Should not contain tilde after expansion
        assert!(!expanded_path.to_string_lossy().contains('~'));
    }

    #[test]
    fn test_path_expansion_mastodon_token_file() {
        let toml_content = r#"
[database]
path = "/tmp/test.db"

[mastodon]
enabled = true
instance = "mastodon.social"
token_file = "~/test/mastodon.token"
"#;

        let config: Config = toml::from_str(toml_content).unwrap();
        let mastodon = config.mastodon.unwrap();

        let expanded_path = mastodon.expand_token_file_path().unwrap();

        // Should not contain tilde after expansion
        assert!(!expanded_path.to_string_lossy().contains('~'));
    }

    #[test]
    fn test_platform_enable_disable_logic() {
        let toml_content = r#"
[database]
path = "/tmp/test.db"

[nostr]
enabled = true
keys_file = "/tmp/nostr.keys"
relays = ["wss://relay1.com"]

[mastodon]
enabled = false
instance = "mastodon.social"
token_file = "/tmp/mastodon.token"
"#;

        let config: Config = toml::from_str(toml_content).unwrap();

        // Nostr should be enabled
        assert!(config.nostr.as_ref().unwrap().enabled);

        // Mastodon should be disabled
        assert!(!config.mastodon.as_ref().unwrap().enabled);
    }

    #[test]
    fn test_default_config_includes_all_platforms() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        Config::create_default_config(&config_path).unwrap();

        let content = std::fs::read_to_string(&config_path).unwrap();

        // Should include comments for all platforms
        assert!(content.contains("Nostr"));
        assert!(content.contains("Mastodon"));
        assert!(content.contains("SSB"));

        // Should include helpful comments
        assert!(content.contains("Enable or disable"));
        assert!(content.contains("disabled by default"));
    }

    // ============================================================================
    // Task 4.4: Credential configuration tests
    // Requirements: 7.5, 10.1
    // ============================================================================

    #[test]
    fn test_parse_credential_config_keyring() {
        let toml_content = r#"
[database]
path = "/tmp/test.db"

[credentials]
storage = "keyring"
path = "~/.config/plurcast/credentials"
"#;

        let config: Config = toml::from_str(toml_content).unwrap();

        assert!(config.credentials.is_some());
        let credentials = config.credentials.unwrap();
        assert_eq!(
            credentials.storage,
            crate::credentials::StorageBackend::Keyring
        );
        assert_eq!(credentials.path, "~/.config/plurcast/credentials");
    }

    #[test]
    fn test_parse_credential_config_encrypted() {
        let toml_content = r#"
[database]
path = "/tmp/test.db"

[credentials]
storage = "encrypted"
path = "/custom/path/credentials"
"#;

        let config: Config = toml::from_str(toml_content).unwrap();

        assert!(config.credentials.is_some());
        let credentials = config.credentials.unwrap();
        assert_eq!(
            credentials.storage,
            crate::credentials::StorageBackend::Encrypted
        );
        assert_eq!(credentials.path, "/custom/path/credentials");
    }

    #[test]
    fn test_credential_config_defaults_when_missing() {
        let toml_content = r#"
[database]
path = "/tmp/test.db"
"#;

        let config: Config = toml::from_str(toml_content).unwrap();

        // Credentials section is optional, should be None
        assert!(config.credentials.is_none());
    }

    #[test]
    fn test_credential_config_default_values() {
        let toml_content = r#"
[database]
path = "/tmp/test.db"

[credentials]
"#;

        let config: Config = toml::from_str(toml_content).unwrap();

        assert!(config.credentials.is_some());
        let credentials = config.credentials.unwrap();

        // Should use default values
        assert_eq!(
            credentials.storage,
            crate::credentials::StorageBackend::Keyring
        );
        assert_eq!(credentials.path, "~/.config/plurcast/credentials");
    }

    #[test]
    fn test_invalid_storage_backend() {
        let toml_content = r#"
[database]
path = "/tmp/test.db"

[credentials]
storage = "invalid_backend"
"#;

        let result: std::result::Result<Config, toml::de::Error> = toml::from_str(toml_content);
        assert!(result.is_err());
    }

    #[test]
    fn test_credential_path_expansion() {
        let toml_content = r#"
[database]
path = "/tmp/test.db"

[credentials]
storage = "encrypted"
path = "~/test/credentials"
"#;

        let config: Config = toml::from_str(toml_content).unwrap();
        let credentials = config.credentials.unwrap();

        let expanded_path = credentials.expand_path();

        // Should not contain tilde after expansion
        assert!(!expanded_path.to_string_lossy().contains('~'));
    }

    #[test]
    fn test_credential_config_validation_success() {
        let toml_content = r#"
[database]
path = "/tmp/test.db"

[credentials]
storage = "keyring"
path = "~/.config/plurcast/credentials"
"#;

        let config: Config = toml::from_str(toml_content).unwrap();

        // Should validate successfully
        assert!(config.validate().is_ok());
    }

    #[test]
    #[serial_test::serial]
    fn test_master_password_from_env() {
        // Save original value if it exists
        let original_value = env::var("PLURCAST_MASTER_PASSWORD").ok();

        env::set_var("PLURCAST_MASTER_PASSWORD", "test-password-123");

        let toml_content = r#"
[database]
path = "/tmp/test.db"

[credentials]
storage = "encrypted"
"#;

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(&config_path, toml_content).unwrap();

        let config = Config::load_from_path(&config_path).unwrap();

        assert!(config.credentials.is_some());
        let credentials = config.credentials.unwrap();
        assert!(credentials.master_password.is_some());
        assert_eq!(credentials.master_password.unwrap(), "test-password-123");

        // Restore original value or remove
        match original_value {
            Some(val) => env::set_var("PLURCAST_MASTER_PASSWORD", val),
            None => env::remove_var("PLURCAST_MASTER_PASSWORD"),
        }
    }

    #[test]
    #[serial_test::serial]
    fn test_master_password_not_in_env() {
        // Save original value if it exists
        let original_value = env::var("PLURCAST_MASTER_PASSWORD").ok();

        // Ensure it's not set for this test
        env::remove_var("PLURCAST_MASTER_PASSWORD");

        let toml_content = r#"
[database]
path = "/tmp/test.db"

[credentials]
storage = "encrypted"
"#;

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(&config_path, toml_content).unwrap();

        let config = Config::load_from_path(&config_path).unwrap();

        assert!(config.credentials.is_some());
        let credentials = config.credentials.unwrap();
        assert!(credentials.master_password.is_none());

        // Restore original value if it existed
        if let Some(val) = original_value {
            env::set_var("PLURCAST_MASTER_PASSWORD", val);
        }
    }

    #[test]
    fn test_backward_compatibility_no_credentials_section() {
        let toml_content = r#"
[database]
path = "/tmp/test.db"

[nostr]
enabled = true
keys_file = "/tmp/nostr.keys"
relays = ["wss://relay1.com"]
"#;

        let config: Config = toml::from_str(toml_content).unwrap();

        // Should parse successfully without credentials section
        assert!(config.credentials.is_none());
        assert!(config.nostr.is_some());

        // Should validate successfully
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_default_config_includes_credentials_section() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        Config::create_default_config(&config_path).unwrap();

        let content = std::fs::read_to_string(&config_path).unwrap();

        // Should include credentials section
        assert!(content.contains("[credentials]"));
        assert!(content.contains("storage = \"keyring\""));
        assert!(content.contains("Credential storage configuration"));

        // Should include helpful comments about storage backends
        assert!(content.contains("keyring"));
        assert!(content.contains("encrypted"));
        assert!(content.contains("plain"));
    }

    #[test]
    fn test_credential_config_with_all_platforms() {
        let toml_content = r#"
[database]
path = "/tmp/test.db"

[credentials]
storage = "keyring"

[nostr]
enabled = true
keys_file = "/tmp/nostr.keys"
relays = ["wss://relay1.com"]

[mastodon]
enabled = true
instance = "mastodon.social"
token_file = "/tmp/mastodon.token"

"#;

        let config: Config = toml::from_str(toml_content).unwrap();

        // All sections should be present
        assert!(config.credentials.is_some());
        assert!(config.nostr.is_some());
        assert!(config.mastodon.is_some());

        // Should validate successfully
        assert!(config.validate().is_ok());
    }

    // ============================================================================
    // Task 2.2: SSB configuration parsing tests
    // Requirements: 2.1, 2.2, 2.3, 2.4, 2.5
    // ============================================================================

    #[test]
    fn test_parse_ssb_config_valid() {
        let toml_content = r#"
[database]
path = "/tmp/test.db"

[ssb]
enabled = true
feed_path = "~/.plurcast-ssb"
pubs = [
    "net:hermies.club:8008~shs:base64key1",
    "net:pub.example.com:8008~shs:base64key2"
]
"#;

        let config: Config = toml::from_str(toml_content).unwrap();

        assert!(config.ssb.is_some());
        let ssb = config.ssb.unwrap();
        assert!(ssb.enabled);
        assert_eq!(ssb.feed_path, "~/.plurcast-ssb");
        assert_eq!(ssb.pubs.len(), 2);
        assert_eq!(ssb.pubs[0], "net:hermies.club:8008~shs:base64key1");
        assert_eq!(ssb.pubs[1], "net:pub.example.com:8008~shs:base64key2");
    }

    #[test]
    fn test_parse_ssb_config_minimal() {
        let toml_content = r#"
[database]
path = "/tmp/test.db"

[ssb]
enabled = true
"#;

        let config: Config = toml::from_str(toml_content).unwrap();

        assert!(config.ssb.is_some());
        let ssb = config.ssb.unwrap();
        assert!(ssb.enabled);
        // Should use default feed_path
        assert_eq!(ssb.feed_path, "~/.plurcast-ssb");
        // Should have empty pubs list (local-only mode)
        assert_eq!(ssb.pubs.len(), 0);
    }

    #[test]
    fn test_parse_ssb_config_disabled() {
        let toml_content = r#"
[database]
path = "/tmp/test.db"

[ssb]
enabled = false
feed_path = "/tmp/ssb"
pubs = []
"#;

        let config: Config = toml::from_str(toml_content).unwrap();

        assert!(config.ssb.is_some());
        let ssb = config.ssb.unwrap();
        assert!(!ssb.enabled);
        assert_eq!(ssb.feed_path, "/tmp/ssb");
        assert_eq!(ssb.pubs.len(), 0);
    }

    #[test]
    fn test_parse_ssb_config_omitted() {
        let toml_content = r#"
[database]
path = "/tmp/test.db"

[nostr]
enabled = true
keys_file = "/tmp/nostr.keys"
relays = ["wss://relay1.com"]
"#;

        let config: Config = toml::from_str(toml_content).unwrap();

        // SSB section is optional, should be None
        assert!(config.ssb.is_none());
    }

    #[test]
    fn test_ssb_config_default_values() {
        let toml_content = r#"
[database]
path = "/tmp/test.db"

[ssb]
"#;

        let config: Config = toml::from_str(toml_content).unwrap();

        assert!(config.ssb.is_some());
        let ssb = config.ssb.unwrap();

        // Should use default values
        assert!(!ssb.enabled); // Default is false
        assert_eq!(ssb.feed_path, "~/.plurcast-ssb"); // Default feed_path
        assert_eq!(ssb.pubs.len(), 0); // Default empty pubs list
    }

    #[test]
    fn test_ssb_config_custom_feed_path() {
        let toml_content = r#"
[database]
path = "/tmp/test.db"

[ssb]
enabled = true
feed_path = "/custom/path/to/ssb"
"#;

        let config: Config = toml::from_str(toml_content).unwrap();

        assert!(config.ssb.is_some());
        let ssb = config.ssb.unwrap();
        assert_eq!(ssb.feed_path, "/custom/path/to/ssb");
    }

    #[test]
    fn test_ssb_config_empty_pubs_list() {
        let toml_content = r#"
[database]
path = "/tmp/test.db"

[ssb]
enabled = true
feed_path = "~/.plurcast-ssb"
pubs = []
"#;

        let config: Config = toml::from_str(toml_content).unwrap();

        assert!(config.ssb.is_some());
        let ssb = config.ssb.as_ref().unwrap();
        assert!(ssb.enabled);
        assert_eq!(ssb.pubs.len(), 0);

        // Should validate successfully (empty pubs is valid for local-only mode)
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_ssb_config_missing_feed_path() {
        let toml_content = r#"
[database]
path = "/tmp/test.db"

[ssb]
enabled = true
feed_path = ""
"#;

        let config: Config = toml::from_str(toml_content).unwrap();
        let result = config.validate();

        assert!(result.is_err());
        match result {
            Err(PlurcastError::Config(ConfigError::MissingField(msg))) => {
                assert!(msg.contains("feed_path"));
            }
            _ => panic!("Expected MissingField error for feed_path"),
        }
    }

    #[test]
    fn test_validate_ssb_config_disabled_skip_validation() {
        let toml_content = r#"
[database]
path = "/tmp/test.db"

[ssb]
enabled = false
feed_path = ""
"#;

        let config: Config = toml::from_str(toml_content).unwrap();
        let result = config.validate();

        // Should pass validation because SSB is disabled
        assert!(result.is_ok());
    }

    #[test]
    fn test_ssb_feed_path_expansion() {
        let toml_content = r#"
[database]
path = "/tmp/test.db"

[ssb]
enabled = true
feed_path = "~/test/ssb-feed"
"#;

        let config: Config = toml::from_str(toml_content).unwrap();
        let ssb = config.ssb.as_ref().unwrap();

        let expanded_path = ssb.expand_feed_path().unwrap();

        // Should not contain tilde after expansion
        assert!(!expanded_path.to_string_lossy().contains('~'));
    }

    #[test]
    fn test_parse_ssb_config_with_all_platforms() {
        let toml_content = r#"
[database]
path = "/tmp/test.db"

[nostr]
enabled = true
keys_file = "/tmp/nostr.keys"
relays = ["wss://relay1.com"]

[mastodon]
enabled = true
instance = "mastodon.social"
token_file = "/tmp/mastodon.token"

[ssb]
enabled = true
feed_path = "~/.plurcast-ssb"
pubs = ["net:hermies.club:8008~shs:key123"]
"#;

        let config: Config = toml::from_str(toml_content).unwrap();

        // All platforms should be present
        assert!(config.nostr.is_some());
        assert!(config.mastodon.is_some());
        assert!(config.ssb.is_some());

        // Verify SSB config
        let ssb = config.ssb.as_ref().unwrap();
        assert!(ssb.enabled);
        assert_eq!(ssb.feed_path, "~/.plurcast-ssb");
        assert_eq!(ssb.pubs.len(), 1);

        // Should validate successfully
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_default_config_includes_ssb_comment() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        Config::create_default_config(&config_path).unwrap();

        let content = std::fs::read_to_string(&config_path).unwrap();

        // Should include SSB section comment
        assert!(content.contains("SSB"));
        assert!(content.contains("Secure Scuttlebutt"));
        assert!(content.contains("feed_path"));
        assert!(content.contains("pubs"));
    }

    #[test]
    fn test_ssb_config_multiserver_address_format() {
        let toml_content = r#"
[database]
path = "/tmp/test.db"

[ssb]
enabled = true
feed_path = "~/.plurcast-ssb"
pubs = [
    "net:hermies.club:8008~shs:base64encodedkey==",
    "net:192.168.1.100:8008~shs:anotherkey123=="
]
"#;

        let config: Config = toml::from_str(toml_content).unwrap();

        assert!(config.ssb.is_some());
        let ssb = config.ssb.unwrap();
        assert_eq!(ssb.pubs.len(), 2);

        // Verify multiserver address format is preserved
        assert!(ssb.pubs[0].starts_with("net:"));
        assert!(ssb.pubs[0].contains("~shs:"));
        assert!(ssb.pubs[1].starts_with("net:"));
        assert!(ssb.pubs[1].contains("~shs:"));
    }
}
