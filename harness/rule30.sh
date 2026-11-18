#!/usr/bin/env bash
# Run a plurcast binary as rule30 from WSL (Windows blocks local Rust builds).
# Usage (from Git Bash or PowerShell):  wsl -e bash /mnt/c/Users/Trist/engineering/plurcast/harness/rule30.sh plur-post "text" --format json
# Usage (inside WSL):                   harness/rule30.sh plur-queue list
set -euo pipefail
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$HOME/.cache/plurcast-target}"
export PLURCAST_CONFIG="$HOME/.config/plurcast-claude/config.toml"
export PLURCAST_DB_PATH="/mnt/c/Users/Trist/AppData/Local/plurcast-claude/posts.db"
bin="${1:?usage: rule30.sh <plur-post|plur-queue|plur-send|plur-history> [args...]}"; shift
# Always build: it is a fast no-op when current, and a stale binary would
# silently run old behaviour after the source changes.
(cd "$REPO" && cargo build -q -p "$bin") >&2
exec "$CARGO_TARGET_DIR/debug/$bin" "$@"
