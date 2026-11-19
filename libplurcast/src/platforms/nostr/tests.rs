//! Unit tests for the Nostr platform.

use super::*;
use crate::config::NostrConfig;
use tempfile::TempDir;

fn create_test_config() -> NostrConfig {
    NostrConfig {
        enabled: true,
        keys_file: "/tmp/test_keys".to_string(),
        relays: vec![
            "wss://relay.damus.io".to_string(),
            "wss://nos.lol".to_string(),
        ],
        default_pow_difficulty: None,
    }
}

#[test]
fn test_key_parsing_hex_format() {
    let temp_dir = TempDir::new().unwrap();
    let keys_file = temp_dir.path().join("hex_keys");

    // Generate a test key and get its hex representation
    let test_keys = Keys::generate();
    let hex_key = test_keys.secret_key().to_secret_hex();

    // Write hex key to file (should be 64 characters)
    assert_eq!(hex_key.len(), 64, "Hex key should be 64 characters");
    std::fs::write(&keys_file, &hex_key).unwrap();

    // Test parsing
    let config = NostrConfig {
        enabled: true,
        keys_file: keys_file.to_str().unwrap().to_string(),
        relays: vec![],
        default_pow_difficulty: None,
    };

    let mut platform = NostrPlatform::new(&config);
    let keys_content = std::fs::read_to_string(&keys_file).unwrap();
    let result = platform.load_keys_from_string(&keys_content);

    assert!(result.is_ok(), "Should parse valid hex key");
    assert!(platform.keys.is_some());
}

#[test]
fn test_key_parsing_bech32_nsec_format() {
    let temp_dir = TempDir::new().unwrap();
    let keys_file = temp_dir.path().join("bech32_keys");

    // Generate a test key and get its bech32 representation
    let test_keys = Keys::generate();
    let bech32_key = test_keys.secret_key().to_bech32().unwrap();

    // Verify it starts with nsec
    assert!(
        bech32_key.starts_with("nsec"),
        "Bech32 key should start with nsec"
    );
    std::fs::write(&keys_file, &bech32_key).unwrap();

    // Test parsing
    let config = NostrConfig {
        enabled: true,
        keys_file: keys_file.to_str().unwrap().to_string(),
        relays: vec![],
        default_pow_difficulty: None,
    };

    let mut platform = NostrPlatform::new(&config);
    let keys_content = std::fs::read_to_string(&keys_file).unwrap();
    let result = platform.load_keys_from_string(&keys_content);

    assert!(result.is_ok(), "Should parse valid bech32 nsec key");
    assert!(platform.keys.is_some());
}

#[test]
fn test_key_parsing_invalid_hex_format() {
    let temp_dir = TempDir::new().unwrap();
    let keys_file = temp_dir.path().join("invalid_hex_keys");

    // Write invalid hex key (wrong length)
    std::fs::write(&keys_file, "invalid_hex_key_too_short").unwrap();

    let config = NostrConfig {
        enabled: true,
        keys_file: keys_file.to_str().unwrap().to_string(),
        relays: vec![],
        default_pow_difficulty: None,
    };

    let mut platform = NostrPlatform::new(&config);
    let keys_content = std::fs::read_to_string(&keys_file).unwrap();
    let result = platform.load_keys_from_string(&keys_content);

    assert!(result.is_err(), "Should fail on invalid hex key");

    match result {
        Err(crate::PlurcastError::Platform(PlatformError::Authentication(msg))) => {
            assert!(msg.contains("must be 64-character hex or bech32 nsec format"));
        }
        _ => panic!("Expected authentication error"),
    }
}

#[test]
fn test_key_parsing_invalid_bech32_format() {
    let temp_dir = TempDir::new().unwrap();
    let keys_file = temp_dir.path().join("invalid_bech32_keys");

    // Write invalid bech32 key
    std::fs::write(&keys_file, "nsec_invalid_checksum_12345").unwrap();

    let config = NostrConfig {
        enabled: true,
        keys_file: keys_file.to_str().unwrap().to_string(),
        relays: vec![],
        default_pow_difficulty: None,
    };

    let mut platform = NostrPlatform::new(&config);
    let keys_content = std::fs::read_to_string(&keys_file).unwrap();
    let result = platform.load_keys_from_string(&keys_content);

    assert!(result.is_err(), "Should fail on invalid bech32 key");
}

#[test]
fn test_key_parsing_missing_file() {
    // Test that file I/O fails appropriately
    let missing_file = std::path::PathBuf::from("/nonexistent/path/keys");
    let file_result = std::fs::read_to_string(&missing_file);
    assert!(
        file_result.is_err(),
        "Should fail when keys file doesn't exist"
    );

    // Test that the parser also rejects invalid content
    let config = NostrConfig {
        enabled: true,
        keys_file: "/nonexistent/path/keys".to_string(),
        relays: vec![],
        default_pow_difficulty: None,
    };

    let mut platform = NostrPlatform::new(&config);
    let result = platform.load_keys_from_string("not_a_valid_key");

    assert!(result.is_err(), "Should fail on invalid key format");

    match result {
        Err(crate::PlurcastError::Platform(PlatformError::Authentication(msg))) => {
            assert!(msg.contains("must be 64-character hex or bech32 nsec format"));
        }
        _ => panic!("Expected authentication error"),
    }
}

#[test]
fn test_content_validation_empty_content() {
    let config = create_test_config();
    let platform = NostrPlatform::new(&config);

    let result = platform.validate_content("");

    assert!(result.is_err(), "Should fail on empty content");

    match result {
        Err(crate::PlurcastError::Platform(PlatformError::Validation(msg))) => {
            assert!(msg.contains("Content cannot be empty"));
            assert!(msg.contains("Nostr"));
        }
        _ => panic!("Expected validation error"),
    }
}

#[test]
fn test_content_validation_normal_content() {
    let config = create_test_config();
    let platform = NostrPlatform::new(&config);

    let result = platform.validate_content("This is a normal post");

    assert!(result.is_ok(), "Should accept normal content");
}

#[test]
fn test_content_validation_long_content() {
    let config = create_test_config();
    let platform = NostrPlatform::new(&config);

    // Create content longer than 280 characters
    let long_content = "a".repeat(300);

    // Should still succeed (Nostr has no hard limit), but may log a warning
    let result = platform.validate_content(&long_content);

    assert!(
        result.is_ok(),
        "Should accept long content (Nostr has no hard limit)"
    );
}

#[test]
fn test_content_validation_exactly_280_chars() {
    let config = create_test_config();
    let platform = NostrPlatform::new(&config);

    let content = "a".repeat(280);
    let result = platform.validate_content(&content);

    assert!(
        result.is_ok(),
        "Should accept content at 280 character boundary"
    );
}

#[tokio::test]
async fn test_posting_without_authentication() {
    let config = create_test_config();
    let platform = NostrPlatform::new(&config);

    // Try to post without authenticating
    let post = crate::Post::new("Test content".to_string());
    let result = platform.post(&post).await;

    assert!(result.is_err(), "Should fail when not authenticated");

    match result {
        Err(crate::PlurcastError::Platform(PlatformError::Authentication(msg))) => {
            assert!(msg.contains("Not authenticated"));
            assert!(msg.contains("Nostr"));
        }
        _ => panic!("Expected authentication error"),
    }
}

#[tokio::test]
async fn test_authenticate_without_keys() {
    let config = create_test_config();
    let mut platform = NostrPlatform::new(&config);

    // Try to authenticate without loading keys
    let result = platform.authenticate().await;

    assert!(result.is_err(), "Should fail when keys not loaded");

    match result {
        Err(crate::PlurcastError::Platform(PlatformError::Authentication(msg))) => {
            assert!(msg.contains("Keys not loaded"));
            assert!(msg.contains("Nostr"));
        }
        _ => panic!("Expected authentication error"),
    }
}

#[tokio::test]
async fn test_authenticate_sets_authenticated_flag() {
    let temp_dir = TempDir::new().unwrap();
    let keys_file = temp_dir.path().join("test_keys");

    // Generate and save test keys
    let test_keys = Keys::generate();
    let hex_key = test_keys.secret_key().to_secret_hex();
    std::fs::write(&keys_file, &hex_key).unwrap();

    let config = NostrConfig {
        enabled: true,
        keys_file: keys_file.to_str().unwrap().to_string(),
        relays: vec![], // Empty relays to avoid actual network connections
        default_pow_difficulty: None,
    };

    let mut platform = NostrPlatform::new(&config);

    // Load keys
    let keys_content = std::fs::read_to_string(&keys_file).unwrap();
    platform.load_keys_from_string(&keys_content).unwrap();

    // Verify not authenticated initially
    assert!(!platform.authenticated);

    // Authenticate (with empty relays, this should succeed without network calls)
    let result = platform.authenticate().await;

    // Should succeed even with no relays
    assert!(
        result.is_ok(),
        "Authentication should succeed with loaded keys"
    );

    // Verify authenticated flag is set
    assert!(platform.authenticated);
}

#[test]
fn test_platform_name() {
    let config = create_test_config();
    let platform = NostrPlatform::new(&config);

    assert_eq!(platform.name(), "nostr");
}

#[test]
fn test_key_parsing_with_whitespace() {
    let temp_dir = TempDir::new().unwrap();
    let keys_file = temp_dir.path().join("keys_with_whitespace");

    // Generate a test key with surrounding whitespace
    let test_keys = Keys::generate();
    let hex_key = test_keys.secret_key().to_secret_hex();
    let key_with_whitespace = format!("\n  {}  \n", hex_key);

    std::fs::write(&keys_file, key_with_whitespace).unwrap();

    let config = NostrConfig {
        enabled: true,
        keys_file: keys_file.to_str().unwrap().to_string(),
        relays: vec![],
        default_pow_difficulty: None,
    };

    let mut platform = NostrPlatform::new(&config);
    let keys_content = std::fs::read_to_string(&keys_file).unwrap();
    let result = platform.load_keys_from_string(&keys_content);

    assert!(result.is_ok(), "Should handle whitespace in keys file");
}

#[test]
fn test_multiple_relays_configuration() {
    let config = NostrConfig {
        enabled: true,
        keys_file: "/tmp/keys".to_string(),
        relays: vec![
            "wss://relay1.example.com".to_string(),
            "wss://relay2.example.com".to_string(),
            "wss://relay3.example.com".to_string(),
        ],
        default_pow_difficulty: None,
    };

    let platform = NostrPlatform::new(&config);

    assert_eq!(platform.relays.len(), 3);
    assert_eq!(platform.relays[0], "wss://relay1.example.com");
    assert_eq!(platform.relays[1], "wss://relay2.example.com");
    assert_eq!(platform.relays[2], "wss://relay3.example.com");
}

#[test]
fn test_character_limit_returns_none() {
    let config = create_test_config();
    let platform = NostrPlatform::new(&config);

    // Nostr has no hard character limit
    assert_eq!(platform.character_limit(), None);
}

#[test]
fn test_is_configured_without_keys() {
    let config = create_test_config();
    let platform = NostrPlatform::new(&config);

    // Platform should not be configured without keys loaded
    assert!(!platform.is_configured());
}

#[test]
fn test_is_configured_with_keys() {
    let temp_dir = TempDir::new().unwrap();
    let keys_file = temp_dir.path().join("test_keys");

    // Generate and save test keys
    let test_keys = Keys::generate();
    let hex_key = test_keys.secret_key().to_secret_hex();
    std::fs::write(&keys_file, &hex_key).unwrap();

    let config = NostrConfig {
        enabled: true,
        keys_file: keys_file.to_str().unwrap().to_string(),
        relays: vec![],
        default_pow_difficulty: None,
    };

    let mut platform = NostrPlatform::new(&config);

    // Should not be configured before loading keys
    assert!(!platform.is_configured());

    // Load keys
    let keys_content = std::fs::read_to_string(&keys_file).unwrap();
    platform.load_keys_from_string(&keys_content).unwrap();

    // Should be configured after loading keys
    assert!(platform.is_configured());
}

#[test]
fn test_is_configured_with_invalid_keys_file() {
    let config = NostrConfig {
        enabled: true,
        keys_file: "/nonexistent/path/keys".to_string(),
        relays: vec![],
        default_pow_difficulty: None,
    };

    let mut platform = NostrPlatform::new(&config);

    // Should not be configured initially
    assert!(!platform.is_configured());

    // Try to load invalid key content (will fail)
    let result = platform.load_keys_from_string("invalid_key_format");
    assert!(result.is_err());

    // Should still not be configured after failed load
    assert!(!platform.is_configured());
}

// =========================================================================
// NIP-10 Reply Tag Tests
// =========================================================================

#[test]
fn test_nip10_reply_tag_format() {
    // Test that we create proper NIP-10 reply tags with "reply" marker
    let event_id =
        EventId::parse("331c0d98945a9b7e4adbc9e9e1a5b7e7eaaabac15dc404b936361bf122c4e293").unwrap();

    // Create the tag the same way we do in post()
    let tag = Tag::custom(
        "e",
        vec![event_id.to_hex(), String::new(), "reply".to_string()],
    );

    // Verify the tag structure: ["e", "<event-id>", "", "reply"]
    let tag_vec = tag.as_slice();
    assert_eq!(tag_vec.len(), 4, "NIP-10 reply tag should have 4 elements");
    assert_eq!(tag_vec[0], "e", "First element should be 'e'");
    assert_eq!(
        tag_vec[1], "331c0d98945a9b7e4adbc9e9e1a5b7e7eaaabac15dc404b936361bf122c4e293",
        "Second element should be event ID in hex"
    );
    assert_eq!(tag_vec[2], "", "Third element should be empty relay URL");
    assert_eq!(
        tag_vec[3], "reply",
        "Fourth element should be 'reply' marker"
    );
}

#[test]
fn test_nip10_reply_tag_with_different_event_id() {
    // Test with a bech32 note ID to ensure parsing works
    let note_id = "note1xvwqmxy5t2dhujkme857rfdhul424wkpthzqfwfkxcdlzgkyu2fsra5prs";
    let event_id = EventId::parse(note_id).unwrap();

    let tag = Tag::custom(
        "e",
        vec![event_id.to_hex(), String::new(), "reply".to_string()],
    );

    let tag_vec = tag.as_slice();
    assert_eq!(tag_vec.len(), 4);
    assert_eq!(tag_vec[0], "e");
    // Verify it's a valid 64-char hex string
    assert_eq!(tag_vec[1].len(), 64, "Event ID should be 64-char hex");
    assert!(
        tag_vec[1].chars().all(|c| c.is_ascii_hexdigit()),
        "Event ID should be hexadecimal"
    );
    assert_eq!(tag_vec[2], "");
    assert_eq!(tag_vec[3], "reply");
}

// =========================================================================
// NIP-65 Relay List Tests
// =========================================================================

#[test]
fn test_supports_relay_list() {
    let config = create_test_config();
    let platform = NostrPlatform::new(&config);
    assert!(platform.supports_relay_list());
}

#[test]
fn test_relay_list_pubkey_without_keys() {
    let config = create_test_config();
    let platform = NostrPlatform::new(&config);
    assert!(platform.relay_list_pubkey().is_none());
}

#[test]
fn test_relay_list_pubkey_with_keys() {
    let config = create_test_config();
    let mut platform = NostrPlatform::new(&config);

    // Load test keys
    let test_keys = Keys::generate();
    let hex_key = test_keys.secret_key().to_secret_hex();
    platform.load_keys_from_string(&hex_key).unwrap();

    let pubkey = platform.relay_list_pubkey();
    assert!(pubkey.is_some());

    // Verify pubkey is 64-char hex
    let pubkey = pubkey.unwrap();
    assert_eq!(pubkey.len(), 64);
    assert!(pubkey.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_relay_count() {
    let config = create_test_config();
    let platform = NostrPlatform::new(&config);
    assert_eq!(platform.relay_count(), 2); // Default config has 2 relays
}

#[test]
fn test_relays_accessor() {
    let config = create_test_config();
    let platform = NostrPlatform::new(&config);
    let relays = platform.relays();
    assert_eq!(relays.len(), 2);
    assert!(relays.contains(&"wss://relay.damus.io".to_string()));
    assert!(relays.contains(&"wss://nos.lol".to_string()));
}

#[tokio::test]
async fn test_publish_relay_list_requires_auth() {
    let config = create_test_config();
    let platform = NostrPlatform::new(&config);

    // Should fail without authentication
    let result = platform.publish_relay_list().await;
    assert!(result.is_err());
}
