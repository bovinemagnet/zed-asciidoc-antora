; The injected language is looked up by its declared name, so this must match `name` in
; languages/asciidoc-inline/config.toml exactly (case aside); an underscored or
; abbreviated spelling resolves to no language at all and the injection is silently dropped.
((block_macro
  (target) @injection.content)
  (#set! injection.language "AsciiDoc Inline"))

((table_cell
  (table_cell_content) @injection.content)
  (#set! injection.language "AsciiDoc Inline"))

((paragraph) @injection.content
  (#set! injection.include-children)
  (#set! injection.language "AsciiDoc Inline"))

; Lines outside paragraphs hold inline AsciiDoc too. Paragraph lines are left out, as the
; paragraph above already covers them.
([
  (admonition (line) @injection.content)
  (block_title (line) @injection.content)
  (callout_list_item (line) @injection.content)
  (checked_list_item (line) @injection.content)
  (description_list_item (line) @injection.content)
  (document_attr (line) @injection.content)
  (document_title (line) @injection.content)
  (ordered_list_item (line) @injection.content)
  (quoted_block (line) @injection.content)
  (quoted_md_block (line) @injection.content)
  (title1 (line) @injection.content)
  (title2 (line) @injection.content)
  (title3 (line) @injection.content)
  (title4 (line) @injection.content)
  (title5 (line) @injection.content)
  (unordered_list_item (line) @injection.content)
]
  (#set! injection.include-children)
  (#set! injection.language "AsciiDoc Inline"))

((section_block
  (element_attr
    (positional_attr
      (block_style))
    (positional_attr) @injection.language)
  (listing_block
    (listing_block_body) @injection.content)))
