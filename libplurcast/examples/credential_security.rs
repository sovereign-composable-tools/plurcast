//! Example demonstrating credential security features
//!
//! This example shows:
//! - How to check if you're using insecure storage
//! - How to configure secure storage backends
//! - Security warnings that users will see

use libplurcast::credentials::{CredentialConfig, CredentialManager, StorageBackend};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Credential Security Example ===\n");

    // Example 1: Plain text storage (INSECURE - will show warnings)
    println!("1. Plain text storage (INSECURE):");
    let plain_config = CredentialConfig {
        storage: StorageBackend::Plain,
        path: "/tmp/plurcast-test".to_string(),
        master_password: None,
    };

    let plain_manager = CredentialManager::new(plain_config)?;

    if plain_manager.is_insecure() {
        eprintln!("⚠️  WARNING: Using insecure plain text storage!");
        eprintln!("   Primary backend: {:?}", plain_manager.primary_backend());
        eprintln!("   Recommendation: Switch to 'keyring' or 'encrypted' storage\n");
    }

    // Storing will also show a warning
    plain_manager.store("plurcast.test", "api_key", "secret123")?;
    println!();

    // Example 2: Encrypted file storage (SECURE)
    println!("2. Encrypted file storage (SECURE):");
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

    // Example 3: OS Keyring storage (MOST SECURE)
    println!("3. OS Keyring storage (MOST SECURE):");
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
    println!("3. AVOID: Plain text files (storage = \"plain\")");
    println!("   - Only for backward compatibility");
    println!("   - Credentials stored unencrypted");
    println!("   - Use 'plur-creds migrate' to upgrade");

    Ok(())
}
