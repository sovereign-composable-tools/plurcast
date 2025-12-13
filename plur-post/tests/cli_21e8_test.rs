//! Integration tests for --21e8 easter egg flag

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_21e8_flag_without_nostr_pow() {
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.arg("test content").arg("--21e8").arg("--draft"); // Use draft to avoid actual posting

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("--21e8 requires --nostr-pow"));
}

#[test]
fn test_21e8_flag_with_nostr_pow() {
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.arg("test content")
        .arg("--nostr-pow")
        .arg("8")
        .arg("--21e8")
        .arg("--draft"); // Draft mode to test parsing

    // Should succeed (parsing works)
    cmd.assert().success();
}

#[test]
fn test_21e8_flag_not_in_help() {
    let mut cmd = Command::cargo_bin("plur-post").unwrap();

    cmd.arg("--help");

    // Verify --21e8 is NOT mentioned in help (true easter egg)
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--21e8").not());
}
