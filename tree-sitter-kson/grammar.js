/**
 * @file Tree-sitter grammar for KSON (https://kson.org), authored as part of
 * the zed-kson extension (WI-2). This grammar is the editor-side syntax
 * layer only: validation, diagnostics and formatting come from the KSON
 * language server, not from this grammar.
 *
 * The grammar mirrors the reference recursive-descent grammar documented in
 * kson-org/kson at pinned revision
 * 928961c0669166b3852c5f67019ff0ceb57b7599
 * (src/commonMain/kotlin/org/kson/parser/Parser.kt).
 *
 * MILESTONE 1: the JSON-compatible core. Every RFC 8259 JSON document is a
 * legal KSON document (KSON is a verified superset of JSON), so this
 * milestone lands the delimited object, bracket list, quoted strings,
 * numbers and literals first, corpus-driven. Plain (un-delimited) objects,
 * dash lists, embed blocks, comments and unquoted/single-quoted strings
 * follow in the next milestone.
 *
 * Commas are optional between properties and elements: the reference parser
 * accepts them anywhere between delimited internals and reports redundant
 * ones as validation errors, so the grammar stays lenient there.
 */

module.exports = grammar({
  name: 'kson',

  extras: $ => [
    /\s/,
  ],

  rules: {
    // `repeat` (like tree-sitter-json) rather than `optional`: a root state
    // with no shiftable tokens lexes trailing whitespace in error mode,
    // producing spurious ERROR nodes on ordinary files ending in a newline.
    // Side effect (accepted, documented): multiple consecutive root values
    // parse without an ERROR node; the server reports those documents.
    document: $ => repeat($._value),

    _value: $ => choice(
      $.delimited_object,
      $.bracket_list,
      $.double_quoted_string,
      $.number,
      $.true,
      $.false,
      $.null,
    ),

    // delimitedObject -> "{" objectInternals "}"
    delimited_object: $ => seq(
      '{',
      repeat(choice($.property, ',')),
      '}',
    ),

    property: $ => prec(1, seq(
      field('key', $._key),
      ':',
      field('value', $._value),
    )),

    _key: $ => choice(
      $.double_quoted_string,
      $.number,
      $.true,
      $.false,
      $.null,
    ),

    // bracketList -> "[" listInternals "]"
    bracket_list: $ => seq(
      '[',
      repeat(choice($._value, ',')),
      ']',
    ),

    double_quoted_string: $ => seq(
      '"',
      repeat(choice($.escape_sequence, $._double_quoted_content)),
      // The close quote is optional so an unclosed string still forms a
      // string node (content runs to the next quote or EOF, exactly like
      // the reference lexer) instead of collapsing into an ERROR blob.
      optional(token.immediate('"')),
    ),

    _double_quoted_content: $ => token.immediate(/[^"\\]+/),

    // Like the reference lexer, a backslash escapes whatever follows it,
    // including a raw newline.
    escape_sequence: $ => token.immediate(/\\[\s\S]/),

    // Numbers follow JSON's grammar; like the reference lexer, candidates
    // are scanned generously (letters, `_`, `+`, `-`, `.` may follow the
    // leading digit) so that invalid numbers stay one token for the server
    // to report, and half-typed numbers keep highlighting while editing.
    number: $ => /-?[0-9][\p{L}\p{N}_\p{Mn}\p{Mc}+\-\.]*/,

    true: $ => 'true',
    false: $ => 'false',
    null: $ => 'null',
  },
});
