//! Parallel Proof of Work mining for Nostr events (NIP-13)
//!
//! This module implements multi-threaded PoW mining to maximize CPU utilization
//! when creating Nostr events with proof of work.

use nostr_sdk::prelude::*;
use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use crate::error::{PlatformError, Result};

/// Mine a Nostr event with parallel proof of work
///
/// This function uses all available CPU cores to mine a valid nonce that meets
/// the specified difficulty requirement. The difficulty is measured in leading
/// zero bits in the event ID hash.
///
/// # Arguments
///
/// * `content` - The text content of the note
/// * `keys` - The keypair to sign the event with
/// * `difficulty` - Target difficulty (number of leading zero bits, 0-64)
///
/// # Returns
///
/// Returns the mined and signed Event with a valid PoW nonce
///
/// # Performance
///
/// - Uses all CPU cores via rayon parallel iterators
/// - Partitions nonce search space across threads
/// - First thread to find solution wins
/// - Expected speedup: ~linear with core count
///
/// # Example
///
/// ```no_run
/// # use libplurcast::platforms::nostr_pow::mine_event_parallel;
/// # use nostr_sdk::Keys;
/// let keys = Keys::generate();
/// let event = mine_event_parallel("Hello Nostr!", &keys, 20).await?;
/// // Event ID will have ~20 leading zero bits
/// # Ok::<(), libplurcast::PlurcastError>(())
/// ```
pub async fn mine_event_parallel(content: &str, keys: &Keys, difficulty: u8) -> Result<Event> {
    let num_threads = num_cpus::get();
    tracing::info!(
        "Mining Nostr event with PoW difficulty {} using {} CPU cores...",
        difficulty,
        num_threads
    );

    // Atomic flags for thread coordination
    let found = Arc::new(AtomicBool::new(false));
    let solution_nonce = Arc::new(AtomicU64::new(0));

    // Partition the nonce search space across threads
    let chunk_size = u64::MAX / num_threads as u64;

    // Create timestamp and pubkey once (shared across all threads)
    let created_at = Timestamp::now();
    let pubkey = keys.public_key();
    let kind = Kind::TextNote;
    let content = content.to_string();

    // Parallel search across threads
    let result: std::result::Result<(), PlatformError> =
        (0..num_threads).into_par_iter().try_for_each(|thread_id| {
            let start_nonce = thread_id as u64 * chunk_size;
            let found = Arc::clone(&found);
            let solution_nonce = Arc::clone(&solution_nonce);
            let content = content.clone();

            // Search this thread's partition of the nonce space
            for nonce in start_nonce.. {
                // Check if another thread found a solution
                if found.load(Ordering::Relaxed) {
                    break;
                }

                // Try every 10,000 nonces, log progress from thread 0
                if nonce % 10_000 == 0 && thread_id == 0 {
                    tracing::debug!("Thread {}: trying nonce {}", thread_id, nonce);
                }

                // Build tags with PoW nonce
                let tags = vec![Tag::pow(nonce as u128, difficulty)];

                // Calculate event ID for this candidate
                let event_id = EventId::new(&pubkey, &created_at, &kind, &tags, &content);

                // Check if this nonce meets the difficulty requirement
                if event_id.check_pow(difficulty) {
                    tracing::info!(
                        "✓ Found valid nonce {} (thread {}) after {} attempts",
                        nonce,
                        thread_id,
                        nonce - start_nonce
                    );

                    solution_nonce.store(nonce, Ordering::SeqCst);
                    found.store(true, Ordering::SeqCst);
                    break;
                }
            }

            Ok(())
        });

    result?;

    // Build and sign final event with the solution nonce
    let final_nonce = solution_nonce.load(Ordering::SeqCst);

    if final_nonce == 0 && !found.load(Ordering::SeqCst) {
        return Err(PlatformError::Posting(
            "PoW mining failed: no valid nonce found".to_string(),
        )
        .into());
    }

    tracing::info!("Building final event with nonce {}...", final_nonce);

    let event = EventBuilder::text_note(&content, [])
        .custom_created_at(created_at)
        .add_tags([Tag::pow(final_nonce as u128, difficulty)])
        .to_event(keys)
        .map_err(|e| {
            PlatformError::Posting(format!("Failed to build final PoW event: {}", e))
        })?;

    // Verify the final event meets difficulty
    if !event.id.check_pow(difficulty) {
        return Err(PlatformError::Posting(format!(
            "Final event does not meet PoW difficulty {} (this is a bug)",
            difficulty
        ))
        .into());
    }

    tracing::info!(
        "✓ PoW mining complete! Event ID: {} (difficulty: {})",
        event.id.to_hex(),
        difficulty
    );

    Ok(event)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_pow_difficulty_0() {
        // Any hash meets difficulty 0
        let keys = Keys::generate();
        let event = EventBuilder::text_note("test", [])
            .to_event(&keys)
            .unwrap();
        assert!(event.id.check_pow(0));
    }

    #[test]
    fn test_check_pow_difficulty_8() {
        // Create an event ID with known leading zeros
        let hash_bytes = [
            0x00, // 8 leading zeros
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF,
        ];

        let event_id = EventId::from_byte_array(hash_bytes);
        assert!(event_id.check_pow(8));
        assert!(!event_id.check_pow(9));
    }

    #[test]
    fn test_check_pow_difficulty_16() {
        let hash_bytes = [
            0x00, 0x00, // 16 leading zeros
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF,
        ];

        let event_id = EventId::from_byte_array(hash_bytes);
        assert!(event_id.check_pow(16));
        assert!(!event_id.check_pow(17));
    }

    #[tokio::test]
    async fn test_mine_event_low_difficulty() {
        let keys = Keys::generate();
        let event = mine_event_parallel("test", &keys, 8).await.unwrap();

        // Verify event is valid
        assert_eq!(event.kind, Kind::TextNote);
        assert_eq!(event.content, "test");

        // Verify PoW difficulty
        assert!(event.id.check_pow(8));
    }

    #[tokio::test]
    async fn test_mine_event_medium_difficulty() {
        let keys = Keys::generate();
        let event = mine_event_parallel("parallel mining test", &keys, 12)
            .await
            .unwrap();

        assert!(event.id.check_pow(12));
    }
}
