# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

The Zed extension for AsciiDoc, Asciidoctor and Antora authoring. It registers the language, Tree-sitter grammars and queries, and snippets, and starts the `adoc-ls` language server. `README.md` tracks what is and is not implemented and must be updated when setup or visible behaviour changes. `AGENTS.md` holds the contribution guidelines.

`adoc-ls` and the crates behind it (`adoc-core`, `adoc-parser`, `adoc-index`, `adoc-antora`, `adoc-lang`, `adoc-render`) live in AdocFoundry (`https://github.com/bovinemagnet/AdocFoundry`, usually checked out beside this repository). Language-server behaviour — diagnostics, completion, hover, definition, Antora resolution, preview — is changed there, not here.

## Commands

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings   # CI treats warnings as errors
cargo test
cargo check --target wasm32-wasip2          # Zed Wasm target
```

Load the repository root as a Zed dev extension. The extension starts `adoc-ls --stdio`, taking it from `PATH` when present (`cargo install --git https://github.com/bovinemagnet/AdocFoundry adoc-ls`, or `cargo install --path crates/adoc-ls` from an AdocFoundry checkout) and otherwise downloading it from the latest AdocFoundry release. Dev-extension builds need rustup with the `wasm32-wasip2` target.

## Structure

- `src/lib.rs` — the whole extension crate (`asciidoc-zed-extension`, built for `wasm32-wasip2`): finds `adoc-ls` on `PATH`, otherwise downloads it from the latest AdocFoundry release, and returns the command Zed runs.
- `extension.toml` — registers the language, snippets (`snippets/asciidoc.json`), the `adoc-ls` language server, and the two Tree-sitter grammars pinned by revision.
- `languages/asciidoc/` and `languages/asciidoc-inline/` — language config plus `.scm` queries (highlights, outline, injections, brackets, text objects, overrides), and `languages/asciidoc/tasks.json` for the AdocFoundry, Asciidoctor and Antora tasks. Tree-sitter is for syntax, outline, folding, text objects and injections only — never for attribute evaluation, include expansion, Antora coordinates or cross-file references.

## Conventions that must be preserved

- Keep the extension thin; all semantics live in AdocFoundry's crates.
- All external dependencies are pinned exactly (`=0.7.0` style).
- `unsafe_code = "forbid"`.
- `crates/zed-asciidoc/` is an empty leftover directory; do not add code there.
