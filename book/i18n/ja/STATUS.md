# Translation status — 日本語 (`ja`)

**Language:** Japanese (日本語), code `ja`.
**Canonical source:** `book/src` (English).
**English revision translated from:** `c28fc0c` (`git rev-parse --short HEAD`, NUDO repository).
**Translation tree:** `book/i18n/ja/src/`, one file per English source file, at the identical relative path.

> **Working-tree note.** At translation time the English working tree was not
> identical to `c28fc0c`: `book/src/index.md`,
> `book/src/03-compiler/pipeline.md`,
> `book/src/04-type-system/generated-verified.md`,
> `book/src/05-agents-and-ai/tasks-tools-models.md` and
> `book/src/06-security/capabilities-effects.md` carried uncommitted edits, and
> `book/src/diagrams/` was untracked. The translation follows the working tree as
> it stood when each file was translated. `translation.json` records the SHA-256
> of each English file as translated, which is what `check_translations.py`
> compares against; if the English changes again, the chapter is reported as
> **behind** rather than silently left in place.

## Chapters

All 33 English Markdown files have a Japanese counterpart. Status is the chapter's
own English status label, whose rank is preserved in translation (never
strengthened).

| # | Chapter (`src/…`) | English status | Translated |
| --- | --- | --- | --- |
| — | `index.md` | Historical / Living | yes |
| — | `SUMMARY.md` | — (navigation) | yes |
| 00.1 | `00-introduction/what-is-nudo.md` | Specified | yes |
| 00.2 | `00-introduction/why-exists.md` | Specified | yes |
| 00.3 | `00-introduction/status.md` | Implemented / Planned | yes |
| 01.1 | `01-philosophy/deterministic-first.md` | Specified | yes |
| 01.2 | `01-philosophy/explicit-trust.md` | Proposed | yes |
| 01.3 | `01-philosophy/local-first.md` | Specified | yes |
| 02.1 | `02-language-design/grammar.md` | Proposed | yes |
| 02.2 | `02-language-design/expressions.md` | Open | yes |
| 02.3 | `02-language-design/modules.md` | Proposed | yes |
| 03.1 | `03-compiler/pipeline.md` | Specified | yes |
| 03.2 | `03-compiler/lexer.md` | Implemented | yes |
| 03.3 | `03-compiler/parser.md` | Planned | yes |
| 03.4 | `03-compiler/diagnostics.md` | Specified / Planned | yes |
| 04.1 | `04-type-system/fundamentals.md` | Proposed | yes |
| 04.2 | `04-type-system/generated-verified.md` | Proposed | yes |
| 05.1 | `05-agents-and-ai/agents.md` | Proposed | yes |
| 05.2 | `05-agents-and-ai/tasks-tools-models.md` | Proposed | yes |
| 06.1 | `06-security/capabilities-effects.md` | Proposed | yes |
| 06.2 | `06-security/threat-model.md` | Specified / Planned | yes |
| 07.1 | `07-runtime/trace-provenance.md` | Proposed | yes |
| 07.2 | `07-runtime/budgets-approvals.md` | Proposed | yes |
| 08.1 | `08-tooling/cli.md` | Implemented / Planned | yes |
| 09.1 | `09-interoperability/mcp-a2a-wasm.md` | Planned | yes |
| 10.1 | `10-internals/repository-architecture.md` | Implemented foundation | yes |
| 11.1 | `11-rationale/why-rust-own-compiler.md` | Specified rationale | yes |
| 11.2 | `11-rationale/why-trust-types.md` | Proposed rationale | yes |
| 12.1 | `12-alternatives/rejected.md` | Historical / Design rationale | yes |
| 13.1 | `13-examples/ordinary-program.md` | Proposed syntax | yes |
| 13.2 | `13-examples/trust-flow.md` | Proposed syntax | yes |
| 14.1 | `14-contributing/spec-nep-adr.md` | Specified process | yes |
| 15.1 | `15-history/milestones.md` | Historical / Living | yes |

## Not translated, and why

* **Diagrams.** `book/src/diagrams/*.svg` are shared, not duplicated: the build
  copies the canonical set into `book/i18n/ja/src/diagrams/`, and the
  `{{#include …}}` lines are byte-identical to the English ones. The SVGs do
  contain English labels (`Orchestrator`, `holds three capabilities`,
  `Delegation may narrow. It may never widen.`). Per
  [`../README.md`](../README.md), that is a signal that the diagram is saying too
  much, and the fix belongs in the diagram, not in five translations. The caption
  line under each include **is** translated.
* **Assets.** `book/src/assets/brand/*` are images.
* **Out of scope by contract.** `book/adr`, `book/quality`, `book/scripts`,
  `book/theme`, `spec/`, `neps/` and the repository `README.md` stay in English;
  see [`../README.md`](../README.md), "What is not translated".
* **Furniture outside `book/i18n/ja/`.** `book/i18n/ja/book.toml` already existed
  and was left untouched; the language is already registered in
  `book/scripts/build_site.py` and in the switcher in `book/theme/nudo-book.js`.

## Verification

```sh
python3 book/scripts/build_site.py ja
python3 book/scripts/stamp_translation.py --lang ja
python3 book/scripts/check_translations.py
```

Real output is recorded in the translation report for this revision; run the
commands again to reproduce it.
