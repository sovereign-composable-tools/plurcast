#!/usr/bin/env bash
#
# zap.sh — send a proper NIP-57 zap as rule30
#
# Creates a kind 9734 zap request, sends it to the recipient's LNURL
# callback, and pays the resulting invoice via Coinos.
#
# Usage:
#   ./harness/zap.sh <lud16> <amount_sats> [message] [--pubkey <hex>] [--event <event_id>] [--dry-run]
#
# Examples:
#   ./harness/zap.sh user@example.com 21 "great post" --pubkey abc123...
#   ./harness/zap.sh user@example.com 1 --dry-run

set -euo pipefail

CREDS_FILE="$(cygpath "$APPDATA" 2>/dev/null || echo "$HOME/.config")/plurcast-claude/credentials.json"
KEY_FILE="$(cygpath "$APPDATA" 2>/dev/null || echo "$HOME/.config")/plurcast-claude/nostr.key"
RELAYS="wss://nos.lol;wss://relay.damus.io;wss://relay.primal.net"

# Parse args
LUD16=""
AMOUNT_SATS=""
MESSAGE=""
EVENT_ID=""
RECIPIENT_PUBKEY=""
DRY_RUN=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        --event)
            EVENT_ID="$2"
            shift 2
            ;;
        --pubkey)
            RECIPIENT_PUBKEY="$2"
            shift 2
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        *)
            if [[ -z "$LUD16" ]]; then
                LUD16="$1"
            elif [[ -z "$AMOUNT_SATS" ]]; then
                AMOUNT_SATS="$1"
            else
                MESSAGE="$1"
            fi
            shift
            ;;
    esac
done

if [[ -z "$LUD16" || -z "$AMOUNT_SATS" ]]; then
    echo "usage: $0 <lud16> <amount_sats> [message] [--event <event_id>] [--dry-run]"
    exit 1
fi

AMOUNT_MSATS=$((AMOUNT_SATS * 1000))

# Load credentials
NOSTR_KEY=$(cat "$KEY_FILE" 2>/dev/null) || { echo "error: nostr key not found at $KEY_FILE"; exit 1; }
COINOS_TOKEN=$(jq -r '.coinos.token' "$CREDS_FILE" 2>/dev/null) || { echo "error: coinos token not found"; exit 1; }

# Step 1: Resolve lud16 → LNURL endpoint
USER="${LUD16%%@*}"
DOMAIN="${LUD16##*@}"
LNURL_ENDPOINT="https://${DOMAIN}/.well-known/lnurlp/${USER}"

echo "resolving $LUD16"

LNURL_META=$(curl -sf "$LNURL_ENDPOINT") || { echo "error: could not resolve lnurl for $LUD16"; exit 1; }

CALLBACK=$(echo "$LNURL_META" | jq -r '.callback')
ALLOWS_NOSTR=$(echo "$LNURL_META" | jq -r '.allowsNostr // false')
NOSTR_PUBKEY=$(echo "$LNURL_META" | jq -r '.nostrPubkey // empty')

if [[ "$ALLOWS_NOSTR" != "true" ]]; then
    echo "warning: recipient does not support nostr zaps, falling back to plain lightning"
    if $DRY_RUN; then
        echo "[dry run] would request invoice from callback and pay $AMOUNT_SATS sats"
        exit 0
    fi
    INVOICE=$(curl -sf "${CALLBACK}?amount=${AMOUNT_MSATS}" | jq -r '.pr') || { echo "error: failed to get invoice"; exit 1; }
    curl -sf -H "Authorization: Bearer $COINOS_TOKEN" \
        -H "Content-Type: application/json" \
        -X POST https://coinos.io/api/payments \
        -d "{\"payreq\": \"$INVOICE\"}" | jq '.' 2>/dev/null
    echo "paid $AMOUNT_SATS sats (anonymous, no zap receipt)"
    exit 0
fi

# Use provided pubkey, or fall back to LNURL nostrPubkey (which may be the wallet's, not the user's)
if [[ -z "$RECIPIENT_PUBKEY" ]]; then
    RECIPIENT_PUBKEY="$NOSTR_PUBKEY"
    echo "warning: using wallet's nostrPubkey as recipient. pass --pubkey for proper attribution."
fi
echo "recipient: ${RECIPIENT_PUBKEY:0:16}... | $AMOUNT_SATS sats | nostr zap supported"

# Step 2: Create kind 9734 zap request event
ZAP_TAGS=(-k 9734 --sec "$NOSTR_KEY")
ZAP_TAGS+=(-t "relays=$RELAYS")
ZAP_TAGS+=(-t "p=$RECIPIENT_PUBKEY")
ZAP_TAGS+=(-t "amount=$AMOUNT_MSATS")

if [[ -n "$EVENT_ID" ]]; then
    ZAP_TAGS+=(-e "$EVENT_ID")
fi

if [[ -n "$MESSAGE" ]]; then
    ZAP_TAGS+=(-c "$MESSAGE")
else
    ZAP_TAGS+=(-c "")
fi

ZAP_REQUEST=$(nak event "${ZAP_TAGS[@]}" 2>/dev/null) || { echo "error: failed to create zap request"; exit 1; }

echo "zap request: $(echo "$ZAP_REQUEST" | jq -r '.id[:16]')..."

# Step 3: URL-encode and send to callback
NOSTR_PARAM=$(echo "$ZAP_REQUEST" | jq -sRr @uri 2>/dev/null) || { echo "error: could not URL-encode zap request"; exit 1; }

CALLBACK_URL="${CALLBACK}?amount=${AMOUNT_MSATS}&nostr=${NOSTR_PARAM}"

if $DRY_RUN; then
    echo "[dry run] would send zap request to: ${CALLBACK}"
    echo "[dry run] zap request event:"
    echo "$ZAP_REQUEST" | jq '.'
    exit 0
fi

INVOICE_RESP=$(curl -sf "$CALLBACK_URL") || { echo "error: callback request failed"; exit 1; }
INVOICE=$(echo "$INVOICE_RESP" | jq -r '.pr // empty')

if [[ -z "$INVOICE" ]]; then
    echo "error: no invoice returned"
    echo "$INVOICE_RESP"
    exit 1
fi

echo "invoice: ${INVOICE:0:40}..."

# Step 4: Pay via Coinos
PAY_RESP=$(curl -sf -H "Authorization: Bearer $COINOS_TOKEN" \
    -H "Content-Type: application/json" \
    -X POST https://coinos.io/api/payments \
    -d "{\"payreq\": \"$INVOICE\"}") || { echo "error: payment failed"; exit 1; }

echo ""
echo "zap sent! $AMOUNT_SATS sats to $LUD16"
[[ -n "$MESSAGE" ]] && echo "message: $MESSAGE"
echo "$PAY_RESP" | jq '{amount, hash}' 2>/dev/null || echo "$PAY_RESP"
