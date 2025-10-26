//! plur-tui - Terminal UI for Plurcast
//!
//! Interactive terminal interface for posting to decentralized social platforms.
//! Provides a rich, SSH-friendly TUI with real-time validation and progress tracking.

use plur_tui::{
    error::Result,
    app::{AppState, reduce, event::EventHandler, Action},
    terminal::{install_panic_hook, setup_terminal, restore_terminal},
    ui,
    services::{ServiceHandle, validation_summary},
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
    
    // Initialize service layer
    let services = ServiceHandle::new()?;
    
    // Track active posting operations (post_id -> progress channel)
    let mut posting_rx: Option<crossbeam_channel::Receiver<libplurcast::service::events::Event>> = None;
    
    // Create textarea for composer (stateful widget)
    let mut textarea = tui_textarea::TextArea::default();
    textarea.set_placeholder_text("Type your post here... (Ctrl+S to post, F1 for help, q to quit)");
    
    // Create event handler with tick rate from config
    let event_handler = EventHandler::new(state.config.tick_rate_ms);
    
    // Main event loop
    loop {
        // Update textarea styling before render
        let border_color = if state.composer.posting {
            ratatui::style::Color::Yellow
        } else if state.composer.valid {
            ratatui::style::Color::Green
        } else {
            ratatui::style::Color::Red
        };
        
        textarea.set_block(
            ratatui::widgets::Block::default()
                .title(" Composer ")
                .borders(ratatui::widgets::Borders::ALL)
                .border_style(ratatui::style::Style::default().fg(border_color))
        );
        
        // Render UI (passing textarea reference)
        terminal.draw(|frame| {
            ui::render(frame, &state, &textarea);
        })?;
        
        // Handle events
        let tui_event = event_handler.next()?;
        
        // Special handling for key events in composer
        let action = match tui_event {
            plur_tui::app::event::TuiEvent::Key(key) => {
                use crossterm::event::{KeyCode, KeyModifiers};
                
                // Let textarea handle input when in composer and not in overlay
                let in_composer = state.current_screen == plur_tui::app::Screen::Composer;
                let no_overlay = !state.help_visible && state.error.is_none();
                let not_posting = !state.composer.posting;
                
                // Check if this is a global hotkey
                let is_global_key = matches!(
                    (key.code, key.modifiers),
                    (KeyCode::Char('q'), KeyModifiers::NONE) |
                    (KeyCode::F(_), _) |
                    (KeyCode::Char('s'), KeyModifiers::CONTROL) |
                    (KeyCode::Char('l'), KeyModifiers::CONTROL) |
                    (KeyCode::Char('m'), KeyModifiers::NONE) |
                    (KeyCode::Esc, _)
                );
                
                if in_composer && no_overlay && not_posting && !is_global_key {
                    // Let textarea handle the input
                    textarea.input(key);
                    
                    // Sync content to state and trigger validation
                    let content = textarea.lines().join("\n");
                    plur_tui::app::Action::ComposerInputChanged(content)
                } else {
                    // Pass to reducer as normal
                    plur_tui::app::Action::Key(key)
                }
            }
            other => other.into(),
        };
        
        // Update state through reducer
        state = reduce(state, action.clone());
        
        // Check for service events (posting progress)
        if let Some(ref rx) = posting_rx {
            while let Ok(event) = rx.try_recv() {
                use libplurcast::service::events::Event;
                
                let action = match event {
                    Event::PostingStarted { .. } => {
                        // Already handled by ComposerPostStarted
                        continue;
                    }
                    Event::PostingProgress { platform, status, .. } => {
                        Action::ComposerPostProgress {
                            platform,
                            message: status,
                        }
                    }
                    Event::PostingCompleted { post_id, results } => {
                        // Convert to TUI PlatformResult
                        let tui_results: Vec<plur_tui::app::actions::PlatformResult> = results.into_iter()
                            .map(|r| plur_tui::app::actions::PlatformResult {
                                platform: r.platform,
                                success: r.success,
                                post_id: r.post_id,
                                error: r.error,
                            })
                            .collect();
                        
                        Action::ComposerPostSucceeded {
                            post_id,
                            results: tui_results,
                        }
                    }
                    Event::PostingFailed { error, .. } => {
                        Action::ComposerPostFailed { error }
                    }
                };
                
                state = reduce(state, action);
            }
        }
        
        // Perform side effects based on action
        match action {
            Action::ComposerInputChanged(ref content) => {
                // Validate content in real-time
                // TODO: Make platforms configurable, for now use nostr and mastodon
                let platforms = vec!["nostr", "mastodon"];
                let validation = services.validate(content, &platforms);
                let (valid, errors, warnings, char_count) = validation_summary(&validation, content);
                
                // Apply validation result
                state = reduce(state, Action::ComposerValidationResult {
                    valid,
                    errors,
                    warnings,
                    char_count,
                });
            }
            Action::ComposerPostRequested => {
                // Start posting if valid
                if state.can_post() {
                    // Mark posting as started
                    state = reduce(state, Action::ComposerPostStarted);
                    
                    // Spawn posting task
                    // TODO: Make platforms configurable
                    let platforms = vec!["nostr".to_string(), "mastodon".to_string()];
                    match services.post(state.composer.content.clone(), platforms) {
                        Ok((_post_id, rx)) => {
                            posting_rx = Some(rx);
                        }
                        Err(e) => {
                            state = reduce(state, Action::ComposerPostFailed {
                                error: format!("Failed to start posting: {}", e),
                            });
                        }
                    }
                }
            }
            _ => {}
        }
        
        // Sync textarea with state if content was cleared
        if state.composer.content.is_empty() && !textarea.is_empty() {
            textarea = tui_textarea::TextArea::default();
            textarea.set_placeholder_text("Type your post here... (Ctrl+S to post, F1 for help, q to quit)");
        }
        
        // Check if we should quit
        if state.should_quit {
            break;
        }
    }
    
    Ok(())
}
