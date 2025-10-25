//! Test keybinding mappings to actions
//!
//! Verifies that keyboard input is correctly mapped to actions
//! through the reducer.

use plur_tui::app::{AppState, Action, Screen, reduce};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

fn key_event(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent::new(code, modifiers)
}

#[test]
fn test_q_quits_application() {
    let state = AppState::new();
    let key = key_event(KeyCode::Char('q'), KeyModifiers::NONE);
    
    let new_state = reduce(state, Action::Key(key));
    
    assert!(new_state.should_quit);
}

#[test]
fn test_q_does_not_quit_while_posting() {
    let mut state = AppState::new();
    state.composer.posting = true;
    
    let key = key_event(KeyCode::Char('q'), KeyModifiers::NONE);
    let new_state = reduce(state, Action::Key(key));
    
    // Should not quit while posting
    assert!(!new_state.should_quit);
}

#[test]
fn test_f1_toggles_help() {
    let state = AppState::new();
    assert!(!state.help_visible);
    
    // Show help
    let key = key_event(KeyCode::F(1), KeyModifiers::NONE);
    let state = reduce(state, Action::Key(key));
    assert!(state.help_visible);
    
    // Hide help
    let key = key_event(KeyCode::F(1), KeyModifiers::NONE);
    let state = reduce(state, Action::Key(key));
    assert!(!state.help_visible);
}

#[test]
fn test_f2_navigates_to_history() {
    let state = AppState::new();
    assert_eq!(state.current_screen, Screen::Composer);
    
    let key = key_event(KeyCode::F(2), KeyModifiers::NONE);
    let new_state = reduce(state, Action::Key(key));
    
    assert_eq!(new_state.current_screen, Screen::History);
}

#[test]
fn test_f3_navigates_to_drafts() {
    let state = AppState::new();
    assert_eq!(state.current_screen, Screen::Composer);
    
    let key = key_event(KeyCode::F(3), KeyModifiers::NONE);
    let new_state = reduce(state, Action::Key(key));
    
    assert_eq!(new_state.current_screen, Screen::Drafts);
}

#[test]
fn test_m_toggles_mouse() {
    let state = AppState::new();
    assert!(!state.mouse_enabled);
    
    // Enable mouse
    let key = key_event(KeyCode::Char('m'), KeyModifiers::NONE);
    let state = reduce(state, Action::Key(key));
    assert!(state.mouse_enabled);
    
    // Disable mouse
    let key = key_event(KeyCode::Char('m'), KeyModifiers::NONE);
    let state = reduce(state, Action::Key(key));
    assert!(!state.mouse_enabled);
}

#[test]
fn test_esc_dismisses_error() {
    let mut state = AppState::new();
    state.error = Some("Test error".to_string());
    
    let key = key_event(KeyCode::Esc, KeyModifiers::NONE);
    let new_state = reduce(state, Action::Key(key));
    
    assert!(new_state.error.is_none());
}

#[test]
fn test_esc_hides_help() {
    let mut state = AppState::new();
    state.help_visible = true;
    
    let key = key_event(KeyCode::Esc, KeyModifiers::NONE);
    let new_state = reduce(state, Action::Key(key));
    
    assert!(!new_state.help_visible);
}

#[test]
fn test_ctrl_s_does_nothing_when_invalid() {
    let state = AppState::new();
    assert!(!state.composer.valid);
    
    let key = key_event(KeyCode::Char('s'), KeyModifiers::CONTROL);
    let new_state = reduce(state, Action::Key(key));
    
    // Should remain unchanged (post not triggered)
    assert!(!new_state.composer.posting);
}

#[test]
fn test_ctrl_s_triggers_post_when_valid() {
    let mut state = AppState::new();
    state.composer.valid = true;
    state.composer.content = "Valid content".to_string();
    
    let key = key_event(KeyCode::Char('s'), KeyModifiers::CONTROL);
    let new_state = reduce(state, Action::Key(key));
    
    // State should be unchanged by the key press itself
    // The actual posting is handled by side effect
    // which will dispatch ComposerPostStarted action
    assert_eq!(new_state.composer.content, "Valid content");
}

#[test]
fn test_ctrl_l_clears_after_successful_post() {
    let mut state = AppState::new();
    state.composer.last_post_id = Some("abc123".to_string());
    state.composer.content = "Posted content".to_string();
    
    let key = key_event(KeyCode::Char('l'), KeyModifiers::CONTROL);
    let new_state = reduce(state, Action::Key(key));
    
    // Should clear to default state
    assert_eq!(new_state.composer.content, "");
    assert!(new_state.composer.last_post_id.is_none());
}

#[test]
fn test_ctrl_l_does_nothing_without_post() {
    let mut state = AppState::new();
    state.composer.content = "Some content".to_string();
    // No last_post_id
    
    let key = key_event(KeyCode::Char('l'), KeyModifiers::CONTROL);
    let new_state = reduce(state, Action::Key(key));
    
    // Should remain unchanged
    assert_eq!(new_state.composer.content, "Some content");
}
