# The NUDO Technical Book

The explanatory manual for NUDO: what the language is for, why it is shaped the
way it is, how the compiler and runtime are put together, and what has been
rejected along the way.

**Read it at <https://smouj.github.io/nudo/>.**

If you want the rules rather than the reasoning, read [`../spec`](../spec/README.md).

## This book is not the specification

The book explains the language. It does not define it. The authority order is
stated in [`../spec/README.md`](../spec/README.md) and is not negotiable:

| Rank | Artefact |
| ---- | -------- |
| 1 | [`../spec/`](../spec/README.md) — normative language rules |
| 2 | accepted [`../neps/`](../neps/README.md) — language evolution |
| 3 | [`../tests/conformance`](../tests/conformance/README.md) — executable agreement |
| 4 | the implementation |
| 5 | **this book** — explanation, rationale and learning material |

If the book disagrees with the specification, **the book is wrong**. Fix the
book.

## Status labels are mandatory

A polished manual about a pre-alpha language can mislead readers. Every chapter
therefore states whether it describes implemented behavior, accepted
specification, a proposal, planned work, an open question or historical context.
The vocabulary is defined in [EDITORIAL.md](EDITORIAL.md).

The English status rank stays uppercase in every translation so its meaning
remains machine-checkable. Translations may localize the label, but preserve the
rank token in parentheses; see [i18n/README.md](i18n/README.md).

[The current project status](src/00-introduction/status.md) reflects the
toolchain today: the lexer, parser, typed AST, HIR and name resolution are
implemented. The CLI invokes the type-checking crate for its partial set of
accepted rules. The interpreter, effect checker, agent runtime and WebAssembly
backend are not available.
## Layout

| Path | Contents |
| ---- | -------- |
| <code>src/</code> | Canonical English Markdown source; <code>SUMMARY.md</code> is the table of contents |
| <code>i18n/</code> | Spanish, Chinese, Japanese and German editions with per-chapter source hashes |
| <code>theme/</code> | The web visual system: tokens, book CSS, print CSS and a small script |
| <code>typst/</code> | The PDF typesetting layer |
| <code>adr/</code> | Architecture decision records for the book system |
| <code>templates/</code> | Chapter and ADR templates |
| <code>quality/</code> | Editor configuration: markdownlint, Vale and lychee |
| <code>scripts/</code> | Build, validation and brand-sync helpers |
| <code>output/</code> | Generated files; never committed |
## Building

```sh
python3 book/scripts/sync_brand_assets.py --repo-root .
mdbook build book          # → book/output/html
mdbook serve book --open   # local preview
```

The PDF needs Typst and Pandoc:

```sh
python3 book/scripts/prepare_typst.py
typst compile book/typst/main.typ book/output/pdf/nudo-technical-book.pdf
```

Validate the source:

```sh
python3 book/scripts/check_book.py
```

## Publishing

[`.github/workflows/book.yml`](../.github/workflows/book.yml) builds the book and
deploys it to GitHub Pages on every push to `main` that touches `book/`.

* The site is generated; `book/output/` is in `.gitignore` and is never committed.
* The brand sync runs **in the workflow**, not in the repository. That keeps
  exactly one copy of each logo in [`../assets/logo`](../assets/logo) and means no
  binary is ever duplicated.
* If the official assets are unavailable, the templates fall back to the plain
  text `NUDO`. Nothing is ever redrawn, recoloured or reconstructed — see
  [`../assets/brand/BRAND.md`](../assets/brand/BRAND.md).

## Adding a chapter

1. Write the page in `src/<section>/<slug>.md` from
   [`templates/chapter-template.md`](templates/chapter-template.md).
2. Add it to [`src/SUMMARY.md`](src/SUMMARY.md) — an orphan page fails the
   validator, and `mdbook` with `create-missing = false` will not build one.
3. Give it a status line.
4. Run `python3 book/scripts/check_book.py`.

Numbering, the beige/oxide/graphite palette and the chapter furniture belong to
the editorial system, not to the brand: they are recorded in
[`theme/DESIGN-SYSTEM.md`](theme/DESIGN-SYSTEM.md) and in
[`adr/`](adr/README.md). The brand itself is `#000000` and `#3F85FF` and is
governed by [`../assets/brand/BRAND.md`](../assets/brand/BRAND.md).
