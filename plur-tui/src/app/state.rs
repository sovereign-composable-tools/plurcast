//! Application state
//!
//! Immutable state structure following functional programming principles.
//! All state transitions happen through the reducer (see `reducer.rs`).

use super::actions::Screen;

/// Root application state
///
/// This is the single source of truth for the entire application.
/// State transitions are pure functions that return new state values.
#[derive(Debug, Clone)]
pub struct AppState {
    /// Should the application quit?
    pub should_quit: bool,
    
    /// Current active screen
    pub current_screen: Screen,
    
    /// Mouse capture enabled?
    pub mouse_enabled: bool,
    
    /// Help overlay visible?
    pub help_visible: bool,
    
    /// Composer state
    pub composer: ComposerState,
    
    /// Status bar state
    pub status: StatusBarState,
    
    /// Error overlay state
    pub error: Option<String>,
    
    /// UI configuration
    pub config: UiConfig,
}

/// Composer screen state
#[derive(Debug, Clone)]
pub struct ComposerState {
    /// Current input content
    pub content: String,
    
    /// Is content valid?
    pub valid: bool,
    
    /// Validation errors
    pub errors: Vec<String>,
    
    /// Validation warnings  
    pub warnings: Vec<String>,
    
    /// Character count
    pub char_count: usize,
    
    /// Posting in progress?
    pub posting: bool,
    
    /// Posting progress messages per platform
    pub progress: Vec<(String, String)>, // (platform, message)
    
    /// Last post result (for showing success message)
    pub last_post_id: Option<String>,
}

/// Status bar state
#[derive(Debug, Clone)]
pub struct StatusBarState {
    /// Current status message
    pub message: Option<String>,
}

/// UI configuration
#[derive(Debug, Clone)]
pub struct UiConfig {
    /// Use colors?
    pub colors_enabled: bool,
    
    /// Use unicode symbols (false = ASCII fallback)
    pub unicode_enabled: bool,
    
    /// Tick rate in milliseconds
    pub tick_rate_ms: u64,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            should_quit: false,
            current_screen: Screen::Composer,
            mouse_enabled: false,
            help_visible: false,
            composer: ComposerState::default(),
            status: StatusBarState::default(),
            error: None,
            config: UiConfig::default(),
        }
    }
}

impl Default for ComposerState {
    fn default() -> Self {
        Self {
            content: String::new(),
            valid: false, // Empty content is invalid
            errors: vec!["Content cannot be empty".to_string()],
            warnings: Vec::new(),
            char_count: 0,
            posting: false,
            progress: Vec::new(),
            last_post_id: None,
        }
    }
}

impl Default for StatusBarState {
    fn default() -> Self {
        Self {
            message: None,
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        // Detect environment for sensible defaults
        let colors_enabled = !std::env::var("NO_COLOR").is_ok() 
            && !std::env::var("PLUR_TUI_NO_COLOR").is_ok();
        
        let unicode_enabled = colors_enabled; // Same heuristic for now
        
        let tick_rate_ms = std::env::var("PLUR_TUI_TICK_MS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(100);
        
        Self {
            colors_enabled,
            unicode_enabled,
            tick_rate_ms,
        }
    }
}

impl AppState {
    /// Create new application state with default values
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Check if posting is allowed (valid content, not currently posting)
    pub fn can_post(&self) -> bool {
        self.composer.valid && !self.composer.posting
    }
}
