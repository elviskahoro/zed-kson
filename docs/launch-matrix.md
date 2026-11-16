# Adapter launch matrix (WI-3)

Verification record for the `kson-language-server` adapter
(`src/lib.rs` and its `node`, `distribution`, `config` modules). The
matrix is part of WI-3's verification contract; WI-4 (feature parity)
and WI-5 (release readiness) build on these results and re-run the
Zed-side rows.

Two verification levels are used, and each row states which applies:

- **E2E** — executed end-to-end against the real server bundle
  (`tooling/lsp-clients/vscode/dist/server.js`, built from the upstream
  pin `928961c0669166b3852c5f67019ff0ceb57b7599` by
  `./gradlew tooling:language-server-protocol:npm_install`,
  `./gradlew tooling:language-server-protocol:npm_run_compile`, `pnpm
  install` in `tooling/lsp-clients`, then `tsx esbuild.ts`; see
  README "Local server setup").
- **Unit** — the adapter's resolution/gate logic is pure and covered by
  `cargo test` (24 tests); the row's observable behavior is the tested
  error string or decision. Full Zed-side reproduction requires the
  Zed extension host, so the procedure for the Zed dev-extension run
  is documented per row.

## Results

| # | Scenario | Level | Result |
| --- | --- | --- | --- |
| 1 | Override path, happy launch: `node <binary.path> --stdio` with a full LSP handshake (initialize → initialized → didOpen → textDocument/diagnostic → shutdown → exit) | E2E | **Pass** on Node 24.0.x (pixi-managed, matches `engines.node: "24.x"`) and Node v26.2.0 (system). Initialize result includes the advertised capabilities (hover, definition, completion, formatting, pull diagnostics, semantic tokens, code lens, folding range, selection range, document highlight, document symbol, execute command); clean exit code 0. |
| 2 | Override path, diagnostics flow: invalid `.kson` document | E2E | **Pass** — `textDocument/diagnostic` returns a severity-1 diagnostic (`source: "kson"`, "Object properties must be `key: value` pairs") for `{ name: "smoke" items ["a" }`. |
| 3 | Transport flag is required: launch without `--stdio` | E2E | **No LSP response on stdio within 5s** — `createConnection()`'s argv detection needs `--stdio`; the flag is required, not decorative. |
| 4 | Defaults: no `binary.path` set | Unit | The pinned distribution (`PINNED_DISTRIBUTION`, a release asset of this repository) is downloaded once into a version-keyed cache and launched (`pinned_distribution_points_at_this_repository_release`, cache tests). Zed procedure: install dev extension with no `binary.path` → the server downloads and starts. |
| 5 | Missing Node runtime (no bundled Node, no `node` on PATH) | Unit | Defined failure naming the prerequisite: "install Node.js 24+ … on PATH, or update Zed so its bundled Node is included" (`node::tests::errors_name_the_prerequisite_to_fix`). The adapter prefers Zed's bundled Node (`zed::node_binary_path`), so PATH absence alone does not trigger this row. |
| 6 | Node too old (major < 24) | Unit | Defined failure with observed version and fix URL (`node::tests::version_gate_matches_upstream_requirement`, `node_too_old_error` wording). Gate accepts ≥ 24; Node 26 verified working in row 1. |
| 7 | Bad `binary.path`: missing / directory / unreadable | Unit | Three distinct actionable errors, each naming the setting and the path (`override_validation_rejects_missing_and_directory_paths`, `override_errors_name_the_setting_and_the_path`, including the `chmod +r` hint). |
| 8 | Offline / restricted network | Unit | Download failures surface as "failed to download … If you are offline or behind a restricted network, build the server locally … set `binary.path`" (`download_failed_error` wording test). A previously cached version keeps working offline because the cache is checked before any download. |
| 9 | Unsupported platform | Unit | Not applicable: the bundle is platform-independent JavaScript, so every platform gets the same asset (`every_platform_gets_the_platform_independent_asset`). Node.js 24+ is the only runtime requirement (rows 5–6). |
| 10 | Settings propagation: initialization options and workspace configuration with user overrides | Unit | Deep merge keeps unset defaults while user keys win (`config::tests`: `user_keys_override_defaults_key_by_key`, `nested_objects_merge_recursively`, `explicit_user_null_wins_over_the_default`); workspace configuration wraps user `settings` under the `"kson"` section. Runtime propagation through Zed is exercised by WI-4's configuration-change fixtures. |
| 11 | Repeat launches / upgrades do not redownload | Unit | Cache is keyed on the pinned version with an `INSTALLED_VERSION` marker; marker match short-circuits the download and stale versions are pruned (`marker_match_ignores_surrounding_whitespace`, `stale_versions_are_removed_but_kept_version_survives`). Unexercised against a real artifact for the same reason as row 8. |

## Reproducing the E2E rows

The E2E harness (JSON-RPC framing over the child's stdio) lives at
`/tmp/kson-smoke/recv_by_id.py` in the WI-3 session; the essential
shape is:

```bash
node <path-to>/dist/server.js --stdio   # speak LSP over stdin/stdout
```

Send `initialize` (with `initializationOptions` `{ "distributionId":
"kson", "enableBundledSchemas": true }`), then `initialized`,
`textDocument/didOpen`, `textDocument/diagnostic`, `shutdown`, `exit`.
The server answers on stdout with `Content-Length`-framed JSON-RPC and
exits 0 after `exit`.

## Observations recorded for WI-4/WI-5

- The server is pull-diagnostics only (`diagnosticProvider`
  capability; no `textDocument/publishDiagnostics` was observed). Whether
  Zed polls `textDocument/diagnostic` for this server is the open
  question OQ1 from `docs/compatibility-inventory.md`; WI-4 must verify
  it inside Zed.
- With no workspace root the server logs "No workspace root available,
  schema configuration disabled" — schema features need a real worktree
  root; WI-4's fixtures should account for that.
- Advertised capabilities seen at initialize (row 1) match the WI-1
  inventory's inspection of the server source.
