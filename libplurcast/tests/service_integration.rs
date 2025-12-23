//! Integration tests for PlurcastService
//!
//! Tests the service layer as a whole, including interactions between services.

use std::collections::HashMap;

use libplurcast::service::{
    history::HistoryQuery, posting::PostRequest, validation::ValidationRequest, PlurcastService,
};
use libplurcast::Config;
use tempfile::TempDir;

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
        ssb: None,
        defaults: libplurcast::config::DefaultsConfig { platforms: vec![] },
        credentials: None,
        scheduling: None,
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
    let draft = service
        .draft()
        .create("Test draft content".to_string())
        .await
        .unwrap();
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
    assert!(
        result.is_some(),
        "Draft should still exist when no platforms are configured"
    );
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
    let long_content = "a".repeat(501); // Exceeds Mastodon's 500 char limit
    let mastodon_request = ValidationRequest {
        content: long_content,
        platforms: vec!["mastodon".to_string()],
    };
    let response = service.validation().validate(mastodon_request);
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
        account: None,
        scheduled_at: None,
        nostr_pow: None,
        nostr_21e8: false,
        reply_to: HashMap::new(),
    };
    let response1 = service.posting().post(request1).await.unwrap();

    let request2 = PostRequest {
        content: "Second post".to_string(),
        platforms: vec![],
        draft: true,
        account: None,
        scheduled_at: None,
        nostr_pow: None,
        nostr_21e8: false,
        reply_to: HashMap::new(),
    };
    let _response2 = service.posting().post(request2).await.unwrap();

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
    let post = service
        .history()
        .get_post(&response1.post_id)
        .await
        .unwrap();
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
        account: None,
        scheduled_at: None,
        nostr_pow: None,
        nostr_21e8: false,
        reply_to: HashMap::new(),
    };

    let response = service.posting().post(request).await.unwrap();
    assert!(!response.post_id.is_empty());

    // Note: Since we're posting in draft mode with no platforms,
    // no events will be emitted. This test verifies that:
    // 1. We can subscribe to the event bus
    // 2. The service doesn't panic when posting with an active subscription
    // 3. The subscription mechanism works (even if no events are sent)

    // Try to receive with timeout (should timeout since no events in draft mode)
    let receive_result =
        tokio::time::timeout(std::time::Duration::from_millis(100), receiver.recv()).await;

    // Should timeout (no events for draft mode)
    assert!(
        receive_result.is_err(),
        "Should timeout - no events in draft mode"
    );
}

#[tokio::test]
async fn test_validation_with_convenience_method() {
    let (service, _temp_dir) = setup_test_service().await;

    // Test is_valid convenience method
    assert!(service
        .validation()
        .is_valid("Valid content", &vec!["nostr".to_string()]));
    assert!(!service
        .validation()
        .is_valid("", &vec!["nostr".to_string()]));

    let long_content = "a".repeat(501);
    assert!(!service
        .validation()
        .is_valid(&long_content, &vec!["mastodon".to_string()]));
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
        account: None,
        scheduled_at: None,
        nostr_pow: None,
        nostr_21e8: false,
        reply_to: HashMap::new(),
    };
    service.posting().post(request).await.unwrap();

    // Should now have 1 post
    let count = service.history().count_posts(query).await.unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn test_scheduled_post_workflow() {
    let (service, _temp_dir) = setup_test_service().await;

    // Step 1: Schedule a post for 1 second in the future
    let scheduled_time = chrono::Utc::now().timestamp() + 1;
    let request = PostRequest {
        content: "Scheduled test post".to_string(),
        platforms: vec![],
        draft: false,
        account: None,
        scheduled_at: Some(scheduled_time),
        nostr_pow: None,
        nostr_21e8: false,
        reply_to: HashMap::new(),
    };

    let response = service.posting().post(request).await.unwrap();
    assert!(response.overall_success);
    let post_id = response.post_id.clone();

    // Step 2: Verify post was created with Scheduled status
    let post_result = service.history().get_post(&post_id).await.unwrap();
    assert!(post_result.is_some());
    let post_detail = post_result.unwrap();
    let post = post_detail.post;
    assert_eq!(post.status, libplurcast::PostStatus::Scheduled);
    assert_eq!(post.scheduled_at, Some(scheduled_time));

    // Step 3: Wait for scheduled time
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // Step 4: Simulate what plur-send daemon does - call post_scheduled()
    let platforms = vec![]; // No actual platforms for this test
    let post_scheduled_response = service
        .posting()
        .post_scheduled(post, platforms, None)
        .await
        .unwrap();

    // Step 5: Verify post status changed from Scheduled to Posted (or Failed if no platforms)
    let updated_post_result = service.history().get_post(&post_id).await.unwrap();
    assert!(updated_post_result.is_some());
    let updated_post = updated_post_result.unwrap().post;

    // With no platforms, status should be Failed
    assert_eq!(updated_post.status, libplurcast::PostStatus::Failed);

    // Verify the post_id remains the same (no duplicate was created)
    assert_eq!(post_scheduled_response.post_id, post_id);
}

#[tokio::test]
async fn test_scheduled_post_no_duplicate_creation() {
    use libplurcast::PostStatus;

    let (service, _temp_dir) = setup_test_service().await;

    // Schedule a post
    let scheduled_time = chrono::Utc::now().timestamp() + 1;
    let request = PostRequest {
        content: "Test duplicate detection".to_string(),
        platforms: vec![],
        draft: false,
        account: None,
        scheduled_at: Some(scheduled_time),
        nostr_pow: None,
        nostr_21e8: false,
        reply_to: HashMap::new(),
    };

    let response = service.posting().post(request).await.unwrap();
    let original_post_id = response.post_id.clone();

    // Get the original post
    let original_post = service
        .history()
        .get_post(&original_post_id)
        .await
        .unwrap()
        .unwrap()
        .post;

    // Wait for scheduled time
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // Call post_scheduled (simulating daemon)
    let _ = service
        .posting()
        .post_scheduled(original_post.clone(), vec![], None)
        .await
        .unwrap();

    // Query database for all posts with this content
    let query = HistoryQuery {
        platform: None,
        status: None,
        since: None,
        until: None,
        search: Some("Test duplicate detection".to_string()),
        limit: Some(100),
        offset: None,
    };
    let posts = service.history().list_posts(query).await.unwrap();

    // Should only be 1 post (not duplicated)
    assert_eq!(
        posts.len(),
        1,
        "post_scheduled should not create duplicate posts"
    );
    assert_eq!(posts[0].post.id, original_post_id);

    // Verify status changed but ID remained same
    let final_post = service
        .history()
        .get_post(&original_post_id)
        .await
        .unwrap()
        .unwrap()
        .post;
    assert_ne!(final_post.status, PostStatus::Scheduled);
    assert_eq!(final_post.id, original_post_id);
}
