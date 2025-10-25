//! Event handling infrastructure
//!
//! Merges UI events (keyboard, mouse, resize, tick) with service events
//! from the EventBus into a unified action stream.

use std::time::Duration;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};
use crate::app::Action;

/// TUI events that can be dispatched
#[derive(Debug, Clone)]
pub enum TuiEvent {
    /// Keyboard input
    Key(KeyEvent),
    
    /// Mouse input (when enabled)
    Mouse(MouseEvent),
    
    /// Terminal resize
    Resize(u16, u16),
    
    /// Periodic tick for animations/progress
    Tick,
    
    /// Service event (from EventBus)
    /// TODO: Wire up EventBus events
    ServiceEvent(String),
}

impl From<TuiEvent> for Action {
    fn from(event: TuiEvent) -> Self {
        match event {
            TuiEvent::Key(key) => Action::Key(key),
            TuiEvent::Mouse(mouse) => Action::Mouse(mouse),
            TuiEvent::Resize(w, h) => Action::Resize(w, h),
            TuiEvent::Tick => Action::Tick,
            TuiEvent::ServiceEvent(_) => {
                // TODO: Parse service events and convert to appropriate actions
                Action::Tick // Placeholder
            }
        }
    }
}

/// Event handler that polls for terminal events
pub struct EventHandler {
    tick_rate: Duration,
}

impl EventHandler {
    /// Create a new event handler with the specified tick rate
    pub fn new(tick_rate_ms: u64) -> Self {
        Self {
            tick_rate: Duration::from_millis(tick_rate_ms),
        }
    }
    
    /// Poll for the next event, blocking up to tick_rate duration
    ///
    /// Returns None if no event occurred within tick_rate (which triggers a Tick)
    pub fn next(&self) -> std::io::Result<TuiEvent> {
        // Poll for events with timeout
        if event::poll(self.tick_rate)? {
            match event::read()? {
                CrosstermEvent::Key(key) => Ok(TuiEvent::Key(key)),
                CrosstermEvent::Mouse(mouse) => Ok(TuiEvent::Mouse(mouse)),
                CrosstermEvent::Resize(w, h) => Ok(TuiEvent::Resize(w, h)),
                _ => Ok(TuiEvent::Tick), // Ignore other events
            }
        } else {
            // Timeout - generate tick
            Ok(TuiEvent::Tick)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_event_handler_creation() {
        let handler = EventHandler::new(100);
        assert_eq!(handler.tick_rate, Duration::from_millis(100));
    }
    
    #[test]
    fn test_custom_tick_rate() {
        let handler = EventHandler::new(250);
        assert_eq!(handler.tick_rate, Duration::from_millis(250));
    }
}
