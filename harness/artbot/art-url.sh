#!/usr/bin/env bash
# art-url.sh — generate art and return a blossom URL
# Usage: ./art-url.sh <technique> [seed]
#
# Generates an image using techniques.sh, uploads to blossom,
# prints the URL to stdout. Cleans up the temp file.
#
# Example:
#   URL=$(./art-url.sh quasicrystal "my-seed")
#   nak event -k 1 -c "some post text $URL" ...

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TECHNIQUE="${1:?Usage: art-url.sh <technique> [seed]}"
SEED="${2:-$(date +%s)}"
SECRET_KEY="${NOSTR_SECRET_KEY:-0eaa52f610709bea13b35ade049f069bec0f9ce0eaedea5d1d92652d55ddd9e3}"

TMPFILE="$SCRIPT_DIR/.tmp_art_$$.png"
trap 'rm -f "$TMPFILE"' EXIT

# Generate
bash "$SCRIPT_DIR/techniques.sh" "$TECHNIQUE" "$TMPFILE" "$SEED" >&2

# Upload
RESULT=$(nak blossom upload -s blossom.primal.net --sec "$SECRET_KEY" "$TMPFILE" 2>/dev/null)
URL=$(echo "$RESULT" | jq -r '.url')

if [ -z "$URL" ] || [ "$URL" = "null" ]; then
    echo "Upload failed: $RESULT" >&2
    exit 1
fi

echo "$URL"
