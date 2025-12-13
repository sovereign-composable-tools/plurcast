//! Backward Compatibility Tests
//!
//! These tests verify that the input validation changes do not break
//! existing functionality for valid content.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

/// Helper to create a test environment with temp database
fn setup_test_env() -> TempDir {
    tempfile::tempdir().unwrap()
}

#[test]
fn test_existing_valid_posts_still_work() {
    let temp_dir = setup_test_env();
    let db_path = temp_dir.path().join("test.db");

    // Test various valid content that should continue to work
    let valid_contents = vec![
        "Hello world",
        "A longer post with multiple words and punctuation!",
        "Post with\nmultiple\nlines",
        "Unicode content: 你好世界 🌍",
        "Special chars: @#$%^&*()",
    ];

    for content in valid_contents {
        let mut cmd = Command::cargo_bin("plur-post").unwrap();
        cmd.env("PLURCAST_DB_PATH", db_path.to_str().unwrap())
            .arg("--draft")
            .arg(content)
            .assert()
            .success()
            .stdout(predicate::str::starts_with("draft:"));
    }
}

#[test]
fn test_cli_interface_unchanged() {
    // Verify all existing flags still work
    let temp_dir = setup_test_env();
    let db_path = temp_dir.path().join("test.db");

    // Test --draft flag
    Command::cargo_bin("plur-post")
        .unwrap()
        .env("PLURCAST_DB_PATH", db_path.to_str().unwrap())
        .arg("--draft")
        .arg("Test content")
        .assert()
        .success();

    // Test --format flag
    Command::cargo_bin("plur-post")
        .unwrap()
        .env("PLURCAST_DB_PATH", db_path.to_str().unwrap())
        .arg("--draft")
        .arg("--format")
        .arg("json")
        .arg("Test content")
        .assert()
        .success();

    // Test --platform flag
    Command::cargo_bin("plur-post")
        .unwrap()
        .env("PLURCAST_DB_PATH", db_path.to_str().unwrap())
        .arg("--draft")
        .arg("--platform")
        .arg("nostr")
        .arg("Test content")
        .assert()
        .success();

    // Test --verbose flag
    Command::cargo_bin("plur-post")
        .unwrap()
        .env("PLURCAST_DB_PATH", db_path.to_str().unwrap())
        .arg("--draft")
        .arg("--verbose")
        .arg("Test content")
        .assert()
        .success();
}

#[test]
fn test_output_format_unchanged_for_valid_content() {
    let temp_dir = setup_test_env();
    let db_path = temp_dir.path().join("test.db");

    // Test text format output (default)
    let output = Command::cargo_bin("plur-post")
        .unwrap()
        .env("PLURCAST_DB_PATH", db_path.to_str().unwrap())
        .arg("--draft")
        .arg("Valid content")
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.starts_with("draft:"),
        "Text format should start with 'draft:'"
    );
    assert!(
        stdout.contains("-"),
        "Text format should contain UUID with dashes"
    );

    // Test JSON format output (draft mode outputs an object)
    let output = Command::cargo_bin("plur-post")
        .unwrap()
        .env("PLURCAST_DB_PATH", db_path.to_str().unwrap())
        .arg("--draft")
        .arg("--format")
        .arg("json")
        .arg("Valid content")
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("\"status\""),
        "JSON should contain status field"
    );
    assert!(
        stdout.contains("\"draft\""),
        "JSON should contain draft status"
    );
    assert!(
        stdout.contains("\"post_id\""),
        "JSON should contain post_id field"
    );
}

#[test]
fn test_stdin_input_still_works() {
    let temp_dir = setup_test_env();
    let db_path = temp_dir.path().join("test.db");

    // Test stdin input
    Command::cargo_bin("plur-post")
        .unwrap()
        .env("PLURCAST_DB_PATH", db_path.to_str().unwrap())
        .arg("--draft")
        .write_stdin("Content from stdin")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("draft:"));
}

#[test]
fn test_argument_input_still_works() {
    let temp_dir = setup_test_env();
    let db_path = temp_dir.path().join("test.db");

    // Test argument input
    Command::cargo_bin("plur-post")
        .unwrap()
        .env("PLURCAST_DB_PATH", db_path.to_str().unwrap())
        .arg("--draft")
        .arg("Content from argument")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("draft:"));
}

#[test]
fn test_exit_codes_unchanged_for_valid_content() {
    let temp_dir = setup_test_env();
    let db_path = temp_dir.path().join("test.db");

    // Valid content should exit with code 0
    Command::cargo_bin("plur-post")
        .unwrap()
        .env("PLURCAST_DB_PATH", db_path.to_str().unwrap())
        .arg("--draft")
        .arg("Valid content")
        .assert()
        .code(0);
}

#[test]
fn test_help_and_version_flags_unchanged() {
    // Test --help flag
    Command::cargo_bin("plur-post")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("plur-post"))
        .stdout(predicate::str::contains("USAGE"));

    // Test --version flag
    Command::cargo_bin("plur-post")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("plur-post"));
}

#[test]
fn test_only_new_behavior_is_size_rejection() {
    let temp_dir = setup_test_env();
    let db_path = temp_dir.path().join("test.db");

    // Content under limit should work (existing behavior)
    // Use stdin to avoid Windows command line length limits
    Command::cargo_bin("plur-post")
        .unwrap()
        .env("PLURCAST_DB_PATH", db_path.to_str().unwrap())
        .arg("--draft")
        .write_stdin("x".repeat(99_999))
        .assert()
        .success();

    // Content over limit should fail (NEW behavior)
    Command::cargo_bin("plur-post")
        .unwrap()
        .env("PLURCAST_DB_PATH", db_path.to_str().unwrap())
        .arg("--draft")
        .write_stdin("x".repeat(100_001))
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("Content too large"));
}

#[test]
fn test_empty_content_error_unchanged() {
    let temp_dir = setup_test_env();
    let db_path = temp_dir.path().join("test.db");

    // Empty content should still fail with exit code 3 (existing behavior)
    Command::cargo_bin("plur-post")
        .unwrap()
        .env("PLURCAST_DB_PATH", db_path.to_str().unwrap())
        .arg("--draft")
        .arg("")
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("empty"));
}

#[test]
fn test_whitespace_trimming_unchanged() {
    let temp_dir = setup_test_env();
    let db_path = temp_dir.path().join("test.db");

    // Content with leading/trailing whitespace should still be trimmed
    Command::cargo_bin("plur-post")
        .unwrap()
        .env("PLURCAST_DB_PATH", db_path.to_str().unwrap())
        .arg("--draft")
        .arg("  Valid content with whitespace  ")
        .assert()
        .success();

    // Whitespace-only content should still fail
    Command::cargo_bin("plur-post")
        .unwrap()
        .env("PLURCAST_DB_PATH", db_path.to_str().unwrap())
        .arg("--draft")
        .arg("   ")
        .assert()
        .failure()
        .code(3);
}

#[test]
fn test_multiline_content_unchanged() {
    let temp_dir = setup_test_env();
    let db_path = temp_dir.path().join("test.db");

    // Multiline content should still work
    let multiline = "Line 1\nLine 2\nLine 3";
    Command::cargo_bin("plur-post")
        .unwrap()
        .env("PLURCAST_DB_PATH", db_path.to_str().unwrap())
        .arg("--draft")
        .arg(multiline)
        .assert()
        .success();
}

#[test]
fn test_unicode_content_unchanged() {
    let temp_dir = setup_test_env();
    let db_path = temp_dir.path().join("test.db");

    // Unicode content should still work
    let unicode_content = "Hello 世界 🌍 مرحبا";
    Command::cargo_bin("plur-post")
        .unwrap()
        .env("PLURCAST_DB_PATH", db_path.to_str().unwrap())
        .arg("--draft")
        .arg(unicode_content)
        .assert()
        .success();
}

#[test]
fn test_special_characters_unchanged() {
    let temp_dir = setup_test_env();
    let db_path = temp_dir.path().join("test.db");

    // Special characters should still work
    let special_chars = "Test @mention #hashtag $money 100% & more!";
    Command::cargo_bin("plur-post")
        .unwrap()
        .env("PLURCAST_DB_PATH", db_path.to_str().unwrap())
        .arg("--draft")
        .arg(special_chars)
        .assert()
        .success();
}
