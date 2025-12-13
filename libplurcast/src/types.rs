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

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "TEXT")]
pub enum PostStatus {
    Draft,
    Scheduled,
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
    pub account_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_post_new_uuid_generation() {
        let post = Post::new("Test content".to_string());

        // Verify UUID format (should be valid UUIDv4)
        let uuid_result = uuid::Uuid::parse_str(&post.id);
        assert!(uuid_result.is_ok(), "Post ID should be a valid UUID");

        // Verify it's a v4 UUID
        let uuid = uuid_result.unwrap();
        assert_eq!(uuid.get_version(), Some(uuid::Version::Random));
    }

    #[test]
    fn test_post_new_unique_ids() {
        let post1 = Post::new("Content 1".to_string());
        let post2 = Post::new("Content 2".to_string());

        // Each post should have a unique ID
        assert_ne!(post1.id, post2.id);
    }

    #[test]
    fn test_post_new_timestamp_generation() {
        let before = chrono::Utc::now().timestamp();
        let post = Post::new("Test content".to_string());
        let after = chrono::Utc::now().timestamp();

        // Timestamp should be within reasonable range (Unix timestamp)
        assert!(post.created_at >= before);
        assert!(post.created_at <= after);

        // Verify it's a valid Unix timestamp (positive number, reasonable range)
        assert!(post.created_at > 1_600_000_000); // After Sept 2020
        assert!(post.created_at < 2_000_000_000); // Before May 2033
    }

    #[test]
    fn test_post_new_default_values() {
        let content = "Test content".to_string();
        let post = Post::new(content.clone());

        assert_eq!(post.content, content);
        assert_eq!(post.scheduled_at, None);
        assert!(matches!(post.status, PostStatus::Pending));
        assert_eq!(post.metadata, None);
    }

    #[test]
    fn test_post_status_pending() {
        let status = PostStatus::Pending;

        // Test serialization
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, r#""Pending""#);

        // Test deserialization
        let deserialized: PostStatus = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, PostStatus::Pending));
    }

    #[test]
    fn test_post_status_posted() {
        let status = PostStatus::Posted;

        // Test serialization
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, r#""Posted""#);

        // Test deserialization
        let deserialized: PostStatus = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, PostStatus::Posted));
    }

    #[test]
    fn test_post_status_failed() {
        let status = PostStatus::Failed;

        // Test serialization
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, r#""Failed""#);

        // Test deserialization
        let deserialized: PostStatus = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, PostStatus::Failed));
    }

    #[test]
    fn test_post_status_draft() {
        let status = PostStatus::Draft;

        // Test serialization
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, r#""Draft""#);

        // Test deserialization
        let deserialized: PostStatus = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, PostStatus::Draft));
    }

    #[test]
    fn test_post_serialization() {
        let post = Post {
            id: "test-id".to_string(),
            content: "Test content".to_string(),
            created_at: 1234567890,
            scheduled_at: Some(1234567900),
            status: PostStatus::Pending,
            metadata: Some(r#"{"tags":["test"]}"#.to_string()),
        };

        let json = serde_json::to_string(&post).unwrap();
        let deserialized: Post = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, post.id);
        assert_eq!(deserialized.content, post.content);
        assert_eq!(deserialized.created_at, post.created_at);
        assert_eq!(deserialized.scheduled_at, post.scheduled_at);
        assert_eq!(deserialized.metadata, post.metadata);
    }

    #[test]
    fn test_post_record_creation_with_all_fields() {
        let record = PostRecord {
            id: Some(1),
            post_id: "post-123".to_string(),
            platform: "nostr".to_string(),
            platform_post_id: Some("note1abc".to_string()),
            posted_at: Some(1234567890),
            success: true,
            error_message: None,
            account_name: "default".to_string(),
        };

        assert_eq!(record.id, Some(1));
        assert_eq!(record.post_id, "post-123");
        assert_eq!(record.platform, "nostr");
        assert_eq!(record.platform_post_id, Some("note1abc".to_string()));
        assert_eq!(record.posted_at, Some(1234567890));
        assert!(record.success);
        assert_eq!(record.error_message, None);
        assert_eq!(record.account_name, "default");
    }

    #[test]
    fn test_post_record_creation_success() {
        let record = PostRecord {
            id: None,
            post_id: "post-456".to_string(),
            platform: "mastodon".to_string(),
            platform_post_id: Some("12345".to_string()),
            posted_at: Some(chrono::Utc::now().timestamp()),
            success: true,
            error_message: None,
            account_name: "default".to_string(),
        };

        assert!(record.success);
        assert_eq!(record.error_message, None);
        assert!(record.platform_post_id.is_some());
    }

    #[test]
    fn test_post_record_creation_failure() {
        let record = PostRecord {
            id: None,
            post_id: "post-789".to_string(),
            platform: "bluesky".to_string(),
            platform_post_id: None,
            posted_at: None,
            success: false,
            error_message: Some("Network timeout".to_string()),
            account_name: "default".to_string(),
        };

        assert!(!record.success);
        assert_eq!(record.error_message, Some("Network timeout".to_string()));
        assert_eq!(record.platform_post_id, None);
        assert_eq!(record.posted_at, None);
    }

    #[test]
    fn test_post_record_serialization() {
        let record = PostRecord {
            id: Some(42),
            post_id: "post-abc".to_string(),
            platform: "nostr".to_string(),
            platform_post_id: Some("note1xyz".to_string()),
            posted_at: Some(1234567890),
            success: true,
            error_message: None,
            account_name: "default".to_string(),
        };

        let json = serde_json::to_string(&record).unwrap();
        let deserialized: PostRecord = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, record.id);
        assert_eq!(deserialized.post_id, record.post_id);
        assert_eq!(deserialized.platform, record.platform);
        assert_eq!(deserialized.platform_post_id, record.platform_post_id);
        assert_eq!(deserialized.posted_at, record.posted_at);
        assert_eq!(deserialized.success, record.success);
        assert_eq!(deserialized.error_message, record.error_message);
        assert_eq!(deserialized.account_name, record.account_name);
    }

    #[test]
    fn test_post_with_metadata() {
        let metadata = serde_json::json!({
            "tags": ["rust", "decentralization"],
            "reply_to": "note1abc"
        });

        let post = Post {
            id: uuid::Uuid::new_v4().to_string(),
            content: "Test with metadata".to_string(),
            created_at: chrono::Utc::now().timestamp(),
            scheduled_at: None,
            status: PostStatus::Pending,
            metadata: Some(metadata.to_string()),
        };

        assert!(post.metadata.is_some());

        // Verify metadata can be parsed back
        let parsed: serde_json::Value =
            serde_json::from_str(post.metadata.as_ref().unwrap()).unwrap();
        assert_eq!(parsed["tags"][0], "rust");
        assert_eq!(parsed["tags"][1], "decentralization");
    }

    #[test]
    fn test_post_clone() {
        let post = Post::new("Original content".to_string());
        let cloned = post.clone();

        assert_eq!(post.id, cloned.id);
        assert_eq!(post.content, cloned.content);
        assert_eq!(post.created_at, cloned.created_at);
    }

    #[test]
    fn test_post_record_clone() {
        let record = PostRecord {
            id: None,
            post_id: "test".to_string(),
            platform: "nostr".to_string(),
            platform_post_id: Some("note1".to_string()),
            posted_at: Some(123),
            success: true,
            error_message: None,
            account_name: "default".to_string(),
        };

        let cloned = record.clone();
        assert_eq!(record.id, cloned.id);
        assert_eq!(record.post_id, cloned.post_id);
    }
}
