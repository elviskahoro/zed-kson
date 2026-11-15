# KSON Compatibility Inventory (WI-1)

Discovery document for the Zed KSON extension. This is the evidence base that
WI-2 (syntax foundation) and WI-3 (LSP adapter hardening) execute against: the
pinned upstream revision, an inspection of the KSON language server, a parity
inventory of the VS Code reference client, and the two ownership decisions the
plan requires. Nothing below is stated from runtime observation unless
explicitly marked; everything else is verified by source inspection at the
pinned revision.

## Upstream pin

- Repository: https://github.com/kson-org/kson
- Pinned commit: `928961c0669166b3852c5f67019ff0ceb57b7599`
  (default branch HEAD at clone time, 2026-11-15; commit date 2026-11-13,
  "Merge pull request #502 from holodorum/refactor/simplify-union-narrowing")
- All `file:line` citations below are relative to the kson repo root at this
  commit. Later work items (WI-2…WI-5) cite this pin so drift is detectable;
  re-run this inventory if the pin moves.

## Language server inspection (`tooling/language-server-protocol`)

### Package and distribution status

- Package `kson-language-server`, version `0.5.0-dev.0`
  (`tooling/language-server-protocol/package.json:2,14`). It is published as a
  **library only**: exports are `.`, `./node`, `./browser` (package.json:16-31)
  and there is **no `bin` entry**, so upstream ships no standalone CLI
  executable.
- Runtime dependencies `kson` and `kson-tooling` are `link:` dependencies on
  gradle-built Kotlin/JS output living inside the kson repo
  (`package.json:45-46`, pointing at `kson-lib/build/dist/js/productionLibrary`
  and `kson-tooling-lib/build/dist/js/productionLibrary`). Nothing the server
  needs is on a package registry.
- Registry check (2026-11-15): `kson-language-server` and `kson-tooling` return
  404 on npm. The `kson` package on npm (0.0.3) belongs to an unrelated project
  (`sakatam/kson`, maintainer sakatam) and is **not** kson-org's KSON library.
  Conclusion: no published, runnable server distribution exists today.
- Build: gradle from the kson repo root; `./gradlew
  tooling:language-server-protocol:npm_install` also builds the Kotlin/JS
  libraries the server links against (`tooling/language-server-protocol/README.md`,
  "Local development"). A locally usable server bundle additionally requires the
  VS Code client's esbuild step (below).
- Node requirement: `engines.node: "24.x"` (server `package.json:42`; the VS
  Code extension declares `node: "24.x"` and `vscode: ^1.99.3` too).

### Transport and CLI flags (the `--stdio` question)

- The Node entry that the VS Code extension bundles is
  `tooling/lsp-clients/vscode/src/server/node/ksonServerMain.ts`: it calls
  `createConnection()` from `vscode-languageserver/node` (ksonServerMain.ts:1,5)
  and hands the connection to `startKsonServer` (ksonServerMain.ts:9). It
  accepts no CLI flags of its own.
- The VS Code client launches the esbuild-bundled `dist/server.js` with
  `TransportKind.ipc` (`tooling/lsp-clients/vscode/src/client/node/ksonClientMain.ts:32,42-43`;
  the `server.js` entry is produced by `tooling/lsp-clients/vscode/esbuild.ts:76-78`).
- The server pins `vscode-languageserver` `^10.1.2`
  (`tooling/language-server-protocol/package.json:47`). In that library,
  `createConnection()` with no arguments selects the transport from the
  process's command line (`--node-ipc`, `--socket`, `--pipe`, `--stdio`) — this
  is documented library behavior, not something the kson repo states itself.
- **Disposition for this repo:** `node <server.js> --stdio` is the
  library-supported stdio path and the Zed adapter's current `--stdio` argument
  is consistent with that contract. However, no shipped server artifact exists
  to execute, so runtime verification is deferred to WI-3 (against a locally
  built server.js). Until that smoke test passes, the repo README keeps its
  "stdio flag unverified" caveat; WI-3 must update the README either way.

### Server capabilities (advertised at initialize)

From `src/startKsonServer.ts:146-207`:

- `textDocumentSync: Full` (full-document sync, not incremental).
- Semantic tokens, full requests only (legend at
  `src/core/features/SemanticTokensService.ts:4`).
- `documentFormattingProvider: true`.
- Diagnostics via the **pull model** (`diagnosticProvider`, identifier =
  `distributionId`, no inter-file dependencies, no workspace diagnostics;
  startKsonServer.ts:158-163). No push `publishDiagnostics` path exists in the
  server source.
- Code lens (`resolveProvider: false`).
- Execute command: `${distributionId}.{plainFormat, delimitedFormat,
  compactFormat, classicFormat, associateSchema, removeSchema}`
  (`src/core/commands/CommandType.ts:8-29`, wire ids built at
  startKsonServer.ts:171-174).
- Folding ranges, selection ranges, document highlight, document symbols,
  hover, definition, workspace folders.
- Completion with trigger characters `"`, `'`, newline, space, no resolve
  provider (startKsonServer.ts:200-207).
- Custom endpoints: request `kson/getDocumentSchema`
  (startKsonServer.ts:236) and notification `kson/schemaConfigurationChanged`
  (emitted by `src/core/schema/notifySchemaChange.ts`, which also requests
  `workspace/diagnostic/refresh`, and consumed by the client at
  ksonClientMain.ts:80).

### Initialization options and configuration shape

- Initialization options (`KsonInitializationOptions`,
  `src/startKsonServer.ts:26-38`): `bundledSchemas` (list matched by file
  extension, contents passed inline), `bundledMetaSchemas` (matched by a
  document's `$schema` content), `enableBundledSchemas` (default `true`),
  `distributionId` (default `"kson"`; used as command-id prefix, diagnostic
  identifier, and the configuration section name — startKsonServer.ts:62,98).
  The current Zed adapter defaults (`distributionId: "kson"`,
  `enableBundledSchemas: true`) match the server defaults.
- Workspace configuration is pulled via `workspace/configuration` with section
  `distributionId` (startKsonServer.ts:217) and pushed via
  `DidChangeConfigurationNotification` registration (startKsonServer.ts:226).
  The settings slice shape (`src/core/KsonSettings.ts`): `format.
  formattingStyle` (`plain`/`delimited` surfaced in VS Code; the shared mapper
  also accepts `compact`/`classic`), `codeLens.enable` (default true), and
  `enableBundledSchemas` (also toggles the bundled provider at runtime,
  startKson.ts onDidChangeConfiguration). `kson.trace.server` is a client-side
  setting only, never read by the server.
- Indentation is deliberately absent from settings: it is carried per-request
  by LSP `FormattingOptions` (KsonSettings.ts doc comment).

### Schema association and bundled schemas

- Workspace schema mapping lives in `.kson-schema.kson` at the workspace root
  (`SCHEMA_CONFIG_FILENAME`, `src/core/schema/SchemaConfig.ts:61`) with shape
  `{schemas: [{fileMatch: [globs], schema: workspace-relative path}]}`
  (SchemaConfig.ts:13-19). Schema/provider changes reload through
  `onDidChangeWatchedFiles` (startKsonServer.ts:268-286).
- Bundled schemas: the server-side `BundledSchemaProvider` consumes the
  initialization options (contents inline; startKsonServer.ts:101-122). The
  VS Code client reads schema files from `dist/extension/schemas` and passes
  the contents (`src/config/bundledSchemaLoader.ts`). The only bundled schema
  today is the draft-07 **metaschema** for `.schema.kson` files
  (`bundledSchemaLoader.ts:11,85-86`); per-language `bundledSchema` mappings
  come from the VS Code `package.json` languages contribution, and the KSON
  language itself declares none (`tooling/lsp-clients/vscode/package.json:32-49`,
  `"bundledSchema": null` at line 47).
- Bundled schemas surface as `bundled://schema/{ext}.schema.kson` and
  `bundled://metaschema/{name}.schema.kson` URIs, which VS Code can open via a
  TextDocumentContentProvider (see parity item P8).

## VS Code client parity inventory

Classifications: **supported** (Zed has a native equivalent today), **adapted**
(achievable in Zed with a different mechanism, owned by a later work item),
**deferred** (no Zed-native mechanism; documented disposition, candidate for
upstream or future Zed API), **blocked** (cannot be done until an external
dependency changes).

| # | Item | Evidence (file:line @ pin) | Disposition | Zed plan |
| --- | --- | --- | --- | --- |
| P1 | Language registration (`.kson`, aliases, icon) | `vscode/package.json:32-50` | supported | Already in `languages/kson/config.toml` (suffix `.kson`, `# ` comments, brackets); no action |
| P2 | Activation `workspaceContains:**/.kson-schema.kson` | `vscode/package.json:21-22` | adapted | Zed starts the LSP when a `.kson` file opens; schema-only workspaces don't need the server. Accept the difference; document it |
| P3 | TextMate grammar `source.kson` | `shared/extension/config/kson.tmLanguage.json`; `vscode/package.json:50-76` | blocked | No Tree-sitter grammar exists in kson-org (repo-wide search at pin finds none); see decision D1 — author one in WI-2 |
| P4 | Language configuration (line comment, brackets, onEnterRules) | `shared/extension/config/language-configuration.json:3,5,29` | adapted | Comment/bracket basics already in `languages/kson/config.toml`; auto-indent rules become Tree-sitter `indents.scm` in WI-2 |
| P5 | Snippets (`kv`, `embed%`, `embed$`, `list`) | `shared/extension/snippets/kson.code-snippets` | adapted | Convert to Zed snippet format in WI-4 (feature parity); conversion is mechanical |
| P6 | `kson.selectSchema` QuickPick schema picker | `vscode/src/client/node/ksonClientMain.ts:91-165` | deferred | Zed extension API 0.7.0 has no arbitrary-QuickPick UI. Zed-native alternative: edit `.kson-schema.kson` directly; server `associateSchema`/`removeSchema` commands stay wire-callable. Document the workflow in README (WI-5) |
| P7 | Status-bar schema indicator | `src/client/common/StatusBarManager.ts:25-29,45-49` | deferred | No status-bar API for Zed extensions. No Zed-native alternative today; record as known gap. Users get schema feedback via hover/completions/diagnostics |
| P8 | `bundled://` virtual document provider | `src/client/common/BundledSchemaContentProvider.ts:5-21` | adapted | Zed has no extension content-provider API; go-to-definition into `bundled://` URIs may fail to open (runtime-verify in WI-4). Bundled schema **contents** still work via init options, so validation/completion are unaffected. Document as known limitation |
| P9 | Client restart/error policy (3 restarts, then message) | `src/config/clientOptions.ts:11,76-77` | supported | Zed core manages server lifecycle/restart; adapter contributes actionable launch errors (WI-3) |
| P10 | `kson.trace.server` setting | `vscode/package.json:108-116` | adapted | Zed surfaces LSP traffic via its own logging/dev tools; no per-server trace enum needed |
| P11 | Middleware injecting editor indentation into format commands | `src/config/clientOptions.ts:57-67` | adapted | `textDocument/formatting` from Zed carries `FormattingOptions` (insertSpaces/tabSize) automatically. Code-lens format buttons get no such injection in Zed (no middleware concept) — best-effort, verify in WI-4 |
| P12 | Client-side file watching (`synchronize.fileEvents`) | `src/config/clientOptions.ts:53-54` | adapted | The server never dynamically registers watchers itself; it relies on the VS Code client's `synchronize.fileEvents`. Zed only watches files a server registers via `client/registerCapability`, so `.kson-schema.kson` edits may not hot-reload. Verify in WI-4; workaround is reopen/reassociate; candidate upstream contribution |
| P13 | User-facing notifications (crash/activation errors) | `ksonClientMain.ts:205-210` | supported | Zed surfaces server stderr and status natively; adapter adds actionable failure messages (WI-3) |
| P14 | Browser (VS Code Web / web worker server) | `src/client/browser/ksonClientMain.ts` | not applicable | Zed extensions run natively; browser target has no Zed counterpart |

Server-feature parity (what Zed can consume once the adapter launches the
server) is owned by WI-3/WI-4 fixtures; the inventory-level notes are:

- **Diagnostics (pull model only)** — the server exposes `diagnosticProvider`
  and no push path (startKsonServer.ts:158-163). Zed's LSP client support for
  pull diagnostics must be confirmed at runtime in WI-3 before relying on it
  (open item OQ1).
- **Semantic tokens** — full requests only; usable alongside Tree-sitter
  highlighting once WI-2 lands.
- **Formatting** — standard `textDocument/formatting`; indentation follows
  `FormattingOptions`, style follows the `format.formattingStyle` setting.

## Decisions

### D1 — Grammar ownership: author the Tree-sitter grammar here

Verified at the pin: kson-org/kson contains no Tree-sitter grammar (repo-wide
search for `tree-sitter`/`tree_sitter`/`treeSitter` at `928961c` matches
nothing relevant; the only syntax artifact is the TextMate grammar at
`tooling/lsp-clients/shared/extension/config/kson.tmLanguage.json`).

**Adopted default:** author the Tree-sitter grammar as part of this extension
work (WI-2), register it in `[grammars.kson]` per Zed's extension packaging
rules, and use the pinned TextMate grammar plus the VS Code extension's test
corpus (`tooling/lsp-clients/vscode/test/`) as the highlighting reference. Do
not wait for kson-org to ship one.

### D2 — Server distribution: keep the user-supplied `binary.path` override

Verified at the pin: the server is a library package with no `bin`, depends on
unpublished `link:` Kotlin/JS artifacts, requires Node 24.x, and nothing
runnable is on npm (see "Package and distribution status"). kson-lib's native
archives are Kotlin library builds, **not** server executables, and must never
be downloaded as such.

**Adopted default:** keep the user-supplied
`lsp.kson-language-server.binary.path` override as the always-works path (the
current adapter behavior). Add pinned-artifact download support only once
upstream publishes a runnable distribution; until then the README documents the
local-build path. Runtime verification of `--stdio` (see "Transport and CLI
flags") happens in WI-3 against a locally built server.js, and the README is
updated with the result.

## Open decisions and unresolved ambiguities

Each carries a recommended default; none blocks WI-2 from starting.

- **OQ1 — Zed pull-diagnostics support.** The server is pull-model only.
  Recommended default: WI-3 smoke-tests diagnostics with a locally built
  server; if Zed does not exercise `textDocument/diagnostic` for this server,
  record it as a hard gap and raise it upstream (request a push path or rely
  on Zed core support, whichever the runtime evidence shows).
- **OQ2 — `bundled://` navigation in Zed.** Recommended default: assume
  go-to-definition into `bundled://` URIs does not open in Zed; WI-4 verifies
  and the README documents it as a known limitation. Validation/completions
  from bundled schemas are unaffected.
- **OQ3 — Schema hot-reload without client-side watching.** Recommended
  default: WI-4 verifies whether `.kson-schema.kson` edits reload in Zed;
  workaround is reopen/reassociate, and an upstream contribution (dynamic
  watcher registration) is the long-term fix.
- **OQ4 — Code-lens format buttons without indentation middleware.**
  Recommended default: treat code-lens formatting as best-effort (server
  defaults apply); primary formatting is Zed's native format action, which
  carries `FormattingOptions`. WI-4 documents actual behavior.
- **OQ5 — Node 24 detection.** The adapter currently does not version-check
  Node. Recommended default: WI-3 adds an actionable startup failure when
  `node` is missing or unusably old, rather than silently spawning a broken
  server.

## Verification (for this document)

- Upstream clone at the pinned commit: `git clone https://github.com/kson-org/kson`
  → HEAD `928961c0669166b3852c5f67019ff0ceb57b7599` (2026-11-15).
- Registry checks: `npm view kson-language-server` → 404; `npm view
  kson-tooling` → 404; `npm view kson` → 0.0.3, `sakatam/kson` (unrelated).
- Every parity row above cites file-and-line evidence at the pin; no
  unverified upstream behavior is stated as fact (Zed-side capabilities are
  explicitly marked for runtime verification in WI-3/WI-4).
- Every open decision carries a recommended default; both required decisions
  (grammar ownership, server distribution) are recorded as adopted defaults.
