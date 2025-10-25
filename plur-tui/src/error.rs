//! Error types for plur-tui
//!
//! Provides TUI-specific error types that wrap service layer errors
//! and terminal/IO errors for unified error handling.

use thiserror::Error;

/// TUI-specific errors
#[derive(Error, Debug)]
pub enum TuiError {
    /// Service layer error
    #[error("Service error: {0}")]
    Service(#[from] libplurcast::PlurcastError),

    /// Terminal/IO error
    #[error("Terminal error: {0}")]
    Terminal(#[from] std::io::Error),

    /// Application state error
    #[error("Application error: {0}")]
    Application(String),

    /// Event handling error
    #[error("Event error: {0}")]
    Event(String),
}

/// Result type for TUI operations
pub type Result<T> = std::result::Result<T, TuiError>;
