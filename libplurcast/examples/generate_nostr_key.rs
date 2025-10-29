// Simple Nostr key generator for testing
// Usage: cargo run --example generate_nostr_key

use nostr_sdk::prelude::*;

fn main() {
    // Generate a new keypair
    let keys = Keys::generate();
    
    // Get secret key (private) in different formats
    let secret_hex = keys.secret_key().to_secret_hex();
    let secret_bech32 = keys.secret_key().to_bech32().unwrap();
    
    // Get public key
    let public_hex = keys.public_key().to_hex();
    let public_bech32 = keys.public_key().to_bech32().unwrap();
    
    println!("=== Nostr Test Keypair ===\n");
    
    println!("Private Key (KEEP SECRET!):");
    println!("  Hex:    {}", secret_hex);
    println!("  Bech32: {}", secret_bech32);
    println!();
    
    println!("Public Key (safe to share):");
    println!("  Hex:    {}", public_hex);
    println!("  Bech32: {}", public_bech32);
    println!();
    
    println!("Save the private key to use with plur-creds:");
    println!("  plur-creds set nostr");
    println!("  # Paste either the hex or bech32 private key when prompted");
}
