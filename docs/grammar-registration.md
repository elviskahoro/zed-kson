# Grammar registration and pinning (WI-2 record)

This document records how the Tree-sitter grammar landed in WI-2 is pinned
for Zed's extension packaging, and how to use it with a local dev install.
It executes against the WI-1 discovery decisions in
`docs/compatibility-inventory.md` (D1: author the grammar here) and
resolves the open pinning question the WI-2 work item carried
("in-repo path vs published grammar repo, confirmed against Zed docs").

## The mechanism (confirmed against Zed source and docs)

Verified against zed-industries/zed at commit
`24d37721115bd410b8f73998f8da5c4598b64a03` (2026-11-15):

- `extension.toml` registers grammars in a `[grammars.<name>]` table whose
  schema (`GrammarManifestEntry`, `crates/extension/src/extension_manifest.rs`)
  is exactly `repository` (git URL), `rev` (a git revision; `commit` is an
  accepted alias) and the optional `path` (a subdirectory of the cloned
  repository that contains `src/parser.c`).
- On every install — dev installs included — Zed's `compile_grammar`
  (`crates/extension/src/extension_builder.rs`) checks the grammar out into
  `<extension-dir>/grammars/<name>` as a **git clone whose `origin` must
  equal the manifest `repository`**, runs `git fetch --depth 1 origin <rev>`
  and `git checkout <rev>`, then compiles `src/parser.c` to wasm with the
  wasi-sdk clang. A failed fetch is tolerated as long as the checkout
  succeeds locally.
- Zed's docs (`docs/src/extensions/languages.md`) additionally bless
  `file://` URLs for `repository` during local development.

Two consequences drove the design:

1. Grammar sources cannot simply live as plain files at
   `grammars/<name>` — Zed requires that path to be its own git clone.
2. A grammar does not need a dedicated published repository: any git
   repository reachable at `repository` with the grammar under `path` works,
   including this extension's own repository.

## What WI-2 committed

- The grammar repository content lives in-repo at [`../tree-sitter-kson/`](../tree-sitter-kson)
  (`grammar.js`, generated `src/parser.c`, corpus, fixtures), tracked by
  this repository's git like any other source.
- `extension.toml` pins it:

  ```toml
  [grammars.kson]
  repository = "https://github.com/elviskahoro/zed-kson"
  rev = "953c3159037e82fc21986c88eeb3d05e3ef13e43"
  path = "tree-sitter-kson"
  ```

  The `rev` is the commit that introduced the completed grammar (milestone
  2 of the corpus-driven build); later commits only changed extension-side
  files outside `tree-sitter-kson/`, so the grammar content at `rev` equals
  the content at HEAD.

This refines WI-1's D1 recommended default ("author the grammar as part of
this extension work and publish/pin it per Zed's grammar packaging rules"):
the grammar is authored here and pinned **to this repository** rather than
to a not-yet-created dedicated grammar repository, which satisfies Zed's
packaging rules today. Moving the grammar to a dedicated repository later
(e.g. `kson-org/tree-sitter-kson`, also useful to non-Zed editors) only
requires updating `repository`/`rev` — `path` would be dropped.

## Trying it locally (dev install)

The `rev` above resolves once the referenced commit is pushed to
`https://github.com/elviskahoro/zed-kson` (pushing is out of scope for
this work item). Until then — or when working from an unpushed checkout —
prepare the grammar checkout the way Zed's installer expects:

```bash
git clone <your-zed-kson-checkout> grammars/kson
git -C grammars/kson remote set-url origin https://github.com/elviskahoro/zed-kson
```

`grammars/` is gitignored; the clone provides the pinned `rev` locally, the
remote matches the manifest, and Zed's fetch failure is tolerated because
the checkout succeeds (both behaviors verified against the Zed source
above). Then use Zed's "Install Dev Extension" on this directory.

Alternatively, point `repository` at a `file://` URL of your checkout and
use the checkout's current `rev` — the mechanism Zed's docs bless for local
development. Restore the published pin before committing.

## Before publication (WI-5)

- If this branch is rebased or squashed on merge, update `rev` to the final
  merged commit that contains `tree-sitter-kson/` (the content is what
  matters, not the exact sha).
- Re-run `./scripts/check-grammar.sh` after any grammar change; the
  committed `src/` output of `tree-sitter generate` is what Zed builds.
- The scaffolded top-level `repository` field in `extension.toml` still
  says `https://github.com/kson-org/zed-kson`; confirm the final home of
  this repository (and make the grammar pin match it) during publication
  prep.
