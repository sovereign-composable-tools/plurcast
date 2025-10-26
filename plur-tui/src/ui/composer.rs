//! Composer widget with tui-textarea integration
//!
//! This module provides a stateful text editor for composing posts.
//! It wraps tui-textarea to provide multi-line editing with cursor support.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders},
};
use tui_textarea::TextArea;
use crate::app::AppState;

/// Stateful composer widget
///
/// This wraps tui-textarea and syncs its content with AppState.
pub struct ComposerWidget<'a> {
    textarea: TextArea<'a>,
}

impl<'a> ComposerWidget<'a> {
    /// Create a new composer widget from current state
    pub fn new(state: &AppState) -> Self {
        let mut textarea = TextArea::default();
        
        // Set initial content if any
        if !state.composer.content.is_empty() {
            // Split content into lines for textarea
            let lines: Vec<String> = state.composer.content.lines().map(|s| s.to_string()).collect();
            textarea = TextArea::from(lines);
        }
        
        // Set block style based on validation state
        let block = Block::default()
            .title(" Composer ")
            .borders(Borders::ALL)
            .border_style(if state.composer.posting {
                Style::default().fg(Color::Yellow)
            } else if state.composer.valid {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            });
        
        textarea.set_block(block);
        
        // Set placeholder text
        if state.composer.content.is_empty() {
            textarea.set_placeholder_text("Type your post here... (Ctrl+S to post, F1 for help, q to quit)");
        }
        
        // Disable input if posting
        if state.composer.posting {
            // textarea has no built-in disabled state, we handle this in event handling
        }
        
        Self { textarea }
    }
    
    /// Render the composer
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_widget(&self.textarea, area);
    }
    
    /// Get the current content
    pub fn content(&self) -> String {
        self.textarea.lines().join("\n")
    }
    
    /// Handle input (returns whether the input was consumed)
    pub fn input(&mut self, input: crossterm::event::KeyEvent) -> bool {
        use crossterm::event::{KeyCode, KeyModifiers};
        
        // Don't handle special keys that should be handled globally
        match (input.code, input.modifiers) {
            // Let these through to global handler
            (KeyCode::Char('q'), KeyModifiers::NONE) => false,
            (KeyCode::F(_), _) => false,
            (KeyCode::Char('s'), KeyModifiers::CONTROL) => false,
            (KeyCode::Char('l'), KeyModifiers::CONTROL) => false,
            (KeyCode::Char('m'), KeyModifiers::NONE) => false,
            (KeyCode::Esc, _) => false,
            
            // Handle in textarea
            _ => {
                self.textarea.input(input);
                true
            }
        }
    }
}
