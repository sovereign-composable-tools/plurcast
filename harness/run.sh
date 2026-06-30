#!/usr/bin/env bash
#
# rule30 harness — persistent autonomous cycle
#
# Replaces cron-based /rule30 invocations with session-continuous
# claude -p calls. Each cycle resumes the previous session, so context
# accumulates instead of cold-starting every 15 minutes.
#
# Usage:
#   ./harness/run.sh              # run one cycle (for testing)
#   ./harness/run.sh --loop       # run continuously (production)
#   ./harness/run.sh --loop 900   # custom interval in seconds (default: 900 = 15min)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
LOG_DIR="$REPO_DIR/harness/logs"
SESSION_FILE="$REPO_DIR/harness/.session"
BUDGET_PER_CYCLE=0.30
MODEL="opus"
INTERVAL=900

mkdir -p "$LOG_DIR"

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*" | tee -a "$LOG_DIR/harness.log"
}

run_cycle() {
    local timestamp
    timestamp="$(date '+%Y%m%d_%H%M%S')"
    local cycle_log="$LOG_DIR/cycle_${timestamp}.log"
    local flags=()

    flags+=(-p)
    flags+=(--model "$MODEL")
    flags+=(--max-budget-usd "$BUDGET_PER_CYCLE")
    flags+=(--permission-mode auto)

    # Resume previous session if one exists
    if [[ -f "$SESSION_FILE" ]]; then
        local session_id
        session_id="$(cat "$SESSION_FILE")"
        if [[ -n "$session_id" ]]; then
            flags+=(--resume "$session_id")
            log "resuming session $session_id"
        fi
    else
        log "first run — starting fresh session"
    fi

    log "cycle start (budget: \$$BUDGET_PER_CYCLE)"

    # Run claude and capture output
    # The prompt is minimal when resuming — context is already there
    local prompt
    if [[ -f "$SESSION_FILE" ]]; then
        prompt="continue your /rule30 cycle. check what time it is, see what's changed since last cycle, and do your thing."
    else
        prompt="/rule30"
    fi

    local exit_code=0
    cd "$REPO_DIR"
    claude "${flags[@]}" \
        --output-format json \
        "$prompt" \
        2>"$cycle_log.stderr" \
        >"$cycle_log.json" || exit_code=$?

    if [[ $exit_code -ne 0 ]]; then
        log "cycle failed (exit $exit_code) — see $cycle_log.stderr"
        # Don't clear session on failure — we can retry resume
        return 1
    fi

    # Extract session ID and cost from JSON output
    # claude -p --output-format json may output multiple JSON objects (one per message)
    # The result message with session_id is typically the last one
    local new_session_id cost
    new_session_id="$(jq -rs '[.[] | select(.session_id)] | last | .session_id // empty' "$cycle_log.json" 2>/dev/null)" || true
    cost="$(jq -rs '[.[] | select(.cost_usd)] | last | .cost_usd // "unknown"' "$cycle_log.json" 2>/dev/null)" || cost="unknown"

    if [[ -n "${new_session_id:-}" ]]; then
        echo "$new_session_id" > "$SESSION_FILE"
        log "session saved: $new_session_id"
    fi
    log "cycle complete (cost: \$$cost)"

    # Trim old logs (keep last 100 cycles)
    local log_count
    log_count="$(find "$LOG_DIR" -name 'cycle_*.json' | wc -l)"
    if [[ "$log_count" -gt 100 ]]; then
        find "$LOG_DIR" -name 'cycle_*' | sort | head -n "$(( (log_count - 100) * 2 ))" | xargs rm -f
        log "trimmed old cycle logs"
    fi
}

# Parse args
LOOP=false
while [[ $# -gt 0 ]]; do
    case "$1" in
        --loop)
            LOOP=true
            if [[ "${2:-}" =~ ^[0-9]+$ ]]; then
                INTERVAL="$2"
                shift
            fi
            shift
            ;;
        --budget)
            BUDGET_PER_CYCLE="$2"
            shift 2
            ;;
        --model)
            MODEL="$2"
            shift 2
            ;;
        --reset)
            rm -f "$SESSION_FILE"
            log "session reset — next cycle starts fresh"
            exit 0
            ;;
        *)
            echo "unknown arg: $1"
            echo "usage: $0 [--loop [interval_secs]] [--budget amount] [--model model] [--reset]"
            exit 1
            ;;
    esac
done

if $LOOP; then
    log "harness starting in loop mode (interval: ${INTERVAL}s, budget: \$$BUDGET_PER_CYCLE/cycle)"
    while true; do
        run_cycle || true
        log "sleeping ${INTERVAL}s until next cycle"
        sleep "$INTERVAL"
    done
else
    run_cycle
fi
