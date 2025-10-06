//! Integration tests for attack scenarios (Security Issue H2)
//!
//! These tests verify that the input validation protects against:
//! - Simulated infinite streams
//! - Very large arguments
//! - Whitespace padding attacks
//! - Performance requirements for validation
//!
//! Requirements: NFR-1, NFR-4, SR-1

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::time::Instant;
use tempfile::TempDir;

/// Helper to escape path for TOML on Windows
fn escape_path_for_toml(path: &str) -> String {
    path.replace('\\', "\\\\")
}

/// Helper to create a test environment with config and database
fn setup_test_env() -> (TempDir, String, String) {
    let temp_dir = TempDir::new().unwrap();
    
    // Create config directory
    let config_dir = temp_dir.path().join("config");
    fs::create_dir_all(&config_dir).unwrap();
    
    // Create data directory
    let data_dir = temp_dir.path().join("data");
    fs::create_dir_all(&data_dir).unwrap();
    
    // Create config file
    let config_path = config_dir.join("config.toml");
    let db_path = data_dir.join("posts.db");
    let keys_path = config_dir.join("nostr.keys");
    
    let config_content = format!(
        r#"
[database]
path = "{}"

[nostr]
enabled = true
keys_file = "{}"
relays = ["wss://relay.damus.io"]

[defaults]
platforms = ["nostr"]
"#,
        escape_path_for_toml(&db_path.to_string_lossy()),
        escape_path_for_toml(&keys_path.to_string_lossy())
    );
    
    fs::write(&config_path, config_content).unwrap();
    
    // Generate test Nostr keys
    let test_keys = nostr_sdk::Keys::generate();
    let hex_key = test_keys.secret_key().to_secret_hex();
    fs::write(&keys_path, hex_key).unwrap();
    
    (
        temp_dir,
        config_path.to_string_lossy().to_string(),
        db_path.to_string_lossy().to_string(),
    )
}

// ============================================================================
// Attack Scenario Tests
// ============================================================================

#[test]
fn test_simulated_infinite_stream_fails_fast() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    // Simulate an infinite stream by providing content much larger than the limit
    // This tests that the tool fails fast without reading the entire stream
    let large_content = "x".repeat(10_000_000); // 10MB
    
    let start = Instant::now();
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path)
        .write_stdin(large_content.as_bytes())
        .arg("--draft")
        .assert()
        .failure()
        .code(3) // Invalid input
        .stderr(predicate::str::contains("Content too large"))
        .stderr(predicate::str::contains("exceeds 100000 bytes"))
        .stderr(predicate::str::contains("maximum: 100000 bytes"));
    
    let elapsed = start.elapsed();
    
    // Verify rejection happens quickly (< 100ms as per NFR-1)
    // Using 500ms as upper bound to account for test overhead and slower CI systems
    assert!(
        elapsed.as_millis() < 500,
        "Infinite stream rejection took {}ms, expected < 500ms",
        elapsed.as_millis()
    );
}

#[test]
fn test_very_large_argument_fails_immediately() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    // Test with very large content via stdin (simulating large argument)
    // Windows has command-line length limits, so we use stdin to simulate
    let huge_content = "a".repeat(5_000_000); // 5MB
    
    let start = Instant::now();
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path)
        .write_stdin(huge_content.as_bytes())
        .arg("--draft")
        .assert()
        .failure()
        .code(3) // Invalid input
        .stderr(predicate::str::contains("Content too large"));
    
    let elapsed = start.elapsed();
    
    // Verify immediate rejection (< 100ms as per NFR-1)
    // Using 500ms as upper bound to account for test overhead
    assert!(
        elapsed.as_millis() < 500,
        "Large argument rejection took {}ms, expected < 500ms",
        elapsed.as_millis()
    );
}

#[test]
fn test_whitespace_padding_attack_fails() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    // Attempt to bypass limit with whitespace padding
    // Create content that is over limit even after trimming
    let content_core = "a".repeat(50_000);
    let whitespace = " ".repeat(50_001); // Total > 100KB
    let padded_content = format!("{}{}", content_core, whitespace);
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path)
        .write_stdin(padded_content.as_bytes())
        .arg("--draft")
        .assert()
        .failure()
        .code(3) // Invalid input
        .stderr(predicate::str::contains("Content too large"));
}

#[test]
fn test_whitespace_padding_attack_with_newlines() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    // Attempt to bypass limit with newlines and spaces
    let content_core = "a".repeat(50_000);
    let whitespace = "\n \t\r\n".repeat(10_001); // Total > 100KB
    let padded_content = format!("{}{}", content_core, whitespace);
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path)
        .write_stdin(padded_content.as_bytes())
        .arg("--draft")
        .assert()
        .failure()
        .code(3) // Invalid input
        .stderr(predicate::str::contains("Content too large"));
}

#[test]
fn test_exit_code_3_for_oversized_content() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    // Verify exit code 3 for all validation failures
    let oversized_content = "x".repeat(100_001);
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    let output = cmd
        .env("PLURCAST_CONFIG", config_path)
        .write_stdin(oversized_content.as_bytes())
        .arg("--draft")
        .output()
        .unwrap();
    
    assert_eq!(output.status.code(), Some(3), "Expected exit code 3 for oversized content");
}

#[test]
fn test_exit_code_3_for_empty_content() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    // Verify exit code 3 for empty content validation failure
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    let output = cmd
        .env("PLURCAST_CONFIG", config_path)
        .write_stdin("   \n\t  ")
        .arg("--draft")
        .output()
        .unwrap();
    
    assert_eq!(output.status.code(), Some(3), "Expected exit code 3 for empty content");
}

#[test]
fn test_exit_code_3_for_whitespace_only() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    // Verify exit code 3 for whitespace-only content
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    let output = cmd
        .env("PLURCAST_CONFIG", config_path)
        .arg("     ")
        .arg("--draft")
        .output()
        .unwrap();
    
    assert_eq!(output.status.code(), Some(3), "Expected exit code 3 for whitespace-only content");
}

// ============================================================================
// Performance Tests (NFR-1)
// ============================================================================

#[test]
fn test_validation_performance_normal_content() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    // Test validation overhead for normal-sized content (< 1ms as per NFR-1)
    let normal_content = "This is a normal post with reasonable length.";
    
    let start = Instant::now();
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path)
        .write_stdin(normal_content)
        .arg("--draft")
        .assert()
        .success();
    
    let elapsed = start.elapsed();
    
    // Validation should be very fast for normal content
    // Using 500ms as upper bound to account for full command execution overhead
    // (process spawn, config loading, database initialization, etc.)
    assert!(
        elapsed.as_millis() < 500,
        "Normal content validation took {}ms, expected < 500ms",
        elapsed.as_millis()
    );
}

#[test]
fn test_validation_performance_10kb_content() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    // Test validation overhead for 10KB content (< 1ms as per NFR-1)
    let content_10kb = "a".repeat(10_000);
    
    let start = Instant::now();
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path)
        .write_stdin(content_10kb.as_bytes())
        .arg("--draft")
        .assert()
        .success();
    
    let elapsed = start.elapsed();
    
    // Validation should complete quickly even for 10KB
    // Using 500ms as upper bound to account for full command execution overhead
    // (process spawn, config loading, database initialization, etc.)
    assert!(
        elapsed.as_millis() < 500,
        "10KB content validation took {}ms, expected < 500ms",
        elapsed.as_millis()
    );
}

#[test]
fn test_rejection_performance_oversized_content() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    // Test rejection performance for oversized content (< 100ms as per NFR-1)
    let oversized_content = "x".repeat(500_000); // 500KB
    
    let start = Instant::now();
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path)
        .write_stdin(oversized_content.as_bytes())
        .arg("--draft")
        .assert()
        .failure()
        .code(3);
    
    let elapsed = start.elapsed();
    
    // Rejection should happen quickly (< 100ms as per NFR-1)
    // Using 500ms as upper bound to account for test overhead and slower CI systems
    assert!(
        elapsed.as_millis() < 500,
        "Oversized content rejection took {}ms, expected < 500ms",
        elapsed.as_millis()
    );
}

#[test]
fn test_rejection_performance_massive_content() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    // Test rejection performance for massive content (should still be fast)
    let massive_content = "x".repeat(10_000_000); // 10MB
    
    let start = Instant::now();
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path)
        .write_stdin(massive_content.as_bytes())
        .arg("--draft")
        .assert()
        .failure()
        .code(3);
    
    let elapsed = start.elapsed();
    
    // Even massive content should be rejected quickly
    // Using 1000ms (1 second) as upper bound for very large content
    assert!(
        elapsed.as_millis() < 1000,
        "Massive content rejection took {}ms, expected < 1000ms",
        elapsed.as_millis()
    );
}

// ============================================================================
// Edge Case Attack Scenarios
// ============================================================================

#[test]
fn test_exactly_at_limit_boundary() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    // Test content exactly at 100,000 bytes (should pass)
    let content_at_limit = "a".repeat(100_000);
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path)
        .write_stdin(content_at_limit.as_bytes())
        .arg("--draft")
        .assert()
        .success()
        .code(0);
}

#[test]
fn test_one_byte_over_limit() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    // Test content at 100,001 bytes (should fail)
    let content_over_limit = "a".repeat(100_001);
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path)
        .write_stdin(content_over_limit.as_bytes())
        .arg("--draft")
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("Content too large"));
}

#[test]
fn test_multibyte_unicode_at_limit() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    // Test with multibyte Unicode characters near the limit
    // Each emoji is typically 4 bytes
    let emoji_count = 25_000; // 25,000 * 4 = 100,000 bytes
    let content = "🌍".repeat(emoji_count);
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path)
        .write_stdin(content.as_bytes())
        .arg("--draft")
        .assert()
        .success()
        .code(0);
}

#[test]
fn test_multibyte_unicode_over_limit() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    // Test with multibyte Unicode characters over the limit
    // Using ASCII to avoid UTF-8 boundary issues when truncating
    let content = format!("{}{}", "🌍".repeat(24_999), "x".repeat(101));
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path)
        .write_stdin(content.as_bytes())
        .arg("--draft")
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("Content too large").or(predicate::str::contains("Failed to read from stdin")));
}

#[test]
fn test_binary_data_attack() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    // Test with binary data (null bytes, control characters)
    let mut binary_content = vec![0u8; 100_001];
    for (i, item) in binary_content.iter_mut().enumerate() {
        *item = (i % 256) as u8;
    }
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path)
        .write_stdin(binary_content)
        .arg("--draft")
        .assert()
        .failure()
        .code(3);
}

#[test]
fn test_repeated_newlines_attack() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    // Test with repeated newlines to inflate size
    let content = "\n".repeat(100_001);
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path)
        .write_stdin(content.as_bytes())
        .arg("--draft")
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("Content too large"));
}

#[test]
fn test_mixed_whitespace_attack() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    // Test with mixed whitespace characters
    let spaces = " ".repeat(30_000);
    let tabs = "\t".repeat(30_000);
    let newlines = "\n".repeat(30_000);
    let carriage_returns = "\r".repeat(10_002);
    let content = format!("{}{}{}{}", spaces, tabs, newlines, carriage_returns);
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path)
        .write_stdin(content.as_bytes())
        .arg("--draft")
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("Content too large"));
}

// ============================================================================
// Security Requirement Tests (SR-1)
// ============================================================================

#[test]
fn test_dos_prevention_rapid_large_inputs() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    // Test that multiple rapid large inputs are all rejected quickly
    let large_content = "x".repeat(500_000);
    
    for _ in 0..5 {
        let start = Instant::now();
        
        let mut cmd = Command::cargo_bin("plur-post").unwrap();
        
        cmd.env("PLURCAST_CONFIG", &config_path)
            .write_stdin(large_content.as_bytes())
            .arg("--draft")
            .assert()
            .failure()
            .code(3);
        
        let elapsed = start.elapsed();
        
        // Each rejection should be fast
        assert!(
            elapsed.as_millis() < 500,
            "Rapid rejection took {}ms, expected < 500ms",
            elapsed.as_millis()
        );
    }
}

#[test]
fn test_memory_bounded_rejection() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    // Test that rejection doesn't consume excessive memory
    // This is a behavioral test - the tool should reject without reading entire stream
    let huge_content = "x".repeat(50_000_000); // 50MB
    
    let start = Instant::now();
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path)
        .write_stdin(huge_content.as_bytes())
        .arg("--draft")
        .assert()
        .failure()
        .code(3);
    
    let elapsed = start.elapsed();
    
    // Should reject quickly without reading entire 50MB
    // If it read the entire stream, it would take much longer
    assert!(
        elapsed.as_millis() < 2000,
        "Memory-bounded rejection took {}ms, expected < 2000ms (should not read entire stream)",
        elapsed.as_millis()
    );
}
