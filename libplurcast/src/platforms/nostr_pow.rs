//! Parallel Proof of Work mining for Nostr events (NIP-13)
//!
//! This module implements multi-threaded PoW mining to maximize CPU utilization
//! when creating Nostr events with proof of work.

use nostr_sdk::prelude::*;
use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use crate::error::{PlatformError, Result};

/// Check if an event ID meets the 21e8 PoW pattern requirement
///
/// The 21e8 pattern requires:
/// 1. N/4 leading zero nibbles (where N is the difficulty in bits)
/// 2. Immediately followed by the hex pattern "21e8"
///
/// # Arguments
///
/// * `event_id` - The event ID to check
/// * `difficulty` - The PoW difficulty in bits
///
/// # Returns
///
/// Returns true if the event ID matches the pattern {zeros}21e8...
///
/// # Examples
///
/// - difficulty 8 = 2 nibbles = "0021e8..."
/// - difficulty 16 = 4 nibbles = "000021e8..."
/// - difficulty 20 = 5 nibbles = "0000021e8..."
pub fn check_pow_21e8(event_id: &EventId, difficulty: u8) -> bool {
    let hex = event_id.to_hex();
    let zero_nibbles = (difficulty / 4) as usize;

    // Need at least zero_nibbles + 4 characters for "21e8"
    if hex.len() < zero_nibbles + 4 {
        return false;
    }

    // Check leading zeros
    for i in 0..zero_nibbles {
        if hex.chars().nth(i) != Some('0') {
            return false;
        }
    }

    // Check for "21e8" pattern immediately after zeros
    let pattern_start = zero_nibbles;
    let expected = "21e8";

    for (i, expected_char) in expected.chars().enumerate() {
        if hex.chars().nth(pattern_start + i) != Some(expected_char) {
            return false;
        }
    }

    true
}

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
/// * `require_21e8` - If true, require the 21e8 pattern after leading zeros
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
/// let event = mine_event_parallel("Hello Nostr!", &keys, 20, false).await?;
/// // Event ID will have ~20 leading zero bits
/// # Ok::<(), libplurcast::PlurcastError>(())
/// ```
pub async fn mine_event_parallel(
    content: &str,
    keys: &Keys,
    difficulty: u8,
    require_21e8: bool,
) -> Result<Event> {
    let num_threads = num_cpus::get();
    let pattern_msg = if require_21e8 {
        " with 21e8 pattern"
    } else {
        ""
    };
    tracing::info!(
        "Mining Nostr event with PoW difficulty {}{} using {} CPU cores...",
        difficulty,
        pattern_msg,
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
                let is_valid = if require_21e8 {
                    check_pow_21e8(&event_id, difficulty)
                } else {
                    event_id.check_pow(difficulty)
                };

                if is_valid {
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
        return Err(
            PlatformError::Posting("PoW mining failed: no valid nonce found".to_string()).into(),
        );
    }

    tracing::info!("Building final event with nonce {}...", final_nonce);

    let event = EventBuilder::text_note(&content, [])
        .custom_created_at(created_at)
        .add_tags([Tag::pow(final_nonce as u128, difficulty)])
        .to_event(keys)
        .map_err(|e| PlatformError::Posting(format!("Failed to build final PoW event: {}", e)))?;

    // Verify the final event meets difficulty
    let verification_passed = if require_21e8 {
        check_pow_21e8(&event.id, difficulty)
    } else {
        event.id.check_pow(difficulty)
    };

    if !verification_passed {
        let pattern_msg = if require_21e8 {
            " with 21e8 pattern"
        } else {
            ""
        };
        return Err(PlatformError::Posting(format!(
            "Final event does not meet PoW difficulty {}{} (this is a bug)",
            difficulty, pattern_msg
        ))
        .into());
    }

    let pattern_msg = if require_21e8 { " (21e8 pattern)" } else { "" };
    tracing::info!(
        "✓ PoW mining complete{}! Event ID: {} (difficulty: {})",
        pattern_msg,
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
        let event = EventBuilder::text_note("test", []).to_event(&keys).unwrap();
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
        let event = mine_event_parallel("test", &keys, 8, false).await.unwrap();

        // Verify event is valid
        assert_eq!(event.kind, Kind::TextNote);
        assert_eq!(event.content, "test");

        // Verify PoW difficulty
        assert!(event.id.check_pow(8));
    }

    #[tokio::test]
    async fn test_mine_event_medium_difficulty() {
        let keys = Keys::generate();
        let event = mine_event_parallel("parallel mining test", &keys, 12, false)
            .await
            .unwrap();

        assert!(event.id.check_pow(12));
    }

    // Tests for 21e8 easter egg pattern validation
    #[test]
    fn test_check_pow_21e8_valid_difficulty_20() {
        // difficulty 20 = 5 nibbles = "0000021e8..."
        // Hex: 00000 21e8 = bytes: 00 00 02 1e 8...
        let hash_bytes = [
            0x00, 0x00, 0x02, 0x1e, 0x8F, // First 5 nibbles: 00000 then 21e8
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        ];

        let event_id = EventId::from_byte_array(hash_bytes);
        assert!(check_pow_21e8(&event_id, 20));
    }

    #[test]
    fn test_check_pow_21e8_valid_difficulty_16() {
        // difficulty 16 = 4 nibbles = "000021e8..."
        let hash_bytes = [
            0x00, 0x00, 0x21, 0xe8, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF,
        ];

        let event_id = EventId::from_byte_array(hash_bytes);
        assert!(check_pow_21e8(&event_id, 16));
    }

    #[test]
    fn test_check_pow_21e8_valid_difficulty_24() {
        // difficulty 24 = 6 nibbles = "00000021e8..."
        // Hex: 000000 21e8 = bytes: 00 00 00 21 e8...
        let hash_bytes = [
            0x00, 0x00, 0x00, 0x21, 0xe8, // First 6 nibbles: 000000 then 21e8
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        ];

        let event_id = EventId::from_byte_array(hash_bytes);
        assert!(check_pow_21e8(&event_id, 24));
    }

    #[test]
    fn test_check_pow_21e8_invalid_pattern() {
        // Has leading zeros but not "21e8" pattern
        // Hex: 00000 1234 = bytes: 00 00 01 23 4...
        let hash_bytes = [
            0x00, 0x00, 0x01, 0x23, 0x4F, // Wrong pattern: 000001234
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        ];

        let event_id = EventId::from_byte_array(hash_bytes);
        assert!(!check_pow_21e8(&event_id, 20));
    }

    #[test]
    fn test_check_pow_21e8_insufficient_leading_zeros() {
        // Has "21e8" but not enough leading zeros
        // Hex: 0021e8 (only 2 leading zero nibbles, not 5)
        let hash_bytes = [
            0x00, 0x21, 0xe8, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF,
        ];

        let event_id = EventId::from_byte_array(hash_bytes);
        assert!(!check_pow_21e8(&event_id, 20));
    }

    #[test]
    fn test_check_pow_21e8_difficulty_not_multiple_of_4() {
        // difficulty 21 = 5.25 nibbles (round down to 5)
        // Should require "0000021e8..."
        // Hex: 00000 21e8 = bytes: 00 00 02 1e 8...
        let hash_bytes = [
            0x00, 0x00, 0x02, 0x1e, 0x8F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF,
        ];

        let event_id = EventId::from_byte_array(hash_bytes);
        assert!(check_pow_21e8(&event_id, 21)); // 21/4 = 5 nibbles
    }

    #[test]
    fn test_check_pow_21e8_edge_case_difficulty_8() {
        // difficulty 8 = 2 nibbles = "0021e8..."
        let hash_bytes = [
            0x00, 0x21, 0xe8, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF,
        ];

        let event_id = EventId::from_byte_array(hash_bytes);
        assert!(check_pow_21e8(&event_id, 8));
    }

    #[tokio::test]
    async fn test_mine_event_21e8_difficulty_8() {
        let keys = Keys::generate();
        let event = mine_event_parallel("test 21e8", &keys, 8, true)
            .await
            .unwrap();

        // Verify the event meets 21e8 pattern
        assert!(check_pow_21e8(&event.id, 8));

        // Verify content is correct
        assert_eq!(event.content, "test 21e8");
    }

    #[tokio::test]
    async fn test_mine_event_21e8_difficulty_16() {
        let keys = Keys::generate();
        let event = mine_event_parallel("harder 21e8 mining", &keys, 16, true)
            .await
            .unwrap();

        assert!(check_pow_21e8(&event.id, 16));
    }
}
