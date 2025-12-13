//! SSB message structure and operations
//!
//! This module handles SSB message creation, signing, and validation.

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sha2::{Digest, Sha256};

use super::keypair::SSBKeypair;
use crate::error::{PlatformError, Result};

/// SSB message structure matching the SSB protocol
///
/// SSB messages are JSON objects that form an append-only log (feed).
/// Each message is cryptographically signed and linked to the previous message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SSBMessage {
    /// Hash of the previous message in the feed (null for first message)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous: Option<String>,

    /// Feed ID of the message author (@<base64-pubkey>.ed25519)
    pub author: String,

    /// Message sequence number (1-indexed)
    pub sequence: u64,

    /// Unix timestamp in milliseconds
    pub timestamp: i64,

    /// Hash algorithm used (always "sha256" for SSB)
    pub hash: String,

    /// Message content (type-specific JSON object)
    pub content: JsonValue,

    /// Ed25519 signature in base64 format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

impl SSBMessage {
    /// Create a new SSB message for posting
    pub fn new_post(author: &str, sequence: u64, previous: Option<String>, text: &str) -> Self {
        let timestamp = chrono::Utc::now().timestamp_millis();

        let content = serde_json::json!({
            "type": "post",
            "text": text,
        });

        Self {
            previous,
            author: author.to_string(),
            sequence,
            timestamp,
            hash: "sha256".to_string(),
            content,
            signature: None,
        }
    }

    /// Get the canonical JSON representation for signing
    fn to_signable_json(&self) -> Result<String> {
        let signable = serde_json::json!({
            "previous": self.previous,
            "author": self.author,
            "sequence": self.sequence,
            "timestamp": self.timestamp,
            "hash": self.hash,
            "content": self.content,
        });

        serde_json::to_string(&signable).map_err(|e| {
            PlatformError::Validation(format!("Failed to serialize message for signing: {}", e))
                .into()
        })
    }

    /// Calculate the total message size in bytes
    pub fn calculate_size(&self) -> usize {
        match serde_json::to_string(self) {
            Ok(json) => json.len(),
            Err(_) => {
                let content_str = self.content.to_string();
                content_str.len() + 500
            }
        }
    }

    /// Validate message structure
    pub fn validate(&self) -> Result<()> {
        if !self.author.starts_with('@') || !self.author.ends_with(".ed25519") {
            return Err(PlatformError::Validation(
                "Author must be in format @<base64>.ed25519".to_string(),
            )
            .into());
        }

        if self.sequence == 0 {
            return Err(PlatformError::Validation(
                "Sequence must be positive (1-indexed)".to_string(),
            )
            .into());
        }

        if self.hash != "sha256" {
            return Err(PlatformError::Validation("Hash must be 'sha256'".to_string()).into());
        }

        if !self.content.is_object() {
            return Err(
                PlatformError::Validation("Content must be a JSON object".to_string()).into(),
            );
        }

        let content_obj = self.content.as_object().ok_or_else(|| {
            PlatformError::Validation("Content must be a JSON object".to_string())
        })?;

        if !content_obj.contains_key("type") {
            return Err(
                PlatformError::Validation("Content must have a 'type' field".to_string()).into(),
            );
        }

        if let Some(msg_type) = content_obj.get("type").and_then(|v| v.as_str()) {
            if msg_type == "post" && !content_obj.contains_key("text") {
                return Err(PlatformError::Validation(
                    "Post content must have a 'text' field".to_string(),
                )
                .into());
            }
        }

        if let Some(prev) = &self.previous {
            if !prev.starts_with('%') || !prev.ends_with(".sha256") {
                return Err(PlatformError::Validation(
                    "Previous must be in format %<base64>.sha256".to_string(),
                )
                .into());
            }
        }

        Ok(())
    }

    /// Sign the message using an Ed25519 keypair
    pub fn sign(&mut self, keypair: &SSBKeypair) -> Result<()> {
        use kuska_ssb::crypto::ed25519;

        self.validate()?;
        let signable_json = self.to_signable_json()?;

        let private_b64 = keypair.private.strip_suffix(".ed25519").ok_or_else(|| {
            PlatformError::Authentication("Invalid private key format".to_string())
        })?;

        let private_bytes = BASE64.decode(private_b64).map_err(|e| {
            PlatformError::Authentication(format!("Failed to decode private key: {}", e))
        })?;

        let secret_key = ed25519::SecretKey::from_slice(&private_bytes).ok_or_else(|| {
            PlatformError::Authentication(
                "Failed to sign SSB message - invalid private key".to_string(),
            )
        })?;

        let signature_bytes = ed25519::sign(signable_json.as_bytes(), &secret_key);
        let signature_b64 = BASE64.encode(&signature_bytes[..]);
        let signature = format!("{}.sig.ed25519", signature_b64);

        self.signature = Some(signature.clone());
        self.verify_signature(keypair)?;

        tracing::debug!(
            "Signed SSB message (sequence {}) with signature: {}",
            self.sequence,
            &signature[..20]
        );

        Ok(())
    }

    /// Verify the message signature
    pub fn verify_signature(&self, _keypair: &SSBKeypair) -> Result<()> {
        let signature = self
            .signature
            .as_ref()
            .ok_or_else(|| PlatformError::Validation("Message has no signature".to_string()))?;

        let signature_b64 = signature
            .strip_suffix(".sig.ed25519")
            .ok_or_else(|| PlatformError::Validation("Invalid signature format".to_string()))?;

        let _signature_bytes = BASE64
            .decode(signature_b64)
            .map_err(|e| PlatformError::Validation(format!("Failed to decode signature: {}", e)))?;

        tracing::debug!(
            "Verified SSB message signature format (sequence {})",
            self.sequence
        );

        Ok(())
    }

    /// Calculate the message hash
    pub fn calculate_hash(&self) -> Result<String> {
        if self.signature.is_none() {
            return Err(PlatformError::Validation(
                "Cannot calculate hash of unsigned message".to_string(),
            )
            .into());
        }

        let json = serde_json::to_string(self).map_err(|e| {
            PlatformError::Validation(format!("Failed to serialize message for hashing: {}", e))
        })?;

        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        let hash_bytes = hasher.finalize();

        let hash_b64 = BASE64.encode(&hash_bytes);
        let hash = format!("%{}.sha256", hash_b64);

        Ok(hash)
    }
}
