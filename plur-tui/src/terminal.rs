//! Terminal management
//!
//! Handles terminal setup, teardown, and restoration.
//! Ensures terminal is properly restored even on panic.

use std::io::{self, Stdout};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use crate::error::Result;

/// Terminal type alias
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

/// Setup terminal for TUI rendering
///
/// - Enables raw mode (no line buffering, no echo)
/// - Enters alternate screen (restore on exit)
/// - Enables mouse capture if requested
pub fn setup_terminal() -> Result<Tui> {
    // Enable raw mode for TUI
    enable_raw_mode()?;
    
    // Enter alternate screen
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    
    // Create terminal backend
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    
    Ok(terminal)
}

/// Restore terminal to normal mode
///
/// - Leaves alternate screen
/// - Disables raw mode
/// - Restores cursor
pub fn restore_terminal(mut terminal: Tui) -> Result<()> {
    // Leave alternate screen
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    
    // Disable raw mode
    disable_raw_mode()?;
    
    Ok(())
}

/// Install panic hook to restore terminal on panic
///
/// This ensures the terminal is properly restored even if the application panics.
pub fn install_panic_hook() {
    let original_hook = std::panic::take_hook();
    
    std::panic::set_hook(Box::new(move |panic_info| {
        // Try to restore terminal
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        
        // Call original panic hook
        original_hook(panic_info);
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_panic_hook_installs() {
        // Just verify it doesn't panic
        install_panic_hook();
    }
}
