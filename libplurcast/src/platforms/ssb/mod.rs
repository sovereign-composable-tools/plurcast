//! SSB (Secure Scuttlebutt) platform implementation
//!
//! This module provides integration with the Secure Scuttlebutt (SSB) protocol,
//! a peer-to-peer, offline-first social protocol.

mod keypair;
mod message;
mod platform;
mod replication;

#[cfg(test)]
mod tests;

pub use keypair::SSBKeypair;
pub use message::SSBMessage;
pub use platform::SSBPlatform;
pub use replication::{PubAddress, PubConnection};
