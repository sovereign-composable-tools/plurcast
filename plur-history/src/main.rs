use anyhow::{Context, Result};
use clap::Parser;
use libplurcast::service::{history::HistoryQuery as ServiceHistoryQuery, PlurcastService};
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(name = "plur-history")]
#[command(version, about = "Query local posting history")]
#[command(
    long_about = r#"Query local posting history with filtering and formatting options.

EXAMPLES:
    # Show last 20 posts (default)
    plur-history

    # Show more posts
    plur-history --limit 50

    # Filter by platform
    plur-history --platform nostr
    plur-history --platform mastodon
    plur-history --platform ssb

    # Filter by date range
    plur-history --since "2025-10-01" --until "2025-10-05"
    plur-history --since "2025-10-01T09:00:00Z"

    # Search content
    plur-history --search "rust"
    plur-history --search "announcement"

    # Combine filters
    plur-history --platform nostr --since "2025-10-01" --limit 10

    # JSON output for scripting
    plur-history --format json
    plur-history --format json | jq '.[] | .content'
    plur-history --format json | jq '.[] | select(.platforms[].success == false)'

    # JSONL output (one JSON object per line)
    plur-history --format jsonl

    # Export to CSV for analysis
    plur-history --format csv > posts.csv
    plur-history --format csv | cut -d, -f3 | sort | uniq -c

    # Unix composability examples
    plur-history --format json | jq -r '.[] | .platforms[] | select(.platform == "nostr") | .platform_post_id'
    plur-history --platform nostr --format csv | grep ",true,"

OUTPUT FORMATS:
    text  - Human-readable text with timestamps and platform status (default)
    json  - JSON array (complete data structure)
    jsonl - JSON lines, one object per line (streaming-friendly)
    csv   - CSV with headers (spreadsheet-compatible)

EXIT CODES:
    0 - Success (including empty results)
    1 - Error (database not found, query failed, etc.)
"#
)]
struct Args {
    /// Filter by platform (nostr, mastodon, ssb)
    #[arg(short, long, value_name = "PLATFORM")]
    #[arg(help = "Filter results to specific platform (nostr, mastodon, or ssb)")]
    platform: Option<String>,

    /// Filter posts since this date (Unix timestamp or ISO 8601 format)
    #[arg(long, value_name = "DATE")]
    #[arg(help = "Show posts since this date (Unix timestamp, YYYY-MM-DD, or ISO 8601 format)")]
    since: Option<String>,

    /// Filter posts until this date (Unix timestamp or ISO 8601 format)
    #[arg(long, value_name = "DATE")]
    #[arg(help = "Show posts until this date (Unix timestamp, YYYY-MM-DD, or ISO 8601 format)")]
    until: Option<String>,

    /// Search posts by content
    #[arg(short, long, value_name = "TERM")]
    #[arg(help = "Search posts containing this text (case-insensitive substring match)")]
    search: Option<String>,

    /// Maximum number of posts to return
    #[arg(short, long, default_value = "20", value_name = "N")]
    #[arg(help = "Maximum number of posts to return (default: 20)")]
    limit: usize,

    /// Output format
    #[arg(short, long, default_value = "text", value_name = "FORMAT")]
    #[arg(
        help = "Output format: text (human-readable), json (array), jsonl (streaming), or csv (spreadsheet)"
    )]
    #[arg(value_parser = ["text", "json", "jsonl", "csv"])]
    format: String,

    /// Verbose output (show additional metadata like SSB sequence numbers and hashes)
    #[arg(short, long)]
    #[arg(help = "Show additional metadata (SSB sequence numbers, message hashes, etc.)")]
    verbose: bool,
}

/// Query parameters for history
#[derive(Debug)]
struct HistoryQuery {
    platform: Option<String>,
    since: Option<i64>,
    until: Option<i64>,
    search: Option<String>,
    limit: usize,
}

/// A single post with its platform results
#[derive(Debug, Serialize, Deserialize)]
struct HistoryEntry {
    post_id: String,
    content: String,
    created_at: i64,
    platforms: Vec<PlatformStatus>,
}

/// Status of a post on a specific platform
#[derive(Debug, Serialize, Deserialize)]
struct PlatformStatus {
    platform: String,
    success: bool,
    platform_post_id: Option<String>,
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sequence: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message_hash: Option<String>,
}

/// Query history using service layer
async fn query_history(
    service: &PlurcastService,
    query: &HistoryQuery,
) -> Result<Vec<HistoryEntry>> {
    // Map CLI query to service layer query
    let service_query = ServiceHistoryQuery {
        platform: query.platform.clone(),
        status: None, // No status filter in CLI
        since: query
            .since
            .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0)),
        until: query
            .until
            .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0)),
        search: query.search.clone(),
        limit: Some(query.limit),
        offset: None,
    };

    // Query via service layer
    let posts_with_records = service
        .history()
        .list_posts(service_query)
        .await
        .context("Failed to query history")?;

    // Map service layer types to CLI types
    let mut entries = Vec::new();
    for pwr in posts_with_records {
        let platforms = pwr
            .records
            .iter()
            .map(|record| {
                // Extract SSB-specific metadata from platform_post_id if available
                let (sequence, message_hash): (Option<i64>, Option<String>) = if record.platform == "ssb" {
                    if let Some(ref post_id) = record.platform_post_id {
                        // SSB message IDs are in format: ssb:%<hash>
                        // For now, extract hash from the ID
                        // Sequence number would come from database metadata (added in earlier tasks)
                        let hash = if post_id.starts_with("ssb:%") {
                            Some(post_id[5..].to_string())
                        } else {
                            Some(post_id.clone())
                        };
                        (None, hash) // Sequence would be populated from DB metadata
                    } else {
                        (None, None)
                    }
                } else {
                    (None, None)
                };

                PlatformStatus {
                    platform: record.platform.clone(),
                    success: record.success,
                    platform_post_id: record.platform_post_id.clone(),
                    error: record.error_message.clone(),
                    sequence,
                    message_hash,
                }
            })
            .collect();

        entries.push(HistoryEntry {
            post_id: pwr.post.id,
            content: pwr.post.content,
            created_at: pwr.post.created_at,
            platforms,
        });
    }

    Ok(entries)
}

/// Parse date string to Unix timestamp
fn parse_date(date_str: &str) -> Result<i64> {
    // Try parsing as Unix timestamp first
    if let Ok(timestamp) = date_str.parse::<i64>() {
        return Ok(timestamp);
    }

    // Try parsing as ISO 8601
    let dt = chrono::DateTime::parse_from_rfc3339(date_str)
        .or_else(|_| {
            // Try parsing as date only (YYYY-MM-DD)
            chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc().fixed_offset())
        })
        .context(format!("Invalid date format: {}. Use Unix timestamp or ISO 8601 (YYYY-MM-DD or YYYY-MM-DDTHH:MM:SSZ)", date_str))?;

    Ok(dt.timestamp())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();

    let args = Args::parse();

    tracing::debug!("plur-history started with args: {:?}", args);

    // Initialize service layer
    let service = PlurcastService::new()
        .await
        .context("Failed to initialize service. Have you posted anything yet?")?;

    // Parse date arguments
    let since = if let Some(ref since_str) = args.since {
        Some(parse_date(since_str)?)
    } else {
        None
    };

    let until = if let Some(ref until_str) = args.until {
        Some(parse_date(until_str)?)
    } else {
        None
    };

    // Build query
    let query = HistoryQuery {
        platform: args.platform,
        since,
        until,
        search: args.search,
        limit: args.limit,
    };

    // Execute query
    let entries = query_history(&service, &query)
        .await
        .context("Failed to query history")?;

    // Output results based on format
    match args.format.as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&entries)?;
            println!("{}", json);
        }
        "jsonl" => {
            for entry in entries {
                let json = serde_json::to_string(&entry)?;
                println!("{}", json);
            }
        }
        "csv" => {
            // CSV format: post_id,timestamp,platform,success,platform_post_id,error,content
            println!("post_id,timestamp,platform,success,platform_post_id,error,content");
            for entry in entries {
                for platform in &entry.platforms {
                    let success = if platform.success { "true" } else { "false" };
                    let platform_post_id = platform.platform_post_id.as_deref().unwrap_or("");
                    let error = platform.error.as_deref().unwrap_or("");
                    let content = entry.content.replace('"', "\"\""); // Escape quotes

                    println!(
                        "{},{},{},{},{},{},\"{}\"",
                        entry.post_id,
                        entry.created_at,
                        platform.platform,
                        success,
                        platform_post_id,
                        error,
                        content
                    );
                }
            }
        }
        "text" => {
            // Human-readable text format
            if entries.is_empty() {
                // Empty results - output nothing and exit 0
                std::process::exit(0);
            }

            for entry in entries {
                // Format timestamp
                let dt = chrono::DateTime::from_timestamp(entry.created_at, 0)
                    .unwrap_or_else(chrono::Utc::now);
                let timestamp = dt.format("%Y-%m-%d %H:%M:%S");

                // Truncate content for preview
                let content_preview = if entry.content.len() > 60 {
                    format!("{}...", &entry.content[..60])
                } else {
                    entry.content.clone()
                };

                println!("{} | {} | {}", timestamp, entry.post_id, content_preview);

                // Show platform results
                for platform in &entry.platforms {
                    let symbol = if platform.success { "✓" } else { "✗" };
                    if let Some(ref post_id) = platform.platform_post_id {
                        println!("  {} {}: {}", symbol, platform.platform, post_id);
                        
                        // Show SSB-specific metadata in verbose mode
                        if args.verbose && platform.platform == "ssb" {
                            if let Some(seq) = platform.sequence {
                                println!("    Sequence: {}", seq);
                            }
                            if let Some(ref hash) = platform.message_hash {
                                println!("    Hash: {}", hash);
                            }
                        }
                    } else if let Some(ref error) = platform.error {
                        println!("  {} {}: {}", symbol, platform.platform, error);
                    } else {
                        println!("  {} {}", symbol, platform.platform);
                    }
                }
                println!(); // Blank line between entries
            }
        }
        _ => {
            eprintln!(
                "Error: Invalid format '{}'. Valid formats: text, json, jsonl, csv",
                args.format
            );
            std::process::exit(1);
        }
    }

    Ok(())
}
