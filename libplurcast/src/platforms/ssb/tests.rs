//! Tests for SSB platform implementation
//!
//! This module contains comprehensive tests for all SSB components.
//! Tests are organized by component: platform, keypair, message, and replication.

use super::*;
use crate::config::SSBConfig;
use crate::credentials::{CredentialConfig, CredentialManager, StorageBackend};
use crate::platforms::Platform;
use tempfile::TempDir;

// ============================================================================
// Platform Tests
// ============================================================================

#[test]
fn test_platform_name() {
    let config = SSBConfig {
        enabled: true,
        feed_path: "~/.plurcast-ssb".to_string(),
        pubs: vec![],
    };
    let platform = SSBPlatform::new(&config);
    assert_eq!(platform.name(), "ssb");
}

#[test]
fn test_character_limit() {
    let config = SSBConfig {
        enabled: true,
        feed_path: "~/.plurcast-ssb".to_string(),
        pubs: vec![],
    };
    let platform = SSBPlatform::new(&config);
    assert_eq!(platform.character_limit(), None);
}

#[test]
fn test_validate_content_within_limit() {
    let config = SSBConfig {
        enabled: true,
        feed_path: "~/.plurcast-ssb".to_string(),
        pubs: vec![],
    };
    let platform = SSBPlatform::new(&config);

    let content = "Hello SSB!";
    assert!(platform.validate_content(content).is_ok());
}

#[test]
fn test_validate_content_exceeds_limit() {
    let config = SSBConfig {
        enabled: true,
        feed_path: "~/.plurcast-ssb".to_string(),
        pubs: vec![],
    };
    let platform = SSBPlatform::new(&config);

    // Create content larger than 8KB
    let content = "x".repeat(8193);
    let result = platform.validate_content(&content);
    assert!(result.is_err());

    if let Err(e) = result {
        assert!(e.to_string().contains("exceeds"));
    }
}

#[test]
fn test_is_configured() {
    let config = SSBConfig {
        enabled: true,
        feed_path: "~/.plurcast-ssb".to_string(),
        pubs: vec![],
    };
    let platform = SSBPlatform::new(&config);
    assert!(platform.is_configured());

    let config_disabled = SSBConfig {
        enabled: false,
        feed_path: "~/.plurcast-ssb".to_string(),
        pubs: vec![],
    };
    let platform_disabled = SSBPlatform::new(&config_disabled);
    assert!(!platform_disabled.is_configured());
}

// ============================================================================
// Keypair Tests
// ============================================================================

#[test]
fn test_keypair_generation() {
    let keypair = SSBKeypair::generate();

    assert_eq!(keypair.curve, "ed25519");
    assert!(keypair.public.ends_with(".ed25519"));
    assert!(keypair.private.ends_with(".ed25519"));
    assert!(keypair.id.starts_with('@'));
    assert!(keypair.id.ends_with(".ed25519"));
    assert!(keypair.validate().is_ok());
}

#[test]
fn test_keypair_validation_valid() {
    let keypair = SSBKeypair::generate();
    assert!(keypair.validate().is_ok());
}

#[test]
fn test_keypair_validation_invalid_curve() {
    let mut keypair = SSBKeypair::generate();
    keypair.curve = "invalid".to_string();

    let result = keypair.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid curve"));
}

#[test]
fn test_keypair_serialization() {
    let keypair = SSBKeypair::generate();
    let json = keypair.to_json().unwrap();

    assert!(json.contains("\"curve\""));
    assert!(json.contains("\"public\""));
    assert!(json.contains("\"private\""));
    assert!(json.contains("\"id\""));
    assert!(json.contains("\"ed25519\""));
}

#[test]
fn test_keypair_deserialization() {
    let keypair = SSBKeypair::generate();
    let json = keypair.to_json().unwrap();
    let deserialized = SSBKeypair::from_json(&json).unwrap();

    assert_eq!(keypair.curve, deserialized.curve);
    assert_eq!(keypair.public, deserialized.public);
    assert_eq!(keypair.private, deserialized.private);
    assert_eq!(keypair.id, deserialized.id);
}

// ============================================================================
// Message Tests
// ============================================================================

#[test]
fn test_message_creation() {
    let keypair = SSBKeypair::generate();
    let message = SSBMessage::new_post(&keypair.id, 1, None, "Hello SSB!");

    assert_eq!(message.author, keypair.id);
    assert_eq!(message.sequence, 1);
    assert_eq!(message.hash, "sha256");
    assert!(message.previous.is_none());
    assert!(message.signature.is_none());

    let content_obj = message.content.as_object().unwrap();
    assert_eq!(content_obj.get("type").unwrap().as_str().unwrap(), "post");
    assert_eq!(
        content_obj.get("text").unwrap().as_str().unwrap(),
        "Hello SSB!"
    );
}

#[test]
fn test_message_signing() {
    let keypair = SSBKeypair::generate();
    let mut message = SSBMessage::new_post(&keypair.id, 1, None, "Hello SSB!");

    message.sign(&keypair).unwrap();
    assert!(message.signature.is_some());

    let signature = message.signature.as_ref().unwrap();
    assert!(signature.ends_with(".sig.ed25519"));
}

#[test]
fn test_message_validation() {
    let keypair = SSBKeypair::generate();
    let message = SSBMessage::new_post(&keypair.id, 1, None, "Hello SSB!");

    assert!(message.validate().is_ok());
}

#[test]
fn test_message_hash_calculation() {
    let keypair = SSBKeypair::generate();
    let mut message = SSBMessage::new_post(&keypair.id, 1, None, "Hello SSB!");

    message.sign(&keypair).unwrap();
    let hash = message.calculate_hash().unwrap();

    assert!(hash.starts_with('%'));
    assert!(hash.ends_with(".sha256"));
}

// ============================================================================
// Credential Manager Integration Tests
// ============================================================================

#[test]
fn test_store_and_retrieve_keypair() {
    let temp_dir = TempDir::new().unwrap();

    let config = CredentialConfig {
        storage: StorageBackend::Encrypted,
        path: temp_dir.path().to_string_lossy().to_string(),
        master_password: Some("test-password-12345".to_string()),
    };

    let credentials = CredentialManager::new(config).unwrap();
    let keypair = SSBKeypair::generate();

    SSBPlatform::store_keypair(&credentials, &keypair, "test-account", true).unwrap();
    let retrieved = SSBPlatform::retrieve_keypair(&credentials, "test-account").unwrap();

    assert_eq!(keypair.curve, retrieved.curve);
    assert_eq!(keypair.public, retrieved.public);
    assert_eq!(keypair.private, retrieved.private);
    assert_eq!(keypair.id, retrieved.id);
}

#[test]
fn test_has_keypair() {
    let temp_dir = TempDir::new().unwrap();

    let config = CredentialConfig {
        storage: StorageBackend::Encrypted,
        path: temp_dir.path().to_string_lossy().to_string(),
        master_password: Some("test-password-12345".to_string()),
    };

    let credentials = CredentialManager::new(config).unwrap();

    assert!(!SSBPlatform::has_keypair(&credentials, "test-account").unwrap());

    let keypair = SSBKeypair::generate();
    SSBPlatform::store_keypair(&credentials, &keypair, "test-account", true).unwrap();

    assert!(SSBPlatform::has_keypair(&credentials, "test-account").unwrap());
}

// ============================================================================
// Replication Tests
// ============================================================================

#[test]
fn test_pub_address_parsing() {
    // Use valid base64 (32 bytes for Ed25519 public key)
    let valid_base64 = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
    let address_str = format!("net:hermies.club:8008~shs:{}", valid_base64);
    let address = PubAddress::parse(&address_str).unwrap();

    assert_eq!(address.protocol, "net");
    assert_eq!(address.host, "hermies.club");
    assert_eq!(address.port, 8008);
    assert_eq!(address.auth, "shs");
    assert_eq!(address.pubkey, valid_base64);
}

#[test]
fn test_pub_address_parsing_invalid() {
    let result = PubAddress::parse("invalid-address");
    assert!(result.is_err());
}

#[test]
fn test_pub_address_to_string() {
    let address = PubAddress {
        protocol: "net".to_string(),
        host: "hermies.club".to_string(),
        port: 8008,
        auth: "shs".to_string(),
        pubkey: "base64key".to_string(),
    };

    assert_eq!(address.to_string(), "net:hermies.club:8008~shs:base64key");
}

// TODO: Extract remaining 70+ tests from ssb.rs.old
// Categories to add:
// - More message validation tests
// - Feed database tests
// - Posting integration tests
// - Replication protocol tests
// - Import/export tests
