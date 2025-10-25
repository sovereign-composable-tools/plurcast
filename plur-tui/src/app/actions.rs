//! Actions for the reducer pattern
//!
//! All state transitions are triggered by actions. This module defines
//! all possible actions that can modify application state.

use crossterm::event::{KeyEvent, MouseEvent};

/// Actions that trigger state transitions
///
/// Following functional programming principles, actions are immutable
/// data structures that describe what should happen. The reducer
/// (see `reducer.rs`) is responsible for applying actions to state.
#[derive(Debug, Clone)]
pub enum Action {
    // === UI Events ===
    /// Keyboard input event
    Key(KeyEvent),
    
    /// Mouse input event (when enabled)
    Mouse(MouseEvent),
    
    /// Periodic tick for animations/progress updates
    Tick,
    
    /// Terminal resize event
    Resize(u16, u16),
    
    // === Navigation ===
    /// Navigate to a different screen
    NavigateTo(Screen),
    
    /// Quit the application
    Quit,
    
    /// Toggle mouse capture on/off
    ToggleMouse,
    
    /// Show help overlay
    ShowHelp,
    
    /// Hide help overlay
    HideHelp,
    
    // === Composer Actions ===
    /// Input content changed in composer
    ComposerInputChanged(String),
    
    /// Trigger validation of current content
    ComposerValidate,
    
    /// Validation completed with results
    ComposerValidationResult {
        valid: bool,
        errors: Vec<String>,
        warnings: Vec<String>,
        char_count: usize,
    },
    
    /// User requested to post
    ComposerPostRequested,
    
    /// Posting started
    ComposerPostStarted,
    
    /// Posting progress update
    ComposerPostProgress {
        platform: String,
        message: String,
    },
    
    /// Posting completed successfully
    ComposerPostSucceeded {
        post_id: String,
        results: Vec<PlatformResult>,
    },
    
    /// Posting failed
    ComposerPostFailed {
        error: String,
    },
    
    /// Clear composer after successful post
    ComposerClear,
    
    // === Error Handling ===
    /// Show error overlay
    ShowError(String),
    
    /// Dismiss error overlay
    DismissError,
    
    // === Status Bar ===
    /// Update status message
    SetStatus(String),
    
    /// Clear status message
    ClearStatus,
}

/// Screen/View identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// Composer screen (MVP)
    Composer,
    
    /// History browser (M2)
    History,
    
    /// Draft manager (M3)
    Drafts,
}

/// Platform posting result
#[derive(Debug, Clone)]
pub struct PlatformResult {
    pub platform: String,
    pub success: bool,
    pub post_id: Option<String>,
    pub error: Option<String>,
}
