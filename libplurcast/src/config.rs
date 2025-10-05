//! Configuration management for Plurcast

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::{ConfigError, Result};

/// Main configuration structure for Plurcast
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Database configuration
    pub database: DatabaseConfig,
    
    /// Nostr platform configuration (optional)
    #[serde(default)]
    pub nostr: Option<NostrConfig>,
    
    /// Default settings
    #[serde(default)]
    pub defaults: DefaultsConfig,
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
        "wss://relay.damus.io".to_string(),
        "wss://nos.lol".to_string(),
        "wss://relay.nostr.band".to_string(),
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

impl Config {
    /// Load configuration from the default location
    ///
    /// If the configuration file doesn't exist, creates a default one
    pub fn load() -> Result<Self> {
        let config_path = resolve_config_path()?;
        
        // If config doesn't exist, create default
        if !config_path.exists() {
            Self::create_default_config(&config_path)?;
        }
        
        Self::load_from_path(&config_path)
    }

    /// Load configuration from a specific path
    ///
    /// Returns detailed error messages for parsing failures
    pub fn load_from_path(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::ReadError(std::io::Error::new(
                e.kind(),
                format!("Failed to read config from {}: {}", path.display(), e)
            )))?;
        
        let config: Config = toml::from_str(&content)
            .map_err(ConfigError::ParseError)?;
        
        Ok(config)
    }

    /// Create a default configuration file at the specified path
    ///
    /// Creates parent directories if they don't exist
    /// Sets file permissions to 600 (owner read/write only) for security
    pub fn create_default_config(path: &PathBuf) -> Result<()> {
        let default_config = Self::default_config();
        
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(ConfigError::ReadError)?;
        }
        
        // Serialize to TOML
        let toml_content = toml::to_string_pretty(&default_config)
            .map_err(|e| ConfigError::MissingField(format!("Failed to serialize default config: {}", e)))?;
        
        // Write to file
        std::fs::write(path, toml_content)
            .map_err(ConfigError::ReadError)?;
        
        // Set file permissions to 600 (owner read/write only) on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(path, permissions)
                .map_err(ConfigError::ReadError)?;
        }
        
        Ok(())
    }

    /// Get a default configuration structure
    ///
    /// This is used for creating new config files
    pub fn default_config() -> Self {
        Self {
            database: DatabaseConfig {
                path: "~/.local/share/plurcast/posts.db".to_string(),
            },
            nostr: Some(NostrConfig {
                enabled: true,
                keys_file: "~/.config/plurcast/nostr.keys".to_string(),
                relays: default_nostr_relays(),
            }),
            defaults: DefaultsConfig::default(),
        }
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
        assert_eq!(nostr.relays.len(), 3);
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
        assert_eq!(nostr.relays.len(), 3); // default_nostr_relays()
    }

    #[test]
    fn test_config_with_nested_paths() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path().join("nested").join("deep").join("config.toml");
        
        // Should create parent directories
        Config::create_default_config(&nested_path).unwrap();
        
        assert!(nested_path.exists());
        assert!(nested_path.parent().unwrap().exists());
    }
}
