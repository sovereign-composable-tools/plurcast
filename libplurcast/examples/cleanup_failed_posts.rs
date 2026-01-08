//! Helper tool to view and clean up failed posts
//!
//! Usage:
//!   cargo run --example cleanup_failed_posts -- list    # Show failed posts
//!   cargo run --example cleanup_failed_posts -- delete  # Delete all failed posts

use libplurcast::{Config, Database};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("list");

    // Load config and connect to database
    let config = Config::load()?;
    let db = Database::new(&config.database.path).await?;

    match command {
        "list" => {
            println!("=== All Posts by Status ===\n");

            // Get all posts (runtime query to avoid compile-time DATABASE_URL requirement)
            let pool = db.pool();

            let counts: Vec<(String, i64)> =
                sqlx::query_as("SELECT status, COUNT(*) as count FROM posts GROUP BY status")
                    .fetch_all(pool)
                    .await?;

            for (status, count) in counts {
                println!("{}: {}", status, count);
            }

            println!("\n=== Failed Posts ===\n");
            let failed = db.get_failed_posts().await?;

            if failed.is_empty() {
                println!("No failed posts found.");
            } else {
                for post in &failed {
                    let preview = if post.content.len() > 60 {
                        format!("{}...", &post.content[..60])
                    } else {
                        post.content.clone()
                    };
                    println!("{} | {}", post.id, preview);
                }

                println!("\nTotal failed posts: {}", failed.len());
                println!("\nTo delete all failed posts, run:");
                println!("  cargo run --example cleanup_failed_posts -- delete");
            }
        }
        "delete" => {
            let failed = db.get_failed_posts().await?;
            let count = failed.len();

            if count == 0 {
                println!("No failed posts to delete.");
                return Ok(());
            }

            println!("Deleting {} failed post(s)...", count);

            for post in failed {
                db.delete_post(&post.id).await?;
                println!("  Deleted: {}", post.id);
            }

            println!("\nDone! Deleted {} post(s).", count);
        }
        "stats" => {
            let pool = db.pool();

            println!("=== Database Statistics ===\n");

            let counts: Vec<(String, i64)> =
                sqlx::query_as("SELECT status, COUNT(*) as count FROM posts GROUP BY status")
                    .fetch_all(pool)
                    .await?;

            println!("Posts by status:");
            for (status, count) in counts {
                println!("  {}: {}", status, count);
            }

            let scheduled = db.get_scheduled_posts().await?;
            println!("\nScheduled posts: {}", scheduled.len());

            let failed = db.get_failed_posts().await?;
            println!("Failed posts: {}", failed.len());
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            eprintln!("Usage: cleanup_failed_posts [list|delete|stats]");
            std::process::exit(1);
        }
    }

    Ok(())
}
