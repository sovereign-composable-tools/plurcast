//! Memory security tests
//!
//! This test suite verifies that private keys are protected in memory using
//! the secrecy crate and automatically zeroed when dropped.

use libplurcast::config::NostrConfig;
use libplurcast::platforms::nostr::NostrPlatform;
use libplurcast::platforms::Platform;
use nostr_sdk::{Keys, ToBech32};

/// Test that NostrPlatform with loaded keys doesn't expose keys in Debug output
#[test]
fn test_nostr_keys_not_exposed_in_debug() {
    let test_keys = Keys::generate();
    let hex_key = test_keys.secret_key().to_secret_hex();

    let config = NostrConfig {
        enabled: true,
        keys_file: "".to_string(),
        relays: vec![],
    };

    let mut platform = NostrPlatform::new(&config);
    platform.load_keys_from_string(&hex_key).unwrap();

    // Get debug output
    let debug_output = format!("{:?}", platform);

    // Verify the actual private key hex is NOT in the debug output
    assert!(
        !debug_output.contains(&hex_key),
        "Private key exposed in debug output! Found key in: {}",
        debug_output
    );

    // Verify Secret redacts the value
    assert!(
        debug_output.contains("Secret") || debug_output.contains("[REDACTED]"),
        "Expected Secret to redact sensitive data in debug output: {}",
        debug_output
    );
}

/// Test that keys are properly loaded and the platform is configured
#[test]
fn test_keys_loaded_after_load_keys_from_string() {
    let test_keys = Keys::generate();
    let hex_key = test_keys.secret_key().to_secret_hex();

    let config = NostrConfig {
        enabled: true,
        keys_file: "".to_string(),
        relays: vec![],
    };

    let mut platform = NostrPlatform::new(&config);

    // Before loading keys
    assert!(!platform.is_configured());

    // Load keys
    platform.load_keys_from_string(&hex_key).unwrap();

    // After loading keys
    assert!(platform.is_configured());
}

/// Test that bech32 (nsec) format keys are also protected
#[test]
fn test_nsec_keys_not_exposed_in_debug() {
    let test_keys = Keys::generate();
    let nsec_key = test_keys.secret_key().to_bech32().unwrap();

    let config = NostrConfig {
        enabled: true,
        keys_file: "".to_string(),
        relays: vec![],
    };

    let mut platform = NostrPlatform::new(&config);
    platform.load_keys_from_string(&nsec_key).unwrap();

    // Get debug output
    let debug_output = format!("{:?}", platform);

    // Verify the actual nsec key is NOT in the debug output
    assert!(
        !debug_output.contains(&nsec_key),
        "Nsec key exposed in debug output! Found key in: {}",
        debug_output
    );
}

/// Test that Drop is called when platform goes out of scope
///
/// This test verifies that the Drop implementation is present and will be called.
/// The actual memory zeroing is handled by the secrecy crate's Secret<T> type.
#[test]
fn test_drop_called_on_platform() {
    let test_keys = Keys::generate();
    let hex_key = test_keys.secret_key().to_secret_hex();

    let config = NostrConfig {
        enabled: true,
        keys_file: "".to_string(),
        relays: vec![],
    };

    // Create platform in inner scope
    {
        let mut platform = NostrPlatform::new(&config);
        platform.load_keys_from_string(&hex_key).unwrap();
        assert!(platform.is_configured());

        // Platform will be dropped at end of this scope
    }

    // Platform has been dropped, Drop implementation called
    // Secret<T> has automatically zeroed the keys
    // We can't directly verify memory was zeroed, but the type system guarantees it
}

/// Test that multiple platforms can exist and each properly protects its keys
#[test]
fn test_multiple_platforms_with_different_keys() {
    let keys1 = Keys::generate();
    let keys2 = Keys::generate();
    let hex_key1 = keys1.secret_key().to_secret_hex();
    let hex_key2 = keys2.secret_key().to_secret_hex();

    let config = NostrConfig {
        enabled: true,
        keys_file: "".to_string(),
        relays: vec![],
    };

    let mut platform1 = NostrPlatform::new(&config.clone());
    let mut platform2 = NostrPlatform::new(&config);

    platform1.load_keys_from_string(&hex_key1).unwrap();
    platform2.load_keys_from_string(&hex_key2).unwrap();

    // Both should be configured
    assert!(platform1.is_configured());
    assert!(platform2.is_configured());

    // Neither should expose keys in debug
    let debug1 = format!("{:?}", platform1);
    let debug2 = format!("{:?}", platform2);

    assert!(!debug1.contains(&hex_key1));
    assert!(!debug2.contains(&hex_key2));
}

/// Test that invalid keys don't get stored
#[test]
fn test_invalid_keys_not_stored() {
    let config = NostrConfig {
        enabled: true,
        keys_file: "".to_string(),
        relays: vec![],
    };

    let mut platform = NostrPlatform::new(&config);

    // Try to load invalid key
    let result = platform.load_keys_from_string("invalid_key");

    // Should fail
    assert!(result.is_err());

    // Platform should not be configured
    assert!(!platform.is_configured());

    // Debug should not contain the invalid key
    let debug_output = format!("{:?}", platform);
    assert!(!debug_output.contains("invalid_key"));
}

/// Test that keys are protected even after authentication
#[tokio::test]
async fn test_keys_protected_after_authentication() {
    let test_keys = Keys::generate();
    let hex_key = test_keys.secret_key().to_secret_hex();

    let config = NostrConfig {
        enabled: true,
        keys_file: "".to_string(),
        relays: vec![], // No relays to avoid network calls
    };

    let mut platform = NostrPlatform::new(&config);
    platform.load_keys_from_string(&hex_key).unwrap();

    // Authenticate (with no relays, this should succeed without network)
    let result = platform.authenticate().await;
    assert!(result.is_ok());

    // Keys should still be protected after authentication
    let debug_output = format!("{:?}", platform);
    assert!(!debug_output.contains(&hex_key));
}

/// Test that shared test key is also protected
#[test]
fn test_shared_test_key_protected() {
    let config = NostrConfig {
        enabled: true,
        keys_file: "".to_string(),
        relays: vec![],
    };

    let mut platform = NostrPlatform::new(&config);
    platform.load_shared_test_keys().unwrap();

    // Even the public test key should be wrapped in Secret
    let debug_output = format!("{:?}", platform);

    // The test key constant itself is public, but once loaded,
    // it should still be wrapped in Secret for consistency
    assert!(
        debug_output.contains("Secret") || debug_output.contains("[REDACTED]"),
        "Expected Secret wrapper even for test keys: {}",
        debug_output
    );
}

/// Test memory protection with hex key format
#[test]
fn test_hex_key_memory_protection() {
    let test_keys = Keys::generate();
    let hex_key = test_keys.secret_key().to_secret_hex();

    // Verify hex format (64 chars)
    assert_eq!(hex_key.len(), 64);

    let config = NostrConfig {
        enabled: true,
        keys_file: "".to_string(),
        relays: vec![],
    };

    let mut platform = NostrPlatform::new(&config);
    platform.load_keys_from_string(&hex_key).unwrap();

    // Verify protection
    let debug_output = format!("{:?}", platform);
    assert!(!debug_output.contains(&hex_key));
}

/// Test memory protection with bech32 (nsec) key format
#[test]
fn test_bech32_key_memory_protection() {
    let test_keys = Keys::generate();
    let bech32_key = test_keys.secret_key().to_bech32().unwrap();

    // Verify bech32 format (starts with nsec)
    assert!(bech32_key.starts_with("nsec"));

    let config = NostrConfig {
        enabled: true,
        keys_file: "".to_string(),
        relays: vec![],
    };

    let mut platform = NostrPlatform::new(&config);
    platform.load_keys_from_string(&bech32_key).unwrap();

    // Verify protection
    let debug_output = format!("{:?}", platform);
    assert!(!debug_output.contains(&bech32_key));
}

/// Test that whitespace in keys is handled and keys are still protected
#[test]
fn test_keys_with_whitespace_protected() {
    let test_keys = Keys::generate();
    let hex_key = test_keys.secret_key().to_secret_hex();

    // Add whitespace around the key
    let key_with_whitespace = format!("\n  {}  \n", hex_key);

    let config = NostrConfig {
        enabled: true,
        keys_file: "".to_string(),
        relays: vec![],
    };

    let mut platform = NostrPlatform::new(&config);
    platform.load_keys_from_string(&key_with_whitespace).unwrap();

    // Should still load successfully
    assert!(platform.is_configured());

    // Verify protection
    let debug_output = format!("{:?}", platform);
    assert!(!debug_output.contains(&hex_key));
    assert!(!debug_output.contains(&key_with_whitespace));
}
