/**
 * @file Tree-sitter grammar for KSON (https://kson.org), authored as part of
 * the zed-kson extension (WI-2). This grammar is the editor-side syntax
 * layer only: validation, diagnostics and formatting come from the KSON
 * language server, not from this grammar.
 *
 * The grammar mirrors the reference recursive-descent grammar documented in
 * kson-org/kson at pinned revision
 * 928961c0669166b3852c5f67019ff0ceb57b7599
 * (src/commonMain/kotlin/org/kson/parser/Parser.kt):
 *
 *   root -> ksonValue <EOF>
 *   ksonValue -> dashList | plainObject | delimitedValue
 *   dashList -> ( LIST_DASH ksonValue )+ "="?
 *   plainObject -> ( keyword ksonValue )+ "."?
 *   delimitedValue -> delimitedObject | delimitedDashList | bracketList
 *                   | literal | embedBlock
 *   delimitedDashList -> "<" ( LIST_DASH ksonValue )* ">"
 *   delimitedObject -> "{" objectInternals "}"   (commas optional)
 *   bracketList -> "[" listInternals "]"          (commas optional)
 *   keyword -> string ":"
 *
 * Plain (un-delimited) structures are greedy: per the KSON spec they absorb
 * every element or property they can, so the innermost structure always wins.
 * This grammar encodes that deterministically with shift-favoring
 * precedences: `property`, `list_element`, `object_end` and `list_end`
 * carry precedence 1, beating the precedence-0 `plain_object`/`dash_list`
 * reductions they compete with. The result matches the reference parser's
 * greedy-innermost parse (including its treatment of deceptively indented
 * documents, which the language server flags as validation errors).
 *
 * Known deliberate divergences from the reference *lexer*, chosen for editor
 * robustness (all affect invalid input only; the server reports the errors):
 *   - `-x` lexes as a list dash followed by an unquoted string, where the
 *     reference lexer scans a single (invalid) NUMBER token.
 *   - A bare `-` with nothing after it lexes as a list dash (dangling dash),
 *     matching the reference parser's DANGLING_LIST_DASH recovery.
 *   - Invalid number candidates (e.g. `5x`, `1.2.3`) lex generously as one
 *     NUMBER-shaped token, exactly like the reference lexer, instead of
 *     being rejected at the syntax level.
 *   - Multiple consecutive root values parse without an ERROR node (the
 *     reference parser reports unexpected trailing content); the language
 *     server reports those documents.
 *   - Unclosed strings and embed blocks keep their node structure (the
 *     reference lexer also scans them to the next delimiter or EOF).
 */

module.exports = grammar({
  name: 'kson',

  extras: $ => [
    /[\s\uFEFF\u2060\u200B]/,
    $.comment,
  ],

  rules: {
    // `repeat` (like tree-sitter-json) rather than `optional`: a root state
    // with no shiftable tokens lexes trailing whitespace in error mode,
    // producing spurious ERROR nodes on ordinary files ending in a newline.
    // Accepted, documented divergence: multiple consecutive root values
    // parse without an ERROR node; the server reports those documents.
    document: $ => repeat($._value),

    _value: $ => choice(
      $.plain_object,
      $.dash_list,
      $._delimited_value,
    ),

    _delimited_value: $ => choice(
      $.delimited_object,
      $.delimited_dash_list,
      $.bracket_list,
      $.double_quoted_string,
      $.single_quoted_string,
      $.unquoted_string,
      $.number,
      $.true,
      $.false,
      $.null,
      $.embed_block,
    ),

    // plainObject -> ( keyword ksonValue )+ "."?
    // Greedy: absorbs every property it can (see header comment).
    plain_object: $ => prec.right(seq(
      repeat1($.property),
      optional($.object_end),
    )),

    property: $ => prec(1, seq(
      field('key', $._key),
      ':',
      field('value', $._value),
    )),

    // A key is a string; numbers and reserved words are accepted as keys the
    // same way the reference parser accepts them (with OBJECT_KEY_* errors
    // reported by the server, not the grammar).
    _key: $ => choice(
      $.double_quoted_string,
      $.single_quoted_string,
      $.unquoted_string,
      $.number,
      $.true,
      $.false,
      $.null,
    ),

    // dashList -> ( LIST_DASH ksonValue )+ "="?
    // Greedy: absorbs every `- ` element it can (see header comment).
    dash_list: $ => prec.right(seq(
      repeat1($.list_element),
      optional($.list_end),
    )),

    list_element: $ => prec(1, seq(
      $.list_dash,
      $._value,
    )),

    // A `-` not followed by a digit is a list dash; `-5` wins as a number by
    // longest-match, which is how the reference lexer distinguishes them.
    list_dash: $ => '-',

    object_end: $ => prec(1, '.'),

    list_end: $ => prec(1, '='),

    // delimitedObject -> "{" objectInternals "}"
    // Commas between properties are optional (JSON compatibility); redundant
    // commas parse leniently and are flagged by the server.
    delimited_object: $ => seq(
      '{',
      repeat(choice($.property, ',')),
      '}',
    ),

    // delimitedDashList -> "<" dashListInternals ">"
    delimited_dash_list: $ => seq(
      '<',
      repeat($.list_element),
      '>',
    ),

    // bracketList -> "[" listInternals "]"
    // Commas between elements are optional (JSON compatibility).
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

    single_quoted_string: $ => seq(
      "'",
      repeat(choice($.escape_sequence, $._single_quoted_content)),
      optional(token.immediate("'")),
    ),

    _single_quoted_content: $ => token.immediate(/[^'\\]+/),

    // Like the reference lexer, a backslash escapes whatever follows it,
    // including a raw newline.
    escape_sequence: $ => token.immediate(/\\[\s\S]/),

    // 'Simple' strings: letters from any alphabet, digits, underscores and
    // dashes, not starting with a digit or dash. Combining marks (Mn, Mc)
    // are body-only characters, following UAX #31 like the reference lexer.
    unquoted_string: $ => /[\p{L}_][\p{L}\p{N}_\-\p{Mn}\p{Mc}]*/,

    // Numbers follow JSON's grammar; like the reference lexer, candidates
    // are scanned generously (letters, `_`, `+`, `-`, `.` may follow the
    // leading digit) so that invalid numbers stay one token for the server
    // to report, and half-typed numbers keep highlighting while editing.
    number: $ => /-?[0-9][\p{L}\p{N}_\p{Mn}\p{Mc}+\-\.]*/,

    true: $ => 'true',
    false: $ => 'false',
    null: $ => 'null',

    comment: $ => /#[^\r\n]*/,

    // embedBlock -> EMBED_OPEN_DELIM EMBED_TAG? EMBED_PREAMBLE_NEWLINE
    //               EMBED_CONTENT? EMBED_CLOSE_DELIM
    // Content always ends at the first raw occurrence of the end delimiter;
    // `%\%`-style escapes contain a backslash and so never terminate the
    // content. The alternative `$` delimiter works identically.
    embed_block: $ => choice(
      seq(
        '%',
        optional($.embed_tag),
        $._embed_preamble_newline,
        optional(alias($._percent_embed_content, $.embed_content)),
        optional(token.immediate('%%')),
      ),
      seq(
        '$',
        optional($.embed_tag),
        $._embed_preamble_newline,
        optional(alias($._dollar_embed_content, $.embed_content)),
        optional(token.immediate('$$')),
      ),
    ),

    // The tag is the whole rest of the opening line: quotes, punctuation and
    // even a `%%` are ordinary tag characters; only a raw newline ends it.
    embed_tag: $ => token.immediate(/[^\r\n]+/),

    _embed_preamble_newline: $ => token.immediate(/\r?\n/),

    _percent_embed_content: $ => token.immediate(/([^%]|%[^%])+/),

    _dollar_embed_content: $ => token.immediate(/([^$]|\$[^$])+/),
  },
});
