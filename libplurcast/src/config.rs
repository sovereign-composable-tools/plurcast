//! Configuration management for Plurcast

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::{ConfigError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub nostr: Option<NostrConfig>,
    pub defaults: DefaultsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NostrConfig {
    pub enabled: bool,
    pub keys_file: String,
    pub relays: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultsConfig {
    pub platforms: Vec<String>,
}

impl Config {
    /// Load configuration from the default location
    pub fn load() -> Result<Self> {
        let config_path = resolve_config_path()?;
        Self::load_from_path(&config_path)
    }

    /// Load configuration from a specific path
    pub fn load_from_path(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(ConfigError::ReadError)?;
        let config: Config = toml::from_str(&content)
            .map_err(ConfigError::ParseError)?;
        Ok(config)
    }

    /// Create a default configuration
    pub fn default_config() -> Self {
        Self {
            database: DatabaseConfig {
                path: "~/.local/share/plurcast/posts.db".to_string(),
            },
            nostr: Some(NostrConfig {
                enabled: true,
                keys_file: "~/.config/plurcast/nostr.keys".to_string(),
                relays: vec![
                    "wss://relay.damus.io".to_string(),
                    "wss://nos.lol".to_string(),
                    "wss://relay.nostr.band".to_string(),
                ],
            }),
            defaults: DefaultsConfig {
                platforms: vec!["nostr".to_string()],
            },
        }
    }
}

/// Resolve the configuration file path following XDG Base Directory spec
pub fn resolve_config_path() -> Result<PathBuf> {
    if let Ok(path) = std::env::var("PLURCAST_CONFIG") {
        return Ok(PathBuf::from(shellexpand::tilde(&path).to_string()));
    }

    let config_dir = dirs::config_dir()
        .ok_or_else(|| ConfigError::MissingField("config directory".to_string()))?;

    Ok(config_dir.join("plurcast").join("config.toml"))
}

/// Resolve the data directory path following XDG Base Directory spec
pub fn resolve_data_path() -> Result<PathBuf> {
    let data_dir = dirs::data_dir()
        .ok_or_else(|| ConfigError::MissingField("data directory".to_string()))?;

    Ok(data_dir.join("plurcast"))
}
