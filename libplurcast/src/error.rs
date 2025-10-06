//! Error types for Plurcast

use thiserror::Error;

pub type Result<T> = std::result::Result<T, PlurcastError>;

#[derive(Error, Debug)]
pub enum PlurcastError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("Database error: {0}")]
    Database(#[from] DbError),

    #[error("Platform error: {0}")]
    Platform(#[from] PlatformError),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

impl PlurcastError {
    /// Returns the appropriate exit code for this error
    pub fn exit_code(&self) -> i32 {
        match self {
            PlurcastError::InvalidInput(_) => 3,
            PlurcastError::Platform(PlatformError::Authentication(_)) => 2,
            PlurcastError::Platform(_) => 1,
            PlurcastError::Config(_) => 1,
            PlurcastError::Database(_) => 1,
        }
    }
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    ReadError(#[from] std::io::Error),

    #[error("Failed to parse config: {0}")]
    ParseError(#[from] toml::de::Error),

    #[error("Missing required field: {0}")]
    MissingField(String),
}

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database operation failed: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("Migration failed: {0}")]
    MigrationError(#[from] sqlx::migrate::MigrateError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Error, Debug, Clone)]
pub enum PlatformError {
    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Content validation failed: {0}")]
    Validation(String),

    #[error("Posting failed: {0}")]
    Posting(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_code_invalid_input() {
        let error = PlurcastError::InvalidInput("Empty content".to_string());
        assert_eq!(error.exit_code(), 3);
    }

    #[test]
    fn test_exit_code_authentication_error() {
        let platform_error = PlatformError::Authentication("Missing keys".to_string());
        let error = PlurcastError::Platform(platform_error);
        assert_eq!(error.exit_code(), 2);
    }

    #[test]
    fn test_exit_code_posting_error() {
        let platform_error = PlatformError::Posting("Network timeout".to_string());
        let error = PlurcastError::Platform(platform_error);
        assert_eq!(error.exit_code(), 1);
    }

    #[test]
    fn test_exit_code_validation_error() {
        let platform_error = PlatformError::Validation("Content too long".to_string());
        let error = PlurcastError::Platform(platform_error);
        assert_eq!(error.exit_code(), 1);
    }

    #[test]
    fn test_exit_code_network_error() {
        let platform_error = PlatformError::Network("Connection refused".to_string());
        let error = PlurcastError::Platform(platform_error);
        assert_eq!(error.exit_code(), 1);
    }

    #[test]
    fn test_exit_code_config_error() {
        let config_error = ConfigError::MissingField("database.path".to_string());
        let error = PlurcastError::Config(config_error);
        assert_eq!(error.exit_code(), 1);
    }

    #[test]
    fn test_exit_code_database_error() {
        let db_error = DbError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "File not found",
        ));
        let error = PlurcastError::Database(db_error);
        assert_eq!(error.exit_code(), 1);
    }

    #[test]
    fn test_error_message_formatting_invalid_input() {
        let error = PlurcastError::InvalidInput("Content cannot be empty".to_string());
        let message = format!("{}", error);
        assert_eq!(message, "Invalid input: Content cannot be empty");
    }

    #[test]
    fn test_error_message_formatting_authentication() {
        let platform_error = PlatformError::Authentication("Keys file not found".to_string());
        let error = PlurcastError::Platform(platform_error);
        let message = format!("{}", error);
        assert_eq!(message, "Platform error: Authentication failed: Keys file not found");
    }

    #[test]
    fn test_error_message_formatting_posting() {
        let platform_error = PlatformError::Posting("Failed to connect to relay".to_string());
        let error = PlurcastError::Platform(platform_error);
        let message = format!("{}", error);
        assert_eq!(message, "Platform error: Posting failed: Failed to connect to relay");
    }

    #[test]
    fn test_error_message_formatting_validation() {
        let platform_error = PlatformError::Validation("Content exceeds limit".to_string());
        let error = PlurcastError::Platform(platform_error);
        let message = format!("{}", error);
        assert_eq!(message, "Platform error: Content validation failed: Content exceeds limit");
    }

    #[test]
    fn test_error_message_formatting_config() {
        let config_error = ConfigError::MissingField("nostr.keys_file".to_string());
        let error = PlurcastError::Config(config_error);
        let message = format!("{}", error);
        assert_eq!(message, "Configuration error: Missing required field: nostr.keys_file");
    }

    #[test]
    fn test_error_conversion_from_config_error() {
        let config_error = ConfigError::MissingField("test".to_string());
        let plurcast_error: PlurcastError = config_error.into();
        
        match plurcast_error {
            PlurcastError::Config(_) => {
                // Success - correct conversion
            }
            _ => panic!("Expected PlurcastError::Config"),
        }
    }

    #[test]
    fn test_error_conversion_from_db_error() {
        let db_error = DbError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "test",
        ));
        let plurcast_error: PlurcastError = db_error.into();
        
        match plurcast_error {
            PlurcastError::Database(_) => {
                // Success - correct conversion
            }
            _ => panic!("Expected PlurcastError::Database"),
        }
    }

    #[test]
    fn test_error_conversion_from_platform_error() {
        let platform_error = PlatformError::Posting("test".to_string());
        let plurcast_error: PlurcastError = platform_error.into();
        
        match plurcast_error {
            PlurcastError::Platform(_) => {
                // Success - correct conversion
            }
            _ => panic!("Expected PlurcastError::Platform"),
        }
    }

    #[test]
    fn test_authentication_error_detection_in_exit_code() {
        // Test that authentication errors specifically return exit code 2
        let auth_error = PlurcastError::Platform(PlatformError::Authentication(
            "Invalid credentials".to_string(),
        ));
        assert_eq!(auth_error.exit_code(), 2);

        // Test that other platform errors return exit code 1
        let posting_error = PlurcastError::Platform(PlatformError::Posting(
            "Failed to post".to_string(),
        ));
        assert_eq!(posting_error.exit_code(), 1);

        let validation_error = PlurcastError::Platform(PlatformError::Validation(
            "Invalid content".to_string(),
        ));
        assert_eq!(validation_error.exit_code(), 1);

        let network_error = PlurcastError::Platform(PlatformError::Network(
            "Connection failed".to_string(),
        ));
        assert_eq!(network_error.exit_code(), 1);
    }

    #[test]
    fn test_config_error_read_error_formatting() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let config_error = ConfigError::ReadError(io_error);
        let message = format!("{}", config_error);
        assert!(message.contains("Failed to read config file"));
    }

    #[test]
    fn test_platform_error_variants() {
        // Test all PlatformError variants format correctly
        let auth = PlatformError::Authentication("test auth".to_string());
        assert_eq!(format!("{}", auth), "Authentication failed: test auth");

        let validation = PlatformError::Validation("test validation".to_string());
        assert_eq!(format!("{}", validation), "Content validation failed: test validation");

        let posting = PlatformError::Posting("test posting".to_string());
        assert_eq!(format!("{}", posting), "Posting failed: test posting");

        let network = PlatformError::Network("test network".to_string());
        assert_eq!(format!("{}", network), "Network error: test network");
    }

    #[test]
    fn test_result_type_alias() {
        // Test that our Result type alias works correctly
        fn returns_ok() -> Result<String> {
            Ok("success".to_string())
        }

        fn returns_err() -> Result<String> {
            Err(PlurcastError::InvalidInput("test".to_string()))
        }

        assert!(returns_ok().is_ok());
        assert!(returns_err().is_err());
    }

    // ============================================================================
    // Task 9.3: Additional error handling tests
    // Requirements: 10.3
    // ============================================================================

    #[test]
    fn test_rate_limit_error_exit_code() {
        let platform_error = PlatformError::RateLimit("Rate limit exceeded".to_string());
        let error = PlurcastError::Platform(platform_error);
        assert_eq!(error.exit_code(), 1);
    }

    #[test]
    fn test_rate_limit_error_formatting() {
        let platform_error = PlatformError::RateLimit("Too many requests".to_string());
        let error = PlurcastError::Platform(platform_error);
        let message = format!("{}", error);
        assert_eq!(message, "Platform error: Rate limit exceeded: Too many requests");
    }

    #[test]
    fn test_platform_error_with_context() {
        // Test that platform errors include context
        let auth_error = PlatformError::Authentication(
            "Nostr authentication failed (load keys): Failed to read keys file".to_string()
        );
        let message = format!("{}", auth_error);
        assert!(message.contains("Nostr"));
        assert!(message.contains("load keys"));
        assert!(message.contains("Failed to read keys file"));
    }

    #[test]
    fn test_platform_error_with_suggestion() {
        // Test that platform errors include suggestions
        let validation_error = PlatformError::Validation(
            "Content exceeds limit. Suggestion: Shorten your content.".to_string()
        );
        let message = format!("{}", validation_error);
        assert!(message.contains("Suggestion"));
    }

    #[test]
    fn test_all_platform_error_variants_have_exit_codes() {
        // Ensure all PlatformError variants map to appropriate exit codes
        let auth = PlurcastError::Platform(PlatformError::Authentication("test".to_string()));
        assert_eq!(auth.exit_code(), 2, "Authentication errors should exit with code 2");

        let validation = PlurcastError::Platform(PlatformError::Validation("test".to_string()));
        assert_eq!(validation.exit_code(), 1, "Validation errors should exit with code 1");

        let posting = PlurcastError::Platform(PlatformError::Posting("test".to_string()));
        assert_eq!(posting.exit_code(), 1, "Posting errors should exit with code 1");

        let network = PlurcastError::Platform(PlatformError::Network("test".to_string()));
        assert_eq!(network.exit_code(), 1, "Network errors should exit with code 1");

        let rate_limit = PlurcastError::Platform(PlatformError::RateLimit("test".to_string()));
        assert_eq!(rate_limit.exit_code(), 1, "Rate limit errors should exit with code 1");
    }

    #[test]
    fn test_error_message_includes_platform_name() {
        // Test that error messages from different platforms include platform name
        let nostr_error = PlatformError::Authentication(
            "Nostr authentication failed: Invalid key".to_string()
        );
        assert!(format!("{}", nostr_error).contains("Nostr"));

        let mastodon_error = PlatformError::Authentication(
            "Mastodon authentication failed: Invalid token".to_string()
        );
        assert!(format!("{}", mastodon_error).contains("Mastodon"));

        let bluesky_error = PlatformError::Authentication(
            "Bluesky authentication failed: Invalid credentials".to_string()
        );
        assert!(format!("{}", bluesky_error).contains("Bluesky"));
    }

    #[test]
    fn test_error_message_includes_operation_context() {
        // Test that error messages include the operation that failed
        let error_with_context = PlatformError::Posting(
            "Nostr posting failed (publish): Connection timeout".to_string()
        );
        let message = format!("{}", error_with_context);
        assert!(message.contains("publish"));
        assert!(message.contains("Posting failed"));
    }

    #[test]
    fn test_network_error_formatting() {
        let network_error = PlatformError::Network(
            "Connection refused: Unable to reach relay".to_string()
        );
        let error = PlurcastError::Platform(network_error);
        let message = format!("{}", error);
        assert!(message.contains("Network error"));
        assert!(message.contains("Connection refused"));
    }

    #[test]
    fn test_validation_error_with_details() {
        let validation_error = PlatformError::Validation(
            "Content exceeds Mastodon's 500 character limit (current: 600 characters)".to_string()
        );
        let message = format!("{}", validation_error);
        assert!(message.contains("500"));
        assert!(message.contains("600"));
        assert!(message.contains("character limit"));
    }

    #[test]
    fn test_authentication_error_with_remediation() {
        let auth_error = PlatformError::Authentication(
            "Invalid token. Suggestion: Verify your OAuth token is valid and has not expired.".to_string()
        );
        let message = format!("{}", auth_error);
        assert!(message.contains("Suggestion"));
        assert!(message.contains("OAuth token"));
    }

    #[test]
    fn test_error_chain_preserves_context() {
        // Test that converting through error types preserves context
        let platform_error = PlatformError::Posting(
            "Nostr posting failed (publish): Network timeout".to_string()
        );
        let plurcast_error: PlurcastError = platform_error.into();
        
        let message = format!("{}", plurcast_error);
        assert!(message.contains("Nostr"));
        assert!(message.contains("publish"));
        assert!(message.contains("Network timeout"));
    }

    #[test]
    fn test_config_error_types() {
        // Test different config error types
        let missing_field = ConfigError::MissingField("database.path".to_string());
        assert!(format!("{}", missing_field).contains("Missing required field"));
        assert!(format!("{}", missing_field).contains("database.path"));
    }

    #[test]
    fn test_exit_code_consistency() {
        // Verify exit code consistency across error types
        
        // All authentication errors should be exit code 2
        let auth1 = PlurcastError::Platform(PlatformError::Authentication("test1".to_string()));
        let auth2 = PlurcastError::Platform(PlatformError::Authentication("test2".to_string()));
        assert_eq!(auth1.exit_code(), auth2.exit_code());
        assert_eq!(auth1.exit_code(), 2);

        // All non-auth platform errors should be exit code 1
        let posting = PlurcastError::Platform(PlatformError::Posting("test".to_string()));
        let network = PlurcastError::Platform(PlatformError::Network("test".to_string()));
        let validation = PlurcastError::Platform(PlatformError::Validation("test".to_string()));
        let rate_limit = PlurcastError::Platform(PlatformError::RateLimit("test".to_string()));
        
        assert_eq!(posting.exit_code(), 1);
        assert_eq!(network.exit_code(), 1);
        assert_eq!(validation.exit_code(), 1);
        assert_eq!(rate_limit.exit_code(), 1);

        // Invalid input should be exit code 3
        let invalid = PlurcastError::InvalidInput("test".to_string());
        assert_eq!(invalid.exit_code(), 3);
    }

    #[test]
    fn test_platform_error_clone() {
        // Test that PlatformError can be cloned (required for retry logic)
        let original = PlatformError::Network("Connection failed".to_string());
        let cloned = original.clone();
        
        assert_eq!(format!("{}", original), format!("{}", cloned));
    }

    #[test]
    fn test_error_debug_output() {
        // Test that debug output is useful for logging
        let error = PlurcastError::Platform(PlatformError::Posting(
            "Failed to post".to_string()
        ));
        
        let debug_output = format!("{:?}", error);
        assert!(debug_output.contains("Platform"));
        assert!(debug_output.contains("Posting"));
    }
}
