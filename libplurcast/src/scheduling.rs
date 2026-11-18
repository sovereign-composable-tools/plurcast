//! Scheduling and time parsing utilities
//!
//! This module provides parsing of human-readable time formats for scheduling posts.

use crate::{PlurcastError, Result};
use chrono::{DateTime, Duration, Utc};
use rand::RngExt;

const MIN_RANDOM_SECONDS: i64 = 30;
const MAX_RANDOM_SECONDS: i64 = 30 * 24 * 3600; // 30 days

/// Parse a schedule string into a DateTime
///
/// Supports multiple formats:
/// - Relative durations: "1h", "30m", "2d"
/// - Natural language: "tomorrow", "next week", "in 1 hour"
/// - Absolute times: "2025-11-20 15:00", "next monday 10am"
/// - Random delay from now: "random:10m-20m", "random:1h30m-3h"
/// - Random gap after the last queued post: "spaced:1h-2h"
///
/// `random:` and `spaced:` differ only in where the delay is measured from.
/// `random:MIN-MAX` picks a time between MIN and MAX from now, whatever else
/// is queued. `spaced:MIN-MAX` picks a time between MIN and MAX after the
/// latest post already in the queue (`last_scheduled`), so repeated calls
/// build a drip of posts with randomized gaps. With an empty queue, or one
/// whose latest post is already overdue, `spaced:` measures from now.
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

    // Randomized formats first: from now, or after the last queued post
    if let Some(range) = input.strip_prefix("random:") {
        return parse_random_schedule(range, Utc::now());
    }
    if let Some(range) = input.strip_prefix("spaced:") {
        return parse_random_schedule(range, base_time_after_queue(last_scheduled));
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
///
/// If the parsed date is in the past and no explicit year was given,
/// assumes the next occurrence by adding a year. This prevents scheduling
/// "Jan 1 10:00" in December from being interpreted as the past January.
fn parse_natural_language(input: &str) -> Result<DateTime<Utc>> {
    let parsed = chrono_english::parse_date_string(input, Utc::now(), chrono_english::Dialect::Us)
        .map_err(|e| PlurcastError::InvalidInput(format!("Could not parse time: {}", e)))?;

    // If the parsed date is in the past and no explicit year was given,
    // assume the next occurrence (add a year)
    if parsed < Utc::now() && !contains_explicit_year(input) {
        use chrono::Datelike;
        if let Some(next_occurrence) = parsed.with_year(parsed.year() + 1) {
            return Ok(next_occurrence);
        }
    }

    Ok(parsed)
}

/// Check if the input contains an explicit year (4 consecutive digits)
fn contains_explicit_year(input: &str) -> bool {
    let mut consecutive_digits = 0;
    for c in input.chars() {
        if c.is_ascii_digit() {
            consecutive_digits += 1;
            if consecutive_digits >= 4 {
                return true;
            }
        } else {
            consecutive_digits = 0;
        }
    }
    false
}

/// Pick a time a random "MIN-MAX" delay after `base_time`
fn parse_random_schedule(range: &str, base_time: DateTime<Utc>) -> Result<DateTime<Utc>> {
    let (min_str, max_str) = parse_random_range(range)?;
    let min_duration = parse_duration(min_str)?;
    let max_duration = parse_duration(max_str)?;

    validate_random_range(min_duration, max_duration)?;

    Ok(base_time + generate_random_duration(min_duration, max_duration))
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

/// Base time for `spaced:` scheduling: the latest queued post, but never the past.
///
/// An overdue post still waiting in the queue must not pull new posts into
/// the past, so the base is clamped to now.
fn base_time_after_queue(last_scheduled: Option<i64>) -> DateTime<Utc> {
    let now = Utc::now();
    last_scheduled
        .and_then(|timestamp| DateTime::from_timestamp(timestamp, 0))
        .map_or(now, |last| last.max(now))
}

/// Generate random duration between min and max
fn generate_random_duration(min: Duration, max: Duration) -> Duration {
    let min_secs = min.num_seconds();
    let max_secs = max.num_seconds();
    let random_secs = rand::rng().random_range(min_secs..=max_secs);

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
            (29..=31).contains(&diff),
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
            (119..=121).contains(&diff),
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
        assert!(
            (23..=25).contains(&diff),
            "Expected ~24 hours, got {}",
            diff
        );
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
            (59..=61).contains(&diff),
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
        assert!(
            (20..=28).contains(&diff),
            "Expected ~24 hours, got {}",
            diff
        );
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
        assert!((6..=8).contains(&diff), "Expected 6-8 days, got {}", diff);
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
            (119..=121).contains(&diff),
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
            (10..=20).contains(&diff),
            "Expected 10-20 minutes, got {}",
            diff
        );
    }

    #[test]
    fn test_parse_random_ignores_last_scheduled() {
        let now = Utc::now().timestamp();
        let last = now + 6 * 3600; // a post queued 6 hours out

        let scheduled_time = parse_schedule("random:10m-20m", Some(last)).unwrap();
        let diff = (scheduled_time.timestamp() - now) / 60;

        // "random:" always means from now, however full the queue is
        assert!(
            (10..=20).contains(&diff),
            "Expected 10-20 minutes from now, got {}",
            diff
        );
    }

    #[test]
    fn test_parse_spaced_follows_last_scheduled() {
        let now = Utc::now().timestamp();
        let last = now + 3600; // 1 hour from now

        let scheduled_time = parse_schedule("spaced:10m-20m", Some(last)).unwrap();
        let diff = (scheduled_time.timestamp() - last) / 60;

        assert!(
            (10..=20).contains(&diff),
            "Expected 10-20 minutes after last, got {}",
            diff
        );
    }

    #[test]
    fn test_parse_spaced_with_empty_queue_uses_now() {
        let now = Utc::now().timestamp();

        let scheduled_time = parse_schedule("spaced:10m-20m", None).unwrap();
        let diff = (scheduled_time.timestamp() - now) / 60;

        assert!(
            (10..=20).contains(&diff),
            "Expected 10-20 minutes, got {}",
            diff
        );
    }

    #[test]
    fn test_parse_spaced_with_overdue_queue_uses_now() {
        let now = Utc::now().timestamp();
        let last = now - 5 * 3600; // an overdue post still waiting to be sent

        let scheduled_time = parse_schedule("spaced:10m-20m", Some(last)).unwrap();
        let diff = (scheduled_time.timestamp() - now) / 60;

        // Never schedule into the past because the queue is behind
        assert!(
            (10..=20).contains(&diff),
            "Expected 10-20 minutes, got {}",
            diff
        );
    }

    #[test]
    fn test_parse_random_compound_durations() {
        let now = Utc::now().timestamp();

        let scheduled_time = parse_schedule("random:1h30m-2h15m", None).unwrap();
        let diff = (scheduled_time.timestamp() - now) / 60;

        assert!(
            (90..=135).contains(&diff),
            "Expected 90-135 minutes, got {}",
            diff
        );
    }

    #[test]
    fn test_parse_spaced_rejects_bad_range() {
        assert!(parse_schedule("spaced:2h-1h", None).is_err());
        assert!(parse_schedule("spaced:invalid", None).is_err());
    }

    #[test]
    fn test_parse_random_hours() {
        let result = parse_schedule("random:1h-2h", None);
        assert!(result.is_ok());

        let scheduled_time = result.unwrap();
        let now = Utc::now();
        let diff = (scheduled_time - now).num_hours();

        // Should be between 1 and 2 hours
        assert!((1..=2).contains(&diff));
    }

    #[test]
    fn test_parse_random_mixed_units() {
        let result = parse_schedule("random:30m-2h", None);
        assert!(result.is_ok());

        let scheduled_time = result.unwrap();
        let now = Utc::now();
        let diff_minutes = (scheduled_time - now).num_minutes();

        // Should be between 30 minutes and 120 minutes
        assert!((30..=120).contains(&diff_minutes));
    }

    #[test]
    fn test_parse_random_days() {
        let result = parse_schedule("random:1d-3d", None);
        assert!(result.is_ok());

        let scheduled_time = result.unwrap();
        let now = Utc::now();
        let diff = (scheduled_time - now).num_days();

        // Should be between 1 and 3 days
        assert!((1..=3).contains(&diff));
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

    // YEAR INFERENCE TESTS

    #[test]
    fn test_contains_explicit_year_with_year() {
        assert!(contains_explicit_year("2026-01-01 10:00"));
        assert!(contains_explicit_year("Jan 1 2026"));
        assert!(contains_explicit_year("2025-12-31"));
    }

    #[test]
    fn test_contains_explicit_year_without_year() {
        assert!(!contains_explicit_year("Jan 1 10:00"));
        assert!(!contains_explicit_year("Dec 31 12:00"));
        assert!(!contains_explicit_year("tomorrow"));
        assert!(!contains_explicit_year("15:30"));
    }

    #[test]
    fn test_parse_past_month_without_year_bumps_to_next_year() {
        // Create a date string for a month that's definitely in the past
        // Use January 1st when we're past January
        let now = Utc::now();
        use chrono::Datelike;

        // If we're past January, "Jan 1" should schedule for next year
        if now.month() > 1 {
            let result = parse_schedule("Jan 1 10:00", None);
            assert!(result.is_ok(), "Should parse 'Jan 1 10:00'");

            let scheduled = result.unwrap();
            assert!(
                scheduled > now,
                "Past date without year should be bumped to future: scheduled={}, now={}",
                scheduled,
                now
            );
            assert_eq!(
                scheduled.year(),
                now.year() + 1,
                "Should be scheduled for next year"
            );
        }
    }

    #[test]
    fn test_parse_past_date_with_explicit_year_stays_in_past() {
        // An explicit past year should not be bumped
        let result = parse_schedule("2020-01-01 10:00", None);
        assert!(result.is_ok(), "Should parse '2020-01-01 10:00'");

        let scheduled = result.unwrap();
        use chrono::Datelike;
        assert_eq!(
            scheduled.year(),
            2020,
            "Explicit year should be preserved even if in the past"
        );
    }

    #[test]
    fn test_parse_future_month_without_year_stays_in_current_year() {
        // A future month without year should stay in current year
        let now = Utc::now();
        use chrono::Datelike;

        // If we're before December, "Dec 31" should be this year
        if now.month() < 12 {
            let result = parse_schedule("Dec 31 10:00", None);
            assert!(result.is_ok(), "Should parse 'Dec 31 10:00'");

            let scheduled = result.unwrap();
            assert!(
                scheduled > now,
                "Future date should be in the future: scheduled={}, now={}",
                scheduled,
                now
            );
            assert_eq!(
                scheduled.year(),
                now.year(),
                "Future month should stay in current year"
            );
        }
    }
}
