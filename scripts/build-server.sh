#!/usr/bin/env bash
# Builds the KSON language-server bundle (server.js) from kson-org/kson at
# the commit pinned in server/KSON_COMMIT, and packages it as the release
# asset the extension's distribution path downloads.
#
#   scripts/build-server.sh <kson-checkout> <output-dir>
#
# <kson-checkout> must be a clone of https://github.com/kson-org/kson at
# the pinned commit. The steps are the ones in README "Local server
# setup". Requirements: Node.js 24+, curl and tar (the Gradle wrapper
# downloads its own JDK). pnpm is run through npx, so it need not be
# installed.
#
# Output:
#   <output-dir>/kson-language-server/server.js   the bundle (plain JS)
#   <output-dir>/kson-language-server/LICENSE     upstream Apache-2.0 license
#   <output-dir>/kson-language-server/SOURCE      upstream repo + commit
#   <output-dir>/kson-language-server.tar.gz      the release asset
#   <output-dir>/SHA256SUMS                       checksum of the asset

set -euo pipefail

if [ $# -ne 2 ]; then
  echo "usage: $0 <kson-checkout> <output-dir>" >&2
  exit 2
fi

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
KSON_DIR="$(cd "$1" && pwd)"
mkdir -p "$2"
OUT_DIR="$(cd "$2" && pwd)"
PINNED="$(tr -d '[:space:]' < "$REPO_ROOT/server/KSON_COMMIT")"

actual="$(git -C "$KSON_DIR" rev-parse HEAD 2>/dev/null || true)"
if [ -n "$actual" ] && [ "$actual" != "$PINNED" ]; then
  echo "build-server: $KSON_DIR is at $actual, expected pinned $PINNED" >&2
  exit 1
fi

cd "$KSON_DIR"
./gradlew --no-daemon tooling:language-server-protocol:npm_install
./gradlew --no-daemon tooling:language-server-protocol:npm_run_compile

cd tooling/lsp-clients
npx --yes pnpm@9 install --no-frozen-lockfile
cd vscode
npx --yes pnpm@9 exec tsx esbuild.ts --production

STAGE="$OUT_DIR/kson-language-server"
rm -rf "$STAGE"
mkdir -p "$STAGE"
cp dist/server.js "$STAGE/server.js"
cp "$KSON_DIR/LICENSE" "$STAGE/LICENSE"
printf 'https://github.com/kson-org/kson\n%s\n' "$PINNED" > "$STAGE/SOURCE"

# The asset unpacks server.js at its root, matching the cache layout in
# src/distribution.rs (<cache>/server/<version>/server.js).
tar -czf "$OUT_DIR/kson-language-server.tar.gz" -C "$STAGE" server.js LICENSE SOURCE
(cd "$OUT_DIR" && sha256sum kson-language-server.tar.gz > SHA256SUMS)

echo "build-server: built $STAGE/server.js from kson-org/kson@$PINNED"
