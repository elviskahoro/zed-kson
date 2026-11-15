#!/usr/bin/env bash
# Feature-parity checks for the zed-kson extension (WI-4).
#
# Runs the LSP parity harness (scripts/parity_harness.py) against a
# locally built KSON server bundle, plus the static snippet checks that
# need no server. Every fixture either passes or carries an explicit
# deferred disposition; nothing is faked.
#
# Set the server bundle path with $KSON_SERVER or pass --server:
#   KSON_SERVER=~/src/kson/tooling/lsp-clients/vscode/dist/server.js \
#     ./scripts/check-parity.sh
#
# Without a server bundle the server fixtures are DEFERRED (exit 0, the
# deferred disposition is explicit in the output); a FAIL exits 1.
#
# WI-5 wires this script into CI. Requires python3 and node (24+) with
# the server bundle built per README "Local server setup".

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SERVER="${KSON_SERVER:-}"
NODE="${KSON_NODE:-node}"

command -v python3 >/dev/null 2>&1 || {
  echo "check-parity: python3 is required" >&2
  exit 1
}

python3 "$REPO_ROOT/scripts/parity_harness.py" --server "$SERVER" --node "$NODE"
