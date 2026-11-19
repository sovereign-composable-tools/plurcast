//! Sending signed events to relays.

use nostr_sdk::prelude::{Client, Event, SendEventOutput};
use std::time::Duration;

/// How long to wait for relay connections before publishing anyway.
pub(super) const RELAY_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// Send a signed event to every configured relay.
///
/// Fails when no relay accepted the event, so a post is never reported as
/// published on the strength of a client-side send alone.
pub(super) async fn send_to_relays(
    client: &Client,
    event: &Event,
) -> std::result::Result<SendEventOutput, String> {
    let output = client.send_event(event).await.map_err(|e| e.to_string())?;
    if output.success.is_empty() {
        let reasons: Vec<String> = output
            .failed
            .iter()
            .map(|(url, reason)| format!("{}: {}", url, reason))
            .collect();
        return Err(format!(
            "no relay accepted the event ({})",
            if reasons.is_empty() {
                "no relays connected".to_string()
            } else {
                reasons.join("; ")
            }
        ));
    }
    Ok(output)
}
