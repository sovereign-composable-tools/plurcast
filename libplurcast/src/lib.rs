//! Plurcast - Unix tools for the decentralized social web
//!
//! This library provides core functionality for posting to decentralized
//! social media platforms following Unix philosophy principles.

pub mod config;
pub mod credentials;
pub mod db;
pub mod error;
pub mod platforms;
pub mod poster;
pub mod service;
pub mod types;

// Re-export commonly used types
pub use config::Config;
pub use credentials::{CredentialConfig, CredentialManager, StorageBackend};
pub use db::{Database, PostWithRecords};
pub use error::{PlurcastError, Result};
pub use types::{Post, PostRecord, PostStatus};
