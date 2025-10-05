//! Core types for Plurcast

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: String,
    pub content: String,
    pub created_at: i64,
    pub scheduled_at: Option<i64>,
    pub status: PostStatus,
    pub metadata: Option<String>,
}

impl Post {
    pub fn new(content: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            content,
            created_at: chrono::Utc::now().timestamp(),
            scheduled_at: None,
            status: PostStatus::Pending,
            metadata: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
pub enum PostStatus {
    Pending,
    Posted,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostRecord {
    pub id: Option<i64>,
    pub post_id: String,
    pub platform: String,
    pub platform_post_id: Option<String>,
    pub posted_at: Option<i64>,
    pub success: bool,
    pub error_message: Option<String>,
}
