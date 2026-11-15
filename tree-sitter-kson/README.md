# tree-sitter-kson

A [Tree-sitter](https://tree-sitter.github.io/) grammar for
[KSON](https://kson.org) (Kson Structured Object Notation), authored as part
of the [zed-kson](https://github.com/elviskahoro/zed-kson) Zed extension
(work item WI-2 of the kson-zed-extension build).

KSON has no upstream Tree-sitter grammar — kson-org/kson ships only a
TextMate grammar for VS Code — so this grammar is the editor-side syntax
foundation for KSON in Zed. It is authored against the reference
recursive-descent grammar and lexer in kson-org/kson at pinned revision
`928961c0669166b3852c5f67019ff0ceb57b7599`
(`src/commonMain/kotlin/org/kson/parser/Parser.kt` and `Lexer.kt`); see the
header comment of [`grammar.js`](./grammar.js) for the grammar-to-parser
mapping and the deliberate, documented divergences.

This grammar is the syntax layer only: parse-level leniency (optional
commas, number-like keys, unclosed strings) intentionally mirrors the
reference parser's recovery so that incomplete edits keep parsing. Validation
and diagnostics come from the KSON language server, not from this grammar.

## Milestones

- **M1 — JSON-compatible core** (done): every RFC 8259 JSON document parses
  correctly; delimited `{...}` objects, `[...]` bracket lists, quoted
  strings with escapes, JSON numbers, `true`/`false`/`null`, optional and
  redundant commas. Corpus: `test/corpus/json.txt`.
- **M2 — full KSON** (done): plain (un-delimited) objects and dash lists
  with greedy absorption and `.`/`=` end markers, `<...>` delimited dash
  lists, `#` comments, unquoted and single-quoted strings, `%`/`$` embed
  blocks. Corpus: `test/corpus/kson.txt`.
- Queries for Zed (`highlights.scm`, `indents.scm`, `injections.scm`) live
  in the extension repo under `languages/kson/`.

## Development

Requires the [tree-sitter CLI](https://tree-sitter.github.io/tree-sitter/creating-parsers/)
(0.25+):

```sh
tree-sitter generate   # regenerate src/parser.c from grammar.js
tree-sitter test       # run the corpus in test/corpus/
```

The generated parser (`src/parser.c`, `src/grammar.json`,
`src/node-types.json`) is committed, which is what Zed extension packaging
consumes.

## License

Apache-2.0, matching the zed-kson extension.
