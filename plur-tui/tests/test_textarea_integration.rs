//! Integration tests for tui-textarea widget rendering
//!
//! Verifies that the textarea widget integrates correctly with our UI rendering pipeline.

use plur_tui::app::{AppState, Action, reduce};
use tui_textarea::TextArea;

/// Test that textarea can be created and synced with composer state
#[test]
fn test_textarea_creation_syncs_with_state() {
    let state = AppState::new();
    let textarea = TextArea::default();
    
    // Verify initial sync
    assert!(textarea.is_empty());
    assert_eq!(state.composer.content, "");
    assert_eq!(state.composer.char_count, 0);
}

/// Test that textarea content can be synced to state via reducer
#[test]
fn test_textarea_content_syncs_to_state() {
    let state = AppState::new();
    let mut textarea = TextArea::default();
    
    // Simulate typing in textarea
    textarea.insert_str("Hello");
    textarea.insert_char('\n');
    textarea.insert_str("World");
    
    let content = textarea.lines().join("\n");
    assert_eq!(content, "Hello\nWorld");
    
    // Sync to state via action
    let state = reduce(state, Action::ComposerInputChanged(content));
    
    assert_eq!(state.composer.content, "Hello\nWorld");
    assert_eq!(state.composer.char_count, 11); // "Hello\nWorld" = 11 chars
}

/// Test that textarea can be cleared when state is cleared
#[test]
fn test_textarea_clears_with_state() {
    let mut state = AppState::new();
    let mut textarea = TextArea::default();
    
    // Add content
    textarea.insert_str("Test content");
    let content = textarea.lines().join("\n");
    state = reduce(state, Action::ComposerInputChanged(content));
    
    assert!(!state.composer.content.is_empty());
    assert!(!textarea.is_empty());
    
    // Clear state
    state = reduce(state, Action::ComposerClear);
    
    assert_eq!(state.composer.content, "");
    
    // In actual app, we'd recreate textarea when content is empty
    if state.composer.content.is_empty() && !textarea.is_empty() {
        textarea = TextArea::default();
    }
    
    assert!(textarea.is_empty());
}

/// Test that textarea preserves multiline content
#[test]
fn test_textarea_preserves_multiline() {
    let state = AppState::new();
    let mut textarea = TextArea::default();
    
    // Insert multiple lines
    let lines = vec!["Line 1", "Line 2", "Line 3"];
    for (i, line) in lines.iter().enumerate() {
        if i > 0 {
            textarea.insert_char('\n');
        }
        textarea.insert_str(line);
    }
    
    let content = textarea.lines().join("\n");
    let state = reduce(state, Action::ComposerInputChanged(content.clone()));
    
    assert_eq!(state.composer.content, "Line 1\nLine 2\nLine 3");
    assert_eq!(content, "Line 1\nLine 2\nLine 3");
    assert_eq!(textarea.lines().len(), 3);
}

/// Test that textarea handles unicode correctly
#[test]
fn test_textarea_handles_unicode() {
    let state = AppState::new();
    let mut textarea = TextArea::default();
    
    // Insert unicode characters
    textarea.insert_str("Hello 👋 🌍");
    
    let content = textarea.lines().join("\n");
    let state = reduce(state, Action::ComposerInputChanged(content));
    
    // Unicode characters should be counted correctly
    assert_eq!(state.composer.char_count, 9); // "Hello 👋 🌍" = 9 unicode chars (H-e-l-l-o-space-emoji-space-emoji)
    assert!(state.composer.content.contains("👋"));
    assert!(state.composer.content.contains("🌍"));
}

/// Test that textarea state can be reconstructed from AppState
#[test]
fn test_textarea_reconstruction_from_state() {
    let state = AppState::new();
    
    // Simulate adding content through state
    let content = "Reconstructed\nContent";
    let state = reduce(state, Action::ComposerInputChanged(content.to_string()));
    
    // Reconstruct textarea from state
    let lines: Vec<String> = state.composer.content.lines().map(|s| s.to_string()).collect();
    let textarea = TextArea::from(lines);
    
    assert_eq!(textarea.lines().len(), 2);
    assert_eq!(textarea.lines()[0], "Reconstructed");
    assert_eq!(textarea.lines()[1], "Content");
}
