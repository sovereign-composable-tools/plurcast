//! Draft service for managing draft posts
//!
//! This module provides CRUD operations for draft posts and publishing.

use std::sync::Arc;
use crate::{Database, Result};
use chrono::{DateTime, Utc};
use super::posting::{PostingService, PostResponse};

/// Draft service
///
/// Manages draft posts (CRUD operations) and delegates publishing to
/// the posting service.
pub struct DraftService {
    db: Arc<Database>,
    posting: PostingService,
}

/// A draft post
#[derive(Debug, Clone)]
pub struct Draft {
    pub id: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl DraftService {
    /// Create a new draft service
    pub fn new(db: Arc<Database>, posting: PostingService) -> Self {
        Self { db, posting }
    }

    /// Create a new draft
    ///
    /// # Errors
    ///
    /// Returns an error if the draft cannot be saved to the database.
    pub async fn create(&self, _content: String) -> Result<Draft> {
        // TODO: Implement in task 6
        unimplemented!("DraftService::create will be implemented in task 6")
    }

    /// Update an existing draft
    ///
    /// # Errors
    ///
    /// Returns an error if the draft doesn't exist or cannot be updated.
    pub async fn update(&self, _id: &str, _content: String) -> Result<Draft> {
        // TODO: Implement in task 6
        unimplemented!("DraftService::update will be implemented in task 6")
    }

    /// Delete a draft
    ///
    /// # Errors
    ///
    /// Returns an error if the draft doesn't exist or cannot be deleted.
    pub async fn delete(&self, _id: &str) -> Result<()> {
        // TODO: Implement in task 6
        unimplemented!("DraftService::delete will be implemented in task 6")
    }

    /// List all drafts
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn list(&self) -> Result<Vec<Draft>> {
        // TODO: Implement in task 6
        unimplemented!("DraftService::list will be implemented in task 6")
    }

    /// Get a single draft by ID
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn get(&self, _id: &str) -> Result<Option<Draft>> {
        // TODO: Implement in task 6
        unimplemented!("DraftService::get will be implemented in task 6")
    }

    /// Publish a draft
    ///
    /// Delegates to the posting service to actually post the content.
    ///
    /// # Errors
    ///
    /// Returns an error if the draft doesn't exist or posting fails.
    pub async fn publish(&self, _id: &str, _platforms: Vec<String>) -> Result<PostResponse> {
        // TODO: Implement in task 6
        unimplemented!("DraftService::publish will be implemented in task 6")
    }
}
