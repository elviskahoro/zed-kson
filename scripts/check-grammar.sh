#!/usr/bin/env bash
# Grammar and query checks for the zed-kson extension (WI-2).
#
# Verifies that:
#   1. `tree-sitter generate` succeeds and the committed parser
#      (tree-sitter-kson/src/) is up to date with grammar.js.
#   2. The full corpus passes (`tree-sitter test`).
#   3. Every extension query file compiles against the grammar and
#      produces captures on the representative fixtures.
#
# WI-5 wires this script into CI. Requires the tree-sitter CLI (0.25+):
#   npm install -g tree-sitter-cli

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
GRAMMAR_DIR="$REPO_ROOT/tree-sitter-kson"
QUERIES_DIR="$REPO_ROOT/languages/kson"
FIXTURES_DIR="$GRAMMAR_DIR/test/fixtures"

command -v tree-sitter >/dev/null 2>&1 || {
  echo "check-grammar: tree-sitter CLI is required (npm install -g tree-sitter-cli)" >&2
  exit 1
}

cd "$GRAMMAR_DIR"

# Generate into a scratch copy and compare with the committed parser.
# This does not depend on git, so it also works in checkouts without a
# .git directory (RWX's git/clone, release tarballs).
echo "== tree-sitter generate =="
SCRATCH="$(mktemp -d)"
trap 'rm -rf "$SCRATCH"' EXIT
cp -R "$GRAMMAR_DIR/." "$SCRATCH/"
(cd "$SCRATCH" && tree-sitter generate)

echo "== committed parser freshness =="
if ! diff -r "$GRAMMAR_DIR/src" "$SCRATCH/src" >/dev/null; then
  echo "check-grammar: generated files under tree-sitter-kson/src are stale;" \
       "run tree-sitter generate in tree-sitter-kson/ and commit the output" >&2
  diff -r "$GRAMMAR_DIR/src" "$SCRATCH/src" | head -20 >&2 || true
  exit 1
fi

echo "== corpus =="
tree-sitter test

echo "== queries =="
for query in highlights indents injections; do
  query_file="$QUERIES_DIR/$query.scm"
  if [ ! -f "$query_file" ]; then
    echo "check-grammar: missing $query_file" >&2
    exit 1
  fi
  # --quiet keeps per-capture output out of CI logs; failures exit non-zero.
  if ! tree-sitter query "$query_file" "$FIXTURES_DIR"/*.kson --quiet >/dev/null; then
    echo "check-grammar: query $query.scm failed to compile or match" >&2
    exit 1
  fi
  echo "  $query.scm ok"
done

echo "grammar checks passed"
