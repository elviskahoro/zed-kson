; Auto-indentation for KSON in Zed.
; https://zed.dev/docs/extensions/languages#auto-indentation

; Delimited structures: indent the contents, outdent the closing delimiter.
(delimited_object "}" @end) @indent
(bracket_list "]" @end) @indent
(delimited_dash_list ">" @end) @indent

; Plain (un-delimited) structures: a property or list element whose value
; starts on a later line indents that value. The `.` end-dot and `=` end-dash
; close the innermost structure, so lines following them align outward again
; without an explicit outdent rule. This mirrors the VS Code reference
; onEnterRules (indent on a key without a value, outdent after an end
; marker) from kson-org/kson's language-configuration.json at the pin.
(property) @indent
(list_element) @indent
