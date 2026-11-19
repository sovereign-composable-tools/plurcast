//! Building the text-note event for a post: options from metadata, tags, signing or mining.

use nostr_sdk::prelude::{Event, EventBuilder, EventId, FinalizeEvent, Keys, Kind, Tag};

use crate::error::{PlatformError, Result};

/// Per-post Nostr options, read from a post's metadata JSON.
///
/// Metadata shape: `{"nostr": {"pow_difficulty": 20, "21e8": true}, "reply_to": {"nostr": "note1..."}}`.
/// Anything missing or malformed is treated as absent.
#[derive(Debug, Default, PartialEq)]
pub(super) struct PostOptions {
    /// NIP-13 proof-of-work difficulty, if requested.
    pub(super) pow_difficulty: Option<u8>,
    /// Whether the mined id must also carry the 21e8 pattern.
    pub(super) require_21e8: bool,
    /// Parent event for a NIP-10 reply, given as bech32 `note1...` or hex.
    pub(super) reply_to: Option<EventId>,
}

impl PostOptions {
    pub(super) fn from_metadata(metadata: Option<&str>) -> Self {
        let Some(value) = metadata.and_then(|m| serde_json::from_str::<serde_json::Value>(m).ok())
        else {
            return Self::default();
        };
        let nostr = value.get("nostr");
        Self {
            pow_difficulty: nostr
                .and_then(|n| n.get("pow_difficulty"))
                .and_then(|d| d.as_u64())
                .map(|d| d as u8),
            require_21e8: nostr
                .and_then(|n| n.get("21e8"))
                .and_then(|f| f.as_bool())
                .unwrap_or(false),
            reply_to: value
                .get("reply_to")
                .and_then(|r| r.get("nostr"))
                .and_then(|r| r.as_str())
                .and_then(|id| EventId::parse(id).ok()),
        }
    }

    /// NIP-10 tags: `["e", <event-id>, <relay>, "reply"]` when replying, otherwise none.
    pub(super) fn tags(&self) -> Vec<Tag> {
        match self.reply_to {
            Some(parent) => {
                tracing::debug!("Adding NIP-10 reply tag for parent event: {}", parent);
                vec![Tag::custom(
                    "e",
                    vec![parent.to_hex(), String::new(), "reply".to_string()],
                )]
            }
            None => vec![],
        }
    }
}

/// Build the signed text note for `content`, mining proof of work when the options ask for it.
pub(super) async fn build_event(
    content: &str,
    keys: &Keys,
    options: &PostOptions,
) -> Result<Event> {
    let tags = options.tags();
    let Some(difficulty) = options.pow_difficulty else {
        return EventBuilder::new(Kind::TextNote, content)
            .tags(tags)
            .finalize(keys)
            .map_err(|e| {
                PlatformError::Posting(format!(
                    "Nostr posting failed (sign): Failed to sign note: {}",
                    e
                ))
                .into()
            });
    };

    if options.require_21e8 {
        tracing::info!(
            "Mining Nostr event with 21e8 pattern (difficulty {})...",
            difficulty
        );
    } else {
        tracing::info!(
            "Mining Nostr event with POW difficulty {} (parallel)...",
            difficulty
        );
    }
    let event = crate::platforms::nostr_pow::mine_event_parallel(
        content,
        keys,
        difficulty,
        options.require_21e8,
        tags,
    )
    .await
    .map_err(|e| {
        PlatformError::Posting(format!(
            "Nostr POW mining failed: {}. \
            Suggestion: Try a lower difficulty value.",
            e
        ))
    })?;
    tracing::info!("POW mining complete, publishing event...");
    Ok(event)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PARENT_HEX: &str = "331c0d98945a9b7e4adbc9e9e1a5b7e7eaaabac15dc404b936361bf122c4e293";

    #[test]
    fn no_metadata_gives_defaults() {
        assert_eq!(PostOptions::from_metadata(None), PostOptions::default());
        assert!(PostOptions::default().tags().is_empty());
    }

    #[test]
    fn malformed_metadata_gives_defaults() {
        assert_eq!(
            PostOptions::from_metadata(Some("not json")),
            PostOptions::default()
        );
    }

    #[test]
    fn reads_pow_and_21e8() {
        let options =
            PostOptions::from_metadata(Some(r#"{"nostr":{"pow_difficulty":20,"21e8":true}}"#));
        assert_eq!(options.pow_difficulty, Some(20));
        assert!(options.require_21e8);
        assert_eq!(options.reply_to, None);
    }

    #[test]
    fn reads_only_the_nostr_reply_target() {
        let metadata = format!(
            r#"{{"reply_to":{{"nostr":"{}","mastodon":"12345"}}}}"#,
            PARENT_HEX
        );
        let options = PostOptions::from_metadata(Some(&metadata));
        assert_eq!(options.reply_to, Some(EventId::parse(PARENT_HEX).unwrap()));
        assert_eq!(options.pow_difficulty, None);
    }

    #[test]
    fn unparseable_reply_target_is_ignored() {
        let options = PostOptions::from_metadata(Some(r#"{"reply_to":{"nostr":"nope"}}"#));
        assert_eq!(options.reply_to, None);
    }

    #[test]
    fn reply_tag_follows_nip10() {
        let options = PostOptions {
            reply_to: Some(EventId::parse(PARENT_HEX).unwrap()),
            ..Default::default()
        };
        let tags = options.tags();
        assert_eq!(tags.len(), 1);
        assert_eq!(
            tags[0].as_slice(),
            [
                "e".to_string(),
                PARENT_HEX.to_string(),
                String::new(),
                "reply".to_string()
            ]
        );
    }

    #[tokio::test]
    async fn unmined_event_is_signed_and_carries_the_reply_tag() {
        let keys = Keys::generate();
        let options = PostOptions {
            reply_to: Some(EventId::parse(PARENT_HEX).unwrap()),
            ..Default::default()
        };
        let event = build_event("hello", &keys, &options).await.unwrap();
        assert_eq!(event.content, "hello");
        assert_eq!(event.pubkey, keys.public_key());
        assert!(event.verify().is_ok());
        assert_eq!(event.tags.len(), 1);
    }
}
