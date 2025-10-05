//! Plurcast - Unix tools for the decentralized social web
//!
//! This library provides core functionality for posting to decentralized
//! social media platforms following Unix philosophy principles.

pub mod config;
pub mod db;
pub mod error;
pub mod platforms;
pub mod types;

// Re-export commonly used types
pub use error::{PlurcastError, Result};
pub use types::{Post, PostRecord, PostStatus};
