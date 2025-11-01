// Quick utility to check what npub corresponds to stored private key
use nostr_sdk::{Keys, ToBech32};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get private key from keyring
    let entry = keyring::Entry::new("plurcast.nostr.default", "private_key")?;
    let private_key_str = entry.get_password()?;
    
    // Parse the key
    let keys = Keys::parse(&private_key_str)?;
    
    // Get public key
    let public_key = keys.public_key();
    let npub = public_key.to_bech32()?;
    
    println!("Public key (npub): {}", npub);
    println!("Public key (hex): {}", public_key);
    
    Ok(())
}
