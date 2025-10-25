//! Application module
//!
//! Contains the core application architecture:
//! - Actions: What can happen
//! - State: What is true right now
//! - Reducer: Pure function (State, Action) -> State
//!
//! This follows functional programming principles with immutable state
//! and pure functions for state transitions.

pub mod actions;
pub mod state;
pub mod reducer;
pub mod event;

// Re-export commonly used types
#[allow(unused_imports)] // Will be used in future modules
pub use actions::{Action, Screen, PlatformResult};
#[allow(unused_imports)] // Will be used in future modules
pub use state::{AppState, ComposerState, StatusBarState, UiConfig};
#[allow(unused_imports)] // Will be used in future modules
pub use reducer::reduce;
