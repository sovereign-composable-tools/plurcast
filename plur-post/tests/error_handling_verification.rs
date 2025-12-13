//! Verification tests for error handling and exit codes
//!
//! This test file verifies Task 4 requirements:
//! - Confirm existing main() error handler maps InvalidInput to exit code 3
//! - Test that all validation errors go to stderr
//! - Verify no changes needed to existing error handling infrastructure

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn verify_invalid_input_exit_code_3() {
    // Verify that InvalidInput errors return exit code 3
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.arg("")
        .assert()
        .failure()
        .code(3) // InvalidInput exit code
        .stderr(predicate::str::contains("Error:"));
}

#[test]
fn verify_validation_errors_go_to_stderr() {
    // Verify that validation errors are written to stderr, not stdout
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    let output = cmd.arg("").output().unwrap();

    // Exit code should be 3
    assert_eq!(output.status.code(), Some(3));

    // Error message should be in stderr
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Error:"));
    assert!(stderr.contains("Content cannot be empty"));

    // Stdout should be empty (no output on error)
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.is_empty() || stdout.trim().is_empty());
}

#[test]
fn verify_oversized_content_error_to_stderr() {
    // Verify that oversized content errors go to stderr with proper exit code
    let content = "a".repeat(100_001);

    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    let output = cmd.write_stdin(content.as_bytes()).output().unwrap();

    // Exit code should be 3 (InvalidInput)
    assert_eq!(output.status.code(), Some(3));

    // Error message should be in stderr
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Error:"));
    assert!(stderr.contains("Content too large"));

    // Stdout should be empty
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.is_empty() || stdout.trim().is_empty());
}

#[test]
fn verify_error_handler_infrastructure_unchanged() {
    // This test verifies that the error handling infrastructure works as expected
    // by testing multiple error scenarios

    // Test 1: Empty content -> exit code 3
    let mut cmd1 = Command::cargo_bin("plur-post").unwrap();
    cmd1.arg("").assert().failure().code(3);

    // Test 2: No content provided -> exit code 3
    let mut cmd2 = Command::cargo_bin("plur-post").unwrap();
    cmd2.assert().failure().code(3);

    // Test 3: Invalid format -> exit code 3
    let mut cmd3 = Command::cargo_bin("plur-post").unwrap();
    cmd3.arg("Test content")
        .arg("--format")
        .arg("invalid")
        .assert()
        .failure()
        .code(3);
}

#[test]
fn verify_all_validation_errors_use_exit_code_3() {
    // Verify that all validation-related errors consistently use exit code 3

    // Empty content
    let mut cmd1 = Command::cargo_bin("plur-post").unwrap();
    cmd1.arg("").assert().failure().code(3);

    // Whitespace-only content
    let mut cmd2 = Command::cargo_bin("plur-post").unwrap();
    cmd2.write_stdin("   \n\t  ").assert().failure().code(3);

    // Oversized content
    let content = "a".repeat(100_001);
    let mut cmd3 = Command::cargo_bin("plur-post").unwrap();
    cmd3.write_stdin(content.as_bytes())
        .assert()
        .failure()
        .code(3);
}

#[test]
fn verify_stderr_contains_error_prefix() {
    // Verify that all errors are prefixed with "Error:" in stderr
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.arg("")
        .assert()
        .failure()
        .stderr(predicate::str::starts_with("Error:"));
}

#[test]
fn verify_stdout_clean_on_error() {
    // Verify that stdout remains clean (empty) when errors occur
    // This is important for Unix composability

    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    let output = cmd.arg("").output().unwrap();

    assert!(!output.status.success());

    // Stdout should be empty
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.is_empty() || stdout.trim().is_empty(),
        "stdout should be empty on error, but got: {}",
        stdout
    );
}
