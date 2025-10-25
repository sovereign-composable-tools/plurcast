//! Integration tests for PlurcastService
//!
//! Tests the service layer as a whole, including interactions between services.

use libplurcast::service::{PlurcastService, posting::PostRequest, history::HistoryQuery, validation::ValidationRequest};
use tempfile::TempDir;
use libplurcast::Config;

/// Setup test service with temporary database
async fn setup_test_service() -> (PlurcastService, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    let config = Config {
        database: libplurcast::config::DatabaseConfig {
            path: db_path.to_str().unwrap().to_string(),
        },
        nostr: None,
        mastodon: None,
        bluesky: None,
        defaults: libplurcast::config::DefaultsConfig {
            platforms: vec![],
        },
        credentials: None,
    };
    
    let service = PlurcastService::from_config(config).await.unwrap();
    
    (service, temp_dir)
}

#[tokio::test]
async fn test_service_initialization() {
    let (_service, _temp_dir) = setup_test_service().await;
    
    // If we got here, initialization succeeded
    // No assertions needed - the test passes if setup doesn't panic
}

#[tokio::test]
async fn test_service_accessor_methods() {
    let (service, _temp_dir) = setup_test_service().await;
    
    // Test that all accessor methods return valid references
    let _posting = service.posting();
    let _history = service.history();
    let _draft = service.draft();
    let _validation = service.validation();
    
    // Test event subscription
    let mut _receiver = service.subscribe();
}

#[tokio::test]
async fn test_draft_to_publish_workflow() {
    let (service, _temp_dir) = setup_test_service().await;
    
    // Step 1: Create a draft
    let draft = service.draft().create("Test draft content".to_string()).await.unwrap();
    assert_eq!(draft.content, "Test draft content");
    
    // Step 2: List drafts
    let drafts = service.draft().list().await.unwrap();
    assert_eq!(drafts.len(), 1);
    assert_eq!(drafts[0].id, draft.id);
    
    // Step 3: Publish draft (no platforms configured)
    let response = service.draft().publish(&draft.id, vec![]).await.unwrap();
    assert!(!response.post_id.is_empty());
    assert_eq!(response.results.len(), 0); // No platforms means no results
    
    // Step 4: When no platforms are configured, overall_success is false (no results),
    // so draft will NOT be deleted. This is correct behavior - publishing with no
    // platforms is essentially a no-op.
    let result = service.draft().get(&draft.id).await.unwrap();
    assert!(result.is_some(), "Draft should still exist when no platforms are configured");
}

#[tokio::test]
async fn test_validation_before_posting() {
    let (service, _temp_dir) = setup_test_service().await;
    
    // Test validation for valid content
    let valid_request = ValidationRequest {
        content: "Hello world!".to_string(),
        platforms: vec!["nostr".to_string()],
    };
    let response = service.validation().validate(valid_request);
    assert!(response.valid);
    
    // Test validation for invalid content (empty)
    let invalid_request = ValidationRequest {
        content: "".to_string(),
        platforms: vec!["nostr".to_string()],
    };
    let response = service.validation().validate(invalid_request);
    assert!(!response.valid);
    
    // Test validation for content exceeding platform limits
    let long_content = "a".repeat(301); // Exceeds Bluesky's 300 char limit
    let bluesky_request = ValidationRequest {
        content: long_content,
        platforms: vec!["bluesky".to_string()],
    };
    let response = service.validation().validate(bluesky_request);
    assert!(!response.valid);
}

#[tokio::test]
async fn test_history_queries_after_posting() {
    let (service, _temp_dir) = setup_test_service().await;
    
    // Create some posts
    let request1 = PostRequest {
        content: "First post".to_string(),
        platforms: vec![],
        draft: true,
    };
    let response1 = service.posting().post(request1).await.unwrap();
    
    let request2 = PostRequest {
        content: "Second post".to_string(),
        platforms: vec![],
        draft: true,
    };
    let response2 = service.posting().post(request2).await.unwrap();
    
    // Query history
    let query = HistoryQuery {
        platform: None,
        status: None,
        since: None,
        until: None,
        search: None,
        limit: Some(10),
        offset: None,
    };
    let posts = service.history().list_posts(query).await.unwrap();
    
    assert_eq!(posts.len(), 2);
    
    // Get specific post
    let post = service.history().get_post(&response1.post_id).await.unwrap();
    assert!(post.is_some());
    let post = post.unwrap();
    assert_eq!(post.post.content, "First post");
    
    // Get stats
    let stats_query = HistoryQuery {
        platform: None,
        status: None,
        since: None,
        until: None,
        search: None,
        limit: Some(100),
        offset: None,
    };
    let stats = service.history().get_stats(stats_query).await.unwrap();
    assert_eq!(stats.total_posts, 2);
}

#[tokio::test]
async fn test_event_subscription() {
    let (service, _temp_dir) = setup_test_service().await;
    
    // Subscribe to events
    let mut receiver = service.subscribe();
    
    // Create a post (in draft mode, no actual events will be emitted for platforms)
    let request = PostRequest {
        content: "Test post for events".to_string(),
        platforms: vec![],
        draft: true,
    };
    
    let response = service.posting().post(request).await.unwrap();
    assert!(!response.post_id.is_empty());
    
    // Note: Since we're posting in draft mode with no platforms,
    // no events will be emitted. This test verifies that:
    // 1. We can subscribe to the event bus
    // 2. The service doesn't panic when posting with an active subscription
    // 3. The subscription mechanism works (even if no events are sent)
    
    // Try to receive with timeout (should timeout since no events in draft mode)
    let receive_result = tokio::time::timeout(
        std::time::Duration::from_millis(100),
        receiver.recv()
    ).await;
    
    // Should timeout (no events for draft mode)
    assert!(receive_result.is_err(), "Should timeout - no events in draft mode");
}

#[tokio::test]
async fn test_validation_with_convenience_method() {
    let (service, _temp_dir) = setup_test_service().await;
    
    // Test is_valid convenience method
    assert!(service.validation().is_valid("Valid content", &vec!["nostr".to_string()]));
    assert!(!service.validation().is_valid("", &vec!["nostr".to_string()]));
    
    let long_content = "a".repeat(301);
    assert!(!service.validation().is_valid(&long_content, &vec!["bluesky".to_string()]));
}

#[tokio::test]
async fn test_count_posts() {
    let (service, _temp_dir) = setup_test_service().await;
    
    // Initially should have 0 posts
    let query = HistoryQuery {
        platform: None,
        status: None,
        since: None,
        until: None,
        search: None,
        limit: Some(100),
        offset: None,
    };
    let count = service.history().count_posts(query.clone()).await.unwrap();
    assert_eq!(count, 0);
    
    // Create a post
    let request = PostRequest {
        content: "Counted post".to_string(),
        platforms: vec![],
        draft: true,
    };
    service.posting().post(request).await.unwrap();
    
    // Should now have 1 post
    let count = service.history().count_posts(query).await.unwrap();
    assert_eq!(count, 1);
}
