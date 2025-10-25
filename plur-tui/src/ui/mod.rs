//! UI rendering
//!
//! Pure rendering functions that transform state into terminal frames.
//! Following FP principles: render functions have no side effects.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use crate::app::{AppState, Screen};

/// Render the application UI
///
/// Pure function: Takes state, returns nothing, but draws to frame.
/// This is the main rendering entry point.
pub fn render(frame: &mut Frame, state: &AppState) {
    let area = frame.size();
    
    // Render based on current screen
    match state.current_screen {
        Screen::Composer => render_composer(frame, area, state),
        Screen::History => render_placeholder(frame, area, "History", "Coming in M2"),
        Screen::Drafts => render_placeholder(frame, area, "Drafts", "Coming in M3"),
    }
    
    // Render help overlay if visible
    if state.help_visible {
        render_help_overlay(frame, area, state);
    }
    
    // Render error overlay if present
    if let Some(ref error) = state.error {
        render_error_overlay(frame, area, error, state);
    }
}

/// Render the composer screen
fn render_composer(frame: &mut Frame, area: Rect, state: &AppState) {
    // Create layout: editor + status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),      // Editor area
            Constraint::Length(3),   // Status bar
        ])
        .split(area);
    
    // Editor block
    let editor_block = Block::default()
        .title(" Composer ")
        .borders(Borders::ALL)
        .border_style(if state.composer.posting {
            Style::default().fg(Color::Yellow)
        } else if state.composer.valid {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Red)
        });
    
    // Content
    let content_text = if state.composer.content.is_empty() {
        if state.config.colors_enabled {
            "Type your post here... (Ctrl+S to post, F1 for help, q to quit)"
        } else {
            "Type your post here... (Ctrl+S to post, F1 for help, q to quit)"
        }
    } else {
        &state.composer.content
    };
    
    let content = Paragraph::new(content_text)
        .block(editor_block)
        .wrap(Wrap { trim: false });
    
    frame.render_widget(content, chunks[0]);
    
    // Status bar
    render_status_bar(frame, chunks[1], state);
}

/// Render status bar with validation status and hints
fn render_status_bar(frame: &mut Frame, area: Rect, state: &AppState) {
    let status_text = if state.composer.posting {
        // Show progress
        let progress_lines: Vec<Line> = state.composer.progress.iter()
            .map(|(platform, msg)| {
                Line::from(vec![
                    Span::styled(platform, Style::default().fg(Color::Cyan)),
                    Span::raw(": "),
                    Span::raw(msg),
                ])
            })
            .collect();
        
        Paragraph::new(progress_lines)
            .block(Block::default().borders(Borders::ALL).title(" Posting... "))
            .style(Style::default().fg(Color::Yellow))
    } else if let Some(ref post_id) = state.composer.last_post_id {
        // Show success message
        let text = format!("✓ Posted! ID: {} | Press Ctrl+L to clear", post_id);
        Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title(" Success "))
            .style(Style::default().fg(Color::Green))
    } else {
        // Show validation status and hints
        let status_symbol = if state.composer.valid { "✓" } else { "✗" };
        let char_info = format!("{} chars", state.composer.char_count);
        
        let validation_info = if state.composer.errors.is_empty() {
            "Valid".to_string()
        } else {
            state.composer.errors.join(", ")
        };
        
        let hints = if state.can_post() {
            "Ctrl+S: Post | F1: Help | q: Quit"
        } else {
            "F1: Help | q: Quit"
        };
        
        let lines = vec![
            Line::from(vec![
                Span::styled(status_symbol, if state.composer.valid {
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                }),
                Span::raw(" | "),
                Span::raw(char_info),
                Span::raw(" | "),
                Span::raw(validation_info),
            ]),
            Line::from(Span::styled(hints, Style::default().fg(Color::Gray))),
        ];
        
        Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL))
    };
    
    frame.render_widget(status_text, area);
}

/// Render placeholder screen for unimplemented features
fn render_placeholder(frame: &mut Frame, area: Rect, title: &str, message: &str) {
    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL);
    
    let text = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(message, Style::default().fg(Color::Yellow))),
        Line::from(""),
        Line::from("Press F1 for help, q to quit"),
    ])
    .block(block)
    .alignment(Alignment::Center);
    
    frame.render_widget(text, area);
}

/// Render help overlay
fn render_help_overlay(frame: &mut Frame, area: Rect, _state: &AppState) {
    // Center the help box
    let popup_area = centered_rect(60, 60, area);
    
    let help_text = vec![
        Line::from(Span::styled("Keyboard Shortcuts", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from("Global:"),
        Line::from("  q        - Quit"),
        Line::from("  F1       - Toggle help"),
        Line::from("  F2       - History (M2)"),
        Line::from("  F3       - Drafts (M3)"),
        Line::from("  m        - Toggle mouse"),
        Line::from(""),
        Line::from("Composer:"),
        Line::from("  Ctrl+S   - Post (when valid)"),
        Line::from("  Ctrl+L   - Clear (after post)"),
        Line::from("  Esc      - Dismiss overlays"),
        Line::from(""),
        Line::from("Press Esc or F1 to close"),
    ];
    
    let help = Paragraph::new(help_text)
        .block(Block::default()
            .title(" Help ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)))
        .wrap(Wrap { trim: false });
    
    frame.render_widget(ratatui::widgets::Clear, popup_area); // Clear background
    frame.render_widget(help, popup_area);
}

/// Render error overlay
fn render_error_overlay(frame: &mut Frame, area: Rect, error: &str, _state: &AppState) {
    let popup_area = centered_rect(70, 30, area);
    
    let error_text = vec![
        Line::from(Span::styled("Error", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(error),
        Line::from(""),
        Line::from("Press Esc to dismiss"),
    ];
    
    let error_widget = Paragraph::new(error_text)
        .block(Block::default()
            .title(" Error ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red)))
        .wrap(Wrap { trim: false })
        .alignment(Alignment::Center);
    
    frame.render_widget(ratatui::widgets::Clear, popup_area);
    frame.render_widget(error_widget, popup_area);
}

/// Helper to create centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
