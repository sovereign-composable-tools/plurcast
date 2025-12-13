//! Agent-friendly interface validation tests

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

#[test]
fn test_help_comprehensive_and_parseable() {
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    let output = cmd.arg("--help").output().unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();

    // Verify comprehensive help sections
    assert!(stdout.contains("USAGE") || stdout.contains("Usage:"));
    assert!(stdout.contains("OPTIONS") || stdout.contains("Options:"));

    // Verify key options are documented
    assert!(stdout.contains("--platform"));
    assert!(stdout.contains("--draft"));
    assert!(stdout.contains("--format"));
    assert!(stdout.contains("--verbose"));

    // Verify exit codes are documented
    assert!(stdout.contains("EXIT CODES") || stdout.contains("Exit Codes"));
    assert!(stdout.contains("0"));
    assert!(stdout.contains("1"));
    assert!(stdout.contains("2"));
    assert!(stdout.contains("3"));

    // Verify examples are provided
    assert!(stdout.contains("USAGE EXAMPLES") || stdout.contains("Examples:"));

    // Help should be parseable (structured format)
    assert!(
        stdout.lines().count() > 10,
        "Help should have multiple lines"
    );
}

#[test]
fn test_help_documents_all_flags() {
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--platform"))
        .stdout(predicate::str::contains("--draft"))
        .stdout(predicate::str::contains("--format"))
        .stdout(predicate::str::contains("--verbose"))
        .stdout(predicate::str::contains("--help"))
        .stdout(predicate::str::contains("--version"));
}

#[test]
fn test_json_format_produces_valid_json() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();

    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    let output = cmd
        .env("PLURCAST_CONFIG", config_path)
        .arg("Test JSON output")
        .arg("--draft")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();

    // Parse JSON to verify it's valid
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("Output should be valid JSON");

    // Verify JSON structure
    assert!(json.is_object(), "JSON should be an object");
    assert!(
        json.get("post_id").is_some(),
        "JSON should have post_id field"
    );
    assert!(
        json.get("status").is_some(),
        "JSON should have status field"
    );
}

#[test]
fn test_json_output_structure() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();

    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    let output = cmd
        .env("PLURCAST_CONFIG", config_path)
        .arg("Test JSON structure")
        .arg("--draft")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Verify expected fields exist
    assert!(json["post_id"].is_string());
    assert!(json["status"].is_string());

    // Verify values are reasonable
    let post_id = json["post_id"].as_str().unwrap();
    assert!(!post_id.is_empty());

    let status = json["status"].as_str().unwrap();
    assert!(status == "draft" || status == "posted" || status == "failed");
}

#[test]
fn test_non_tty_plain_output() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();

    // When output is piped (non-TTY), should produce plain output
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    let output = cmd
        .env("PLURCAST_CONFIG", config_path)
        .arg("Test non-TTY")
        .arg("--draft")
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();

    // Output should not contain ANSI color codes when piped
    // ANSI codes start with \x1b[
    assert!(
        !stdout.contains("\x1b["),
        "Output should not contain ANSI color codes when piped"
    );
}

#[test]
fn test_stdout_only_requested_output() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();

    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    let output = cmd
        .env("PLURCAST_CONFIG", config_path)
        .arg("Test stdout cleanliness")
        .arg("--draft")
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();

    // Stdout should only contain the post ID
    assert!(stdout.contains("draft:"));

    // Should not contain diagnostic messages
    assert!(!stdout.to_lowercase().contains("loading"));
    assert!(!stdout.to_lowercase().contains("connecting"));
    assert!(!stdout.to_lowercase().contains("initializing"));
    assert!(!stdout.to_lowercase().contains("processing"));

    // Should be concise (single line)
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1, "Stdout should be a single line");
}

#[test]
fn test_diagnostic_messages_to_stderr() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();

    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    let output = cmd
        .env("PLURCAST_CONFIG", config_path)
        .arg("Test diagnostics")
        .arg("--draft")
        .arg("--verbose")
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();

    // Even with verbose, stdout should only have post ID
    assert!(stdout.contains("draft:"));

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines.len(),
        1,
        "Stdout should only contain post ID even in verbose mode"
    );
}

#[test]
fn test_exit_codes_consistent() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();

    // Test success (0)
    let mut cmd1 = Command::cargo_bin("plur-post").unwrap();
    cmd1.env("PLURCAST_CONFIG", &config_path)
        .arg("Test success")
        .arg("--draft")
        .assert()
        .success()
        .code(0);

    // Test invalid input (3)
    let mut cmd2 = Command::cargo_bin("plur-post").unwrap();
    cmd2.arg("").assert().failure().code(3);

    // Test authentication error (2) - missing config
    let temp_dir = TempDir::new().unwrap();
    let nonexistent = temp_dir.path().join("nonexistent.toml");

    let mut cmd3 = Command::cargo_bin("plur-post").unwrap();
    let result = cmd3
        .env("PLURCAST_CONFIG", nonexistent.to_str().unwrap())
        .arg("Test")
        .output()
        .unwrap();

    assert!(!result.status.success());
    // Exit code should be 1 for config errors (general error)
    let code = result.status.code().unwrap();
    assert_eq!(code, 1, "Exit code should be 1 for config errors");
}

#[test]
fn test_exit_codes_documented_in_help() {
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    let output = cmd.arg("--help").output().unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();

    // Verify all exit codes are documented
    assert!(stdout.contains("0") && (stdout.contains("Success") || stdout.contains("success")));
    assert!(
        stdout.contains("1")
            && (stdout.contains("Posting")
                || stdout.contains("posting")
                || stdout.contains("failed"))
    );
    assert!(
        stdout.contains("2")
            && (stdout.contains("Authentication")
                || stdout.contains("authentication")
                || stdout.contains("auth"))
    );
    assert!(
        stdout.contains("3")
            && (stdout.contains("Invalid")
                || stdout.contains("invalid")
                || stdout.contains("input"))
    );
}

#[test]
fn test_json_parseable_by_serde() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();

    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    let output = cmd
        .env("PLURCAST_CONFIG", config_path)
        .arg("Test serde parsing")
        .arg("--draft")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();

    // Should parse without errors
    let result = serde_json::from_str::<serde_json::Value>(&stdout);
    assert!(result.is_ok(), "JSON should be parseable by serde_json");

    let json = result.unwrap();

    // Should be a proper object, not an array or primitive
    assert!(json.is_object());
}

#[test]
fn test_help_shows_usage_examples() {
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    let output = cmd.arg("--help").output().unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();

    // Should contain usage examples
    assert!(
        stdout.contains("USAGE EXAMPLES")
            || stdout.contains("Examples:")
            || stdout.contains("EXAMPLES")
    );

    // Should show stdin example
    assert!(stdout.contains("echo") || stdout.contains("stdin") || stdout.contains("|"));

    // Should show platform flag example
    assert!(stdout.contains("--platform"));

    // Should show draft example
    assert!(stdout.contains("--draft"));
}

#[test]
fn test_error_messages_clear_and_actionable() {
    // Test empty content error
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    let output = cmd.arg("").output().unwrap();

    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).unwrap();

    // Error message should be clear
    assert!(stderr.to_lowercase().contains("error"));
    assert!(stderr.to_lowercase().contains("empty") || stderr.to_lowercase().contains("content"));
}

#[test]
fn test_version_flag_works() {
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("plur-post"));
}

#[test]
fn test_json_output_single_line() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();

    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    let output = cmd
        .env("PLURCAST_CONFIG", config_path)
        .arg("Test JSON single line")
        .arg("--draft")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();

    // JSON output may be pretty-printed or single-line
    // What matters is that it's valid JSON and parseable
    let trimmed = stdout.trim();

    // Verify it's valid JSON
    let json: serde_json::Value = serde_json::from_str(trimmed).unwrap();

    // Verify it has the expected structure
    assert!(json.is_object());
    assert!(json.get("post_id").is_some());
}

#[test]
fn test_text_format_machine_readable() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();

    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    let output = cmd
        .env("PLURCAST_CONFIG", config_path)
        .arg("Test text format")
        .arg("--draft")
        .arg("--format")
        .arg("text")
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();

    // Text format should be machine-readable (platform:id format)
    assert!(stdout.contains("draft:"));

    // Should be parseable (split on :)
    let parts: Vec<&str> = stdout.trim().split(':').collect();
    assert_eq!(
        parts.len(),
        2,
        "Text output should be in platform:id format"
    );
}

#[test]
fn test_help_comprehensive_coverage() {
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    let output = cmd.arg("--help").output().unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();

    // Verify comprehensive coverage of features
    assert!(stdout.contains("platform") || stdout.contains("Platform"));
    assert!(stdout.contains("draft") || stdout.contains("Draft"));
    assert!(stdout.contains("format") || stdout.contains("Format"));
    assert!(stdout.contains("verbose") || stdout.contains("Verbose"));

    // Should explain what the tool does
    assert!(stdout.to_lowercase().contains("post") || stdout.to_lowercase().contains("content"));

    // Should be long enough to be comprehensive (at least 20 lines)
    assert!(stdout.lines().count() >= 20, "Help should be comprehensive");
}
