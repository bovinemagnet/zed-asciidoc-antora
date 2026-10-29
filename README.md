# AsciiDoc and Antora for Zed

AsciiDoc and Antora for Zed (`asciidoc-antora`) is a Zed extension for first-class AsciiDoc, Asciidoctor, and Antora authoring.

This repository is the Zed extension: language registration, Tree-sitter queries, snippets and the
glue that starts the `adoc-ls` language server. The language server and the crates behind it
(`adoc-core`, `adoc-parser`, `adoc-index`, `adoc-antora`, `adoc-lang`, `adoc-render`, `adoc-ls`) now
live in [AdocFoundry](https://github.com/bovinemagnet/AdocFoundry), where the same code also powers the
AdocFoundry desktop editor, so improvements made there reach Zed users too. The feature list below
describes what the extension delivers together with `adoc-ls`.

Zed's extension API has no webview, so an extension cannot show a rendered preview beside the
editor. That preview pane is why AdocFoundry exists: its desktop editor renders the page natively
beside the source as you type, with scroll sync both ways and no browser, Asciidoctor, Node.js or
Java. The `AdocFoundry: preview page` task opens the current page there.

## Current Status

Implemented:

- Zed language registration for `.adoc`, `.asciidoc`, and `.ad`.
- Pinned block and inline Tree-sitter grammars with highlighting, outline, text-object and injection queries.
  The grammars come from a fork, maintained on its `master` branch since upstream was archived, which
  carries three changes upstream never took: a `table_header_row` node, so a table's header row can be
  styled apart from its body; and table cells written as `| text` that can carry block content, so an
  admonition or listing belonging to a cell no longer breaks the table around it, and the cell's text
  resumes after the block; and consecutive prose lines parsed as one paragraph, so `*bold*` or
  `_italic_` broken over two lines of a paragraph is still recognised. It also has the fixes that went upstream before the archive,
  among them document attribute values that accept globs (`src/**/*.kt`) and empty values (`:name: `), and
  constrained spans that keep word-internal delimiters, so `_"COMP1010_TUT01"_` is one italic run
  rather than several.
- Source blocks highlighted in their declared language (`[source,rust]`), even when a comment sits
  between the attribute line and the listing, `latexmath:[…]` as LaTeX
  and `pass:[…]` as HTML, when Zed has those languages. `stem:[…]` is left alone, since its notation
  follows the document's `:stem:` attribute and defaults to AsciiMath.
- Vim-mode text objects: `ac` / `ic` select a section (inside is everything below its title),
  `af` / `if` a delimited block (inside is everything between its delimiter lines), and `gc` a
  comment.
- Identifiers such as `my-section-id` and `{my-attr}` count as one word for word movement and
  double-click selection.
- Typing `*` or `_` over a selection wraps it as bold or italic, and typing it again wraps the
  same text once more, giving `**bold**` or `__italic__` for use inside a word. Typed with
  nothing selected, each inserts a single character. Zed does not let an extension bind keys,
  so for Ctrl+B and Ctrl+I add this to your keymap (`zed: open keymap`); in AsciiDoc files
  it replaces Zed's own use of those keys:

  ```json
  {
    "context": "Editor && (extension == adoc || extension == asciidoc || extension == ad)",
    "bindings": {
      "ctrl-b": ["workspace::SendKeystrokes", "*"],
      "ctrl-i": ["workspace::SendKeystrokes", "_"]
    }
  }
  ```
- `adoc-ls` stdio transport with incremental synchronization, document symbols, diagnostics, and Go to Definition.
- Workspace indexing and conservative diagnostics for missing local xrefs, anchors, includes,
  and block images. An image is judged only in a file nothing includes, since Asciidoctor
  resolves it against the top-level document; inside an Antora module it is resolved as an
  `image$` resource of the page's module, and partials are not judged on their own.
  Opening, editing or closing any document refreshes the diagnostics of every open document,
  so a change in one file updates the warnings it causes or clears in another.
- Reference resolution that follows Asciidoctor and Antora semantics: module-root-relative page IDs,
  implicit and natural section references, anchors declared in included partials, and bibliography anchors.
- Antora descriptor parsing plus deterministic component, module, and resource-family discovery.
- Same-component Antora xref/include navigation and unknown module/resource diagnostics; components
  absent from the workspace are assumed to come from elsewhere in the playbook and are not reported.
  A target written as a resource ID that does not parse is reported as an invalid family
  (`partials$x.adoc`) or an invalid coordinate (too many, empty, or a malformed version); path
  errors are not, since Antora accepts relative forms the parser rejects. A file inside a
  component whose `antora.yml` cannot be read, or lacks `name`, is warned that Antora features
  are off for it; descriptors are read at start-up, so a fix shows after a server restart.
- Cross-component and cross-version Antora resolution following Antora 3.2: a target naming a
  component but no version resolves to that component's latest version (the versionless `~`
  version first, then released versions above `prerelease` ones), `version@component:module:page`
  selects exactly, and a component without a module means its `ROOT` module. A version of a known
  component that is not in the workspace is assumed to be published elsewhere and not reported.
  Pages listed in another page's `:page-aliases:` resolve to that page, with an information
  diagnostic suggesting the current id.
- Completion for `xref:` targets, `include::` targets, `image:` targets, and anchors.
  Inside an Antora module the current module's pages are offered as bare IDs and other
  modules' pages module-qualified, `include::` offers the family prefixes and then that
  family's resources, and anchors come from the file a target names. In a plain workspace
  the same constructs complete relative paths, including non-AsciiDoc include targets.
- Completion for attribute references after `{`, including inside macro targets such as
  `include::{partialsdir}`. It offers the document's own declarations, those of every file it
  includes, the component's `antora.yml` attributes, Antora's page and family-directory
  attributes, and Asciidoctor's built-ins, in that order of precedence, with each value shown
  alongside. Accepting a name supplies its closing brace, replacing one already present, and
  attributes the document unsets are left out.
- Hover over an xref, `<<anchor>>`, `include::` or `image::` to see the target's title, the
  section an anchor leads to, and where the target lives, as an Antora resource ID inside a
  component or a relative path elsewhere. Hover over an `{attribute}` to see its value and
  whether it comes from the document, an included file, `antora.yml`, Antora, or Asciidoctor.
  Targets resolve exactly as Go to Definition resolves them; unresolved ones show nothing.
- Workspace symbols: Zed's project symbol search finds document titles, section headings,
  explicit anchors and attribute declarations across the workspace by the start of any word
  in their names, so `auth` and `flow` both find "Authentication Flow".
- Antora navigation files (a module's `nav.adoc`, or a file `antora.yml` lists under `nav`)
  get page completion, Go to Definition and missing-page warnings like any page, and their
  outline is the navigation tree: `.Title` lines and `*`, `**`, … items nested by level, each
  linking item showing its `xref:` target. Ordinary documents' outlines nest sections by level.
  Zed builds outlines from Tree-sitter unless told otherwise, so to see these, set:

  ```json
  { "languages": { "AsciiDoc": { "document_symbols": "on" } } }
  ```
- Zed language-server registration that runs an `adoc-ls` found on `PATH`, or otherwise downloads
  `adoc-ls` from the latest [AdocFoundry release](https://github.com/bovinemagnet/AdocFoundry/releases)
  for Linux, macOS and Windows (x86-64 and AArch64) and keeps it up to date. When the release cannot be
  fetched, for example offline, it starts the most recent earlier download instead.
- HTML preview via two code actions. `AsciiDoc: render preview` renders once;
  `AsciiDoc: render live preview` additionally re-renders whenever the document is saved
  and embeds a reloader so the page keeps up; while a document is being followed that
  action becomes `AsciiDoc: stop live preview`, and closing the buffer stops it too. Both render the
  open buffer (saved or not), with Asciidoctor when `asciidoctor` is on `PATH` and otherwise with
  AdocFoundry's own renderer, which needs no external tool and draws diagrams and typeset STEM.
  Either way the preview
  merges the component's `antora.yml` `asciidoc.attributes` (where `true` sets an
  attribute, `false` and `~` unset it, and a trailing `@` lets the page override it)
  and Antora page and family-directory attributes (`moduledir`, `pagesdir`,
  `partialsdir`, `examplesdir`, `attachmentsdir`, `imagesdir`), rewrites
  family-qualified includes such as
  `partial$note.adoc` to absolute paths,
  and opens the result in the default browser. With Asciidoctor, includes nested inside an
  included file are rewritten too, via copies in a scratch directory; reading those
  copies is why preview renders with Asciidoctor's `unsafe` safe mode. Cyclic includes
  stop at the repeat rather than recursing.
- Find References: on a page, the pages that link to it and the includes that include it; on an
  anchor or a reference to one, every reference to that anchor.
- Rename: renaming an anchor id renames it and its references, and renaming an `.adoc` file in
  Zed's project panel first rewrites the xrefs and navigation entries that name it.
- Tasks (`task: spawn`) for AsciiDoc files, each needing its tool on `PATH`:
  - `AdocFoundry: preview page` opens the page in the AdocFoundry desktop editor (`adocfoundry`).
  - `AdocFoundry: check workspace` prints every diagnostic in the worktree as
    `path:line:col: severity code message`, failing on warnings (`adocfoundry-cli check`).
  - `AdocFoundry: render page to HTML` writes `<page>.html` beside the page (`adocfoundry-cli render`).
  - `AdocFoundry: export page to PDF` writes `<page>.pdf` beside the page (`adocfoundry-cli pdf`).
  - `Asciidoctor: render page to HTML` runs `asciidoctor` on the page.
  - `Antora: build site` runs `npx antora antora-playbook.yml` in the worktree root.

  `adocfoundry` and `adocfoundry-cli` come with every AdocFoundry release package, and
  `adocfoundry-cli` is also published as an archive of its own. The workspace check and PDF export
  are server commands that `adoc-ls` does not offer as code actions, so in Zed they are reached
  through these tasks.
- `adoc-ls` settings come from AdocFoundry's settings files, the user settings file and the
  workspace's `.adocfoundry.toml`, not from Zed. Zed's `lsp.adoc-ls.binary` setting chooses the
  binary, its arguments (`--stdio` unless given) and extra environment variables:

  ```json
  { "lsp": { "adoc-ls": { "binary": { "path": "/path/to/adoc-ls", "arguments": ["--stdio"] } } } }
  ```

Not implemented yet:

- Diagnostics inside `antora.yml` itself, and re-reading descriptors when they change.
- A preview pane inside Zed. The extension API exposes no webview or preview capability,
  so preview opens in the browser, or beside the source in AdocFoundry through the
  `AdocFoundry: preview page` task; see
  `docs/prd/DESIGN-Preview_Pipeline_and_Native_Rendering.md`.

## Development

Run the baseline checks before completing a change:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo check --target wasm32-wasip2
```

The extension downloads `adoc-ls` from the latest AdocFoundry release the first time it is needed. To run a local build of the language server instead, install it from AdocFoundry so that it is on `PATH`, where it takes precedence over the download:

```sh
cargo install --git https://github.com/bovinemagnet/AdocFoundry adoc-ls
```

The same command is needed on platforms without a prebuilt release, such as 32-bit x86.

Changes to the language server, its diagnostics, completion or Antora resolution belong in AdocFoundry.

Zed dev extension builds require Rust installed through `rustup` with the `wasm32-wasip2` target. Once loaded, the extension starts `adoc-ls --stdio` for AsciiDoc buffers.

Use `docs/prd/PRD-AsciiDoc_for_Zed_Initial_Implementation_Specification.md` as the implementation source of truth.
