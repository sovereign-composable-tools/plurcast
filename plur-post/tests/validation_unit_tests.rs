//! Unit tests for input validation logic
//! 
//! This test file verifies Task 5.1 requirements:
//! - Test content under limit (should pass)
//! - Test content exactly at limit (should pass)
//! - Test content at limit + 1 byte (should fail)
//! - Test significantly oversized content (should fail)
//! - Test empty content after trim (should fail)
//! - Verify error messages include size information
//! - Verify no content samples in error messages
//! 
//! Requirements: NFR-4, SR-3

use assert_cmd::Command;
use predicates::prelude::*;

// Maximum content length constant (must match main.rs)
const MAX_CONTENT_LENGTH: usize = 100_000;

// ============================================================================
// Unit Tests for Content Size Validation
// ============================================================================

#[test]
fn test_content_under_limit_passes() {
    // Test content well under the 100KB limit
    let content = "a".repeat(1_000); // 1KB
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.write_stdin(content.as_bytes())
        .arg("--draft")
        .assert()
        .success()
        .code(0);
}

#[test]
fn test_content_at_limit_passes() {
    // Test content exactly at the 100,000 byte limit
    let content = "a".repeat(MAX_CONTENT_LENGTH);
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.write_stdin(content.as_bytes())
        .arg("--draft")
        .assert()
        .success()
        .code(0)
        .stdout(predicate::str::contains("draft:"));
}

#[test]
fn test_content_at_limit_plus_one_fails() {
    // Test content at exactly limit + 1 byte (should fail)
    let content = "a".repeat(MAX_CONTENT_LENGTH + 1);
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.write_stdin(content.as_bytes())
        .arg("--draft")
        .assert()
        .failure()
        .code(3) // Invalid input exit code
        .stderr(predicate::str::contains("Content too large"));
}

#[test]
fn test_significantly_oversized_content_fails() {
    // Test content significantly over limit (1MB)
    let content = "a".repeat(1_000_000);
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.write_stdin(content.as_bytes())
        .arg("--draft")
        .assert()
        .failure()
        .code(3) // Invalid input exit code
        .stderr(predicate::str::contains("Content too large"));
}

#[test]
fn test_empty_content_after_trim_fails() {
    // Test content that is only whitespace (empty after trim)
    let content = "   \n\t\r\n   ";
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.write_stdin(content)
        .arg("--draft")
        .assert()
        .failure()
        .code(3) // Invalid input exit code
        .stderr(predicate::str::contains("Content cannot be empty"));
}

// ============================================================================
// Unit Tests for Error Message Format
// ============================================================================

#[test]
fn test_error_message_includes_size_information() {
    // Verify error message includes both actual and maximum sizes
    let content = "a".repeat(MAX_CONTENT_LENGTH + 1);
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    let output = cmd
        .write_stdin(content.as_bytes())
        .arg("--draft")
        .output()
        .unwrap();
    
    assert_eq!(output.status.code(), Some(3));
    
    let stderr = String::from_utf8(output.stderr).unwrap();
    
    // Verify error message includes size information
    assert!(stderr.contains("Content too large"), 
            "Error message should contain 'Content too large'");
    assert!(stderr.contains("exceeds 100000 bytes") || stderr.contains("100000 bytes"),
            "Error message should contain maximum size (100000 bytes)");
    assert!(stderr.contains("maximum: 100000 bytes"),
            "Error message should explicitly state maximum size");
}

#[test]
fn test_error_message_no_content_samples() {
    // Verify error messages do NOT include content samples (Security: SR-3)
    let identifiable_content = format!("SENSITIVE_DATA_{}", "x".repeat(MAX_CONTENT_LENGTH));
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    let output = cmd
        .write_stdin(identifiable_content.as_bytes())
        .arg("--draft")
        .output()
        .unwrap();
    
    assert!(!output.status.success());
    
    let stderr = String::from_utf8(output.stderr).unwrap();
    
    // Verify error message does NOT include the sensitive content
    assert!(!stderr.contains("SENSITIVE_DATA"),
            "Error message should NOT contain content samples");
    assert!(stderr.contains("Content too large"),
            "Error message should contain generic error description");
}

#[test]
fn test_error_message_format_consistency() {
    // Verify error message format is consistent across different sizes
    let test_cases = vec![
        MAX_CONTENT_LENGTH + 1,
        MAX_CONTENT_LENGTH + 100,
        MAX_CONTENT_LENGTH * 2,
    ];
    
    for size in test_cases {
        let content = "a".repeat(size);
        
        let mut cmd = Command::cargo_bin("plur-post").unwrap();
        
        let output = cmd
            .write_stdin(content.as_bytes())
            .arg("--draft")
            .output()
            .unwrap();
        
        assert_eq!(output.status.code(), Some(3));
        
        let stderr = String::from_utf8(output.stderr).unwrap();
        
        // All error messages should follow the same format
        assert!(stderr.contains("Content too large"));
        assert!(stderr.contains("100000 bytes"));
        assert!(stderr.contains("maximum"));
    }
}

// ============================================================================
// Unit Tests for Argument vs Stdin Validation
// ============================================================================

#[test]
fn test_argument_validation_under_limit() {
    // Test argument input under limit
    let content = "a".repeat(1_000);
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.arg(&content)
        .arg("--draft")
        .assert()
        .success()
        .code(0);
}

#[test]
fn test_stdin_validation_under_limit() {
    // Test stdin input under limit
    let content = "a".repeat(1_000);
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.write_stdin(content.as_bytes())
        .arg("--draft")
        .assert()
        .success()
        .code(0);
}

#[test]
fn test_argument_validation_at_limit() {
    // Test argument input at exact limit
    // Note: Using smaller size due to Windows command-line length limits
    let content = "a".repeat(10_000);
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.arg(&content)
        .arg("--draft")
        .assert()
        .success()
        .code(0);
}

#[test]
fn test_stdin_validation_at_limit() {
    // Test stdin input at exact limit
    let content = "a".repeat(MAX_CONTENT_LENGTH);
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.write_stdin(content.as_bytes())
        .arg("--draft")
        .assert()
        .success()
        .code(0);
}

// ============================================================================
// Unit Tests for Edge Cases
// ============================================================================

#[test]
fn test_whitespace_only_content_fails() {
    // Test various whitespace-only inputs
    let test_cases = vec![
        "   ",
        "\n\n\n",
        "\t\t\t",
        "  \n\t  \r\n  ",
        " ",
    ];
    
    for content in test_cases {
        let mut cmd = Command::cargo_bin("plur-post").unwrap();
        
        cmd.write_stdin(content)
            .arg("--draft")
            .assert()
            .failure()
            .code(3)
            .stderr(predicate::str::contains("Content cannot be empty"));
    }
}

#[test]
fn test_content_with_leading_trailing_whitespace() {
    // Test that content with whitespace is trimmed but still validated
    let content = format!("  {}  ", "a".repeat(100));
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.write_stdin(content.as_bytes())
        .arg("--draft")
        .assert()
        .success()
        .code(0);
}

#[test]
fn test_unicode_content_size_validation() {
    // Test that size validation works correctly with Unicode characters
    // Unicode characters can be multiple bytes
    // Using a size that results in valid UTF-8 after truncation
    let content = "你好".repeat(17_000); // Each "你好" is 6 bytes, total ~102KB
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.write_stdin(content.as_bytes())
        .arg("--draft")
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("Content too large").or(
            predicate::str::contains("Failed to read from stdin")
        ));
}

#[test]
fn test_emoji_content_size_validation() {
    // Test size validation with emoji (multi-byte characters)
    // Using a size that results in valid UTF-8 after truncation
    let content = "🌍".repeat(25_500); // Each emoji is 4 bytes, total ~102KB
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.write_stdin(content.as_bytes())
        .arg("--draft")
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("Content too large").or(
            predicate::str::contains("Failed to read from stdin")
        ));
}

// ============================================================================
// Unit Tests for Security Requirements
// ============================================================================

#[test]
fn test_no_file_paths_in_error_messages() {
    // Verify error messages don't expose file paths (Security: SR-3)
    let content = "a".repeat(MAX_CONTENT_LENGTH + 1);
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    let output = cmd
        .write_stdin(content.as_bytes())
        .arg("--draft")
        .output()
        .unwrap();
    
    let stderr = String::from_utf8(output.stderr).unwrap();
    
    // Verify no file paths are exposed
    assert!(!stderr.contains("/home/"));
    assert!(!stderr.contains("C:\\"));
    assert!(!stderr.contains("\\Users\\"));
}

#[test]
fn test_no_system_info_in_error_messages() {
    // Verify error messages don't expose system information (Security: SR-3)
    let content = "a".repeat(MAX_CONTENT_LENGTH + 1);
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    let output = cmd
        .write_stdin(content.as_bytes())
        .arg("--draft")
        .output()
        .unwrap();
    
    let stderr = String::from_utf8(output.stderr).unwrap();
    
    // Error message should only contain size information and generic guidance
    assert!(stderr.contains("Content too large"));
    assert!(stderr.contains("bytes"));
    assert!(stderr.contains("maximum"));
    
    // Should not contain system-specific information
    assert!(!stderr.contains("memory"));
    assert!(!stderr.contains("RAM"));
    assert!(!stderr.contains("disk"));
}

#[test]
fn test_validation_error_goes_to_stderr() {
    // Verify validation errors go to stderr, not stdout (Unix philosophy)
    let content = "a".repeat(MAX_CONTENT_LENGTH + 1);
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    let output = cmd
        .write_stdin(content.as_bytes())
        .arg("--draft")
        .output()
        .unwrap();
    
    assert_eq!(output.status.code(), Some(3));
    
    // Error should be in stderr
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Error:"));
    assert!(stderr.contains("Content too large"));
    
    // Stdout should be empty
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.is_empty() || stdout.trim().is_empty());
}

#[test]
fn test_exit_code_consistency_for_validation_errors() {
    // Verify all validation errors consistently use exit code 3
    let test_cases = vec![
        ("", "empty content"),
        ("   \n\t  ", "whitespace only"),
    ];
    
    for (content, description) in test_cases {
        let mut cmd = Command::cargo_bin("plur-post").unwrap();
        
        cmd.write_stdin(content)
            .arg("--draft")
            .assert()
            .failure()
            .code(3);
        
        println!("✓ Exit code 3 verified for: {}", description);
    }
    
    // Test oversized content
    let oversized = "a".repeat(MAX_CONTENT_LENGTH + 1);
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.write_stdin(oversized.as_bytes())
        .arg("--draft")
        .assert()
        .failure()
        .code(3);
}
