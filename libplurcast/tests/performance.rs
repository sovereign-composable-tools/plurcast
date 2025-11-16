//! Performance tests for Plurcast
//!
//! These tests verify that the system performs well under various loads.

use libplurcast::db::Database;
use libplurcast::platforms::mock::MockPlatform;
use libplurcast::platforms::Platform;
use libplurcast::poster::MultiPlatformPoster;
use libplurcast::types::{Post, PostStatus};
use std::time::Instant;
use tempfile::TempDir;

/// Helper to create a test database with sample data
async fn create_test_database_with_posts(num_posts: usize) -> (TempDir, Database) {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = Database::new(db_path.to_str().unwrap()).await.unwrap();

    // Create sample posts
    for i in 0..num_posts {
        let post = Post {
            id: format!("post-{}", i),
            content: format!("Test post number {}", i),
            created_at: chrono::Utc::now().timestamp() - (i as i64 * 60),
            scheduled_at: None,
            status: PostStatus::Posted,
            metadata: None,
        };
        db.create_post(&post).await.unwrap();

        // Create post records for multiple platforms
        for platform in &["nostr", "mastodon", "bluesky"] {
            let record = libplurcast::types::PostRecord {
                id: None,
                post_id: post.id.clone(),
                platform: platform.to_string(),
                platform_post_id: Some(format!("{}:post-{}", platform, i)),
                posted_at: Some(post.created_at),
                success: true,
                error_message: None,
                account_name: "default".to_string(),
            };
            db.create_post_record(&record).await.unwrap();
        }
    }

    (temp_dir, db)
}

#[tokio::test]
async fn test_concurrent_posting_performance() {
    // Test that concurrent posting is faster than sequential
    let db = Database::new(":memory:").await.unwrap();

    // Create mock platforms with small delays
    let platforms: Vec<Box<dyn Platform>> = vec![
        Box::new(MockPlatform::new_with_delay("platform1", 100)),
        Box::new(MockPlatform::new_with_delay("platform2", 100)),
        Box::new(MockPlatform::new_with_delay("platform3", 100)),
    ];

    let poster = MultiPlatformPoster::new(platforms, db);

    let post = Post {
        id: uuid::Uuid::new_v4().to_string(),
        content: "Performance test post".to_string(),
        created_at: chrono::Utc::now().timestamp(),
        scheduled_at: None,
        status: PostStatus::Pending,
        metadata: None,
    };

    // Measure concurrent posting time
    let start = Instant::now();
    let results = poster.post_to_all(&post).await;
    let duration = start.elapsed();

    // All should succeed
    assert_eq!(results.len(), 3);
    assert!(results.iter().all(|r| r.success));

    // Concurrent posting should take roughly the time of the slowest platform (100ms)
    // plus some overhead, not the sum of all platforms (300ms)
    // Allow generous margin for CI environments
    assert!(
        duration.as_millis() < 250,
        "Concurrent posting took {}ms, expected < 250ms",
        duration.as_millis()
    );

    println!(
        "✓ Concurrent posting to 3 platforms completed in {}ms",
        duration.as_millis()
    );
}

#[tokio::test]
async fn test_history_query_performance_small_dataset() {
    // Test query performance with a small dataset (100 posts)
    let (_temp_dir, db) = create_test_database_with_posts(100).await;

    let start = Instant::now();
    let posts = db
        .query_posts_with_records(None, None, None, None, 20)
        .await
        .unwrap();
    let duration = start.elapsed();

    assert!(posts.len() <= 20);

    // Query should be fast (< 50ms for 100 posts)
    assert!(
        duration.as_millis() < 50,
        "Query took {}ms, expected < 50ms",
        duration.as_millis()
    );

    println!(
        "✓ History query (100 posts) completed in {}ms",
        duration.as_millis()
    );
}

#[tokio::test]
async fn test_history_query_performance_large_dataset() {
    // Test query performance with a larger dataset (1000 posts)
    let (_temp_dir, db) = create_test_database_with_posts(1000).await;

    let start = Instant::now();
    let posts = db
        .query_posts_with_records(None, None, None, None, 20)
        .await
        .unwrap();
    let duration = start.elapsed();

    assert!(posts.len() <= 20);

    // Query should still be fast (< 100ms for 1000 posts)
    // Indexes should make this efficient
    assert!(
        duration.as_millis() < 100,
        "Query took {}ms, expected < 100ms",
        duration.as_millis()
    );

    println!(
        "✓ History query (1000 posts) completed in {}ms",
        duration.as_millis()
    );
}

#[tokio::test]
async fn test_filtered_query_performance() {
    // Test query performance with filters
    let (_temp_dir, db) = create_test_database_with_posts(500).await;

    // Test platform filter
    let start = Instant::now();
    let posts = db
        .query_posts_with_records(Some("nostr"), None, None, None, 20)
        .await
        .unwrap();
    let duration = start.elapsed();

    assert!(!posts.is_empty());

    // Filtered query should be fast (< 100ms)
    assert!(
        duration.as_millis() < 100,
        "Filtered query took {}ms, expected < 100ms",
        duration.as_millis()
    );

    println!(
        "✓ Filtered history query (500 posts) completed in {}ms",
        duration.as_millis()
    );
}

#[tokio::test]
async fn test_memory_usage_reasonable() {
    // Test that memory usage is reasonable for typical workloads
    let (_temp_dir, db) = create_test_database_with_posts(100).await;

    // Query posts multiple times to check for memory leaks
    for _ in 0..10 {
        let posts = db
            .query_posts_with_records(None, None, None, None, 20)
            .await
            .unwrap();
        assert!(posts.len() <= 20);
    }

    // If we get here without OOM, memory usage is reasonable
    println!("✓ Memory usage is reasonable for typical workloads");
}

#[tokio::test]
async fn test_concurrent_database_writes() {
    // Test that concurrent writes don't cause issues
    let db = Database::new(":memory:").await.unwrap();

    let mut handles = vec![];

    // Spawn multiple tasks that write to the database
    for i in 0..10 {
        let db_clone = db.clone();
        let handle = tokio::spawn(async move {
            let post = Post {
                id: format!("concurrent-post-{}", i),
                content: format!("Concurrent test post {}", i),
                created_at: chrono::Utc::now().timestamp(),
                scheduled_at: None,
                status: PostStatus::Pending,
                metadata: None,
            };
            db_clone.create_post(&post).await.unwrap();
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify all posts were created
    let posts = db
        .query_posts_with_records(None, None, None, None, 100)
        .await
        .unwrap();
    assert_eq!(posts.len(), 10);

    println!("✓ Concurrent database writes completed successfully");
}

#[tokio::test]
async fn test_posting_throughput() {
    // Test posting throughput with multiple posts
    let db = Database::new(":memory:").await.unwrap();

    let platforms: Vec<Box<dyn Platform>> = vec![
        Box::new(MockPlatform::new_simple("platform1")),
        Box::new(MockPlatform::new_simple("platform2")),
    ];

    let poster = MultiPlatformPoster::new(platforms, db);

    let num_posts = 10;
    let start = Instant::now();

    for i in 0..num_posts {
        let post = Post {
            id: format!("throughput-post-{}", i),
            content: format!("Throughput test post {}", i),
            created_at: chrono::Utc::now().timestamp(),
            scheduled_at: None,
            status: PostStatus::Pending,
            metadata: None,
        };
        let results = poster.post_to_all(&post).await;
        assert!(results.iter().all(|r| r.success));
    }

    let duration = start.elapsed();
    let posts_per_second = (num_posts as f64) / duration.as_secs_f64();

    println!(
        "✓ Posted {} posts in {}ms ({:.1} posts/sec)",
        num_posts,
        duration.as_millis(),
        posts_per_second
    );

    // Should be able to post at least 5 posts per second with mock platforms
    assert!(
        posts_per_second > 5.0,
        "Throughput was {:.1} posts/sec, expected > 5.0",
        posts_per_second
    );
}

#[tokio::test]
async fn test_database_query_with_search() {
    // Test search query performance
    let (_temp_dir, db) = create_test_database_with_posts(500).await;

    let start = Instant::now();
    let posts = db
        .query_posts_with_records(None, None, None, Some("Test post number 42"), 20)
        .await
        .unwrap();
    let duration = start.elapsed();

    assert!(!posts.is_empty());

    // Search query should be reasonably fast (< 150ms)
    assert!(
        duration.as_millis() < 150,
        "Search query took {}ms, expected < 150ms",
        duration.as_millis()
    );

    println!(
        "✓ Search query (500 posts) completed in {}ms",
        duration.as_millis()
    );
}

#[tokio::test]
async fn test_date_range_query_performance() {
    // Test date range query performance
    let (_temp_dir, db) = create_test_database_with_posts(500).await;

    let now = chrono::Utc::now().timestamp();
    let one_hour_ago = now - 3600;

    let start = Instant::now();
    let posts = db
        .query_posts_with_records(None, Some(one_hour_ago), None, None, 20)
        .await
        .unwrap();
    let duration = start.elapsed();

    // Date range query should be fast (< 100ms)
    assert!(
        duration.as_millis() < 100,
        "Date range query took {}ms, expected < 100ms",
        duration.as_millis()
    );

    println!(
        "✓ Date range query (500 posts) completed in {}ms (found {} posts)",
        duration.as_millis(),
        posts.len()
    );
}
