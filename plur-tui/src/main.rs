//! plur-tui - Terminal UI for Plurcast
//!
//! Interactive terminal interface for posting to decentralized social platforms.
//! Provides a rich, SSH-friendly TUI with real-time validation and progress tracking.

use plur_tui::{
    error::Result,
    app::{AppState, reduce, event::EventHandler},
    terminal::{install_panic_hook, setup_terminal, restore_terminal},
    ui,
};

fn main() -> Result<()> {
    // Install panic hook to restore terminal on panic
    install_panic_hook();
    
    // Setup terminal
    let mut terminal = setup_terminal()?;
    
    // Run the application
    let result = run_app(&mut terminal);
    
    // Restore terminal
    restore_terminal(terminal)?;
    
    result
}

fn run_app(terminal: &mut plur_tui::terminal::Tui) -> Result<()> {
    // Initialize application state
    let mut state = AppState::new();
    
    // Create event handler with tick rate from config
    let event_handler = EventHandler::new(state.config.tick_rate_ms);
    
    // Main event loop
    loop {
        // Render UI
        terminal.draw(|frame| {
            ui::render(frame, &state);
        })?;
        
        // Handle events
        let tui_event = event_handler.next()?;
        let action = tui_event.into();
        
        // Update state through reducer
        state = reduce(state, action);
        
        // Check if we should quit
        if state.should_quit {
            break;
        }
    }
    
    Ok(())
}
