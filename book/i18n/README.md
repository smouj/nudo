# Translations

The manual is published in five languages. English is canonical: every other
language is a translation of it, and a translation is never the place where a
fact is decided.

| Code | Language | Directory | Status |
| ---- | -------- | --------- | ------ |
| `en` | English | [`../src`](../src) — canonical | Source of truth |
| `es` | Español | [`es/`](es/) | See `es/STATUS.md` |
| `zh` | 简体中文 | [`zh/`](zh/) | See `zh/STATUS.md` |
| `ja` | 日本語 | [`ja/`](ja/) | See `ja/STATUS.md` |
| `de` | Deutsch | [`de/`](de/) | See `de/STATUS.md` |

## Why per-language source trees instead of gettext

mdBook has no built-in translation support. The two real options are gettext
catalogues (`mdbook-i18n-helpers`, the tool Rust's own books use) and one source
tree per language.

NUDO uses the second, for reasons that are about this repository rather than
about taste:

* **No toolchain in CI.** Gettext needs an extra Rust binary installed on every
  build. This repository's dependency policy is strict, and a translation
  pipeline that fails when a third-party tool cannot be downloaded is a pipeline
  that fails for a reason unrelated to the translation.
* **Reviewable as what they are.** A translation is Markdown, and it is reviewed
  as Markdown. A `.po` catalogue of 1 500 message fragments is harder to read
  than the chapter it contains.
* **Honest staleness.** Gettext tracks staleness automatically, which is its real
  advantage. That is recovered here by
  [`../scripts/check_translations.py`](../scripts/check_translations.py), which
  records the English source hash each chapter was translated from. A chapter
  whose English source has changed is reported as **behind**, not silently left
  in place.

The cost is duplication of structure across five trees. That cost is paid
knowingly, and the staleness check is what keeps it from becoming drift.

## The translation contract

Rules that apply to every language. They exist because a translated manual about
a pre-alpha language is an unusually effective way to mislead people.

1. **Never translate a status label into something stronger.** The four ranks are
   `IMPLEMENTED`, `SPECIFIED`, `PLANNED` and `PROPOSED`, and they mean exactly
   what [`../../AGENTS.md`](../../AGENTS.md) says they mean.

   A chapter's status line is `<rank>` or `<rank> / <rank>`, optionally followed by
   a short descriptor: `Implemented foundation`, `Proposed syntax`. The rank is
   **kept verbatim in uppercase English** so that it stays machine-checkable and
   cannot drift, and the descriptor is translated.

   The label itself — `Status:` — may be translated (`状态`, `ステータス`) or kept as
   the English field name; both are in use and neither changes the meaning. The
   value must carry the English rank token in parentheses after the translated
   wording, as in `> **Status:** Especificado (SPECIFIED)`, which is the part a
   check can act on. A translation that turns
   "planned" into "available" is a defect, not a nuance.

   There is no `Open` rank. "The design question this chapter names has not been
   settled" is written as `Proposed`, and a chapter whose question *has* been
   settled is updated rather than left carrying its old label — which is what
   happened to `02-language-design/expressions.md` once the expression grammar was
   frozen.
2. **Never translate a claim into a promise.** If the English says a feature is
   designed and not built, every language says that.
3. **Code, identifiers, file paths, `NDO` codes, type names and command names are
   not translated.** `nudo check`, `Generated<T>`, `NDO1003`, `spec/grammar.md`
   and `book/src` stay exactly as they are.
4. **Links point at the translated sibling chapter**, not at the English one. A
   reader who chose a language should stay in it.
5. **Never invent a feature to make a sentence flow.** If a translation needs a
   fact the source does not have, the source is what is missing. Say so.
6. **Diagrams are shared, not duplicated.** They are included from
   `../../diagrams` by relative path and contain almost no prose. Any label that
   must be translated is a signal that the diagram is doing too much talking.
7. **The canonical title stays recognisable.** "The NUDO Technical Book" may be
   rendered in the target language, but `NUDO` does not become anything else.

## Adding a language

1. Copy the structure: `mkdir -p <code>/src` and copy the `book.toml` from
   [`es/book.toml`](es/book.toml), changing `language` and the title.
2. Translate `SUMMARY.md` first: it is the navigation, and an untranslated table
   of contents is the first thing a reader sees.
3. Translate the chapters. Keep the file names and the directory structure
   identical to the English tree, so that links and the staleness check work.
4. Write `<code>/STATUS.md` from the template in `es/STATUS.md`, and record the
   source hash of every chapter.
5. Run `python3 book/scripts/check_translations.py`.
6. Add the language to `LANGUAGES` in
   [`../scripts/build_site.py`](../scripts/build_site.py) and to the switcher in
   [`../theme/nudo-book.js`](../theme/nudo-book.js).

## What is not translated

* [`../adr`](../adr) — decision records of the book system itself, read by the
  people maintaining it.
* [`../quality`](../quality) and [`../scripts`](../scripts) — tooling.
* [`../../spec`](../../spec), [`../../neps`](../../neps) and
  [`../../README.md`](../../README.md) — the repository is in English, and
  translating the normative documents would create five specifications.
