//! SSB keypair management
//!
//! This module handles Ed25519 keypair generation, validation, and serialization.

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};

use crate::error::{PlatformError, Result};

/// SSB keypair for Ed25519 cryptographic identity
///
/// SSB uses Ed25519 keypairs for identity and message signing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SSBKeypair {
    /// Curve type (always "ed25519" for SSB)
    pub curve: String,
    
    /// Base64-encoded public key with .ed25519 suffix
    pub public: String,
    
    /// Base64-encoded private key with .ed25519 suffix
    pub private: String,
    
    /// Feed ID (@ prefix + base64-encoded public key + .ed25519 suffix)
    pub id: String,
}

impl SSBKeypair {
    /// Generate a new random Ed25519 keypair
    pub fn generate() -> Self {
        use kuska_ssb::crypto::ed25519;
        
        let (pk, sk) = ed25519::gen_keypair();
        
        let pk_bytes = pk.as_ref();
        let sk_bytes = sk.as_ref();
        
        let public_b64 = BASE64.encode(pk_bytes);
        let private_b64 = BASE64.encode(sk_bytes);
        
        let public = format!("{}.ed25519", public_b64);
        let private = format!("{}.ed25519", private_b64);
        let id = format!("@{}.ed25519", public_b64);
        
        Self {
            curve: "ed25519".to_string(),
            public,
            private,
            id,
        }
    }
    
    /// Validate that the keypair is well-formed
    pub fn validate(&self) -> Result<()> {
        if self.curve != "ed25519" {
            return Err(PlatformError::Authentication(
                format!("Invalid curve: expected 'ed25519', got '{}'", self.curve)
            ).into());
        }
        
        if !self.public.ends_with(".ed25519") {
            return Err(PlatformError::Authentication(
                "Public key must end with '.ed25519'".to_string()
            ).into());
        }
        
        if !self.private.ends_with(".ed25519") {
            return Err(PlatformError::Authentication(
                "Private key must end with '.ed25519'".to_string()
            ).into());
        }
        
        if !self.id.starts_with('@') || !self.id.ends_with(".ed25519") {
            return Err(PlatformError::Authentication(
                "ID must start with '@' and end with '.ed25519'".to_string()
            ).into());
        }
        
        let public_b64 = self.public.strip_suffix(".ed25519")
            .ok_or_else(|| PlatformError::Authentication("Invalid public key format".to_string()))?;
        let private_b64 = self.private.strip_suffix(".ed25519")
            .ok_or_else(|| PlatformError::Authentication("Invalid private key format".to_string()))?;
        let id_b64 = self.id.strip_prefix('@')
            .and_then(|s| s.strip_suffix(".ed25519"))
            .ok_or_else(|| PlatformError::Authentication("Invalid ID format".to_string()))?;
        
        if public_b64 != id_b64 {
            return Err(PlatformError::Authentication(
                "ID does not match public key".to_string()
            ).into());
        }
        
        BASE64.decode(public_b64)
            .map_err(|e| PlatformError::Authentication(
                format!("Invalid base64 in public key: {}", e)
            ))?;
        
        BASE64.decode(private_b64)
            .map_err(|e| PlatformError::Authentication(
                format!("Invalid base64 in private key: {}", e)
            ))?;
        
        let public_bytes = BASE64.decode(public_b64).unwrap();
        let private_bytes = BASE64.decode(private_b64).unwrap();
        
        if public_bytes.len() != 32 {
            return Err(PlatformError::Authentication(
                format!("Invalid public key length: expected 32 bytes, got {}", public_bytes.len())
            ).into());
        }
        
        if private_bytes.len() != 64 {
            return Err(PlatformError::Authentication(
                format!("Invalid private key length: expected 64 bytes, got {}", private_bytes.len())
            ).into());
        }
        
        Ok(())
    }
    
    /// Serialize the keypair to JSON string
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| PlatformError::Authentication(
                format!("Failed to serialize keypair: {}", e)
            ).into())
    }
    
    /// Deserialize a keypair from JSON string
    pub fn from_json(json: &str) -> Result<Self> {
        let keypair: Self = serde_json::from_str(json)
            .map_err(|e| PlatformError::Authentication(
                format!("Failed to parse keypair JSON: {}", e)
            ))?;
        
        keypair.validate()?;
        
        Ok(keypair)
    }
}
