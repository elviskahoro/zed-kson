# zed-kson

KSON support for Zed. Work in progress.

## Status
- Language config and Rust LSP adapter scaffolded; `cargo check --target wasm32-wasip2` passes.
- **Tree-sitter grammar landed (WI-2).** Authored in-repo at [`tree-sitter-kson/`](./tree-sitter-kson) (none exists in kson-org; see `docs/compatibility-inventory.md` decision D1), registered under `[grammars.kson]` in `extension.toml`, with `highlights.scm`, `indents.scm` and `injections.scm` under `languages/kson/`. Run `./scripts/check-grammar.sh` to re-verify the grammar, corpus and queries. See [`docs/grammar-registration.md`](./docs/grammar-registration.md) for how the grammar is pinned and how to try it with a Zed dev install.
- **Blocked: language server distribution.** The KSON server is not published; build it from kson-org/kson (`tooling/language-server-protocol`) and set:

```json
{ "lsp": { "kson-language-server": { "binary": { "path": "/path/to/server.js" } } } }
```

Requires Node 24 on PATH. The adapter runs `node <path> --stdio` (stdio flag unverified).

`binary.arguments` and `binary.env` are forwarded to the server script (after `--stdio`), not to `node`. Node itself is not version-checked.
