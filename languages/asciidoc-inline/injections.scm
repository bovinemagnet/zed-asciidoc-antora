; `latexmath:[…]` holds LaTeX. `stem:[…]` is left alone: its notation follows the
; document's `:stem:` attribute and defaults to AsciiMath, which Zed has no language for.
((stem_macro
  (macro_name) @_name
  (attr) @injection.content)
  (#eq? @_name "latexmath")
  (#set! injection.language "LaTeX"))

; `pass:[…]` reaches the output untouched, which for the HTML backend is HTML.
((macro_passthrough
  (attr) @injection.content)
  (#set! injection.language "HTML"))
