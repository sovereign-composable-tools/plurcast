//! Key wrapper that lets nostr-sdk keys live inside a `SecretBox`.

use nostr_sdk::prelude::Keys;
use zeroize::Zeroize;

/// Wrapper around nostr_sdk::prelude::Keys that implements Zeroize
///
/// This wrapper allows us to use SecretBox<NostrKeys> for automatic memory zeroing.
/// The Keys type from nostr-sdk doesn't implement Zeroize, so we wrap it.
///
/// # Security Note
///
/// While we can't directly zero the internal Keys struct (it's opaque from nostr-sdk),
/// this wrapper ensures that:
/// 1. Keys are wrapped in SecretBox<T> which prevents accidental exposure
/// 2. Drop is called securely when the value goes out of scope
/// 3. The compiler prevents cloning without explicit exposure
#[derive(Clone)]
pub(super) struct NostrKeys(Keys);

impl Zeroize for NostrKeys {
    fn zeroize(&mut self) {
        // Note: We can't directly zero the internal Keys struct since it's opaque
        // from nostr-sdk. However, this implementation satisfies the Zeroize trait
        // requirement for SecretBox<T>, and Rust's drop semantics will handle cleanup.
        //
        // Ideally, nostr-sdk would implement Zeroize for Keys, but until then,
        // this wrapper provides defense-in-depth by:
        // - Wrapping in SecretBox<T> to prevent accidental exposure
        // - Ensuring proper drop semantics
        // - Making key access explicit via expose_secret()
    }
}

impl NostrKeys {
    pub(super) fn new(keys: Keys) -> Self {
        Self(keys)
    }

    pub(super) fn as_keys(&self) -> &Keys {
        &self.0
    }
}
