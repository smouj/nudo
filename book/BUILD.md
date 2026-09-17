# Build guide

The canonical source is Markdown under `src/`.

## Supported production stack

- mdBook 0.5.x for the web book.
- Typst 0.15.x for the production PDF.
- Pandoc for the Markdown-to-Typst preparation stage.
- Python 3.11+ for validation and deterministic helper scripts.

No font files are vendored. The theme uses system-font fallbacks.

## Install

### mdBook

```bash
cargo install mdbook --no-default-features --features search --vers "^0.5.4" --locked
```

### Typst

Install the Typst CLI from the official distribution or your package manager,
then verify:

```bash
typst --version
```

### Other tools

```bash
pandoc --version
python --version
```

Optional editorial tools: Vale, markdownlint-cli2 and lychee.

## Build web

```bash
cd book
mdbook build
```

The output is written to `output/html/`.

Development server:

```bash
mdbook serve --open
```

## Build PDF

```bash
python scripts/prepare_typst.py
typst compile typst/main.typ output/pdf/nudo-technical-book.pdf
```

`prepare_typst.py` reads `src/SUMMARY.md` and converts chapters in that exact
order. The Markdown therefore remains the only editorial source.

## Validate

```bash
python scripts/check_book.py
```

Optional complete quality gate:

```bash
bash scripts/check_book.sh
```

## Brand assets

When this directory lives inside the NUDO repository:

```bash
python scripts/sync_brand_assets.py --repo-root ..
```

The book will use the official logo files in `assets/logo/`. If those files are
not available, the templates deliberately use the plain text name `NUDO` rather
than constructing an unofficial logo.
