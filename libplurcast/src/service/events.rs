//! Event system for progress tracking
//!
//! This module provides an in-process event bus for distributing progress
//! events to subscribers without blocking operations.
//!
//! # Architecture
//!
//! The event bus uses `tokio::sync::broadcast` for multi-subscriber support.
//! Events are emitted by services during long-running operations and can be
//! consumed by any number of subscribers (CLI progress bars, TUI updates, etc.).
//!
//! # Non-Blocking Behavior
//!
//! If no subscribers exist, events are dropped immediately without allocation
//! or blocking. Subscribers can lag without blocking emitters.
//!
//! # Example
//!
//! ```no_run
//! use libplurcast::service::events::{EventBus, Event};
//!
//! # async fn example() {
//! let event_bus = EventBus::new(100);
//!
//! // Subscribe to events
//! let mut receiver = event_bus.subscribe();
//!
//! // Emit events (non-blocking)
//! event_bus.emit(Event::PostingStarted {
//!     post_id: "abc123".to_string(),
//!     platforms: vec!["nostr".to_string()],
//! });
//!
//! // Receive events
//! if let Ok(event) = receiver.recv().await {
//!     println!("Received: {:?}", event);
//! }
//! # }
//! ```

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

/// Event receiver type alias
pub type EventReceiver = broadcast::Receiver<Event>;

/// Event bus for distributing progress events
///
/// The event bus uses a broadcast channel to distribute events to multiple
/// subscribers. Events are dropped if no subscribers exist, ensuring
/// non-blocking behavior.
#[derive(Clone)]
pub struct EventBus {
    sender: broadcast::Sender<Event>,
}

impl EventBus {
    /// Create a new event bus with the specified capacity
    ///
    /// The capacity determines how many events can be buffered per subscriber
    /// before older events are dropped (if the subscriber is lagging).
    ///
    /// # Arguments
    ///
    /// * `capacity` - Buffer capacity per subscriber (recommended: 100)
    ///
    /// # Example
    ///
    /// ```
    /// use libplurcast::service::events::EventBus;
    ///
    /// let event_bus = EventBus::new(100);
    /// ```
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Subscribe to events
    ///
    /// Returns a receiver that will receive all events emitted after
    /// subscription. Multiple subscribers are supported.
    ///
    /// # Example
    ///
    /// ```
    /// use libplurcast::service::events::EventBus;
    ///
    /// let event_bus = EventBus::new(100);
    /// let mut receiver1 = event_bus.subscribe();
    /// let mut receiver2 = event_bus.subscribe();
    /// ```
    pub fn subscribe(&self) -> EventReceiver {
        self.sender.subscribe()
    }

    /// Emit an event to all subscribers
    ///
    /// This is a non-blocking operation. If no subscribers exist, the event
    /// is dropped immediately. If subscribers are lagging, they may miss
    /// events (oldest events are dropped first).
    ///
    /// # Arguments
    ///
    /// * `event` - The event to emit
    ///
    /// # Example
    ///
    /// ```
    /// use libplurcast::service::events::{EventBus, Event};
    ///
    /// let event_bus = EventBus::new(100);
    /// event_bus.emit(Event::PostingStarted {
    ///     post_id: "abc123".to_string(),
    ///     platforms: vec!["nostr".to_string()],
    /// });
    /// ```
    pub fn emit(&self, event: Event) {
        // send() returns Err if no receivers exist, which is fine
        // We don't want to block or fail if nobody is listening
        let _ = self.sender.send(event);
    }

    /// Get the number of active subscribers
    ///
    /// This is useful for debugging or metrics, but should not be used
    /// for control flow decisions.
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

/// Events emitted by services during operations
///
/// All events are cloneable and serializable for flexibility in how
/// they're consumed (logging, UI updates, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    /// Posting operation started
    PostingStarted {
        /// Unique identifier for the post
        post_id: String,
        /// List of platforms being posted to
        platforms: Vec<String>,
    },

    /// Progress update for a specific platform
    PostingProgress {
        /// Unique identifier for the post
        post_id: String,
        /// Platform being posted to
        platform: String,
        /// Status message (e.g., "Connecting...", "Uploading...")
        status: String,
    },

    /// Posting operation completed
    PostingCompleted {
        /// Unique identifier for the post
        post_id: String,
        /// Results for each platform
        results: Vec<PlatformResult>,
    },

    /// Posting operation failed
    PostingFailed {
        /// Unique identifier for the post
        post_id: String,
        /// Error message
        error: String,
    },
}

/// Result of posting to a single platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformResult {
    /// Platform name (e.g., "nostr", "mastodon")
    pub platform: String,
    /// Whether the post succeeded
    pub success: bool,
    /// Platform-specific post ID (if successful)
    pub post_id: Option<String>,
    /// Error message (if failed)
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_emission_and_subscription() {
        let event_bus = EventBus::new(10);
        let mut receiver = event_bus.subscribe();

        let event = Event::PostingStarted {
            post_id: "test123".to_string(),
            platforms: vec!["nostr".to_string()],
        };

        event_bus.emit(event.clone());

        let received = receiver.recv().await.unwrap();
        match received {
            Event::PostingStarted { post_id, platforms } => {
                assert_eq!(post_id, "test123");
                assert_eq!(platforms, vec!["nostr"]);
            }
            _ => panic!("Wrong event type received"),
        }
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let event_bus = EventBus::new(10);
        let mut receiver1 = event_bus.subscribe();
        let mut receiver2 = event_bus.subscribe();

        let event = Event::PostingProgress {
            post_id: "test456".to_string(),
            platform: "mastodon".to_string(),
            status: "Uploading...".to_string(),
        };

        event_bus.emit(event.clone());

        // Both receivers should get the event
        let received1 = receiver1.recv().await.unwrap();
        let received2 = receiver2.recv().await.unwrap();

        match (received1, received2) {
            (
                Event::PostingProgress {
                    post_id: id1,
                    platform: p1,
                    status: s1,
                },
                Event::PostingProgress {
                    post_id: id2,
                    platform: p2,
                    status: s2,
                },
            ) => {
                assert_eq!(id1, "test456");
                assert_eq!(id2, "test456");
                assert_eq!(p1, "mastodon");
                assert_eq!(p2, "mastodon");
                assert_eq!(s1, "Uploading...");
                assert_eq!(s2, "Uploading...");
            }
            _ => panic!("Wrong event types received"),
        }
    }

    #[tokio::test]
    async fn test_no_subscribers() {
        let event_bus = EventBus::new(10);

        // Emit event with no subscribers - should not panic or block
        event_bus.emit(Event::PostingStarted {
            post_id: "test789".to_string(),
            platforms: vec!["ssb".to_string()],
        });

        // Verify subscriber count is 0
        assert_eq!(event_bus.subscriber_count(), 0);
    }

    #[tokio::test]
    async fn test_event_cloning() {
        let event = Event::PostingCompleted {
            post_id: "clone_test".to_string(),
            results: vec![
                PlatformResult {
                    platform: "nostr".to_string(),
                    success: true,
                    post_id: Some("note1abc".to_string()),
                    error: None,
                },
                PlatformResult {
                    platform: "mastodon".to_string(),
                    success: false,
                    post_id: None,
                    error: Some("Rate limited".to_string()),
                },
            ],
        };

        let cloned = event.clone();

        match (event, cloned) {
            (
                Event::PostingCompleted {
                    post_id: id1,
                    results: r1,
                },
                Event::PostingCompleted {
                    post_id: id2,
                    results: r2,
                },
            ) => {
                assert_eq!(id1, id2);
                assert_eq!(r1.len(), r2.len());
                assert_eq!(r1[0].platform, r2[0].platform);
                assert_eq!(r1[1].error, r2[1].error);
            }
            _ => panic!("Event cloning failed"),
        }
    }

    #[tokio::test]
    async fn test_event_serialization() {
        let event = Event::PostingFailed {
            post_id: "serial_test".to_string(),
            error: "Network timeout".to_string(),
        };

        // Serialize to JSON
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("posting_failed"));
        assert!(json.contains("serial_test"));
        assert!(json.contains("Network timeout"));

        // Deserialize back
        let deserialized: Event = serde_json::from_str(&json).unwrap();
        match deserialized {
            Event::PostingFailed { post_id, error } => {
                assert_eq!(post_id, "serial_test");
                assert_eq!(error, "Network timeout");
            }
            _ => panic!("Deserialization failed"),
        }
    }

    #[tokio::test]
    async fn test_subscriber_count() {
        let event_bus = EventBus::new(10);
        assert_eq!(event_bus.subscriber_count(), 0);

        let _receiver1 = event_bus.subscribe();
        assert_eq!(event_bus.subscriber_count(), 1);

        let _receiver2 = event_bus.subscribe();
        assert_eq!(event_bus.subscriber_count(), 2);

        drop(_receiver1);
        // Note: subscriber count may not update immediately after drop
        // This is a limitation of broadcast channels
    }

    #[tokio::test]
    async fn test_all_event_variants() {
        let event_bus = EventBus::new(10);
        let mut receiver = event_bus.subscribe();

        // Test PostingStarted
        event_bus.emit(Event::PostingStarted {
            post_id: "1".to_string(),
            platforms: vec!["nostr".to_string()],
        });
        assert!(matches!(
            receiver.recv().await.unwrap(),
            Event::PostingStarted { .. }
        ));

        // Test PostingProgress
        event_bus.emit(Event::PostingProgress {
            post_id: "2".to_string(),
            platform: "mastodon".to_string(),
            status: "Connecting...".to_string(),
        });
        assert!(matches!(
            receiver.recv().await.unwrap(),
            Event::PostingProgress { .. }
        ));

        // Test PostingCompleted
        event_bus.emit(Event::PostingCompleted {
            post_id: "3".to_string(),
            results: vec![],
        });
        assert!(matches!(
            receiver.recv().await.unwrap(),
            Event::PostingCompleted { .. }
        ));

        // Test PostingFailed
        event_bus.emit(Event::PostingFailed {
            post_id: "4".to_string(),
            error: "Test error".to_string(),
        });
        assert!(matches!(
            receiver.recv().await.unwrap(),
            Event::PostingFailed { .. }
        ));
    }
}
