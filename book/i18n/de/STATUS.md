# de — Status der Übersetzung

**Sprache:** Deutsch (`de`)
**Englische Revision, aus der übersetzt wurde:** `66774d2`
**Stand:** 2026-09-17
**Umfang:** 33 von 33 Dateien (31 Kapitel, `index.md`, `SUMMARY.md`)

Der englische Quelltext wurde beim Lesen als Arbeitsbaum über `c28fc0c` gelesen; die
fünf geänderten Kapitel (`index.md`, `03-compiler/pipeline.md`,
`04-type-system/generated-verified.md`, `05-agents-and-ai/tasks-tools-models.md`,
`06-security/capabilities-effects.md`) und die Diagramme sind inzwischen als
`66774d2` festgeschrieben, und es ist genau dieser Inhalt, der hier übersetzt wurde.
`book/scripts/check_translations.py` meldet die Ausgabe als *complete and current*.

Diese Ausgabe ist eine Übersetzung von [`../../src`](../../src). Englisch ist kanonisch:
Wenn diese Übersetzung und der englische Quelltext sich widersprechen, ist die
Übersetzung falsch. Statuslabels behalten ihren Rang — `Geplant` wird niemals zu
„verfügbar".

## Übersetzte Kapitel

| Datei | Titel | Status im Kapitel |
| ----- | ----- | ----------------- |
| `index.md` | Das NUDO Technical Book | Historisch / Lebend |
| `SUMMARY.md` | Navigation (Summary) | — |
| `00-introduction/what-is-nudo.md` | Was ist NUDO? | Spezifiziert |
| `00-introduction/why-exists.md` | Warum es NUDO gibt | Spezifiziert |
| `00-introduction/status.md` | Aktueller Projektstatus | Implementiert / Geplant |
| `01-philosophy/deterministic-first.md` | Deterministische Software zuerst | Spezifiziert |
| `01-philosophy/explicit-trust.md` | Explizites Vertrauen | Vorgeschlagen |
| `01-philosophy/local-first.md` | Local-first und anbieterunabhängig | Spezifiziert |
| `02-language-design/grammar.md` | Grammatik und Syntaxdisziplin | Vorgeschlagen |
| `02-language-design/expressions.md` | Ausdrücke und Präzedenz | Offen |
| `02-language-design/modules.md` | Module und Programm-Grenzen | Vorgeschlagen |
| `03-compiler/pipeline.md` | Compiler-Pipeline | Spezifiziert |
| `03-compiler/lexer.md` | Quellmodell und Lexer | Implementiert |
| `03-compiler/parser.md` | Parser, verlustfreie Syntax und AST | Geplant |
| `03-compiler/diagnostics.md` | Diagnostik als Produktdesign | Spezifiziert / Geplant |
| `04-type-system/fundamentals.md` | Grundlagen des Typsystems | Vorgeschlagen |
| `04-type-system/generated-verified.md` | `Generated<T>` und `Verified<T>` | Vorgeschlagen |
| `05-agents-and-ai/agents.md` | Agenten als begrenzte Ausführer | Vorgeschlagen |
| `05-agents-and-ai/tasks-tools-models.md` | Tasks, Tools und Modelle | Vorgeschlagen |
| `06-security/capabilities-effects.md` | Capabilities, Effekte und Autorität | Vorgeschlagen |
| `06-security/threat-model.md` | Bedrohungsmodell | Spezifiziert / Geplant |
| `07-runtime/trace-provenance.md` | Traces und Provenienz | Vorgeschlagen |
| `07-runtime/budgets-approvals.md` | Budgets und Freigaben | Vorgeschlagen |
| `08-tooling/cli.md` | Toolchain und CLI | Implementiert / Geplant |
| `09-interoperability/mcp-a2a-wasm.md` | MCP, A2A und WebAssembly | Geplant |
| `10-internals/repository-architecture.md` | Repository-Architektur | Implementiertes Fundament |
| `11-rationale/why-rust-own-compiler.md` | Warum Rust und warum ein eigener Compiler? | Spezifizierte Begründung |
| `11-rationale/why-trust-types.md` | Warum Vertrauen zur Typgeschichte gehört | Vorgeschlagene Begründung |
| `12-alternatives/rejected.md` | Bewusst verworfene Alternativen | Historisch / Entwurfsbegründung |
| `13-examples/ordinary-program.md` | Gewöhnliches deterministisches Programm | Vorgeschlagene Syntax |
| `13-examples/trust-flow.md` | Modellausgabe und Verifikationsablauf | Vorgeschlagene Syntax |
| `14-contributing/spec-nep-adr.md` | Spezifikation, NEPs und ADRs | Spezifizierter Prozess |
| `15-history/milestones.md` | Meilensteine und Belege | Historisch / Lebend |

## Nicht übersetzt

Nichts innerhalb von `book/src`: Alle 33 Dateien liegen in `de/src` am identischen
relativen Pfad. Die Stempel in [`translation.json`](translation.json) wurden mit
`book/scripts/stamp_translation.py --lang de` aus der aktuellen englischen Revision
gesetzt.

Bewusst **nicht** übersetzt, weil die Übersetzungsvereinbarung das ausschließt
([`../README.md`](../README.md)):

* `book/adr`, `book/quality`, `book/scripts` — Werkzeuge und Entscheidungsprotokolle
  des Buchsystems selbst;
* `spec/`, `neps/`, `README.md` des Repositories — das Repository bleibt englisch,
  damit es nicht fünf Spezifikationen gibt;
* Diagramme unter `src/diagrams/` — sie werden beim Build aus dem kanonischen Baum
  kopiert und enthalten keinen übersetzbaren Fließtext.

## Übersetzungsentscheidungen (Kurzfassung)

* Statuslabels werden übersetzt, ihr Rang bleibt: `Spezifiziert`, `Implementiert`,
  `Geplant`, `Vorgeschlagen`, `Offen`.
* Innerhalb von Codeblöcken, Diagramm-Captions ausgenommen, bleibt alles englisch —
  auch die Wörter `implemented` / `planned` in den Pipeline-Blöcken.
* `{{#include ../diagrams/*.svg}}` und `{{#include diagrams/status.svg}}` sind
  byte-identisch zum englischen Quelltext.
* Etablierte Fachbegriffe bleiben englisch, wo sie im Deutschen so benutzt werden:
  `Compiler`, `Lexer`, `Parser`, `AST`, `Token`, `Span`, `Crate`, `Tool`,
  `Capability`, `Policy`, `Trace`, `Budget`, `Monorepo`, `Formatter`, `Typechecker`.
* Kapitelübergreifende Verweise zeigen auf das deutsche Geschwisterkapitel; relative
  Pfade sind gegenüber dem englischen Quelltext unverändert.
* Der Verweis auf `ROADMAP.md` in `index.md` zeigt auf die absolute Repository-URL
  (`https://github.com/smouj/nudo/blob/main/ROADMAP.md`). Der englische Quelltext
  nutzt `../../ROADMAP.md`, was aus `book/src/` die Repository-Wurzel trifft; der
  deutsche Baum liegt zwei Ebenen tiefer, dort löst derselbe relative Pfad nicht
  mehr auf. Die anderen Sprachausgaben verfahren ebenso.
* Die erste Zeile von `SUMMARY.md` bleibt `# Summary`: mdBook verlangt dort eine
  Überschrift, sie ist keine sichtbare Kapitelüberschrift.

## Offene Punkte

* Keine. Der Build von `de` läuft ohne Fehler- oder Warnzeilen (siehe
  `book/scripts/build_site.py de`).
