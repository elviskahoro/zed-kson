# Dev-install smoke test (WI-5)

The clean dev-install verification required by US-5 / WI-5: install this
extension as a dev extension in a real Zed and verify highlighting plus
server features end-to-end. This document is both the procedure and the
record; the "Record" section states exactly what has and has not been
executed, so no coverage is claimed that was not tested.

## Preconditions

- A clean Zed install (no prior dev install of this extension).
- Node.js ≥ 24 on PATH (or Zed's bundled Node current enough).
- Network access to github.com (the extension downloads the pinned
  server bundle from this repository's releases on first use). Without
  it, build the server per README "Local server setup" and set
  `binary.path`.

## Procedure

### 1. Clean state

Remove any previous dev install and cached grammar clones:

- Zed → Extensions page → uninstall the `kson` dev extension if present.
- Delete Zed's dev-extension grammar clone for `kson` if it exists
  (Zed clones `[grammars.kson]` `repository`/`rev` on first install;
  the local-override trick is in `docs/grammar-registration.md`).

### 2. Install dev extension

1. Open this repository in Zed.
2. Command palette → `zed: extensions page` → **Install Dev Extension**.
3. Expect: the extension installs; the `kson` grammar is fetched from
   the `[grammars.kson]` pin and compiled.

### 3. Syntax features (no server needed)

Open `fixtures/parity/documents/valid_plain.kson`:

- [ ] Highlighting: keys, strings, comments and the `%kotlin` embed
  block get distinct captures (`languages/kson/highlights.scm`; embeds
  via `injections.scm`).
- [ ] `#` line comment toggling works.
- [ ] Bracket/quote pairing and rainbow-indent behavior match
  `languages/kson/config.toml`.
- [ ] Indentation on newlines follows `indents.scm` (object members and
  list items indent; `=`/`.` end markers dedent).

Open `tree-sitter-kson/test/fixtures/readme_plain.kson` and repeat —
this is the same document class the grammar corpus covers.

### 4. Snippets

In an empty `.kson` buffer:

- [ ] Typing the prefix `kv` offers the `kv` snippet; inserting it
  expands `key: value` with tabstops on key and value.
- [ ] `embed%`, `embed$` and `list` snippets expand; `embed$` keeps its
  literal `$` escapes (`\$`, `$$` end markers).

### 5. Server features

Leave `lsp.kson-language-server.binary.path` unset and open any `.kson`
file:

- [ ] the server starts without configuration: `zed: open log` shows the
  bundle downloaded once into the extension's `server/<version>/`
  directory, and a second launch (restart Zed) does not download again.

Then:

- Open `fixtures/parity/documents/invalid_trailing.kson`:
  [ ] exactly one error diagnostic renders ("Unexpected trailing
  content…", severity 1, source `kson`) — the protocol behavior
  fixture-verified as F3/F5, here observed rendering in the editor.
- Open `fixtures/parity/documents/invalid_indent.kson`:
  [ ] one error + one misleading-indentation warning render (F4/F6).
- Open `fixtures/parity/documents/valid_plain.kson`:
  [ ] no diagnostics (F1); run the native format action
  (`editor: format`) — [ ] output honors the editor's indentation
  settings (F8–F10 protocol behavior).
- Schema features. The schema mapping (`.kson-schema.kson`) only applies
  when its folder is the project root, and the test documents live in
  `fixtures/parity/documents/`, so combine them in a scratch folder
  (this mirrors what `scripts/parity_harness.py` does):

  ```bash
  rm -rf /tmp/kson-schema-test && mkdir -p /tmp/kson-schema-test
  cp -R fixtures/parity/workspace/. /tmp/kson-schema-test/
  cp fixtures/parity/documents/*.config.kson /tmp/kson-schema-test/
  zed /tmp/kson-schema-test
  ```

  In that window:
  [ ] `schema_invalid.config.kson` shows "Missing required properties:
  appName, version" (F18);
  [ ] in `schema_valid.config.kson`, hover on `appName` shows "Application
  Name / The name of the application" (F19);
  [ ] on its empty line 3, ctrl-space offers `features` (F20);
  [ ] in `features.config.kson`, ctrl-space inside the `""` on line 3
  offers `logging`, `metrics`, `tracing`, `caching` (F21).
  The server reads `.kson-schema.kson` at startup, so open the folder
  fresh rather than adding the file to an open project.
- Open `fixtures/parity/documents/metaschema_bad.schema.kson`:
  [ ] bundled-metaschema violations render (F27) — proves the bundled
  draft-07 metaschema is wired through the adapter defaults.
- Code lens: [ ] format buttons appear on documents and apply the
  server's 2-space default (F14/F15 — the documented P11 limitation).
- Check `zed: log`: [ ] no adapter errors; the launch line shows
  `node <binary.path> --stdio`.

### 6. Known limitations to confirm (documented behavior, not defects)

- `.kson-schema.kson` edits do not hot-reload; reopening the document
  or restarting the server picks up changes (P12).
- Go-to-definition into `bundled://` URIs cannot open (P8).

## Record

Executed and verified (this build, on the WI-5 worktree commit):

| Evidence | Command / source | Result |
| --- | --- | --- |
| Extension compile gate | `cargo check --target wasm32-wasip2` | Pass |
| Adapter unit tests (24) | `cargo test` | Pass |
| Grammar: generate freshness, corpus, query validity | `./scripts/check-grammar.sh` | Pass |
| Static snippet fixtures S1–S3 | `./scripts/check-parity.sh` (no bundle) | Pass (server fixtures print explicit DEFERRED) |
| Full server-backed suite F1–F28 | `KSON_SERVER=<bundle> ./scripts/check-parity.sh` (WI-4 record) | All 31 pass — recorded in `fixtures/parity/README.md` |
| Launch contract E2E (initialize → diagnostics → shutdown → exit) | WI-3 record | Pass — `docs/launch-matrix.md` rows 1–3 |

**Interactive (GUI) pass: not yet executed.** The steps above that
require a running Zed GUI (sections 1–5 checkboxes) were **not**
performed in this build: the WI-5 execution environment is headless
and cannot run Zed's "Install Dev Extension" flow or observe in-editor
rendering. The protocol-level equivalents of every server behavior in
sections 3–5 were verified against the real server over stdio (WI-3
launch matrix, WI-4 fixture suite); the in-app rendering pass remains
an explicit deferred disposition (see
`docs/compatibility-inventory.md` OQ1 and
`fixtures/parity/README.md` "Dispositions carried into the inventory").

Before publication, run this procedure end-to-end in a clean Zed
install and check every box; the publication guide
(`docs/zed-publication-guide.md`) lists this as a blocking
prerequisite.
