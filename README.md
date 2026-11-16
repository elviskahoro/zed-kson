# zed-kson

[KSON](https://github.com/kson-org/kson) language support for
[Zed](https://zed.dev): a Tree-sitter grammar with highlighting,
indentation and injection queries, snippets, and a hardened language-server
adapter (diagnostics, formatting, schema features).

[![CI](https://github.com/elviskahoro/zed-kson/actions/workflows/ci.yml/badge.svg)](https://github.com/elviskahoro/zed-kson/actions/workflows/ci.yml)

## Status

- **Syntax support is complete (WI-2).** Tree-sitter grammar authored
  in-repo at [`tree-sitter-kson/`](./tree-sitter-kson) (none exists in
  kson-org; decision D1 in `docs/compatibility-inventory.md`), registered
  under `[grammars.kson]` in `extension.toml`, with `highlights.scm`,
  `indents.scm` and `injections.scm` under `languages/kson/`.
- **The LSP adapter is hardened and verified (WI-3).** `cargo check
  --target wasm32-wasip2` passes and the adapter logic is unit-tested
  (`cargo test`, 24 tests); the launch contract `node <bundle> --stdio`
  was exercised end-to-end against a real server build — see
  [`docs/launch-matrix.md`](./docs/launch-matrix.md).
- **Feature parity is exercised (WI-4).** Diagnostics, formatting, schema
  features and the bundled metaschema verified against the real server by
  `./scripts/check-parity.sh`; VS Code-only surfaces carry documented
  dispositions ([`docs/compatibility-inventory.md`](./docs/compatibility-inventory.md),
  [`fixtures/parity/README.md`](./fixtures/parity/README.md)).
- **CI runs all automated gates (WI-5)** on every push and PR:
  extension compile, grammar/query validity, fixture checks
  ([`.github/workflows/ci.yml`](./.github/workflows/ci.yml)).
- **Blocked upstream: language-server distribution.** The KSON server is
  not published as a runnable artifact (see
  `docs/compatibility-inventory.md` at the upstream pin `928961c`). Until
  that changes you build it locally and point the extension at it via
  `binary.path` (below). Nothing else is blocked: highlighting, indents,
  injections and snippets work with no server at all.

## Installation

The extension is not yet in the Zed extension listing (publication prep
is tracked in [`docs/zed-publication-guide.md`](./docs/zed-publication-guide.md);
submitting that PR is deliberately out of scope). To use it today, install
it as a dev extension:

1. Build the server bundle (next section) or skip it to use syntax
   features only.
2. Open this repository in Zed.
3. Open the command palette → `zed: extensions page` → **Install Dev
   Extension** (installs from the open repository).
4. Configure the server as shown under "Local server setup".

The full dev-install walkthrough — including what to verify after
installing — is [`docs/dev-install-smoke-test.md`](./docs/dev-install-smoke-test.md).

## Local server setup

The extension launches a server you build yourself:

```bash
git clone https://github.com/kson-org/kson
cd kson
./gradlew tooling:language-server-protocol:npm_install   # builds Kotlin/JS deps + installs (JDK auto-downloaded)
./gradlew tooling:language-server-protocol:npm_run_compile
cd tooling/lsp-clients
pnpm install --no-frozen-lockfile
cd vscode
../node_modules/.bin/tsx esbuild.ts                      # produces dist/server.js
```

Then configure Zed:

```json
{ "lsp": { "kson-language-server": { "binary": { "path": "/path/to/kson/tooling/lsp-clients/vscode/dist/server.js" } } } }
```

## Runtime requirements

- **Node.js ≥ 24.** The server declares `engines.node: "24.x"`. The
  adapter prefers Zed's bundled Node and falls back to `node` on PATH;
  every candidate is version-checked (`node --version`) before launch,
  and a too-old or unusable Node fails with an actionable message.
  Verified runtimes: Node 24 (pixi-managed) and Node v26.2.0 — see
  `docs/launch-matrix.md`.
- The server bundle is a plain JavaScript file; no native binaries are
  involved, and `kson-lib` native archives must never be used as the
  server executable.
- Syntax-only use (no server configured) has no requirements beyond Zed
  itself; highlighting/indents/injections come from the Tree-sitter
  grammar, not the server.

## Server launch flags (verified)

The adapter runs `node <binary.path> --stdio`. `--stdio` is required:
the server's `createConnection()` (vscode-languageserver v10) selects
its transport from argv, and without the flag the server never speaks
LSP on stdio. This is verified end-to-end against a locally built
server at the pinned revision (initialize → diagnostics → shutdown →
exit; results in `docs/launch-matrix.md`).

## Configuration

| Setting | Effect |
| --- | --- |
| `lsp.kson-language-server.binary.path` | Path to a locally built `server.js`. Takes precedence over any future downloaded distribution. Validated at launch: missing paths, directories, and unreadable files each fail with a message naming the setting. |
| `lsp.kson-language-server.binary.arguments` | Extra arguments passed to the server after `--stdio` (not to `node`). |
| `lsp.kson-language-server.binary.env` | Extra environment variables for the server. |
| `lsp.kson-language-server.initialization_options` | Merged (deep, key-by-key) over the defaults `{ "distributionId": "kson", "enableBundledSchemas": true, "bundledSchemas": [], "bundledMetaSchemas": […draft-07 metaschema, shipped in `schemas/`] }`; your keys win, unset keys keep defaults (arrays replace, not merge). |
| `lsp.kson-language-server.settings` | Passed to the server under its workspace-configuration section, which is the effective `distributionId` (`"kson"` unless you override it in `initialization_options`), e.g. `{ "format": { "formattingStyle": "delimited" } }` for delimited output. |

## Feature coverage

Everything below is verified either by the automated checks
(`scripts/check-grammar.sh`, `scripts/check-parity.sh`, `cargo test`,
CI) or by the recorded verification matrices
(`docs/launch-matrix.md`, `fixtures/parity/README.md`):

- **Language**: `.kson` recognition, `#` line comments, bracket and
  quote pairing (`languages/kson/config.toml`).
- **Syntax**: highlighting, indentation and `%kotlin`/`%` embed
  injections via Tree-sitter queries; corpus covers valid KSON,
  JSON-compatible syntax, comments, strings, collections, embedded
  blocks, end markers, Unicode and incomplete edits.
- **Snippets**: `kv`, `embed%`, `embed$`, `list`
  ([`snippets/kson.json`](./snippets/kson.json)).
- **Diagnostics** (server): pull-model validation — valid documents are
  clean; syntax errors and misleading-indentation warnings render with
  severities (fixtures F1–F7).
- **Formatting** (server): the native format action honors editor
  indentation (spaces/tabs, any width); output style
  (plain/delimited/compact/classic) follows the `format.formattingStyle`
  setting; formatting is idempotent (F8–F16).
- **Schema features** (server): workspace association via
  `.kson-schema.kson`, validation, hover documentation, property and
  enum completions (F17–F24); the draft-07 metaschema ships bundled for
  `.schema.kson` files (F25–F28).
- **Other server surfaces**: code-lens format commands, folding ranges,
  selection ranges, document highlight, document symbols, hover,
  definition (file URIs).

## Parity gaps (known limitations)

Verified against the VS Code reference client; evidence and Zed-native
alternatives in `docs/compatibility-inventory.md` (items P6–P14):

- **No schema picker** (VS Code QuickPick): edit `.kson-schema.kson`
  directly; the server's `associateSchema`/`removeSchema` commands stay
  wire-callable (P6).
- **No status-bar schema indicator**: no Zed extension API for it;
  schema feedback comes from hover/completions/diagnostics (P7).
- **`bundled://` schema URIs cannot be opened**: Zed's LSP
  implementation supports file URIs only; validation/completions are
  unaffected (P8).
- **`.kson-schema.kson` edits do not hot-reload**: the server registers
  no file watchers and Zed only watches server-registered globs;
  reopen the document or restart the server (P12).
- **Code-lens format buttons use the server's 2-space default**: Zed
  has no middleware to inject editor indentation; the native format
  action honors your settings (P11).
- **Browser/web-worker target**: not applicable — Zed extensions run
  natively (P14).

## Future: downloaded server distribution

Once kson-org publishes a runnable server release, the adapter will
download a pinned version through the Zed extension API into a
version-keyed cache (repeat launches never redownload; upgrades
download once and prune stale versions). That path is implemented and
unit-tested in `src/distribution.rs` but currently unexercised — no
artifact exists to download — and stays dormant behind
`PINNED_DISTRIBUTION = None` while your `binary.path` override remains
the always-works path. Offline/restricted environments and platforms
without a release asset get defined errors pointing back at
`binary.path`.

## Troubleshooting

Every adapter failure names the setting or prerequisite to fix:

- *"no server script configured"* — set `binary.path` (see Local server setup).
- *"no Node.js runtime found"* / *"Node.js … is vX, but the KSON server requires Node.js 24 or newer"* — install Node 24+ on PATH or update Zed so its bundled Node is current.
- *"binary.path … does not exist / is not a file / is not readable"* — fix the path or its permissions (`chmod +r`).
- Launch failures inside Zed (as opposed to resolution failures) show up as the server exiting immediately; check `zed: log` for the adapter's error message.

See `docs/launch-matrix.md` for the full verification record and
`docs/compatibility-inventory.md` for the upstream evidence base
(pinned revision `928961c0669166b3852c5f67019ff0ceb57b7599`).

## Development

Prerequisites: Rust with the `wasm32-wasip2` target
(`rustup target add wasm32-wasip2`), the tree-sitter CLI 0.25+
(`npm install -g tree-sitter-cli`), python3, and Node 24+ for the
server-backed fixtures.

```bash
cargo check --target wasm32-wasip2   # extension compile gate
cargo test                           # adapter unit tests (host)
./scripts/check-grammar.sh           # grammar freshness, corpus, queries
KSON_SERVER=<bundle> ./scripts/check-parity.sh   # full parity suite
./scripts/check-parity.sh            # static fixtures only (server fixtures DEFERRED)
```

CI (`.github/workflows/ci.yml`) runs the compile, grammar and static
fixture gates on every push and PR. After any adapter or grammar
change, walk through
[`docs/dev-install-smoke-test.md`](./docs/dev-install-smoke-test.md)
in a real Zed install.

Grammar development notes (local overrides via `file://` URLs, how the
`[grammars.kson]` pin works) live in
[`docs/grammar-registration.md`](./docs/grammar-registration.md).

## Contributing

- Keep changes inside the extension's boundary: the adapter stays
  dependency-light (`zed_extension_api` is the only crate dependency),
  and KSON semantics belong to kson-org upstream — reuse, never
  reimplement.
- Every claimed behavior needs evidence: a fixture, a unit test, or a
  recorded observation in the docs. Do not claim platform or feature
  coverage that was not tested; give untested surfaces an explicit
  disposition instead.
- Run the full Development gate set before submitting; commits follow
  conventional-commit style (`feat: …`, `docs: …`, matching the
  existing history).

## Release process

1. Confirm CI is green on `main` and the dev-install smoke test
   (`docs/dev-install-smoke-test.md`) passes in a clean Zed install.
2. Bump `version` in `extension.toml` (and `Cargo.toml` to match) and
   update the `[grammars.kson]` `rev` if the grammar changed.
3. Follow [`docs/zed-publication-guide.md`](./docs/zed-publication-guide.md)
   for the zed-industries/extensions submission checklist and the
   update procedure for future versions.

## License

Apache-2.0 (see [LICENSE](./LICENSE)). The bundled draft-07 metaschema
(`schemas/metaschema.draft7.kson`) is copied verbatim from the upstream
kson VS Code extension, under the same license.
