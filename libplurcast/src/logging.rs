//! Centralized logging configuration for all Plurcast binaries
//!
//! Provides consistent logging setup with support for:
//! - Text, JSON, and pretty-printed output
//! - Environment variable configuration
//! - Per-module log level filtering
//!
//! # Examples
//!
//! ```no_run
//! use libplurcast::logging::{LoggingConfig, LogFormat};
//!
//! // Initialize with JSON format
//! let config = LoggingConfig::new(LogFormat::Json, "info".to_string(), false);
//! config.init();
//!
//! // Or use default settings (respects env vars)
//! libplurcast::logging::init_default();
//! ```

use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    /// Human-readable text output (no colors, for piping)
    Text,
    /// Machine-parseable JSON (one JSON object per line)
    Json,
    /// Pretty-printed with colors (for development)
    Pretty,
}

impl FromStr for LogFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "text" => Ok(LogFormat::Text),
            "json" => Ok(LogFormat::Json),
            "pretty" => Ok(LogFormat::Pretty),
            _ => Err(format!(
                "Invalid log format: '{}'. Valid options: text, json, pretty",
                s
            )),
        }
    }
}

impl std::fmt::Display for LogFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogFormat::Text => write!(f, "text"),
            LogFormat::Json => write!(f, "json"),
            LogFormat::Pretty => write!(f, "pretty"),
        }
    }
}

/// Configuration for logging initialization
pub struct LoggingConfig {
    pub format: LogFormat,
    pub level: String,
    pub verbose: bool,
}

impl LoggingConfig {
    /// Create a new logging configuration
    ///
    /// # Arguments
    ///
    /// * `format` - Log output format (text, json, or pretty)
    /// * `level` - Minimum log level (error, warn, info, debug, trace)
    /// * `verbose` - If true, defaults to debug level
    pub fn new(format: LogFormat, level: String, verbose: bool) -> Self {
        Self {
            format,
            level,
            verbose,
        }
    }

    /// Initialize logging with the configured settings
    ///
    /// This should be called once at the start of your program.
    ///
    /// # Panics
    ///
    /// Panics if the logging subscriber has already been initialized
    pub fn init(&self) {
        use tracing_subscriber::EnvFilter;

        // Determine the filter based on verbose flag and level
        let filter = if self.verbose {
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug"))
        } else {
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&self.level))
        };

        match self.format {
            LogFormat::Json => {
                // JSON output for machine parsing (production/monitoring)
                // Outputs one JSON object per line to stderr
                tracing_subscriber::fmt()
                    .json()
                    .with_env_filter(filter)
                    .with_writer(std::io::stderr)
                    .with_current_span(true)
                    .with_span_list(true)
                    .flatten_event(true)
                    .with_target(true)
                    .with_line_number(true)
                    .with_file(true)
                    .init();
            }
            LogFormat::Pretty => {
                // Pretty output with colors for development
                tracing_subscriber::fmt()
                    .pretty()
                    .with_env_filter(filter)
                    .with_writer(std::io::stderr)
                    .with_target(true)
                    .with_line_number(true)
                    .with_file(true)
                    .init();
            }
            LogFormat::Text => {
                // Plain text output for piping/basic usage
                // Less verbose for end users
                tracing_subscriber::fmt()
                    .with_env_filter(filter)
                    .with_writer(std::io::stderr)
                    .with_target(false)
                    .with_level(true)
                    .init();
            }
        }
    }
}

/// Initialize logging with default settings
///
/// Respects `PLURCAST_LOG_FORMAT` and `PLURCAST_LOG_LEVEL` environment variables.
/// Falls back to text format with info level if not set.
///
/// # Examples
///
/// ```bash
/// # Use JSON logging
/// export PLURCAST_LOG_FORMAT=json
/// export PLURCAST_LOG_LEVEL=debug
/// plur-post "Hello world"
/// ```
pub fn init_default() {
    let format = std::env::var("PLURCAST_LOG_FORMAT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(LogFormat::Text);

    let level = std::env::var("PLURCAST_LOG_LEVEL").unwrap_or_else(|_| "info".to_string());

    LoggingConfig::new(format, level, false).init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_format_from_str() {
        assert_eq!("text".parse::<LogFormat>().unwrap(), LogFormat::Text);
        assert_eq!("json".parse::<LogFormat>().unwrap(), LogFormat::Json);
        assert_eq!("pretty".parse::<LogFormat>().unwrap(), LogFormat::Pretty);

        // Case insensitive
        assert_eq!("TEXT".parse::<LogFormat>().unwrap(), LogFormat::Text);
        assert_eq!("Json".parse::<LogFormat>().unwrap(), LogFormat::Json);
        assert_eq!("PRETTY".parse::<LogFormat>().unwrap(), LogFormat::Pretty);
    }

    #[test]
    fn test_log_format_from_str_invalid() {
        let result = "invalid".parse::<LogFormat>();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Invalid log format: 'invalid'"));
    }

    #[test]
    fn test_log_format_display() {
        assert_eq!(LogFormat::Text.to_string(), "text");
        assert_eq!(LogFormat::Json.to_string(), "json");
        assert_eq!(LogFormat::Pretty.to_string(), "pretty");
    }

    #[test]
    fn test_logging_config_new() {
        let config = LoggingConfig::new(LogFormat::Json, "debug".to_string(), true);
        assert_eq!(config.format, LogFormat::Json);
        assert_eq!(config.level, "debug");
        assert!(config.verbose);
    }
}
