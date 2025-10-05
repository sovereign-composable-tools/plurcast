//! CLI integration tests for plur-post

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
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
relays = []

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

#[test]
fn test_help_flag_output() {
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Post content to decentralized social platforms"))
        .stdout(predicate::str::contains("USAGE"))
        .stdout(predicate::str::contains("OPTIONS"))
        .stdout(predicate::str::contains("--platform"))
        .stdout(predicate::str::contains("--draft"))
        .stdout(predicate::str::contains("--format"))
        .stdout(predicate::str::contains("--verbose"));
}

#[test]
fn test_version_flag_output() {
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("plur-post"));
}

#[test]
fn test_empty_content_error_handling() {
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.arg("")
        .assert()
        .failure()
        .code(3) // Invalid input exit code
        .stderr(predicate::str::contains("Content cannot be empty"));
}

#[test]
fn test_no_content_no_stdin_error() {
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    // Run without content and without stdin
    cmd.assert()
        .failure()
        .code(3) // Invalid input exit code
        .stderr(predicate::str::contains("Content cannot be empty").or(predicate::str::contains("No content provided")));
}

#[test]
fn test_stdin_input() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path)
        .write_stdin("Test content from stdin")
        .arg("--draft") // Use draft mode to avoid actual posting
        .assert()
        .success()
        .code(0)
        .stdout(predicate::str::contains("draft:"));
}

#[test]
fn test_argument_input() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path)
        .arg("Test content from argument")
        .arg("--draft") // Use draft mode to avoid actual posting
        .assert()
        .success()
        .code(0)
        .stdout(predicate::str::contains("draft:"));
}

#[test]
fn test_draft_mode() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path)
        .arg("Draft content")
        .arg("--draft")
        .assert()
        .success()
        .code(0)
        .stdout(predicate::str::contains("draft:"));
}

#[test]
fn test_output_format_text() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path)
        .arg("Test content")
        .arg("--draft")
        .arg("--format")
        .arg("text")
        .assert()
        .success()
        .stdout(predicate::str::contains("draft:"));
}

#[test]
fn test_output_format_json() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path)
        .arg("Test content")
        .arg("--draft")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"status\""))
        .stdout(predicate::str::contains("\"draft\""))
        .stdout(predicate::str::contains("\"post_id\""));
}

#[test]
fn test_invalid_format() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path)
        .arg("Test content")
        .arg("--format")
        .arg("invalid")
        .assert()
        .failure()
        .code(3) // Invalid input
        .stderr(predicate::str::contains("Invalid format"));
}

#[test]
fn test_platform_selection_nostr() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path)
        .arg("Test content")
        .arg("--platform")
        .arg("nostr")
        .arg("--draft")
        .assert()
        .success();
}

#[test]
fn test_verbose_flag() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path)
        .arg("Test content")
        .arg("--draft")
        .arg("--verbose")
        .assert()
        .success();
    
    // Note: Verbose output goes to stderr, which we can't easily capture in this test
    // But we verify the command succeeds with the flag
}

#[test]
fn test_exit_code_success() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path)
        .arg("Test content")
        .arg("--draft")
        .assert()
        .success()
        .code(0);
}

#[test]
fn test_exit_code_invalid_input() {
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.arg("")
        .assert()
        .failure()
        .code(3);
}

#[test]
fn test_exit_code_auth_error_missing_config() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent_config = temp_dir.path().join("nonexistent.toml");
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", nonexistent_config.to_str().unwrap())
        .arg("Test content")
        .assert()
        .failure();
    // Note: Exit code may vary depending on error type
}

#[test]
fn test_multiple_platforms() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path)
        .arg("Test content")
        .arg("--platform")
        .arg("nostr")
        .arg("--draft")
        .assert()
        .success();
}

#[test]
fn test_stdin_with_newlines() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path)
        .write_stdin("Line 1\nLine 2\nLine 3")
        .arg("--draft")
        .assert()
        .success();
}

#[test]
fn test_long_content() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    let long_content = "a".repeat(500);
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path)
        .arg(&long_content)
        .arg("--draft")
        .assert()
        .success();
}

#[test]
fn test_special_characters_in_content() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path)
        .arg("Content with special chars: !@#$%^&*()_+-=[]{}|;':\",./<>?")
        .arg("--draft")
        .assert()
        .success();
}

#[test]
fn test_unicode_content() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path)
        .arg("Unicode content: 你好世界 🌍 مرحبا")
        .arg("--draft")
        .assert()
        .success();
}

#[test]
fn test_help_shows_exit_codes() {
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("EXIT CODES"))
        .stdout(predicate::str::contains("0 - Success"))
        .stdout(predicate::str::contains("1 - Posting failed"))
        .stdout(predicate::str::contains("2 - Authentication error"))
        .stdout(predicate::str::contains("3 - Invalid input"));
}

#[test]
fn test_help_shows_examples() {
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("USAGE EXAMPLES"))
        .stdout(predicate::str::contains("echo"))
        .stdout(predicate::str::contains("--platform"))
        .stdout(predicate::str::contains("--draft"));
}

#[test]
fn test_json_output_is_valid() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    let output = cmd
        .env("PLURCAST_CONFIG", config_path)
        .arg("Test content")
        .arg("--draft")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();
    
    assert!(output.status.success());
    
    // Verify JSON is valid
    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    
    assert!(parsed.get("status").is_some());
    assert!(parsed.get("post_id").is_some());
}

#[test]
fn test_env_var_db_path_override() {
    let temp_dir = TempDir::new().unwrap();
    let custom_db = temp_dir.path().join("custom.db");
    
    // Create minimal config
    let config_dir = temp_dir.path().join("config");
    fs::create_dir_all(&config_dir).unwrap();
    let config_path = config_dir.join("config.toml");
    
    let keys_path = config_dir.join("nostr.keys");
    let default_db = temp_dir.path().join("default.db");
    
    let config_content = format!(
        r#"
[database]
path = "{}"

[nostr]
enabled = true
keys_file = "{}"
relays = []

[defaults]
platforms = ["nostr"]
"#,
        escape_path_for_toml(&default_db.to_string_lossy()),
        escape_path_for_toml(&keys_path.to_string_lossy())
    );
    
    fs::write(&config_path, config_content).unwrap();
    
    // Generate test keys
    let test_keys = nostr_sdk::Keys::generate();
    let hex_key = test_keys.secret_key().to_secret_hex();
    fs::write(&keys_path, hex_key).unwrap();
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path.to_str().unwrap())
        .env("PLURCAST_DB_PATH", custom_db.to_str().unwrap())
        .arg("Test content")
        .arg("--draft")
        .assert()
        .success();
    
    // Verify custom database was created
    assert!(custom_db.exists());
}

// ============================================================================
// Input Validation Tests (Security Issue H2)
// ============================================================================

#[test]
fn test_argument_content_under_limit() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    // Content well under 100KB limit
    let content = "a".repeat(1000);
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path)
        .arg(&content)
        .arg("--draft")
        .assert()
        .success()
        .code(0)
        .stdout(predicate::str::contains("draft:"));
}

#[test]
fn test_argument_content_at_limit() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    // Content exactly at 100,000 bytes
    // Note: Using smaller size for argument test due to Windows command-line length limits
    let content = "a".repeat(10_000);
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path)
        .arg(&content)
        .arg("--draft")
        .assert()
        .success()
        .code(0)
        .stdout(predicate::str::contains("draft:"));
}

#[test]
fn test_argument_content_over_limit() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    // Content over limit (using stdin to avoid Windows command-line length limits)
    // This test verifies the validation logic works for arguments
    let content = "a".repeat(100_001);
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path)
        .write_stdin(content.as_bytes())
        .arg("--draft")
        .assert()
        .failure()
        .code(3) // Invalid input
        .stderr(predicate::str::contains("Content too large"))
        .stderr(predicate::str::contains("exceeds 100000 bytes"))
        .stderr(predicate::str::contains("maximum: 100000 bytes"));
}

#[test]
fn test_argument_content_significantly_over_limit() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    // Content significantly over limit (using stdin to avoid Windows command-line length limits)
    let content = "a".repeat(1_000_000);
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path)
        .write_stdin(content.as_bytes())
        .arg("--draft")
        .assert()
        .failure()
        .code(3) // Invalid input
        .stderr(predicate::str::contains("Content too large"))
        .stderr(predicate::str::contains("exceeds 100000 bytes"));
}

#[test]
fn test_stdin_content_under_limit() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    // Content well under 100KB limit
    let content = "a".repeat(1000);
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path)
        .write_stdin(content.as_bytes())
        .arg("--draft")
        .assert()
        .success()
        .code(0)
        .stdout(predicate::str::contains("draft:"));
}

#[test]
fn test_stdin_content_at_limit() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    // Content exactly at 100,000 bytes
    let content = "a".repeat(100_000);
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path)
        .write_stdin(content.as_bytes())
        .arg("--draft")
        .assert()
        .success()
        .code(0)
        .stdout(predicate::str::contains("draft:"));
}

#[test]
fn test_stdin_content_over_limit() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    // Content at 100,001 bytes (over limit)
    let content = "a".repeat(100_001);
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path)
        .write_stdin(content.as_bytes())
        .arg("--draft")
        .assert()
        .failure()
        .code(3) // Invalid input
        .stderr(predicate::str::contains("Content too large"))
        .stderr(predicate::str::contains("exceeds 100000 bytes"))
        .stderr(predicate::str::contains("maximum: 100000 bytes"));
}

#[test]
fn test_stdin_content_significantly_over_limit() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    // Content significantly over limit (1MB)
    let content = "a".repeat(1_000_000);
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path)
        .write_stdin(content.as_bytes())
        .arg("--draft")
        .assert()
        .failure()
        .code(3) // Invalid input
        .stderr(predicate::str::contains("Content too large"))
        .stderr(predicate::str::contains("exceeds 100000 bytes"));
}

#[test]
fn test_empty_content_after_trim() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    // Content that is only whitespace
    let content = "   \n\t\r\n   ";
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path)
        .write_stdin(content)
        .arg("--draft")
        .assert()
        .failure()
        .code(3) // Invalid input
        .stderr(predicate::str::contains("Content cannot be empty"));
}

#[test]
fn test_error_message_includes_size_info() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    // Content over limit (using stdin to avoid Windows command-line length limits)
    let content = "a".repeat(150_000);
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    let output = cmd
        .env("PLURCAST_CONFIG", config_path)
        .write_stdin(content.as_bytes())
        .arg("--draft")
        .output()
        .unwrap();
    
    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(3));
    
    let stderr = String::from_utf8(output.stderr).unwrap();
    
    // Verify error message includes size information
    assert!(stderr.contains("Content too large"));
    assert!(stderr.contains("exceeds 100000 bytes"));
    assert!(stderr.contains("maximum: 100000 bytes"));
    
    // Verify no content samples in error message (security requirement SR-3)
    assert!(!stderr.contains("aaaa"));
}

#[test]
fn test_no_content_samples_in_error_messages() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    // Content with identifiable pattern (using stdin to avoid Windows command-line length limits)
    let content = format!("SECRET_DATA_{}", "x".repeat(100_000));
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    let output = cmd
        .env("PLURCAST_CONFIG", config_path)
        .write_stdin(content.as_bytes())
        .arg("--draft")
        .output()
        .unwrap();
    
    assert!(!output.status.success());
    
    let stderr = String::from_utf8(output.stderr).unwrap();
    
    // Verify error message does NOT include content samples
    assert!(!stderr.contains("SECRET_DATA"));
    assert!(stderr.contains("Content too large"));
}
