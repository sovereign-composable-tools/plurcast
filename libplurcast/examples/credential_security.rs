//! Example demonstrating credential security features
//!
//! This example shows:
//! - How to configure secure storage backends
//! - Encrypted file storage with master password
//! - OS keyring storage

use libplurcast::credentials::{CredentialConfig, CredentialManager, StorageBackend};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Credential Security Example ===\n");

    // Example 1: Encrypted file storage (SECURE)
    println!("1. Encrypted file storage (SECURE):");
    let encrypted_config = CredentialConfig {
        storage: StorageBackend::Encrypted,
        path: "/tmp/plurcast-test-encrypted".to_string(),
        master_password: Some("my-strong-password-12345".to_string()),
    };

    let encrypted_manager = CredentialManager::new(encrypted_config)?;

    if !encrypted_manager.is_insecure() {
        println!("✓ Using secure storage!");
        println!(
            "  Primary backend: {:?}",
            encrypted_manager.primary_backend()
        );
    }

    encrypted_manager.store("plurcast.test", "api_key", "secret123")?;
    println!();

    // Example 2: OS Keyring storage (MOST SECURE)
    println!("2. OS Keyring storage (MOST SECURE):");
    let keyring_config = CredentialConfig {
        storage: StorageBackend::Keyring,
        path: "/tmp/plurcast-test".to_string(), // Not used for keyring
        master_password: None,
    };

    match CredentialManager::new(keyring_config) {
        Ok(keyring_manager) => {
            if !keyring_manager.is_insecure() {
                println!("✓ Using OS keyring!");
                println!("  Primary backend: {:?}", keyring_manager.primary_backend());
            }
        }
        Err(e) => {
            eprintln!("⚠️  OS keyring not available: {}", e);
            eprintln!("   This is common in CI/CD or headless environments");
        }
    }

    println!("\n=== Security Recommendations ===");
    println!("1. BEST: Use OS keyring (storage = \"keyring\")");
    println!("   - macOS: Keychain");
    println!("   - Windows: Credential Manager");
    println!("   - Linux: Secret Service (GNOME Keyring/KWallet)");
    println!();
    println!("2. GOOD: Use encrypted files (storage = \"encrypted\")");
    println!("   - Requires master password");
    println!("   - Files encrypted with age encryption");
    println!();
    println!("Note: All storage backends now support multi-account credentials.");
    println!("Use --account flag to manage multiple accounts per platform.");

    Ok(())
}
