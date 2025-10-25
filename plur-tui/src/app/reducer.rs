//! Pure reducer function for state transitions
//!
//! Following functional programming principles, the reducer is a pure function:
//! `(State, Action) -> State`
//!
//! The reducer has NO side effects - it only computes new state values.
//! All business logic and I/O happens outside the reducer.

use super::actions::{Action, Screen};
use super::state::{AppState, ComposerState};
use crossterm::event::{KeyCode, KeyModifiers};

/// Pure reducer function
///
/// Takes current state and an action, returns new state.
/// This function is completely pure - no I/O, no side effects.
///
/// # Purity Guarantees
///
/// - No network requests
/// - No file I/O
/// - No database calls
/// - No mutations (returns new state)
/// - Deterministic (same inputs -> same output)
pub fn reduce(state: AppState, action: Action) -> AppState {
    match action {
        // === UI Events ===
        Action::Key(key) => handle_key(state, key),
        Action::Mouse(_) => state, // TODO: Handle mouse in future
        Action::Tick => state, // No-op for now, used for animations
        Action::Resize(_, _) => state, // Terminal auto-handles resize
        
        // === Navigation ===
        Action::NavigateTo(screen) => AppState {
            current_screen: screen,
            ..state
        },
        
        Action::Quit => AppState {
            should_quit: true,
            ..state
        },
        
        Action::ToggleMouse => AppState {
            mouse_enabled: !state.mouse_enabled,
            ..state
        },
        
        Action::ShowHelp => AppState {
            help_visible: true,
            ..state
        },
        
        Action::HideHelp => AppState {
            help_visible: false,
            ..state
        },
        
        // === Composer Actions ===
        Action::ComposerInputChanged(content) => {
            let char_count = content.chars().count();
            AppState {
                composer: ComposerState {
                    content,
                    char_count,
                    // Validation will be triggered separately
                    ..state.composer
                },
                ..state
            }
        },
        
        Action::ComposerValidate => {
            // Validation is triggered but handled outside reducer
            // The result will come back as ComposerValidationResult
            state
        },
        
        Action::ComposerValidationResult { valid, errors, warnings, char_count } => {
            AppState {
                composer: ComposerState {
                    valid,
                    errors,
                    warnings,
                    char_count,
                    ..state.composer
                },
                ..state
            }
        },
        
        Action::ComposerPostRequested => {
            // Post request is handled outside reducer
            // State transitions happen via ComposerPostStarted
            state
        },
        
        Action::ComposerPostStarted => {
            AppState {
                composer: ComposerState {
                    posting: true,
                    progress: Vec::new(),
                    ..state.composer
                },
                ..state
            }
        },
        
        Action::ComposerPostProgress { platform, message } => {
            let mut progress = state.composer.progress.clone();
            // Update or add progress for this platform
            if let Some(pos) = progress.iter().position(|(p, _)| p == &platform) {
                progress[pos] = (platform, message);
            } else {
                progress.push((platform, message));
            }
            
            AppState {
                composer: ComposerState {
                    progress,
                    ..state.composer
                },
                ..state
            }
        },
        
        Action::ComposerPostSucceeded { post_id, results: _ } => {
            AppState {
                composer: ComposerState {
                    posting: false,
                    progress: Vec::new(),
                    last_post_id: Some(post_id),
                    ..state.composer
                },
                status: super::state::StatusBarState {
                    message: Some("Post successful!".to_string()),
                },
                ..state
            }
        },
        
        Action::ComposerPostFailed { error } => {
            AppState {
                composer: ComposerState {
                    posting: false,
                    progress: Vec::new(),
                    ..state.composer
                },
                error: Some(error),
                ..state
            }
        },
        
        Action::ComposerClear => {
            AppState {
                composer: ComposerState::default(),
                ..state
            }
        },
        
        // === Error Handling ===
        Action::ShowError(error) => {
            AppState {
                error: Some(error),
                ..state
            }
        },
        
        Action::DismissError => {
            AppState {
                error: None,
                ..state
            }
        },
        
        // === Status Bar ===
        Action::SetStatus(message) => {
            AppState {
                status: super::state::StatusBarState {
                    message: Some(message),
                },
                ..state
            }
        },
        
        Action::ClearStatus => {
            AppState {
                status: super::state::StatusBarState {
                    message: None,
                },
                ..state
            }
        },
    }
}

/// Handle keyboard input
///
/// Maps keys to high-level actions. This is where keybindings are defined.
fn handle_key(state: AppState, key: crossterm::event::KeyEvent) -> AppState {
    // Global keybindings (work everywhere)
    match (key.code, key.modifiers) {
        // Quit
        (KeyCode::Char('q'), KeyModifiers::NONE) if !state.composer.posting => {
            return reduce(state, Action::Quit);
        }
        
        // Help
        (KeyCode::F(1), _) => {
            let action = if state.help_visible { Action::HideHelp } else { Action::ShowHelp };
            return reduce(state, action);
        }
        
        // Navigation
        (KeyCode::F(2), _) => {
            return reduce(state, Action::NavigateTo(Screen::History));
        }
        (KeyCode::F(3), _) => {
            return reduce(state, Action::NavigateTo(Screen::Drafts));
        }
        
        // Toggle mouse
        (KeyCode::Char('m'), KeyModifiers::NONE) => {
            return reduce(state, Action::ToggleMouse);
        }
        
        // Dismiss error
        (KeyCode::Esc, _) if state.error.is_some() => {
            return reduce(state, Action::DismissError);
        }
        
        // Hide help
        (KeyCode::Esc, _) if state.help_visible => {
            return reduce(state, Action::HideHelp);
        }
        
        _ => {}
    }
    
    // Screen-specific keybindings
    match state.current_screen {
        Screen::Composer => handle_composer_key(state, key),
        Screen::History | Screen::Drafts => state, // TODO: Handle in M2/M3
    }
}

/// Handle composer-specific keys
fn handle_composer_key(state: AppState, key: crossterm::event::KeyEvent) -> AppState {
    match (key.code, key.modifiers) {
        // Post (Ctrl+S)
        (KeyCode::Char('s'), KeyModifiers::CONTROL) if state.can_post() => {
            reduce(state, Action::ComposerPostRequested)
        }
        
        // Clear (Ctrl+L) - after successful post
        (KeyCode::Char('l'), KeyModifiers::CONTROL) if state.composer.last_post_id.is_some() => {
            reduce(state, Action::ComposerClear)
        }
        
        _ => state,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_reducer_is_pure() {
        let state = AppState::new();
        let state_clone = state.clone();
        
        let action = Action::SetStatus("Test".to_string());
        let new_state = reduce(state_clone.clone(), action);
        
        // Original state unchanged
        assert!(state_clone.status.message.is_none());
        
        // New state has the change
        assert_eq!(new_state.status.message, Some("Test".to_string()));
    }
    
    #[test]
    fn test_quit_action() {
        let state = AppState::new();
        assert!(!state.should_quit);
        
        let new_state = reduce(state, Action::Quit);
        assert!(new_state.should_quit);
    }
    
    #[test]
    fn test_composer_validation_result() {
        let state = AppState::new();
        assert!(!state.composer.valid);
        
        let action = Action::ComposerValidationResult {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            char_count: 100,
        };
        
        let new_state = reduce(state, action);
        assert!(new_state.composer.valid);
        assert_eq!(new_state.composer.char_count, 100);
    }
    
    #[test]
    fn test_posting_flow() {
        let mut state = AppState::new();
        state.composer.valid = true;
        
        // Start posting
        state = reduce(state, Action::ComposerPostStarted);
        assert!(state.composer.posting);
        
        // Progress update
        state = reduce(state, Action::ComposerPostProgress {
            platform: "nostr".to_string(),
            message: "Connecting...".to_string(),
        });
        assert_eq!(state.composer.progress.len(), 1);
        
        // Success
        state = reduce(state, Action::ComposerPostSucceeded {
            post_id: "abc123".to_string(),
            results: Vec::new(),
        });
        assert!(!state.composer.posting);
        assert_eq!(state.composer.last_post_id, Some("abc123".to_string()));
    }
}
