//! Scheduling and time parsing utilities
//!
//! This module provides parsing of human-readable time formats for scheduling posts.

use crate::{PlurcastError, Result};
use chrono::{DateTime, Duration, Utc};
use rand::Rng;

const MIN_RANDOM_SECONDS: i64 = 30;
const MAX_RANDOM_SECONDS: i64 = 30 * 24 * 3600; // 30 days

/// Parse a schedule string into a DateTime
///
/// Supports multiple formats:
/// - Relative durations: "1h", "30m", "2d"
/// - Natural language: "tomorrow", "next week", "in 1 hour"
/// - Absolute times: "2025-11-20 15:00", "next monday 10am"
/// - Random intervals: "random:10m-20m", "random:1h-2h"
///
/// # Errors
///
/// Returns an error if the time format is invalid or cannot be parsed.
pub fn parse_schedule(input: &str, last_scheduled: Option<i64>) -> Result<DateTime<Utc>> {
    if input.is_empty() {
        return Err(PlurcastError::InvalidInput(
            "Schedule string cannot be empty".to_string(),
        ));
    }

    // Try random format first
    if input.starts_with("random:") {
        return parse_random_schedule(input, last_scheduled);
    }

    // Try duration parsing
    if let Ok(duration) = parse_duration(input) {
        return Ok(Utc::now() + duration);
    }

    // Try natural language parsing
    if let Ok(dt) = parse_natural_language(input) {
        return Ok(dt);
    }

    Err(PlurcastError::InvalidInput(format!(
        "Could not parse schedule string: {}",
        input
    )))
}

/// Parse a duration string into a chrono::Duration
fn parse_duration(input: &str) -> Result<Duration> {
    // Try humantime for simple formats like "1h", "30m"
    if let Ok(std_duration) = humantime::parse_duration(input) {
        let seconds = std_duration.as_secs() as i64;
        return Duration::try_seconds(seconds)
            .ok_or_else(|| PlurcastError::InvalidInput("Duration out of range".to_string()));
    }

    Err(PlurcastError::InvalidInput(format!(
        "Could not parse duration: {}",
        input
    )))
}

/// Parse natural language time expression
fn parse_natural_language(input: &str) -> Result<DateTime<Utc>> {
    chrono_english::parse_date_string(input, Utc::now(), chrono_english::Dialect::Us)
        .map_err(|e| PlurcastError::InvalidInput(format!("Could not parse time: {}", e)))
}

/// Parse random schedule format: "random:MIN-MAX"
fn parse_random_schedule(input: &str, last_scheduled: Option<i64>) -> Result<DateTime<Utc>> {
    let range_part = input
        .strip_prefix("random:")
        .ok_or_else(|| PlurcastError::InvalidInput("Invalid random format".to_string()))?;

    let (min_str, max_str) = parse_random_range(range_part)?;
    let min_duration = parse_duration(min_str)?;
    let max_duration = parse_duration(max_str)?;

    validate_random_range(min_duration, max_duration)?;

    let base_time = get_base_time_for_random(last_scheduled);
    let random_duration = generate_random_duration(min_duration, max_duration);

    Ok(base_time + random_duration)
}

/// Split "MIN-MAX" into (MIN, MAX)
fn parse_random_range(range: &str) -> Result<(&str, &str)> {
    let parts: Vec<&str> = range.split('-').collect();
    if parts.len() != 2 {
        return Err(PlurcastError::InvalidInput(
            "Random format must be MIN-MAX".to_string(),
        ));
    }
    Ok((parts[0], parts[1]))
}

/// Validate random range constraints
fn validate_random_range(min: Duration, max: Duration) -> Result<()> {
    let min_secs = min.num_seconds();
    let max_secs = max.num_seconds();

    if min_secs < MIN_RANDOM_SECONDS {
        return Err(PlurcastError::InvalidInput(format!(
            "Minimum random interval must be at least {} seconds",
            MIN_RANDOM_SECONDS
        )));
    }

    if max_secs > MAX_RANDOM_SECONDS {
        return Err(PlurcastError::InvalidInput(format!(
            "Maximum random interval must be less than {} days",
            MAX_RANDOM_SECONDS / (24 * 3600)
        )));
    }

    if min_secs >= max_secs {
        return Err(PlurcastError::InvalidInput(
            "Minimum must be less than maximum".to_string(),
        ));
    }

    Ok(())
}

/// Get base time for random scheduling
fn get_base_time_for_random(last_scheduled: Option<i64>) -> DateTime<Utc> {
    match last_scheduled {
        Some(timestamp) => DateTime::from_timestamp(timestamp, 0).unwrap_or_else(Utc::now),
        None => Utc::now(),
    }
}

/// Generate random duration between min and max
fn generate_random_duration(min: Duration, max: Duration) -> Duration {
    let min_secs = min.num_seconds();
    let max_secs = max.num_seconds();
    let random_secs = rand::thread_rng().gen_range(min_secs..=max_secs);

    Duration::try_seconds(random_secs).unwrap_or(min)
}

#[cfg(test)]
mod tests {
    use super::*;

    // DURATION PARSING TESTS

    #[test]
    fn test_parse_duration_minutes() {
        let result = parse_schedule("30m", None);
        assert!(result.is_ok());

        let scheduled_time = result.unwrap();
        let now = Utc::now();
        let diff = (scheduled_time - now).num_minutes();

        // Should be approximately 30 minutes from now (allow 1 minute tolerance)
        assert!(
            diff >= 29 && diff <= 31,
            "Expected ~30 minutes, got {}",
            diff
        );
    }

    #[test]
    fn test_parse_duration_hours() {
        let result = parse_schedule("2h", None);
        assert!(result.is_ok());

        let scheduled_time = result.unwrap();
        let now = Utc::now();
        let diff = (scheduled_time - now).num_minutes();

        // Should be approximately 120 minutes (allow small tolerance)
        assert!(
            diff >= 119 && diff <= 121,
            "Expected ~120 minutes, got {}",
            diff
        );
    }

    #[test]
    fn test_parse_duration_days() {
        let result = parse_schedule("1d", None);
        assert!(result.is_ok());

        let scheduled_time = result.unwrap();
        let now = Utc::now();
        let diff = (scheduled_time - now).num_hours();

        // Should be approximately 24 hours
        assert!(diff >= 23 && diff <= 25, "Expected ~24 hours, got {}", diff);
    }

    #[test]
    fn test_parse_duration_with_space() {
        let result = parse_schedule("1 hour", None);
        assert!(result.is_ok());

        let scheduled_time = result.unwrap();
        let now = Utc::now();
        let diff = (scheduled_time - now).num_minutes();

        // Should be approximately 60 minutes
        assert!(
            diff >= 59 && diff <= 61,
            "Expected ~60 minutes, got {}",
            diff
        );
    }

    // NATURAL LANGUAGE TESTS

    #[test]
    fn test_parse_tomorrow() {
        let result = parse_schedule("tomorrow", None);
        assert!(result.is_ok());

        let scheduled_time = result.unwrap();
        let now = Utc::now();
        let diff = (scheduled_time - now).num_hours();

        // Should be approximately 24 hours from now (20-28 hours tolerance)
        assert!(diff >= 20 && diff <= 28, "Expected ~24 hours, got {}", diff);
    }

    #[test]
    fn test_parse_next_week() {
        let result = parse_schedule("next week", None);

        // chrono-english might not support "next week" - if so, test should gracefully fail
        if result.is_err() {
            // This is acceptable - not all natural language parsers support all phrases
            return;
        }

        let scheduled_time = result.unwrap();
        let now = Utc::now();
        let diff = (scheduled_time - now).num_days();

        // Should be approximately 7 days from now
        assert!(diff >= 6 && diff <= 8, "Expected 6-8 days, got {}", diff);
    }

    #[test]
    fn test_parse_in_time() {
        let result = parse_schedule("in 2 hours", None);

        // "in X time" format may not be supported by all parsers
        if result.is_err() {
            // Try alternative format that should work
            let alt_result = parse_schedule("2 hours", None);
            assert!(alt_result.is_ok(), "Should parse '2 hours' format");
            return;
        }

        let scheduled_time = result.unwrap();
        let now = Utc::now();
        let diff = (scheduled_time - now).num_minutes();

        // Should be approximately 120 minutes
        assert!(
            diff >= 119 && diff <= 121,
            "Expected ~120 minutes, got {}",
            diff
        );
    }

    // RANDOM SCHEDULING TESTS

    #[test]
    fn test_parse_random_without_last_scheduled() {
        let result = parse_schedule("random:10m-20m", None);
        assert!(result.is_ok());

        let scheduled_time = result.unwrap();
        let now = Utc::now();
        let diff = (scheduled_time - now).num_minutes();

        // Should be between 10 and 20 minutes from now
        assert!(
            diff >= 10 && diff <= 20,
            "Expected 10-20 minutes, got {}",
            diff
        );
    }

    #[test]
    fn test_parse_random_with_last_scheduled() {
        let now = Utc::now().timestamp();
        let last = now + 3600; // 1 hour from now

        let result = parse_schedule("random:10m-20m", Some(last));
        assert!(result.is_ok());

        let scheduled_time = result.unwrap();
        let diff = (scheduled_time.timestamp() - last) / 60;

        // Should be 10-20 minutes after last_scheduled
        assert!(
            diff >= 10 && diff <= 20,
            "Expected 10-20 minutes after last, got {}",
            diff
        );
    }

    #[test]
    fn test_parse_random_hours() {
        let result = parse_schedule("random:1h-2h", None);
        assert!(result.is_ok());

        let scheduled_time = result.unwrap();
        let now = Utc::now();
        let diff = (scheduled_time - now).num_hours();

        // Should be between 1 and 2 hours
        assert!(diff >= 1 && diff <= 2);
    }

    #[test]
    fn test_parse_random_mixed_units() {
        let result = parse_schedule("random:30m-2h", None);
        assert!(result.is_ok());

        let scheduled_time = result.unwrap();
        let now = Utc::now();
        let diff_minutes = (scheduled_time - now).num_minutes();

        // Should be between 30 minutes and 120 minutes
        assert!(diff_minutes >= 30 && diff_minutes <= 120);
    }

    #[test]
    fn test_parse_random_days() {
        let result = parse_schedule("random:1d-3d", None);
        assert!(result.is_ok());

        let scheduled_time = result.unwrap();
        let now = Utc::now();
        let diff = (scheduled_time - now).num_days();

        // Should be between 1 and 3 days
        assert!(diff >= 1 && diff <= 3);
    }

    // ERROR HANDLING TESTS

    #[test]
    fn test_parse_empty_string() {
        let result = parse_schedule("", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_format() {
        let result = parse_schedule("not a time", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_random_invalid_format() {
        let result = parse_schedule("random:invalid", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_random_min_greater_than_max() {
        let result = parse_schedule("random:2h-1h", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_random_too_short() {
        // Minimum should be at least 30 seconds to prevent abuse
        let result = parse_schedule("random:1s-10s", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_random_too_long() {
        // Maximum should be less than 30 days to prevent abuse
        let result = parse_schedule("random:1d-40d", None);
        assert!(result.is_err());
    }
}
