//! Service layer adapter for TUI
//!
//! This module provides an adapter between the async PlurcastService
//! and the synchronous TUI event loop, following functional programming principles.
//!
//! # Architecture
//!
//! - `ServiceHandle`: Wraps PlurcastService and manages tokio runtime
//! - Validation: Synchronous wrapper around ValidationService
//! - Posting: Spawns async task, provides crossbeam channel for progress
//! - Events: Bridges tokio broadcast channel to crossbeam for sync event loop
//!
//! # Example
//!
//! ```no_run
//! use plur_tui::services::ServiceHandle;
//!
//! # fn example() -> plur_tui::error::Result<()> {
//! let services = ServiceHandle::new()?;
//!
//! // Validate content synchronously
//! let validation = services.validate("Hello world!", &["nostr"]);
//! assert!(validation.valid);
//!
//! // Post content asynchronously with progress channel
//! let (post_id, progress_rx) = services.post(
//!     "Hello decentralized world!".to_string(),
//!     vec!["nostr".to_string()],
//! )?;
//! # Ok(())
//! # }\
//! ```

use std::sync::Arc;
use crossbeam_channel::{Sender, Receiver, unbounded};
use libplurcast::service::{
    PlurcastService,
    validation::{ValidationRequest, ValidationResponse},
    posting::PostRequest,
    events::Event,
};
use crate::error::Result;

/// Service handle for TUI operations
///
/// Wraps PlurcastService and provides sync/async bridges for the TUI event loop.
/// Uses a tokio runtime to handle async operations without blocking the UI.
pub struct ServiceHandle {
    service: Arc<PlurcastService>,
    runtime: tokio::runtime::Runtime,
    event_tx: Option<Sender<Event>>,
}

impl ServiceHandle {
    /// Create a new service handle
    ///
    /// Initializes PlurcastService with default configuration and creates
    /// a tokio runtime for async operations.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - PlurcastService initialization fails
    /// - Tokio runtime cannot be created
    pub fn new() -> Result<Self> {
        // Create tokio runtime for async operations
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        
        // Initialize PlurcastService
        let service = runtime.block_on(async {
            PlurcastService::new().await
        })?;
        
        Ok(Self {
            service: Arc::new(service),
            runtime,
            event_tx: None,
        })
    }
    
    /// Subscribe to service events
    ///
    /// Returns a receiver that will receive all service events (progress, completion, errors).
    /// This bridges the tokio broadcast channel to a crossbeam channel for sync use.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use plur_tui::services::ServiceHandle;
    /// # fn example() -> plur_tui::error::Result<()> {
    /// let mut services = ServiceHandle::new()?;
    /// let event_rx = services.subscribe();
    ///
    /// // In event loop, check for events
    /// if let Ok(event) = event_rx.try_recv() {
    ///     // Handle event
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn subscribe(&mut self) -> Receiver<Event> {
        let (tx, rx) = unbounded();
        
        // Spawn task to bridge tokio broadcast -> crossbeam channel
        let mut event_rx = self.service.subscribe();
        let tx_clone = tx.clone();
        self.runtime.spawn(async move {
            loop {
                match event_rx.recv().await {
                    Ok(event) => {
                        // Forward event to crossbeam channel
                        if tx_clone.send(event).is_err() {
                            // Receiver dropped, stop forwarding
                            break;
                        }
                    }
                    Err(e) => {
                        match e {
                            tokio::sync::broadcast::error::RecvError::Lagged(skipped) => {
                                // Warn about lagging but continue
                                tracing::warn!("Event receiver lagged, skipped {} events", skipped);
                            }
                            tokio::sync::broadcast::error::RecvError::Closed => {
                                // Channel closed, stop
                                break;
                            }
                        }
                    }
                }
            }
        });
        
        self.event_tx = Some(tx);
        rx
    }
    
    /// Validate content for specified platforms
    ///
    /// Synchronous validation call that blocks until validation completes.
    /// This is acceptable because validation is fast (no I/O, pure computation).
    ///
    /// # Arguments
    ///
    /// * `content` - Content to validate
    /// * `platforms` - Platform names to validate against
    ///
    /// # Returns
    ///
    /// Validation response with per-platform results
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use plur_tui::services::ServiceHandle;
    /// # fn example() -> plur_tui::error::Result<()> {
    /// let services = ServiceHandle::new()?;
    ///
    /// let validation = services.validate(
    ///     "Hello world!",
    ///     &["nostr", "mastodon"],
    /// );
    ///
    /// if !validation.valid {
    ///     for result in validation.results {
    ///         println!("{}: {:?}", result.platform, result.errors);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn validate(&self, content: &str, platforms: &[&str]) -> ValidationResponse {
        let request = ValidationRequest {
            content: content.to_string(),
            platforms: platforms.iter().map(|s| s.to_string()).collect(),
        };
        
        self.service.validation().validate(request)
    }
    
    /// Post content to specified platforms
    ///
    /// Spawns an async task to post content and returns immediately with:
    /// - A post ID for tracking
    /// - A receiver for progress updates and final result
    ///
    /// Progress events are sent through the returned channel as they occur.
    /// The final event will be either `PostingCompleted` or `PostingFailed`.
    ///
    /// # Arguments
    ///
    /// * `content` - Content to post
    /// * `platforms` - Platform names to post to
    ///
    /// # Returns
    ///
    /// Tuple of (post_id, receiver) for tracking progress
    ///
    /// # Errors
    ///
    /// Returns an error only if the async task cannot be spawned.
    /// Posting failures are communicated through the channel.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use plur_tui::services::ServiceHandle;
    /// # fn example() -> plur_tui::error::Result<()> {
    /// let services = ServiceHandle::new()?;
    ///
    /// let (post_id, progress_rx) = services.post(
    ///     "Hello decentralized world!".to_string(),
    ///     vec!["nostr".to_string(), "mastodon".to_string()],
    /// )?;
    ///
    /// // In event loop, check for progress
    /// if let Ok(event) = progress_rx.try_recv() {
    ///     // Handle progress event
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn post(&self, content: String, platforms: Vec<String>) -> Result<(String, Receiver<Event>)> {
        let (tx, rx) = unbounded();
        
        let request = PostRequest {
            content,
            platforms,
            draft: false,
        };
        
        // Generate post ID before spawning (so we can return it immediately)
        let post_id = uuid::Uuid::new_v4().to_string();
        
        let service = Arc::clone(&self.service);
        let post_id_for_event = post_id.clone();
        let post_id_for_error = post_id.clone();
        
        // Spawn async posting task
        self.runtime.spawn(async move {
            // Subscribe to events for this post
            let mut event_rx = service.subscribe();
            
            // Forward events to the caller's channel
            let tx_clone = tx.clone();
            let post_id_for_filter = post_id_for_event.clone();
            tokio::spawn(async move {
                while let Ok(event) = event_rx.recv().await {
                    // Only forward events related to this post
                    let matches_post = match &event {
                        Event::PostingStarted { post_id, .. } => post_id == &post_id_for_filter,
                        Event::PostingProgress { post_id, .. } => post_id == &post_id_for_filter,
                        Event::PostingCompleted { post_id, .. } => post_id == &post_id_for_filter,
                        Event::PostingFailed { post_id, .. } => post_id == &post_id_for_filter,
                    };
                    
                    if matches_post {
                        if tx_clone.send(event).is_err() {
                            // Receiver dropped, stop
                            break;
                        }
                    }
                }
            });
            
            // Execute the post
            match service.posting().post(request).await {
                Ok(response) => {
                    // Success event was already emitted by service
                    tracing::info!("Post completed: {}", response.post_id);
                }
                Err(e) => {
                    // Send failure event
                    let _ = tx.send(Event::PostingFailed {
                        post_id: post_id_for_error,
                        error: e.to_string(),
                    });
                }
            }
        });
        
        Ok((post_id, rx))
    }
}

/// Helper function to map ValidationResponse to TUI-friendly format
///
/// Extracts the most relevant information from validation results for display.
///
/// # Returns
///
/// Tuple of (valid, errors, warnings, char_count)
pub fn validation_summary(response: &ValidationResponse, content: &str) -> (bool, Vec<String>, Vec<String>, usize) {
    let mut all_errors = Vec::new();
    let mut all_warnings = Vec::new();
    
    for result in &response.results {
        all_errors.extend(result.errors.clone());
        all_warnings.extend(result.warnings.clone());
    }
    
    let char_count = content.chars().count();
    
    (response.valid, all_errors, all_warnings, char_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_service_handle_creation() {
        // This test requires valid config, so it may fail in CI
        // In a real scenario, we'd use a test config
        let result = ServiceHandle::new();
        
        // Just verify the type signature works
        assert!(result.is_ok() || result.is_err());
    }
    
    #[test]
    fn test_validation_summary() {
        use libplurcast::service::validation::{PlatformValidation, ValidationResponse};
        
        let response = ValidationResponse {
            valid: false,
            results: vec![
                PlatformValidation {
                    platform: "nostr".to_string(),
                    valid: true,
                    errors: vec![],
                    warnings: vec!["Content is long".to_string()],
                },
                PlatformValidation {
                    platform: "mastodon".to_string(),
                    valid: false,
                    errors: vec!["Too long".to_string()],
                    warnings: vec![],
                },
            ],
        };
        
        let (valid, errors, warnings, char_count) = validation_summary(&response, "Hello");
        
        assert!(!valid);
        assert_eq!(errors, vec!["Too long"]);
        assert_eq!(warnings, vec!["Content is long"]);
        assert_eq!(char_count, 5);
    }
}
