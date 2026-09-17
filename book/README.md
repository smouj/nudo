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

A polished manual about a pre-alpha language is a machine for misleading people.
Every chapter therefore carries a status line, and a chapter that describes
proposed behaviour must say so in the chapter, not only in a global disclaimer.

The label vocabulary is the repository's:
**IMPLEMENTED**, **SPECIFIED**, **PLANNED**, **PROPOSED** — used alone or in
pairs, optionally with a short descriptor such as `Implemented foundation`. The
rank word stays uppercase English in every language so that it remains
machine-checkable; see [`i18n/README.md`](i18n/README.md).

[`src/00-introduction/status.md`](src/00-introduction/status.md) is the
chapter that states the real state of the toolchain, and it is the one chapter
that must never drift: it says the lexer and `nudo check` work, and that the
parser, type checker, interpreter, effect checker, agent runtime and WASM backend
are design work.

## Layout

| Path | Contents |
| ---- | -------- |
| `src/` | The canonical Markdown source. `SUMMARY.md` is the table of contents |
| `theme/` | The web visual system: tokens, book CSS, print CSS, one small script |
| `typst/` | The PDF typesetting layer |
| `adr/` | Architecture decision records for the book system itself |
| `templates/` | Chapter and ADR templates |
| `quality/` | Editor configuration: markdownlint, Vale, lychee |
| `scripts/` | Build, validation and brand-sync helpers |
| `translations/` | Where a translated edition will live, if one is made |
| `output/` | **Generated.** Never committed |

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
