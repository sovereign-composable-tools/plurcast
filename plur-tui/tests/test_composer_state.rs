//! Test composer state transitions
//!
//! Verifies that composer state updates correctly through
//! input changes, validation, and posting workflows.

use plur_tui::app::{AppState, Action, reduce};

#[test]
fn test_input_change_updates_content_and_char_count() {
    let state = AppState::new();
    
    let content = "Hello world!".to_string();
    let new_state = reduce(state, Action::ComposerInputChanged(content.clone()));
    
    assert_eq!(new_state.composer.content, content);
    assert_eq!(new_state.composer.char_count, 12);
}

#[test]
fn test_input_change_with_unicode() {
    let state = AppState::new();
    
    let content = "Hello 世界 🚀".to_string();
    let new_state = reduce(state, Action::ComposerInputChanged(content.clone()));
    
    assert_eq!(new_state.composer.content, content);
    // Counts characters, not bytes
    assert_eq!(new_state.composer.char_count, 10); // "Hello " (6) + "世界" (2) + " " (1) + "🚀" (1)
}

#[test]
fn test_validation_result_updates_validity() {
    let state = AppState::new();
    
    let action = Action::ComposerValidationResult {
        valid: true,
        errors: Vec::new(),
        warnings: Vec::new(),
        char_count: 100,
    };
    
    let new_state = reduce(state, action);
    
    assert!(new_state.composer.valid);
    assert!(new_state.composer.errors.is_empty());
    assert_eq!(new_state.composer.char_count, 100);
}

#[test]
fn test_validation_result_with_errors() {
    let state = AppState::new();
    
    let action = Action::ComposerValidationResult {
        valid: false,
        errors: vec!["Content too long".to_string()],
        warnings: vec!["Consider shortening".to_string()],
        char_count: 600,
    };
    
    let new_state = reduce(state, action);
    
    assert!(!new_state.composer.valid);
    assert_eq!(new_state.composer.errors, vec!["Content too long"]);
    assert_eq!(new_state.composer.warnings, vec!["Consider shortening"]);
}

#[test]
fn test_post_started_sets_posting_flag() {
    let state = AppState::new();
    
    let new_state = reduce(state, Action::ComposerPostStarted);
    
    assert!(new_state.composer.posting);
    assert!(new_state.composer.progress.is_empty());
}

#[test]
fn test_post_progress_updates() {
    let state = AppState::new();
    let state = reduce(state, Action::ComposerPostStarted);
    
    let state = reduce(state, Action::ComposerPostProgress {
        platform: "nostr".to_string(),
        message: "Connecting...".to_string(),
    });
    
    assert_eq!(state.composer.progress.len(), 1);
    assert_eq!(state.composer.progress[0].0, "nostr");
    assert_eq!(state.composer.progress[0].1, "Connecting...");
}

#[test]
fn test_post_progress_multiple_platforms() {
    let state = AppState::new();
    let state = reduce(state, Action::ComposerPostStarted);
    
    let state = reduce(state, Action::ComposerPostProgress {
        platform: "nostr".to_string(),
        message: "Posting...".to_string(),
    });
    
    let state = reduce(state, Action::ComposerPostProgress {
        platform: "mastodon".to_string(),
        message: "Authenticating...".to_string(),
    });
    
    assert_eq!(state.composer.progress.len(), 2);
    assert!(state.composer.progress.iter().any(|(p, _)| p == "nostr"));
    assert!(state.composer.progress.iter().any(|(p, _)| p == "mastodon"));
}

#[test]
fn test_post_progress_updates_existing_platform() {
    let state = AppState::new();
    let state = reduce(state, Action::ComposerPostStarted);
    
    let state = reduce(state, Action::ComposerPostProgress {
        platform: "nostr".to_string(),
        message: "Connecting...".to_string(),
    });
    
    let state = reduce(state, Action::ComposerPostProgress {
        platform: "nostr".to_string(),
        message: "Posting...".to_string(),
    });
    
    // Should only have one entry for nostr, updated
    assert_eq!(state.composer.progress.len(), 1);
    assert_eq!(state.composer.progress[0].1, "Posting...");
}

#[test]
fn test_post_succeeded_clears_posting_flag() {
    let mut state = AppState::new();
    state.composer.posting = true;
    state.composer.progress = vec![
        ("nostr".to_string(), "Done".to_string()),
    ];
    
    let new_state = reduce(state, Action::ComposerPostSucceeded {
        post_id: "abc123".to_string(),
        results: Vec::new(),
    });
    
    assert!(!new_state.composer.posting);
    assert!(new_state.composer.progress.is_empty());
    assert_eq!(new_state.composer.last_post_id, Some("abc123".to_string()));
    assert_eq!(new_state.status.message, Some("Post successful!".to_string()));
}

#[test]
fn test_post_failed_clears_posting_and_shows_error() {
    let mut state = AppState::new();
    state.composer.posting = true;
    state.composer.content = "Test content".to_string();
    
    let error_msg = "Network error".to_string();
    let new_state = reduce(state, Action::ComposerPostFailed {
        error: error_msg.clone(),
    });
    
    assert!(!new_state.composer.posting);
    assert!(new_state.composer.progress.is_empty());
    assert_eq!(new_state.error, Some(error_msg));
    // Content should be preserved for retry
    assert_eq!(new_state.composer.content, "Test content");
}

#[test]
fn test_composer_clear_resets_to_default() {
    let mut state = AppState::new();
    state.composer.content = "Some content".to_string();
    state.composer.valid = true;
    state.composer.char_count = 100;
    state.composer.last_post_id = Some("abc".to_string());
    
    let new_state = reduce(state, Action::ComposerClear);
    
    assert_eq!(new_state.composer.content, "");
    assert!(!new_state.composer.valid);
    assert_eq!(new_state.composer.char_count, 0);
    assert!(new_state.composer.last_post_id.is_none());
}

#[test]
fn test_can_post_requires_valid_and_not_posting() {
    let mut state = AppState::new();
    
    // Invalid content
    assert!(!state.can_post());
    
    // Valid content
    state.composer.valid = true;
    assert!(state.can_post());
    
    // Valid but posting
    state.composer.posting = true;
    assert!(!state.can_post());
}
