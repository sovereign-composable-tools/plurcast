#!/usr/bin/env bash
#
# respond.sh — check for mentions and generate pubkey art replies
#
# Polls for kind 1 events mentioning rule30, generates a visual
# fingerprint from each sender's pubkey, and replies with the image.
#
# Usage:
#   ./respond.sh [--dry-run] [--since <unix_timestamp>]
#
# Requires: nak, node, ffmpeg, plur-post

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
MY_PUBKEY="a8fb6d8883fb9deb38e3f66af5cf082f6d38e390a68abf9dfea7dc488fe60bf6"
NOSTR_KEY=$(cat "$(cygpath "$APPDATA" 2>/dev/null || echo "$HOME/.config")/plurcast-claude/nostr.key" 2>/dev/null) || { echo "error: nostr key not found"; exit 1; }
RELAY="wss://relay.damus.io"
STATE_FILE="$SCRIPT_DIR/.last_seen"
DRY_RUN=false

# Parse args
SINCE=""
while [[ $# -gt 0 ]]; do
    case "$1" in
        --dry-run) DRY_RUN=true; shift ;;
        --since) SINCE="$2"; shift 2 ;;
        *) shift ;;
    esac
done

# Default: check from last seen timestamp, or last hour
if [[ -z "$SINCE" ]]; then
    if [[ -f "$STATE_FILE" ]]; then
        SINCE=$(cat "$STATE_FILE")
    else
        SINCE=$(($(date +%s) - 3600))
    fi
fi

echo "checking mentions since $(date -d @"$SINCE" 2>/dev/null || date -r "$SINCE" 2>/dev/null || echo "$SINCE")"

# Fetch mentions (kind 1 events tagging my pubkey, not from me)
MENTIONS=$(nak req -k 1 --tag p="$MY_PUBKEY" --since "$SINCE" -l 20 "$RELAY" 2>/dev/null | \
    jq -c "select(.pubkey != \"$MY_PUBKEY\")" 2>/dev/null) || true

if [[ -z "$MENTIONS" ]]; then
    echo "no new mentions"
    # Update timestamp
    date +%s > "$STATE_FILE"
    exit 0
fi

# Check which ones we've already replied to
MY_REPLIES=$(nak req -k 1 -a "$MY_PUBKEY" -l 50 "$RELAY" 2>/dev/null | \
    jq -r '[.tags[] | select(.[0]=="e") | .[1]] | .[]' 2>/dev/null) || true

REPLIED_SET=$(echo "$MY_REPLIES" | sort -u)

# Process each mention
echo "$MENTIONS" | while IFS= read -r event; do
    EVENT_ID=$(echo "$event" | jq -r '.id')
    SENDER=$(echo "$event" | jq -r '.pubkey')
    CONTENT=$(echo "$event" | jq -r '.content')

    # Skip if we already replied to this event
    if echo "$REPLIED_SET" | grep -q "^${EVENT_ID}$" 2>/dev/null; then
        echo "  skip $EVENT_ID (already replied)"
        continue
    fi

    echo "  mention from ${SENDER:0:16}... | ${CONTENT:0:60}"

    # Only respond with art if the mention contains a trigger keyword
    # This prevents spamming art on every casual mention
    LOWER_CONTENT=$(echo "$CONTENT" | tr '[:upper:]' '[:lower:]')
    if ! echo "$LOWER_CONTENT" | grep -qE "(fingerprint|art|generate|pubkey art|my art|draw me|make me)" 2>/dev/null; then
        echo "  skip (no trigger keyword)"
        continue
    fi

    if $DRY_RUN; then
        echo "  [dry run] would generate art for $SENDER and reply to $EVENT_ID"
        continue
    fi

    # Generate their fingerprint
    OUTPUT="/tmp/fingerprint_${SENDER:0:16}.png"
    bash "$SCRIPT_DIR/generate.sh" "$SENDER" "$OUTPUT" > /dev/null 2>&1 || {
        echo "  error: art generation failed for $SENDER"
        continue
    }

    # Upload to blossom
    UPLOAD=$(nak blossom upload -s blossom.primal.net --sec "$NOSTR_KEY" "$OUTPUT" 2>/dev/null) || {
        echo "  error: blossom upload failed"
        continue
    }
    URL=$(echo "$UPLOAD" | jq -r '.url')

    # Reply with the image
    PLURCAST_CONFIG="$(cygpath "$APPDATA" 2>/dev/null || echo "$HOME/.config")/plurcast-claude/config.toml"
    PLURCAST_DB_PATH="$(cygpath "$LOCALAPPDATA" 2>/dev/null || echo "$HOME/.local/share")/plurcast-claude/posts.db"

    PLURCAST_CONFIG="$PLURCAST_CONFIG" PLURCAST_DB_PATH="$PLURCAST_DB_PATH" \
        cargo run -p plur-post -- "your visual fingerprint. deterministic interference pattern from your pubkey — same key always makes the same image.

$URL" \
        --platform nostr --nostr-pow 20 --reply-to "$EVENT_ID" --format json 2>/dev/null || {
        echo "  error: reply failed"
        continue
    }

    echo "  replied to ${EVENT_ID:0:16} with fingerprint for ${SENDER:0:16}"
done

# Update last seen timestamp
date +%s > "$STATE_FILE"
echo "done"
