# Parity fixtures (WI-4)

Feature-parity fixtures for the KSON language server as consumed through
the zed-kson extension's launch contract (`node <bundle> --stdio`,
initialization options from `src/config.rs`). They exercise the parity
surface from the WI-1 inventory (`docs/compatibility-inventory.md`):
diagnostics, formatting (indentation preferences, plain/delimited
output, idempotency), schema validation/hover/completion, workspace
schema association and reload, and the bundled metaschema.

Every expected behavior below was **observed** against the real server
built from the upstream pin `928961c0669166b3852c5f67019ff0ceb57b7599`
(`tooling/lsp-clients/vscode/dist/server.js`; build steps in the repo
README "Local server setup"). Nothing is stated from inspection alone;
nothing is faked. Run them with:

```bash
KSON_SERVER=~/src/kson/tooling/lsp-clients/vscode/dist/server.js \
  ./scripts/check-parity.sh
```

Without `KSON_SERVER` the server fixtures print an explicit `DEFERRED`
disposition (exit 0); the static snippet fixtures always run.

Layout:

- `documents/` — fixture documents, one scenario each
- `workspace/` — a workspace root (`.kson-schema.kson` +
  `config.schema.kson`, adapted from the upstream server examples) that
  the harness copies into a temp directory and passes as
  `workspaceFolders[0]` — the server reads its workspace root from
  there, **not** from `rootUri`

## Static fixtures (no server)

| ID | Fixture | Expected behavior | Observed |
| --- | --- | --- | --- |
| S1 | `snippets/kson.json` | All four VS Code reference snippets (`kv`, `embed%`, `embed$`, `list`) converted with names and prefixes preserved | Pass |
| S2 | `snippets/kson.json` | `embed$` body keeps `\$` escapes for the literal `$` and `$$` end markers (LSP snippet grammar, which Zed implements) | Pass |
| S3 | `snippets/kson.json` | Tabstops `${1:key}`/`${2:value}` and final `$0` survive conversion | Pass |

## Diagnostics (server fixtures F1–F7)

| ID | Fixture | Expected behavior | Observed |
| --- | --- | --- | --- |
| F1 | `documents/valid_plain.kson` | Plain-style document (indentation, dash lists, `=`/`.` end markers, `%kotlin` embed) produces no diagnostics | Pass — 0 items |
| F2 | `documents/valid_json_compat.kson` | JSON-compatible syntax is valid KSON; 0 diagnostics | Pass — 0 items |
| F3 | `documents/invalid_trailing.kson` | `key: "value" extraValue` → exactly one severity-1 diagnostic, "Unexpected trailing content…" | Pass |
| F4 | `documents/invalid_indent.kson` | Bad object pair + misleading list indent → one error ("Object properties must be `key: value` pairs") and one warning ("Misleading indentation…") | Pass |
| F5 | same as F3 | Diagnostic carries `severity: 1`, `source: "kson"` | Pass |
| F6 | same as F4 | Severities are exactly `[1, 2]` (error plus warning) | Pass |
| F7 | — | Server exits 0 after `shutdown`/`exit` | Pass |

Zed-side disposition (OQ1): the server is pull-diagnostics only; Zed
core supports the pull model (`textDocument/diagnostic` via
`GetDocumentDiagnostics`, refreshed on `workspace/diagnostic/refresh` —
`crates/project/src/lsp_store.rs`), so these diagnostics surface in Zed
without a push path. Verified at the protocol level here; the in-app
rendering is part of the WI-5 dev-install smoke test.

## Formatting (F8–F16)

Formatting `{"a":1,"b":{"c":2}}` via `textDocument/formatting`:

| ID | Fixture | Expected behavior | Observed |
| --- | --- | --- | --- |
| F8 | `insertSpaces: true, tabSize: 2` | `a: 1\nb:\n  c: 2` (plain style, 2-space indent) | Pass |
| F9 | `insertSpaces: true, tabSize: 8` | Same shape with 8-space indent — indentation follows `FormattingOptions` | Pass |
| F10 | `insertSpaces: false, tabSize: 4` | Tab-indented output | Pass |
| F11 | default settings | Plain style is the default output | Pass |
| F12 | config change | After `workspace/didChangeConfiguration` with `{"kson": {"format": {"formattingStyle": "delimited"}}}` (the shape `src/config.rs` sends from `lsp.kson-language-server.settings`), formatting returns the braced/delimited form `{\n  a: 1\n  b: {\n    c: 2\n  }\n}` | Pass |
| F13 | idempotency | Formatting the delimited output again is a no-op (no diff) | Pass |
| F14 | code lens | The server offers `kson.{plain,delimited,compact,classic}Format` code-lens commands | Pass |
| F15 | code lens format | Executing `kson.plainFormat` with the lens arguments (no indentation injected) applies a `workspace/applyEdit` with the server's 2-space default | Pass |
| F16 | — | Server exits 0 | Pass |

Zed users reach the style setting through:

```json
{ "lsp": { "kson-language-server": { "settings": { "format": { "formattingStyle": "delimited" } } } } }
```

(P11/OQ4) In VS Code, client middleware injects the active editor's
indentation into code-lens format commands. Zed has no middleware, so
code-lens buttons use the server's default (2 spaces); Zed's native
format action (`textDocument/formatting`) always carries the editor's
`FormattingOptions` and is the primary path (F8–F10). Zed handles
`workspace/applyEdit` (`crates/project/src/lsp_store.rs`), so the lens
buttons do apply their edits.

## Workspace schemas (F17–F24)

With `workspace/` as the root (`.kson-schema.kson` mapping
`**/*.config.kson` to `config.schema.kson`):

| ID | Fixture | Expected behavior | Observed |
| --- | --- | --- | --- |
| F17 | `schema_valid.config.kson` | Document satisfying the schema: 0 diagnostics | Pass |
| F18 | `schema_invalid.config.kson` | `key: "value"` (missing required `appName`/`version`) → one severity-2 diagnostic "Missing required properties: appName, version" | Pass |
| F19 | hover on `appName` | Markdown hover from the schema: "**Application Name**\n\nThe name of the application…" | Pass |
| F20 | completion at line 3 col 0 | Remaining schema properties suggested (`features`) | Pass |
| F21 | completion inside `features:- ""` | Enum values `logging`, `metrics`, `tracing`, `caching` suggested | Pass |
| F22 | `kson/getDocumentSchema` (note: `{uri}`, not textDocument) | `hasSchema: true`, schemaUri resolving to `config.schema.kson` | Pass |
| F23 | schema reload | After deleting `.kson-schema.kson` and notifying `workspace/didChangeWatchedFiles` (Deleted), the schema diagnostic disappears | Pass |
| F24 | — | Server exits 0 | Pass |

(P12/OQ3) The reload path above is what a client with file watching
sends (VS Code's `synchronize.fileEvents`). Zed only sends
`didChangeWatchedFiles` for globs a server registers via
`client/registerCapability` (`crates/project/src/lsp_store.rs`), and the
KSON server never registers watchers (`startKsonServer.ts` at the pin
registers only `DidChangeConfigurationNotification`). So in Zed today,
editing `.kson-schema.kson` does **not** hot-reload; the workaround is
reopening the document (or restarting the server), and the long-term fix
is an upstream watcher registration. Deferred with that disposition.

## Bundled metaschema (F25–F28)

The server only wires its bundled-schema provider when
`bundledSchemas`/`bundledMetaSchemas` arrive non-empty in the
initialization options (`startKsonServer.ts` at the pin). The adapter
therefore ships `schemas/metaschema.draft7.kson` (copied verbatim from
the upstream VS Code extension, Apache-2.0) and passes its contents
inline (`src/config.rs`); before WI-4 the defaults enabled nothing.

| ID | Fixture | Expected behavior | Observed |
| --- | --- | --- | --- |
| F25 | `metaschema_good.schema.kson` | `kson/getDocumentSchema` returns `bundled://metaschema/draft-07.schema.kson`, `hasSchema: true` | Pass |
| F26 | same | Valid `.schema.kson` document: 0 diagnostics | Pass |
| F27 | `metaschema_bad.schema.kson` | Metaschema violations reported (`type: 123` → "Value must match at least one sub-schema", `pattern: 5` → "Expected one of: string, but got: integer") | Pass |
| F28 | — | Server exits 0 | Pass |

(P8/OQ2) Validation and completions from bundled schemas work (above).
Navigation into `bundled://` URIs does not: Zed's LSP implementation
does not support non-file URI schemes
(`crates/project/src/lsp_store.rs`: "we don't support non-file URI
schemes in our LSP impl"), and Zed has no extension content-provider
API, so go-to-definition into a `bundled://` location cannot open the
schema. Known limitation, documented; classification stays **adapted**.

## Dispositions carried into the inventory

- **P5 snippets — adopted.** `snippets/kson.json` registered in
  `extension.toml`; conversion verified by S1–S3.
- **P11 code-lens indentation — adapted (best-effort).** F14/F15 record
  the actual behavior; native formatting is primary.
- **P12 schema hot-reload — deferred.** F23 proves the server-side path;
  the Zed-side watcher gap is source-verified with a workaround.
- **P8 `bundled://` — adapted.** Contents work (F25–F27); navigation is
  a known limitation.
- **OQ1 pull diagnostics — resolved (supported).** See the Zed-side note
  under Diagnostics.
- In-app (GUI) verification of the above — OQ1 rendering, snippet
  expansion, code-lens buttons — is **deferred** to the WI-5 dev-install
  smoke test; this harness verifies the same behaviors at the protocol
  level against the real server bundle.
