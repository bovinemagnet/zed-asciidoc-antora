# Repository Guidelines

## Project Structure & Module Organization

This repository is the Zed extension for AsciiDoc. The language server `adoc-ls` and its semantic crates live in AdocFoundry (https://github.com/bovinemagnet/AdocFoundry); language-server changes belong there.

- `docs/prd/` contains the product requirements and initial implementation specification.
- `src/lib.rs` is the whole extension crate: it starts `adoc-ls --stdio` from `PATH`, or downloads `adoc-ls` from the latest AdocFoundry release when it is not on `PATH`.
- `languages/` and `extension.toml` define Zed language metadata and Tree-sitter queries.
- `snippets/` contains editor snippets.

Keep the Zed extension thin; semantic behaviour belongs in AdocFoundry's crates.

## Build, Test, and Development Commands

- `cargo fmt --check` to enforce Rust formatting.
- `cargo clippy --all-targets -- -D warnings` to enforce lint-clean code.
- `cargo test` to run the extension's unit tests.
- `cargo check --target wasm32-wasip2` to verify the Zed Wasm target.

## Coding Style & Naming Conventions

Use Markdown headings and concise, actionable prose in documentation. Keep examples in fenced code blocks when they represent commands, paths, or AsciiDoc snippets.

For Rust code, follow standard `rustfmt` output. Use kebab-case crate names (`adoc-ls`, `adoc-core`) and snake_case modules/functions. Keep LSP transport types out of core domain crates, and avoid adding Node.js tooling for language-server implementation.

## Testing Guidelines

Add unit tests next to the extension code in `src/lib.rs`. Parser, resolver, renderer and LSP behaviour, with its fixtures, is tested in AdocFoundry.

Do not rely on absolute developer-machine paths or external executables unless the test explicitly verifies missing-executable behavior.

## Commit & Pull Request Guidelines

Current history uses short initial commits, so no detailed convention is established yet. Use concise imperative commit messages, for example `Add initial parser fixtures`.

Pull requests should include a summary, linked issue or PRD section where relevant, tests run, and screenshots only for Zed UI or rendering changes. Keep PRs incremental; each step should leave the workspace compiling once code exists.
