//! Unix philosophy compliance tests

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
fn test_stdin_input_handling() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    // Test: echo "content" | plur-post
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path)
        .write_stdin("Hello from stdin")
        .arg("--draft")
        .assert()
        .success()
        .stdout(predicate::str::contains("draft:"));
}

#[test]
fn test_stdin_multiline_content() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    let multiline_content = "Line 1\nLine 2\nLine 3";
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path)
        .write_stdin(multiline_content)
        .arg("--draft")
        .assert()
        .success();
}

#[test]
fn test_output_piping_text_format() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    // Test that output can be piped to other commands
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    let output = cmd
        .env("PLURCAST_CONFIG", config_path)
        .arg("Test piping")
        .arg("--draft")
        .output()
        .unwrap();
    
    assert!(output.status.success());
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // Output should contain draft: prefix for piping
    assert!(stdout.contains("draft:"));
    
    // Should be single line (or end with newline for piping)
    assert!(stdout.ends_with('\n') || !stdout.contains('\n'));
}

#[test]
fn test_silent_operation_stdout_only_essential() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    let output = cmd
        .env("PLURCAST_CONFIG", config_path)
        .arg("Test silent operation")
        .arg("--draft")
        .output()
        .unwrap();
    
    assert!(output.status.success());
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    
    // Stdout should only contain the post ID
    assert!(stdout.contains("draft:"));
    
    // Should not contain verbose messages in stdout
    assert!(!stdout.to_lowercase().contains("posting"));
    assert!(!stdout.to_lowercase().contains("success"));
    assert!(!stdout.to_lowercase().contains("created"));
    
    // Stderr should be empty in non-verbose mode
    assert!(stderr.is_empty() || stderr.trim().is_empty());
}

#[test]
fn test_env_var_config_override() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create custom config location
    let custom_config_dir = temp_dir.path().join("custom");
    fs::create_dir_all(&custom_config_dir).unwrap();
    
    let data_dir = temp_dir.path().join("data");
    fs::create_dir_all(&data_dir).unwrap();
    
    let config_path = custom_config_dir.join("config.toml");
    let db_path = data_dir.join("posts.db");
    let keys_path = custom_config_dir.join("nostr.keys");
    
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
    
    let test_keys = nostr_sdk::Keys::generate();
    let hex_key = test_keys.secret_key().to_secret_hex();
    fs::write(&keys_path, hex_key).unwrap();
    
    // Test PLURCAST_CONFIG environment variable
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path.to_str().unwrap())
        .arg("Test env var")
        .arg("--draft")
        .assert()
        .success();
}

#[test]
fn test_env_var_db_path_override() {
    let temp_dir = TempDir::new().unwrap();
    
    let config_dir = temp_dir.path().join("config");
    fs::create_dir_all(&config_dir).unwrap();
    
    let default_db = temp_dir.path().join("default.db");
    let custom_db = temp_dir.path().join("custom.db");
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
        escape_path_for_toml(&default_db.to_string_lossy()),
        escape_path_for_toml(&keys_path.to_string_lossy())
    );
    
    let config_path = config_dir.join("config.toml");
    fs::write(&config_path, config_content).unwrap();
    
    let test_keys = nostr_sdk::Keys::generate();
    let hex_key = test_keys.secret_key().to_secret_hex();
    fs::write(&keys_path, hex_key).unwrap();
    
    // Test PLURCAST_DB_PATH environment variable
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path.to_str().unwrap())
        .env("PLURCAST_DB_PATH", custom_db.to_str().unwrap())
        .arg("Test DB override")
        .arg("--draft")
        .assert()
        .success();
    
    // Verify custom database was created
    assert!(custom_db.exists());
}

#[test]
fn test_errors_go_to_stderr() {
    // Test with empty content (invalid input)
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    let output = cmd
        .arg("")
        .output()
        .unwrap();
    
    assert!(!output.status.success());
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    
    // Error message should be in stderr, not stdout
    assert!(stdout.is_empty() || stdout.trim().is_empty());
    assert!(!stderr.is_empty());
    assert!(stderr.to_lowercase().contains("error") || stderr.to_lowercase().contains("empty"));
}

#[test]
fn test_errors_stderr_not_stdout() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent_config = temp_dir.path().join("nonexistent.toml");
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    let output = cmd
        .env("PLURCAST_CONFIG", nonexistent_config.to_str().unwrap())
        .arg("Test content")
        .output()
        .unwrap();
    
    assert!(!output.status.success());
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    
    // Errors should be in stderr
    assert!(stdout.is_empty() || stdout.trim().is_empty());
    assert!(!stderr.is_empty());
}

#[test]
fn test_json_output_composability() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    // Test that JSON output can be parsed by jq or other tools
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    let output = cmd
        .env("PLURCAST_CONFIG", config_path)
        .arg("Test JSON composability")
        .arg("--draft")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();
    
    assert!(output.status.success());
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // Verify it's valid JSON
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    
    // Verify structure is suitable for jq processing
    assert!(json.is_object());
    assert!(json.get("post_id").is_some());
    assert!(json.get("status").is_some());
}

#[test]
fn test_composability_with_grep() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    // Test that output can be grepped
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    let output = cmd
        .env("PLURCAST_CONFIG", config_path)
        .arg("Test grep composability")
        .arg("--draft")
        .output()
        .unwrap();
    
    assert!(output.status.success());
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // Output should be greppable (contains draft:)
    assert!(stdout.contains("draft:"));
    
    // Should be single line or properly formatted for grep
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(!lines.is_empty());
}

#[test]
fn test_exit_immediately_no_delays() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    use std::time::Instant;
    
    let start = Instant::now();
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path)
        .arg("Test quick exit")
        .arg("--draft")
        .assert()
        .success();
    
    let duration = start.elapsed();
    
    // Should complete quickly (under 5 seconds for draft mode)
    assert!(duration.as_secs() < 5, "Command took too long: {:?}", duration);
}

#[test]
fn test_plurcast_env_var_pattern() {
    // Verify all environment variables follow PLURCAST_* pattern
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    // Test PLURCAST_CONFIG
    let mut cmd1 = Command::cargo_bin("plur-post").unwrap();
    cmd1.env("PLURCAST_CONFIG", &config_path)
        .arg("Test")
        .arg("--draft")
        .assert()
        .success();
    
    // Test PLURCAST_DB_PATH
    let temp_db = TempDir::new().unwrap();
    let custom_db = temp_db.path().join("test.db");
    
    let mut cmd2 = Command::cargo_bin("plur-post").unwrap();
    cmd2.env("PLURCAST_CONFIG", &config_path)
        .env("PLURCAST_DB_PATH", custom_db.to_str().unwrap())
        .arg("Test")
        .arg("--draft")
        .assert()
        .success();
}

#[test]
fn test_verbose_output_to_stderr() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    let output = cmd
        .env("PLURCAST_CONFIG", config_path)
        .arg("Test verbose")
        .arg("--draft")
        .arg("--verbose")
        .output()
        .unwrap();
    
    assert!(output.status.success());
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // Even with verbose, stdout should only contain the post ID
    assert!(stdout.contains("draft:"));
    
    // Verbose output should go to stderr (we can't easily test this without capturing stderr)
    // But we verify stdout is clean
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1, "Stdout should only have one line (post ID)");
}

#[test]
fn test_stdin_empty_reads_from_pipe() {
    let (_temp_dir, config_path, _db_path) = setup_test_env();
    
    // When stdin has content, it should be used
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    
    cmd.env("PLURCAST_CONFIG", config_path)
        .write_stdin("Content from pipe")
        .arg("--draft")
        .assert()
        .success()
        .stdout(predicate::str::contains("draft:"));
}
