//! Service layer for Plurcast
//!
//! This module provides a clean, testable API for business logic that can be
//! consumed by multiple interfaces (CLI, TUI, GUI) without code duplication.
//!
//! # Architecture
//!
//! The service layer follows a facade pattern with `PlurcastService` as the
//! main entry point, coordinating specialized sub-services:
//!
//! - `PostingService`: Multi-platform posting with retry logic
//! - `HistoryService`: Query and analyze post history
//! - `DraftService`: Manage draft posts
//! - `ValidationService`: Real-time content validation
//! - `EventBus`: Progress event distribution
//!
//! # Example
//!
//! ```no_run
//! use std::collections::HashMap;
//! use libplurcast::service::PlurcastService;
//! use libplurcast::service::posting::PostRequest;
//!
//! # async fn example() -> libplurcast::Result<()> {
//! let service = PlurcastService::new().await?;
//!
//! let request = PostRequest {
//!     content: "Hello decentralized world!".to_string(),
//!     platforms: vec!["nostr".to_string(), "mastodon".to_string()],
//!     draft: false,
//!     account: None,
//!     scheduled_at: None,
//!     nostr_pow: None,
//!     nostr_21e8: false,
//!     reply_to: HashMap::new(),
//! };
//!
//! let response = service.posting().post(request).await?;
//! println!("Posted to {} platforms", response.results.len());
//! # Ok(())
//! # }
//! ```

pub mod draft;
pub mod events;
pub mod history;
pub mod posting;
pub mod validation;

// Re-export commonly used types
pub use events::PlatformResult;

use self::draft::DraftService;
use self::events::EventBus;
use self::history::HistoryService;
use self::posting::PostingService;
use self::validation::ValidationService;
use crate::{Config, Database, Result};
use std::sync::Arc;

/// Main service facade that coordinates all sub-services
///
/// `PlurcastService` provides a single entry point for all service operations,
/// managing shared resources (Database, Config) and providing access to
/// specialized sub-services.
///
/// # Shared State
///
/// All sub-services share the same `Arc<Database>` and `Arc<Config>` instances,
/// enabling efficient concurrent access without duplication.
///
/// # Example
///
/// ```no_run
/// use libplurcast::service::PlurcastService;
///
/// # async fn example() -> libplurcast::Result<()> {
/// // Create service with default configuration
/// let service = PlurcastService::new().await?;
///
/// // Access sub-services
/// let posting = service.posting();
/// let history = service.history();
/// let drafts = service.draft();
/// let validation = service.validation();
///
/// // Subscribe to events
/// let mut events = service.subscribe();
/// # Ok(())
/// # }
/// ```
pub struct PlurcastService {
    db: Arc<Database>,
    posting: PostingService,
    history: HistoryService,
    draft: DraftService,
    validation: ValidationService,
    event_bus: EventBus,
}

impl PlurcastService {
    /// Create a new service with default configuration
    ///
    /// This loads configuration from the default location and initializes
    /// the database.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Configuration cannot be loaded
    /// - Database cannot be initialized
    /// - Database migrations fail
    pub async fn new() -> Result<Self> {
        let config = Config::load()?;
        Self::from_config(config).await
    }

    /// Create a service with custom configuration
    ///
    /// This allows providing a pre-configured `Config` instance, useful for
    /// testing or custom setups.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Database cannot be initialized
    /// - Database migrations fail
    pub async fn from_config(config: Config) -> Result<Self> {
        // Initialize shared resources
        let db_path = crate::config::resolve_db_path(Some(&config.database.path))?;
        let db_path_str = db_path.to_str().ok_or_else(|| {
            crate::error::PlurcastError::Config(crate::error::ConfigError::MissingField(
                "Invalid database path".to_string(),
            ))
        })?;
        let db = Database::new(db_path_str).await?;

        let db = Arc::new(db);
        let config = Arc::new(config);
        let event_bus = EventBus::new(100);

        // Create sub-services with shared state
        let posting = PostingService::new(Arc::clone(&db), Arc::clone(&config), event_bus.clone());
        let history = HistoryService::new(Arc::clone(&db));
        let validation = ValidationService::new(Arc::clone(&config));
        let draft = DraftService::new(Arc::clone(&db), posting.clone());

        Ok(Self {
            db,
            posting,
            history,
            draft,
            validation,
            event_bus,
        })
    }

    /// Access the database directly
    ///
    /// Provides direct access to the database for operations like looking up
    /// platform-specific post IDs from a plurcast UUID.
    pub fn database(&self) -> &Database {
        &self.db
    }

    /// Access the posting service
    ///
    /// The posting service handles multi-platform posting, retry logic,
    /// and progress tracking.
    pub fn posting(&self) -> &PostingService {
        &self.posting
    }

    /// Access the history service
    ///
    /// The history service provides querying and analysis of post history.
    pub fn history(&self) -> &HistoryService {
        &self.history
    }

    /// Access the draft service
    ///
    /// The draft service manages draft posts (CRUD operations and publishing).
    pub fn draft(&self) -> &DraftService {
        &self.draft
    }

    /// Access the validation service
    ///
    /// The validation service provides real-time content validation against
    /// platform requirements.
    pub fn validation(&self) -> &ValidationService {
        &self.validation
    }

    /// Subscribe to service events
    ///
    /// Returns a receiver that will receive progress events from service
    /// operations. Multiple subscribers are supported.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use libplurcast::service::PlurcastService;
    ///
    /// # async fn example() -> libplurcast::Result<()> {
    /// let service = PlurcastService::new().await?;
    /// let mut events = service.subscribe();
    ///
    /// // In a separate task, listen for events
    /// tokio::spawn(async move {
    ///     while let Ok(event) = events.recv().await {
    ///         println!("Event: {:?}", event);
    ///     }
    /// });
    /// # Ok(())
    /// # }
    /// ```
    pub fn subscribe(&self) -> events::EventReceiver {
        self.event_bus.subscribe()
    }
}
