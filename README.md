# zed-kson

KSON support for Zed. Work in progress.

## Status
- Language config and Rust LSP adapter hardened; `cargo check --target wasm32-wasip2` passes and the adapter logic is unit-tested (`cargo test`, 21 tests) — see `docs/launch-matrix.md`.
- **Tree-sitter grammar landed (WI-2).** Authored in-repo at [`tree-sitter-kson/`](./tree-sitter-kson) (none exists in kson-org; see `docs/compatibility-inventory.md` decision D1), registered under `[grammars.kson]` in `extension.toml`, with `highlights.scm`, `indents.scm` and `injections.scm` under `languages/kson/`. Run `./scripts/check-grammar.sh` to re-verify the grammar, corpus and queries. See [`docs/grammar-registration.md`](./docs/grammar-registration.md) for how the grammar is pinned and how to try it with a Zed dev install.
- **Blocked: language server distribution.** The KSON server is not published as a runnable artifact (see `docs/compatibility-inventory.md` at the upstream pin `928961c`). Build it locally and point the extension at it via `binary.path` (below).

## Local server setup

The extension launches the server you build yourself:

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

## Server launch flags (verified)

The adapter runs `node <binary.path> --stdio`. `--stdio` is required:
the server's `createConnection()` (vscode-languageserver v10) selects
its transport from argv, and without the flag the server never speaks
LSP on stdio. This is verified end-to-end against a locally built
server at the pinned revision (initialize → diagnostics → shutdown →
exit; results in `docs/launch-matrix.md`), replacing the earlier
"unverified" caveat.

## Configuration

| Setting | Effect |
| --- | --- |
| `lsp.kson-language-server.binary.path` | Path to a locally built `server.js`. Takes precedence over any future downloaded distribution. Validated at launch: missing paths, directories, and unreadable files each fail with a message naming the setting. |
| `lsp.kson-language-server.binary.arguments` | Extra arguments passed to the server after `--stdio` (not to `node`). |
| `lsp.kson-language-server.binary.env` | Extra environment variables for the server. |
| `lsp.kson-language-server.initialization_options` | Merged (deep, key-by-key) over the defaults `{ "distributionId": "kson", "enableBundledSchemas": true }`; your keys win, unset keys keep defaults. |
| `lsp.kson-language-server.settings` | Passed to the server as its `"kson"` workspace-configuration section. |

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
