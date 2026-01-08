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

// ============================================================================
// Attachment Types
// ============================================================================

/// Supported image MIME types for attachments
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ImageMimeType {
    Jpeg,
    Png,
    Gif,
    WebP,
}

impl ImageMimeType {
    /// Parse MIME type from a MIME string (e.g., "image/jpeg")
    pub fn from_mime_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "image/jpeg" | "image/jpg" => Some(Self::Jpeg),
            "image/png" => Some(Self::Png),
            "image/gif" => Some(Self::Gif),
            "image/webp" => Some(Self::WebP),
            _ => None,
        }
    }

    /// Detect MIME type from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "jpg" | "jpeg" => Some(Self::Jpeg),
            "png" => Some(Self::Png),
            "gif" => Some(Self::Gif),
            "webp" => Some(Self::WebP),
            _ => None,
        }
    }

    /// Get the MIME type string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Jpeg => "image/jpeg",
            Self::Png => "image/png",
            Self::Gif => "image/gif",
            Self::WebP => "image/webp",
        }
    }

    /// Get the typical file extension for this MIME type
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Jpeg => "jpg",
            Self::Png => "png",
            Self::Gif => "gif",
            Self::WebP => "webp",
        }
    }
}

impl std::fmt::Display for ImageMimeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Status of an attachment upload to a platform
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
pub enum AttachmentStatus {
    /// Not yet uploaded to the platform
    Pending,
    /// Successfully uploaded to the platform
    Uploaded,
    /// Upload failed
    Failed,
}

impl std::fmt::Display for AttachmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Uploaded => write!(f, "uploaded"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

/// An image attachment for a post
///
/// Attachments are stored as file references on disk, not embedded in the database.
/// The file_hash provides integrity verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    /// Unique identifier for the attachment (UUID v4)
    pub id: String,
    /// Post ID this attachment belongs to
    pub post_id: String,
    /// Absolute path to the image file on disk
    pub file_path: String,
    /// MIME type of the image
    pub mime_type: ImageMimeType,
    /// File size in bytes
    pub file_size: u64,
    /// SHA-256 hash of the file content (hex encoded)
    pub file_hash: String,
    /// Optional alt text for accessibility
    pub alt_text: Option<String>,
    /// When the attachment record was created (Unix timestamp)
    pub created_at: i64,
}

impl Attachment {
    /// Create a new attachment with auto-generated ID and timestamp
    pub fn new(
        post_id: String,
        file_path: String,
        mime_type: ImageMimeType,
        file_size: u64,
        file_hash: String,
        alt_text: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            post_id,
            file_path,
            mime_type,
            file_size,
            file_hash,
            alt_text,
            created_at: chrono::Utc::now().timestamp(),
        }
    }
}

/// Platform-specific upload result for an attachment
///
/// Tracks the upload status of an attachment to each platform.
/// A single attachment may have multiple upload records (one per platform).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentUpload {
    /// Database row ID (None for new records)
    pub id: Option<i64>,
    /// Reference to the attachment
    pub attachment_id: String,
    /// Platform name (e.g., "nostr", "mastodon")
    pub platform: String,
    /// Platform-specific attachment ID (e.g., Mastodon media_id)
    pub platform_attachment_id: Option<String>,
    /// Remote URL after upload (for Nostr imeta tags)
    pub remote_url: Option<String>,
    /// When the upload completed (Unix timestamp)
    pub uploaded_at: Option<i64>,
    /// Current upload status
    pub status: AttachmentStatus,
    /// Error message if upload failed
    pub error_message: Option<String>,
}

impl AttachmentUpload {
    /// Create a new pending upload record
    pub fn new_pending(attachment_id: String, platform: String) -> Self {
        Self {
            id: None,
            attachment_id,
            platform,
            platform_attachment_id: None,
            remote_url: None,
            uploaded_at: None,
            status: AttachmentStatus::Pending,
            error_message: None,
        }
    }

    /// Mark the upload as successful
    pub fn mark_uploaded(&mut self, platform_attachment_id: String, remote_url: Option<String>) {
        self.platform_attachment_id = Some(platform_attachment_id);
        self.remote_url = remote_url;
        self.uploaded_at = Some(chrono::Utc::now().timestamp());
        self.status = AttachmentStatus::Uploaded;
        self.error_message = None;
    }

    /// Mark the upload as failed
    pub fn mark_failed(&mut self, error_message: String) {
        self.uploaded_at = Some(chrono::Utc::now().timestamp());
        self.status = AttachmentStatus::Failed;
        self.error_message = Some(error_message);
    }
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

    // ========================================================================
    // Attachment Type Tests
    // ========================================================================

    #[test]
    fn test_image_mime_type_from_extension_jpeg() {
        assert_eq!(
            ImageMimeType::from_extension("jpg"),
            Some(ImageMimeType::Jpeg)
        );
        assert_eq!(
            ImageMimeType::from_extension("jpeg"),
            Some(ImageMimeType::Jpeg)
        );
        assert_eq!(
            ImageMimeType::from_extension("JPG"),
            Some(ImageMimeType::Jpeg)
        );
        assert_eq!(
            ImageMimeType::from_extension("JPEG"),
            Some(ImageMimeType::Jpeg)
        );
    }

    #[test]
    fn test_image_mime_type_from_extension_png() {
        assert_eq!(
            ImageMimeType::from_extension("png"),
            Some(ImageMimeType::Png)
        );
        assert_eq!(
            ImageMimeType::from_extension("PNG"),
            Some(ImageMimeType::Png)
        );
    }

    #[test]
    fn test_image_mime_type_from_extension_gif() {
        assert_eq!(
            ImageMimeType::from_extension("gif"),
            Some(ImageMimeType::Gif)
        );
        assert_eq!(
            ImageMimeType::from_extension("GIF"),
            Some(ImageMimeType::Gif)
        );
    }

    #[test]
    fn test_image_mime_type_from_extension_webp() {
        assert_eq!(
            ImageMimeType::from_extension("webp"),
            Some(ImageMimeType::WebP)
        );
        assert_eq!(
            ImageMimeType::from_extension("WEBP"),
            Some(ImageMimeType::WebP)
        );
    }

    #[test]
    fn test_image_mime_type_from_extension_unsupported() {
        assert_eq!(ImageMimeType::from_extension("txt"), None);
        assert_eq!(ImageMimeType::from_extension("pdf"), None);
        assert_eq!(ImageMimeType::from_extension("mp4"), None);
        assert_eq!(ImageMimeType::from_extension(""), None);
    }

    #[test]
    fn test_image_mime_type_from_mime_str() {
        assert_eq!(
            ImageMimeType::from_mime_str("image/jpeg"),
            Some(ImageMimeType::Jpeg)
        );
        assert_eq!(
            ImageMimeType::from_mime_str("image/jpg"),
            Some(ImageMimeType::Jpeg)
        );
        assert_eq!(
            ImageMimeType::from_mime_str("image/png"),
            Some(ImageMimeType::Png)
        );
        assert_eq!(
            ImageMimeType::from_mime_str("image/gif"),
            Some(ImageMimeType::Gif)
        );
        assert_eq!(
            ImageMimeType::from_mime_str("image/webp"),
            Some(ImageMimeType::WebP)
        );
        assert_eq!(
            ImageMimeType::from_mime_str("IMAGE/JPEG"),
            Some(ImageMimeType::Jpeg)
        );
    }

    #[test]
    fn test_image_mime_type_from_mime_str_unsupported() {
        assert_eq!(ImageMimeType::from_mime_str("text/plain"), None);
        assert_eq!(ImageMimeType::from_mime_str("video/mp4"), None);
        assert_eq!(ImageMimeType::from_mime_str("application/pdf"), None);
    }

    #[test]
    fn test_image_mime_type_as_str() {
        assert_eq!(ImageMimeType::Jpeg.as_str(), "image/jpeg");
        assert_eq!(ImageMimeType::Png.as_str(), "image/png");
        assert_eq!(ImageMimeType::Gif.as_str(), "image/gif");
        assert_eq!(ImageMimeType::WebP.as_str(), "image/webp");
    }

    #[test]
    fn test_image_mime_type_extension() {
        assert_eq!(ImageMimeType::Jpeg.extension(), "jpg");
        assert_eq!(ImageMimeType::Png.extension(), "png");
        assert_eq!(ImageMimeType::Gif.extension(), "gif");
        assert_eq!(ImageMimeType::WebP.extension(), "webp");
    }

    #[test]
    fn test_image_mime_type_display() {
        assert_eq!(format!("{}", ImageMimeType::Jpeg), "image/jpeg");
        assert_eq!(format!("{}", ImageMimeType::Png), "image/png");
    }

    #[test]
    fn test_image_mime_type_serialization() {
        let mime = ImageMimeType::Jpeg;
        let json = serde_json::to_string(&mime).unwrap();
        assert_eq!(json, r#""Jpeg""#);

        let deserialized: ImageMimeType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, ImageMimeType::Jpeg);
    }

    #[test]
    fn test_attachment_status_display() {
        assert_eq!(format!("{}", AttachmentStatus::Pending), "pending");
        assert_eq!(format!("{}", AttachmentStatus::Uploaded), "uploaded");
        assert_eq!(format!("{}", AttachmentStatus::Failed), "failed");
    }

    #[test]
    fn test_attachment_status_serialization() {
        let status = AttachmentStatus::Uploaded;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, r#""Uploaded""#);

        let deserialized: AttachmentStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, AttachmentStatus::Uploaded);
    }

    #[test]
    fn test_attachment_new() {
        let attachment = Attachment::new(
            "post-123".to_string(),
            "/path/to/image.jpg".to_string(),
            ImageMimeType::Jpeg,
            1024,
            "abc123hash".to_string(),
            Some("A beautiful sunset".to_string()),
        );

        // Verify UUID format
        let uuid_result = uuid::Uuid::parse_str(&attachment.id);
        assert!(uuid_result.is_ok(), "Attachment ID should be a valid UUID");

        assert_eq!(attachment.post_id, "post-123");
        assert_eq!(attachment.file_path, "/path/to/image.jpg");
        assert_eq!(attachment.mime_type, ImageMimeType::Jpeg);
        assert_eq!(attachment.file_size, 1024);
        assert_eq!(attachment.file_hash, "abc123hash");
        assert_eq!(attachment.alt_text, Some("A beautiful sunset".to_string()));
        assert!(attachment.created_at > 1_600_000_000);
    }

    #[test]
    fn test_attachment_new_without_alt_text() {
        let attachment = Attachment::new(
            "post-456".to_string(),
            "/path/to/image.png".to_string(),
            ImageMimeType::Png,
            2048,
            "def456hash".to_string(),
            None,
        );

        assert_eq!(attachment.alt_text, None);
    }

    #[test]
    fn test_attachment_unique_ids() {
        let attachment1 = Attachment::new(
            "post-1".to_string(),
            "/path/1.jpg".to_string(),
            ImageMimeType::Jpeg,
            100,
            "hash1".to_string(),
            None,
        );
        let attachment2 = Attachment::new(
            "post-1".to_string(),
            "/path/2.jpg".to_string(),
            ImageMimeType::Jpeg,
            100,
            "hash2".to_string(),
            None,
        );

        assert_ne!(attachment1.id, attachment2.id);
    }

    #[test]
    fn test_attachment_serialization() {
        let attachment = Attachment {
            id: "attach-123".to_string(),
            post_id: "post-456".to_string(),
            file_path: "/path/to/image.gif".to_string(),
            mime_type: ImageMimeType::Gif,
            file_size: 4096,
            file_hash: "ghi789hash".to_string(),
            alt_text: Some("Animated GIF".to_string()),
            created_at: 1234567890,
        };

        let json = serde_json::to_string(&attachment).unwrap();
        let deserialized: Attachment = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, attachment.id);
        assert_eq!(deserialized.post_id, attachment.post_id);
        assert_eq!(deserialized.file_path, attachment.file_path);
        assert_eq!(deserialized.mime_type, attachment.mime_type);
        assert_eq!(deserialized.file_size, attachment.file_size);
        assert_eq!(deserialized.file_hash, attachment.file_hash);
        assert_eq!(deserialized.alt_text, attachment.alt_text);
        assert_eq!(deserialized.created_at, attachment.created_at);
    }

    #[test]
    fn test_attachment_upload_new_pending() {
        let upload =
            AttachmentUpload::new_pending("attach-123".to_string(), "mastodon".to_string());

        assert_eq!(upload.id, None);
        assert_eq!(upload.attachment_id, "attach-123");
        assert_eq!(upload.platform, "mastodon");
        assert_eq!(upload.platform_attachment_id, None);
        assert_eq!(upload.remote_url, None);
        assert_eq!(upload.uploaded_at, None);
        assert_eq!(upload.status, AttachmentStatus::Pending);
        assert_eq!(upload.error_message, None);
    }

    #[test]
    fn test_attachment_upload_mark_uploaded() {
        let mut upload =
            AttachmentUpload::new_pending("attach-123".to_string(), "mastodon".to_string());

        upload.mark_uploaded(
            "media-456".to_string(),
            Some("https://example.com/media/456".to_string()),
        );

        assert_eq!(upload.platform_attachment_id, Some("media-456".to_string()));
        assert_eq!(
            upload.remote_url,
            Some("https://example.com/media/456".to_string())
        );
        assert!(upload.uploaded_at.is_some());
        assert_eq!(upload.status, AttachmentStatus::Uploaded);
        assert_eq!(upload.error_message, None);
    }

    #[test]
    fn test_attachment_upload_mark_uploaded_without_url() {
        let mut upload =
            AttachmentUpload::new_pending("attach-123".to_string(), "mastodon".to_string());

        upload.mark_uploaded("media-789".to_string(), None);

        assert_eq!(upload.platform_attachment_id, Some("media-789".to_string()));
        assert_eq!(upload.remote_url, None);
        assert_eq!(upload.status, AttachmentStatus::Uploaded);
    }

    #[test]
    fn test_attachment_upload_mark_failed() {
        let mut upload =
            AttachmentUpload::new_pending("attach-123".to_string(), "nostr".to_string());

        upload.mark_failed("Network timeout".to_string());

        assert_eq!(upload.platform_attachment_id, None);
        assert!(upload.uploaded_at.is_some());
        assert_eq!(upload.status, AttachmentStatus::Failed);
        assert_eq!(upload.error_message, Some("Network timeout".to_string()));
    }

    #[test]
    fn test_attachment_upload_serialization() {
        let upload = AttachmentUpload {
            id: Some(42),
            attachment_id: "attach-abc".to_string(),
            platform: "nostr".to_string(),
            platform_attachment_id: Some("nip96-xyz".to_string()),
            remote_url: Some("https://nostr.build/image.jpg".to_string()),
            uploaded_at: Some(1234567890),
            status: AttachmentStatus::Uploaded,
            error_message: None,
        };

        let json = serde_json::to_string(&upload).unwrap();
        let deserialized: AttachmentUpload = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, upload.id);
        assert_eq!(deserialized.attachment_id, upload.attachment_id);
        assert_eq!(deserialized.platform, upload.platform);
        assert_eq!(
            deserialized.platform_attachment_id,
            upload.platform_attachment_id
        );
        assert_eq!(deserialized.remote_url, upload.remote_url);
        assert_eq!(deserialized.uploaded_at, upload.uploaded_at);
        assert_eq!(deserialized.status, upload.status);
        assert_eq!(deserialized.error_message, upload.error_message);
    }

    #[test]
    fn test_attachment_clone() {
        let attachment = Attachment::new(
            "post-123".to_string(),
            "/path/to/image.webp".to_string(),
            ImageMimeType::WebP,
            8192,
            "jkl012hash".to_string(),
            Some("WebP image".to_string()),
        );

        let cloned = attachment.clone();

        assert_eq!(attachment.id, cloned.id);
        assert_eq!(attachment.post_id, cloned.post_id);
        assert_eq!(attachment.file_path, cloned.file_path);
        assert_eq!(attachment.mime_type, cloned.mime_type);
    }

    #[test]
    fn test_attachment_upload_clone() {
        let upload =
            AttachmentUpload::new_pending("attach-test".to_string(), "mastodon".to_string());

        let cloned = upload.clone();

        assert_eq!(upload.attachment_id, cloned.attachment_id);
        assert_eq!(upload.platform, cloned.platform);
        assert_eq!(upload.status, cloned.status);
    }
}
