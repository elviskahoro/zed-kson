; Code injections for KSON embed blocks in Zed.
; https://zed.dev/docs/extensions/languages#code-injections
;
; An embed block's tag (the rest of the opening line, e.g. `%sql ...`)
; names the embedded language. The tag list mirrors the pinned VS Code
; TextMate grammar's generated embed list (kson-org/kson
; tooling/lsp-clients/vscode, generate-tm-embed-block.ts at
; 928961c0669166b3852c5f67019ff0ceb57b7599). Only the first word of the
; tag selects the language, so tags like `sql "server=..."` still inject.
;
; Zed resolves language names case-insensitively; languages provided by
; extensions (sql, html, java, ruby, xml, kotlin) inject only when their
; extension is installed, and are skipped gracefully otherwise. Unknown
; tags (the generic case) get no injection and render as plain text.

((embed_block
   (embed_tag) @_tag
   (embed_content) @injection.content)
 (#match? @_tag "^\\s*(typescript|ts)(\\s|$)")
 (#set! injection.language "typescript"))

((embed_block
   (embed_tag) @_tag
   (embed_content) @injection.content)
 (#match? @_tag "^\\s*(javascript|js)(\\s|$)")
 (#set! injection.language "javascript"))

((embed_block
   (embed_tag) @_tag
   (embed_content) @injection.content)
 (#match? @_tag "^\\s*(python|py)(\\s|$)")
 (#set! injection.language "python"))

((embed_block
   (embed_tag) @_tag
   (embed_content) @injection.content)
 (#match? @_tag "^\\s*(json)(\\s|$)")
 (#set! injection.language "json"))

((embed_block
   (embed_tag) @_tag
   (embed_content) @injection.content)
 (#match? @_tag "^\\s*(sql)(\\s|$)")
 (#set! injection.language "sql"))

((embed_block
   (embed_tag) @_tag
   (embed_content) @injection.content)
 (#match? @_tag "^\\s*(html)(\\s|$)")
 (#set! injection.language "html"))

((embed_block
   (embed_tag) @_tag
   (embed_content) @injection.content)
 (#match? @_tag "^\\s*(css)(\\s|$)")
 (#set! injection.language "css"))

((embed_block
   (embed_tag) @_tag
   (embed_content) @injection.content)
 (#match? @_tag "^\\s*(java)(\\s|$)")
 (#set! injection.language "java"))

((embed_block
   (embed_tag) @_tag
   (embed_content) @injection.content)
 (#match? @_tag "^\\s*(cpp|c\\+\\+|cxx)(\\s|$)")
 (#set! injection.language "cpp"))

((embed_block
   (embed_tag) @_tag
   (embed_content) @injection.content)
 (#match? @_tag "^\\s*(csharp|cs|c#)(\\s|$)")
 (#set! injection.language "c#"))

((embed_block
   (embed_tag) @_tag
   (embed_content) @injection.content)
 (#match? @_tag "^\\s*(go)(\\s|$)")
 (#set! injection.language "go"))

((embed_block
   (embed_tag) @_tag
   (embed_content) @injection.content)
 (#match? @_tag "^\\s*(rust|rs)(\\s|$)")
 (#set! injection.language "rust"))

((embed_block
   (embed_tag) @_tag
   (embed_content) @injection.content)
 (#match? @_tag "^\\s*(ruby|rb)(\\s|$)")
 (#set! injection.language "ruby"))

((embed_block
   (embed_tag) @_tag
   (embed_content) @injection.content)
 (#match? @_tag "^\\s*(yaml|yml)(\\s|$)")
 (#set! injection.language "yaml"))

((embed_block
   (embed_tag) @_tag
   (embed_content) @injection.content)
 (#match? @_tag "^\\s*(xml)(\\s|$)")
 (#set! injection.language "xml"))

((embed_block
   (embed_tag) @_tag
   (embed_content) @injection.content)
 (#match? @_tag "^\\s*(markdown|md)(\\s|$)")
 (#set! injection.language "markdown"))

((embed_block
   (embed_tag) @_tag
   (embed_content) @injection.content)
 (#match? @_tag "^\\s*(shell|sh|bash|zsh)(\\s|$)")
 (#set! injection.language "bash"))

((embed_block
   (embed_tag) @_tag
   (embed_content) @injection.content)
 (#match? @_tag "^\\s*(kotlin)(\\s|$)")
 (#set! injection.language "kotlin"))
