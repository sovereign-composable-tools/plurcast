//! Test application initialization and boot sequence
//!
//! Verifies that the app initializes with correct defaults
//! based on environment variables.

use plur_tui::app::{AppState, Screen};

#[test]
fn test_app_initializes_to_composer_screen() {
    let state = AppState::new();
    
    assert_eq!(state.current_screen, Screen::Composer);
    assert!(!state.should_quit);
}

#[test]
fn test_mouse_disabled_by_default() {
    let state = AppState::new();
    
    assert!(!state.mouse_enabled);
}

#[test]
fn test_help_hidden_by_default() {
    let state = AppState::new();
    
    assert!(!state.help_visible);
}

#[test]
fn test_no_error_on_boot() {
    let state = AppState::new();
    
    assert!(state.error.is_none());
}

#[test]
fn test_composer_starts_empty_and_invalid() {
    let state = AppState::new();
    
    assert_eq!(state.composer.content, "");
    assert!(!state.composer.valid);
    assert!(!state.composer.errors.is_empty());
    assert_eq!(state.composer.char_count, 0);
}

#[test]
fn test_composer_not_posting_on_boot() {
    let state = AppState::new();
    
    assert!(!state.composer.posting);
    assert!(state.composer.progress.is_empty());
}

#[test]
fn test_colors_disabled_with_no_color_env() {
    std::env::set_var("NO_COLOR", "1");
    let state = AppState::new();
    std::env::remove_var("NO_COLOR");
    
    assert!(!state.config.colors_enabled);
}

#[test]
fn test_colors_disabled_with_plur_tui_no_color_env() {
    std::env::set_var("PLUR_TUI_NO_COLOR", "1");
    let state = AppState::new();
    std::env::remove_var("PLUR_TUI_NO_COLOR");
    
    assert!(!state.config.colors_enabled);
}

#[test]
fn test_tick_rate_from_env() {
    std::env::set_var("PLUR_TUI_TICK_MS", "250");
    let state = AppState::new();
    std::env::remove_var("PLUR_TUI_TICK_MS");
    
    assert_eq!(state.config.tick_rate_ms, 250);
}

#[test]
fn test_tick_rate_default_100ms() {
    // Ensure env var is not set
    std::env::remove_var("PLUR_TUI_TICK_MS");
    let state = AppState::new();
    
    assert_eq!(state.config.tick_rate_ms, 100);
}

#[test]
fn test_cannot_post_initially() {
    let state = AppState::new();
    
    // Empty content is invalid, so cannot post
    assert!(!state.can_post());
}
