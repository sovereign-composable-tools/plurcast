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

#[derive(Error, Debug)]
pub enum PlatformError {
    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Content validation failed: {0}")]
    Validation(String),

    #[error("Posting failed: {0}")]
    Posting(String),

    #[error("Network error: {0}")]
    Network(String),
}
