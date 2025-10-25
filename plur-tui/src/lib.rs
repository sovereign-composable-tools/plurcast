//! plur-tui library
//!
//! Exports types and modules for testing and potential reuse.

pub mod error;
pub mod app;
pub mod terminal;
pub mod ui;

// Re-export commonly used types
pub use error::{TuiError, Result};
pub use app::{AppState, Action, Screen, reduce};
