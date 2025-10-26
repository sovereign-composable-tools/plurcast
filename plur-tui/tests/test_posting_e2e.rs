//! End-to-end integration tests for posting flow
//!
//! Tests the complete posting workflow from user action to completion,
//! including progress tracking, error handling, and state management.

use plur_tui::app::{AppState, Action, reduce};
use plur_tui::services::ServiceHandle;
use std::time::Duration;
use std::thread;

/// Helper to wait for a condition with timeout
fn wait_for<F>(mut condition: F, timeout: Duration) -> bool
where
    F: FnMut() -> bool,
{
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        if condition() {
            return true;
        }
        thread::sleep(Duration::from_millis(10));
    }
    false
}

/// Test successful posting flow
#[test]
#[ignore] // Requires configured platforms - run manually with credentials
fn test_successful_posting_flow() {
    let services = ServiceHandle::new().expect("Failed to create service handle");
    let mut state = AppState::new();
    
    // Compose valid content
    let content = "Test post from plur-tui integration test";
    state = reduce(state, Action::ComposerInputChanged(content.to_string()));
    
    // Validate
    let validation = services.validate(content, &["nostr"]);
    let (valid, errors, warnings, char_count) = plur_tui::services::validation_summary(&validation, content);
    
    state = reduce(state, Action::ComposerValidationResult {
        valid,
        errors,
        warnings,
        char_count,
    });
    
    assert!(state.can_post(), "Should be able to post valid content");
    
    // Start posting
    state = reduce(state, Action::ComposerPostRequested);
    
    if state.can_post() {
        state = reduce(state, Action::ComposerPostStarted);
        
        let (_post_id, rx) = services.post(content.to_string(), vec!["nostr".to_string()])
            .expect("Failed to start posting");
        
        // Wait for completion
        let mut completed = false;
        let timeout = Duration::from_secs(30);
        let start = std::time::Instant::now();
        
        while start.elapsed() < timeout {
            if let Ok(event) = rx.try_recv() {
                match event {
                    libplurcast::service::events::Event::PostingProgress { platform, status, .. } => {
                        println!("Progress: {} - {}", platform, status);
                        state = reduce(state, Action::ComposerPostProgress {
                            platform,
                            message: status,
                        });
                    }
                    libplurcast::service::events::Event::PostingCompleted { post_id, results } => {
                        println!("Completed: {}", post_id);
                        
                        let tui_results: Vec<plur_tui::app::actions::PlatformResult> = results.into_iter()
                            .map(|r| plur_tui::app::actions::PlatformResult {
                                platform: r.platform,
                                success: r.success,
                                post_id: r.post_id,
                                error: r.error,
                            })
                            .collect();
                        
                        state = reduce(state, Action::ComposerPostSucceeded {
                            post_id,
                            results: tui_results,
                        });
                        
                        completed = true;
                        break;
                    }
                    libplurcast::service::events::Event::PostingFailed { error, .. } => {
                        println!("Failed: {}", error);
                        state = reduce(state, Action::ComposerPostFailed { error });
                        break;
                    }
                    _ => {}
                }
            }
            thread::sleep(Duration::from_millis(100));
        }
        
        assert!(completed, "Posting should complete within timeout");
        assert!(!state.composer.posting, "Should no longer be posting");
        assert!(state.composer.last_post_id.is_some(), "Should have post ID");
    }
}

/// Test posting with invalid content
#[test]
fn test_posting_invalid_content() {
    let services = ServiceHandle::new().expect("Failed to create service handle");
    let mut state = AppState::new();
    
    // Empty content is invalid
    let content = "";
    state = reduce(state, Action::ComposerInputChanged(content.to_string()));
    
    // Validate
    let validation = services.validate(content, &["nostr", "mastodon"]);
    let (valid, errors, warnings, char_count) = plur_tui::services::validation_summary(&validation, content);
    
    state = reduce(state, Action::ComposerValidationResult {
        valid,
        errors,
        warnings,
        char_count,
    });
    
    assert!(!state.can_post(), "Should not be able to post invalid content");
    
    // Attempt to post should fail
    state = reduce(state, Action::ComposerPostRequested);
    
    // State should remain unchanged (can't post)
    assert!(!state.composer.posting, "Should not start posting invalid content");
}

/// Test posting with very long content
#[test]
fn test_posting_long_content() {
    let services = ServiceHandle::new().expect("Failed to create service handle");
    let mut state = AppState::new();
    
    // Create content longer than mastodon limit (500)
    let content = "a".repeat(600);
    state = reduce(state, Action::ComposerInputChanged(content.clone()));
    
    // Validate for mastodon
    let validation = services.validate(&content, &["mastodon"]);
    let (valid, errors, _warnings, char_count) = plur_tui::services::validation_summary(&validation, &content);
    
    state = reduce(state, Action::ComposerValidationResult {
        valid,
        errors: errors.clone(),
        warnings: vec![],
        char_count,
    });
    
    assert!(!valid, "Long content should be invalid for mastodon");
    assert_eq!(char_count, 600);
    assert!(!errors.is_empty(), "Should have validation errors");
    assert!(!state.can_post(), "Should not be able to post");
}

/// Test progress tracking during posting
#[test]
fn test_progress_tracking() {
    let mut state = AppState::new();
    
    // Start posting
    state = reduce(state, Action::ComposerPostStarted);
    assert!(state.composer.posting);
    assert!(state.composer.progress.is_empty());
    
    // Simulate progress updates
    state = reduce(state, Action::ComposerPostProgress {
        platform: "nostr".to_string(),
        message: "Connecting...".to_string(),
    });
    
    assert_eq!(state.composer.progress.len(), 1);
    assert_eq!(state.composer.progress[0].0, "nostr");
    assert_eq!(state.composer.progress[0].1, "Connecting...");
    
    // Update same platform
    state = reduce(state, Action::ComposerPostProgress {
        platform: "nostr".to_string(),
        message: "Uploading...".to_string(),
    });
    
    assert_eq!(state.composer.progress.len(), 1);
    assert_eq!(state.composer.progress[0].1, "Uploading...");
    
    // Add another platform
    state = reduce(state, Action::ComposerPostProgress {
        platform: "mastodon".to_string(),
        message: "Posting...".to_string(),
    });
    
    assert_eq!(state.composer.progress.len(), 2);
}

/// Test posting success clears state correctly
#[test]
fn test_posting_success_state() {
    let mut state = AppState::new();
    
    // Set up initial content and posting state
    state = reduce(state, Action::ComposerInputChanged("Test content".to_string()));
    state = reduce(state, Action::ComposerPostStarted);
    state = reduce(state, Action::ComposerPostProgress {
        platform: "nostr".to_string(),
        message: "Posting...".to_string(),
    });
    
    assert!(state.composer.posting);
    assert!(!state.composer.progress.is_empty());
    
    // Complete posting
    state = reduce(state, Action::ComposerPostSucceeded {
        post_id: "test123".to_string(),
        results: vec![
            plur_tui::app::actions::PlatformResult {
                platform: "nostr".to_string(),
                success: true,
                post_id: Some("note1abc".to_string()),
                error: None,
            },
        ],
    });
    
    // Verify state after success
    assert!(!state.composer.posting, "Should no longer be posting");
    assert!(state.composer.progress.is_empty(), "Progress should be cleared");
    assert_eq!(state.composer.last_post_id, Some("test123".to_string()));
    assert!(state.status.message.is_some(), "Should have success message");
}

/// Test posting failure shows error
#[test]
fn test_posting_failure_state() {
    let mut state = AppState::new();
    
    // Set up posting state
    state = reduce(state, Action::ComposerInputChanged("Test content".to_string()));
    state = reduce(state, Action::ComposerPostStarted);
    
    assert!(state.composer.posting);
    
    // Fail posting
    state = reduce(state, Action::ComposerPostFailed {
        error: "Network error".to_string(),
    });
    
    // Verify state after failure
    assert!(!state.composer.posting, "Should no longer be posting");
    assert!(state.composer.progress.is_empty(), "Progress should be cleared");
    assert_eq!(state.error, Some("Network error".to_string()));
    assert_eq!(state.composer.content, "Test content", "Content should be preserved for retry");
}

/// Test that posting is disabled while already posting
#[test]
fn test_cannot_post_while_posting() {
    let mut state = AppState::new();
    
    // Set up valid content
    state = reduce(state, Action::ComposerInputChanged("Valid content".to_string()));
    state = reduce(state, Action::ComposerValidationResult {
        valid: true,
        errors: vec![],
        warnings: vec![],
        char_count: 13,
    });
    
    assert!(state.can_post());
    
    // Start posting
    state = reduce(state, Action::ComposerPostStarted);
    
    // Should no longer be able to post
    assert!(!state.can_post(), "Should not be able to post while already posting");
}

/// Test composer clear after successful post
#[test]
fn test_composer_clear_after_post() {
    let mut state = AppState::new();
    
    // Set up and complete a post (with validation)
    state = reduce(state, Action::ComposerInputChanged("Test content".to_string()));
    state = reduce(state, Action::ComposerValidationResult {
        valid: true,
        errors: vec![],
        warnings: vec![],
        char_count: 12,
    });
    state = reduce(state, Action::ComposerPostSucceeded {
        post_id: "test123".to_string(),
        results: vec![],
    });
    
    assert_eq!(state.composer.last_post_id, Some("test123".to_string()));
    
    // Clear composer
    state = reduce(state, Action::ComposerClear);
    
    // Verify everything is reset to default
    // Default ComposerState has "Content cannot be empty" error
    assert!(state.composer.content.is_empty());
    assert_eq!(state.composer.char_count, 0);
    assert!(!state.composer.valid);
    assert!(!state.composer.posting);
    assert!(state.composer.progress.is_empty());
    assert!(state.composer.last_post_id.is_none());
    assert_eq!(state.composer.errors, vec!["Content cannot be empty"]);
    assert!(state.composer.warnings.is_empty());
}

/// Test multiple platform posting
#[test]
fn test_multi_platform_posting() {
    let mut state = AppState::new();
    
    state = reduce(state, Action::ComposerPostStarted);
    
    // Simulate progress from multiple platforms
    state = reduce(state, Action::ComposerPostProgress {
        platform: "nostr".to_string(),
        message: "Connecting...".to_string(),
    });
    
    state = reduce(state, Action::ComposerPostProgress {
        platform: "mastodon".to_string(),
        message: "Connecting...".to_string(),
    });
    
    state = reduce(state, Action::ComposerPostProgress {
        platform: "bluesky".to_string(),
        message: "Connecting...".to_string(),
    });
    
    assert_eq!(state.composer.progress.len(), 3);
    
    // Complete with mixed results
    state = reduce(state, Action::ComposerPostSucceeded {
        post_id: "multi123".to_string(),
        results: vec![
            plur_tui::app::actions::PlatformResult {
                platform: "nostr".to_string(),
                success: true,
                post_id: Some("note1abc".to_string()),
                error: None,
            },
            plur_tui::app::actions::PlatformResult {
                platform: "mastodon".to_string(),
                success: true,
                post_id: Some("12345".to_string()),
                error: None,
            },
            plur_tui::app::actions::PlatformResult {
                platform: "bluesky".to_string(),
                success: false,
                post_id: None,
                error: Some("Rate limited".to_string()),
            },
        ],
    });
    
    assert!(!state.composer.posting);
    assert_eq!(state.composer.last_post_id, Some("multi123".to_string()));
}

/// Test validation and posting integration
#[test]
fn test_validation_posting_integration() {
    let services = ServiceHandle::new().expect("Failed to create service handle");
    let mut state = AppState::new();
    
    // Type invalid content
    state = reduce(state, Action::ComposerInputChanged("".to_string()));
    
    let validation = services.validate("", &["nostr"]);
    let (valid, errors, warnings, char_count) = plur_tui::services::validation_summary(&validation, "");
    
    state = reduce(state, Action::ComposerValidationResult {
        valid,
        errors: errors.clone(),
        warnings,
        char_count,
    });
    
    assert!(!state.can_post());
    assert!(!errors.is_empty(), "Empty content should have validation errors");
    
    // Type valid content
    let content = "Now this is valid content!";
    state = reduce(state, Action::ComposerInputChanged(content.to_string()));
    
    let validation = services.validate(content, &["nostr"]);
    let (valid, errors, warnings, char_count) = plur_tui::services::validation_summary(&validation, content);
    
    state = reduce(state, Action::ComposerValidationResult {
        valid,
        errors,
        warnings,
        char_count,
    });
    
    assert!(state.can_post(), "Valid content should allow posting");
    assert_eq!(state.composer.char_count, 26);
    assert!(state.composer.errors.is_empty(), "Valid content should have no errors");
}
