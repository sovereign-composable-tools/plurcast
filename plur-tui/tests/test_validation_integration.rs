//! Integration tests for real-time validation
//!
//! Tests the integration between ValidationService and the composer state.

use plur_tui::app::{AppState, Action, reduce};
use plur_tui::services::{ServiceHandle, validation_summary};

/// Test that empty content is invalid
#[test]
fn test_empty_content_is_invalid() {
    let services = ServiceHandle::new().expect("Failed to create service handle");
    let state = AppState::new();
    
    // Validate empty content
    let validation = services.validate("", &["nostr", "mastodon"]);
    let (valid, errors, _warnings, char_count) = validation_summary(&validation, "");
    
    assert!(!valid, "Empty content should be invalid");
    assert_eq!(char_count, 0);
    assert!(!errors.is_empty(), "Should have validation errors");
    
    // Apply to state
    let state = reduce(state, Action::ComposerValidationResult {
        valid,
        errors,
        warnings: vec![],
        char_count,
    });
    
    assert!(!state.composer.valid);
    assert!(!state.can_post());
}

/// Test that valid content passes validation
#[test]
fn test_valid_content_passes() {
    let services = ServiceHandle::new().expect("Failed to create service handle");
    let state = AppState::new();
    
    let content = "Hello decentralized world!";
    
    // Validate content
    let validation = services.validate(content, &["nostr", "mastodon"]);
    let (valid, errors, _warnings, char_count) = validation_summary(&validation, content);
    
    assert!(valid, "Valid content should pass validation");
    assert_eq!(char_count, 26);
    assert!(errors.is_empty(), "Should have no validation errors");
    
    // Update state with content first
    let state = reduce(state, Action::ComposerInputChanged(content.to_string()));
    
    // Apply validation result
    let state = reduce(state, Action::ComposerValidationResult {
        valid,
        errors,
        warnings: vec![],
        char_count,
    });
    
    assert!(state.composer.valid);
    assert_eq!(state.composer.char_count, 26);
    assert!(state.can_post());
}

/// Test that unicode characters are counted correctly
#[test]
fn test_unicode_character_counting() {
    let services = ServiceHandle::new().expect("Failed to create service handle");
    
    let content = "Hello 👋 🌍 world!";
    
    // Validate content
    let validation = services.validate(content, &["nostr", "mastodon"]);
    let (_valid, _errors, _warnings, char_count) = validation_summary(&validation, content);
    
    // "Hello " = 6, "👋" = 1, " " = 1, "🌍" = 1, " world!" = 7
    // Total = 16 unicode characters
    assert_eq!(char_count, 16, "Unicode chars should be counted correctly");
}

/// Test that very long content is invalid
#[test]
fn test_very_long_content_is_invalid() {
    let services = ServiceHandle::new().expect("Failed to create service handle");
    
    // Create content that exceeds mastodon's 500 char limit
    let content = "a".repeat(600);
    
    // Validate content
    let validation = services.validate(&content, &["mastodon"]);
    let (valid, errors, _warnings, char_count) = validation_summary(&validation, &content);
    
    assert!(!valid, "Very long content should be invalid for mastodon");
    assert_eq!(char_count, 600);
    assert!(!errors.is_empty(), "Should have validation errors about length");
}

/// Test that different platforms have different limits
#[test]
fn test_platform_specific_limits() {
    let services = ServiceHandle::new().expect("Failed to create service handle");
    
    // Create content between mastodon (500) and bluesky (300) limits
    let content = "a".repeat(400);
    
    // Validate for mastodon (should pass)
    let validation_mastodon = services.validate(&content, &["mastodon"]);
    let (valid_mastodon, _, _, _) = validation_summary(&validation_mastodon, &content);
    
    // Validate for bluesky (should fail)
    let validation_bluesky = services.validate(&content, &["bluesky"]);
    let (valid_bluesky, _, _, _) = validation_summary(&validation_bluesky, &content);
    
    assert!(valid_mastodon, "400 chars should be valid for mastodon (500 limit)");
    assert!(!valid_bluesky, "400 chars should be invalid for bluesky (300 limit)");
}

/// Test that nostr has no hard character limit
#[test]
fn test_nostr_no_hard_limit() {
    let services = ServiceHandle::new().expect("Failed to create service handle");
    
    // Create very long content
    let content = "a".repeat(1000);
    
    // Validate for nostr (should pass but may have warnings)
    let validation = services.validate(&content, &["nostr"]);
    let (valid, errors, warnings, _) = validation_summary(&validation, &content);
    
    // Nostr has no hard limit, but may warn at 280
    assert!(valid, "Nostr should accept long content");
    assert!(errors.is_empty(), "Nostr should not error on length");
    
    // May have warnings about recommended length
    // (depends on ValidationService implementation details)
    if !warnings.is_empty() {
        println!("Nostr warnings for 1000 chars: {:?}", warnings);
    }
}

/// Test validation result updates state correctly
#[test]
fn test_validation_updates_state() {
    let mut state = AppState::new();
    
    // Start with invalid state
    assert!(!state.composer.valid);
    
    // Apply validation result for valid content
    state = reduce(state, Action::ComposerValidationResult {
        valid: true,
        errors: vec![],
        warnings: vec!["Long content".to_string()],
        char_count: 350,
    });
    
    assert!(state.composer.valid);
    assert_eq!(state.composer.char_count, 350);
    assert_eq!(state.composer.warnings, vec!["Long content"]);
    assert!(state.composer.errors.is_empty());
    
    // Apply validation result for invalid content
    state = reduce(state, Action::ComposerValidationResult {
        valid: false,
        errors: vec!["Too long".to_string()],
        warnings: vec![],
        char_count: 600,
    });
    
    assert!(!state.composer.valid);
    assert_eq!(state.composer.char_count, 600);
    assert_eq!(state.composer.errors, vec!["Too long"]);
    assert!(state.composer.warnings.is_empty());
}

/// Test that content change triggers validation in the reducer
#[test]
fn test_content_change_flow() {
    let services = ServiceHandle::new().expect("Failed to create service handle");
    let mut state = AppState::new();
    
    // Simulate user typing valid content
    let content = "Valid post content";
    state = reduce(state, Action::ComposerInputChanged(content.to_string()));
    
    // At this point, content is updated but validation hasn't run yet
    assert_eq!(state.composer.content, content);
    assert_eq!(state.composer.char_count, 18);
    
    // Now validate (this would happen in main loop)
    let validation = services.validate(content, &["nostr", "mastodon"]);
    let (valid, errors, warnings, char_count) = validation_summary(&validation, content);
    
    state = reduce(state, Action::ComposerValidationResult {
        valid,
        errors,
        warnings,
        char_count,
    });
    
    // Now state should be fully validated
    assert!(state.composer.valid);
    assert!(state.can_post());
}
