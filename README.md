# zed-kson

KSON support for Zed. Work in progress.

## Status
- Language config and Rust LSP adapter scaffolded; `cargo check --target wasm32-wasip2` passes.
- **Blocked: Tree-sitter grammar.** None exists in kson-org; `[grammars.kson]` is not yet in `extension.toml`, and highlight/indent/injection queries are pending it.
- **Blocked: language server distribution.** The KSON server is not published; build it from kson-org/kson (`tooling/language-server-protocol`) and set:

```json
{ "lsp": { "kson-language-server": { "binary": { "path": "/path/to/server.js" } } } }
```

Requires Node 24 on PATH. The adapter runs `node <path> --stdio` (stdio flag unverified).

`binary.arguments` and `binary.env` are forwarded to the server script (after `--stdio`), not to `node`. Node itself is not version-checked.
