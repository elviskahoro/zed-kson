; Syntax highlighting for KSON in Zed.
; Capture names follow Zed's supported highlight classes:
; https://zed.dev/docs/extensions/languages#syntax-highlighting
;
; Highlighting choices track the pinned VS Code TextMate grammar
; (kson-org/kson tooling/lsp-clients/shared/extension/config/kson.tmLanguage.json
; at 928961c0669166b3852c5f67019ff0ceb57b7599) with two deliberate
; improvements: unquoted strings are highlighted as strings (they are
; strings; TextMate leaves them unstyled), and object keys are
; highlighted as @property like Zed's built-in YAML does.

(comment) @comment

[
  (double_quoted_string)
  (single_quoted_string)
  (unquoted_string)
] @string

(escape_sequence) @string.escape

(number) @number

[
  (true)
  (false)
  (null)
] @constant.builtin

(embed_tag) @tag

[
  ":"
  ","
] @punctuation.delimiter

[
  "{"
  "}"
  "["
  "]"
  "<"
  ">"
] @punctuation.bracket

(list_dash) @punctuation.list_marker

[
  (object_end)
  (list_end)
  "%"
  "$"
  "%%"
  "$$"
] @punctuation.special

(property
  key: [
    (double_quoted_string)
    (single_quoted_string)
    (unquoted_string)
  ] @property)
