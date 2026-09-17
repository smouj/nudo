# Estado de la traducción — Español (`es`)

> **Status:** En curso (edición completa)
> **Summary:** Esta edición española traduce la edición inglesa canónica del libro. El inglés sigue siendo la fuente de verdad: si esta traducción y el original discrepan, prevalece el original.

| Campo | Valor |
| ----- | ----- |
| Idioma | Español (`es`) |
| Directorio | `book/i18n/es/src` |
| Edición inglesa de origen | revisión `66774d2` de `book/src`, más la reescritura todavía sin commitear de `02-language-design/expressions.md` (estado cambiado a `Specified`) |
| Alcance | 33 archivos: `index.md`, `SUMMARY.md` y los 31 capítulos |
| Manifiesto de hashes | `book/i18n/es/translation.json` |

La revisión de origen se registra capítulo a capítulo con
`python3 book/scripts/stamp_translation.py --lang es`. Cada entrada de
`translation.json` es el hash SHA-256 del archivo inglés del que se tradujo ese
capítulo; `python3 book/scripts/check_translations.py` compara esos hashes con el
original actual y marca como **behind** todo capítulo cuyo inglés haya cambiado
después de la traducción.

## Capítulos

| Archivo | Original inglés | Estado |
| ------- | --------------- | ------ |
| `index.md` | `index.md` | Traducido |
| `SUMMARY.md` | `SUMMARY.md` | Traducido |
| `00-introduction/what-is-nudo.md` | `00-introduction/what-is-nudo.md` | Traducido |
| `00-introduction/why-exists.md` | `00-introduction/why-exists.md` | Traducido |
| `00-introduction/status.md` | `00-introduction/status.md` | Traducido |
| `01-philosophy/deterministic-first.md` | `01-philosophy/deterministic-first.md` | Traducido |
| `01-philosophy/explicit-trust.md` | `01-philosophy/explicit-trust.md` | Traducido |
| `01-philosophy/local-first.md` | `01-philosophy/local-first.md` | Traducido |
| `02-language-design/grammar.md` | `02-language-design/grammar.md` | Traducido |
| `02-language-design/expressions.md` | `02-language-design/expressions.md` | Traducido |
| `02-language-design/modules.md` | `02-language-design/modules.md` | Traducido |
| `03-compiler/pipeline.md` | `03-compiler/pipeline.md` | Traducido |
| `03-compiler/lexer.md` | `03-compiler/lexer.md` | Traducido |
| `03-compiler/parser.md` | `03-compiler/parser.md` | Traducido |
| `03-compiler/diagnostics.md` | `03-compiler/diagnostics.md` | Traducido |
| `04-type-system/fundamentals.md` | `04-type-system/fundamentals.md` | Traducido |
| `04-type-system/generated-verified.md` | `04-type-system/generated-verified.md` | Traducido |
| `05-agents-and-ai/agents.md` | `05-agents-and-ai/agents.md` | Traducido |
| `05-agents-and-ai/tasks-tools-models.md` | `05-agents-and-ai/tasks-tools-models.md` | Traducido |
| `06-security/capabilities-effects.md` | `06-security/capabilities-effects.md` | Traducido |
| `06-security/threat-model.md` | `06-security/threat-model.md` | Traducido |
| `07-runtime/trace-provenance.md` | `07-runtime/trace-provenance.md` | Traducido |
| `07-runtime/budgets-approvals.md` | `07-runtime/budgets-approvals.md` | Traducido |
| `08-tooling/cli.md` | `08-tooling/cli.md` | Traducido |
| `09-interoperability/mcp-a2a-wasm.md` | `09-interoperability/mcp-a2a-wasm.md` | Traducido |
| `10-internals/repository-architecture.md` | `10-internals/repository-architecture.md` | Traducido |
| `11-rationale/why-rust-own-compiler.md` | `11-rationale/why-rust-own-compiler.md` | Traducido |
| `11-rationale/why-trust-types.md` | `11-rationale/why-trust-types.md` | Traducido |
| `12-alternatives/rejected.md` | `12-alternatives/rejected.md` | Traducido |
| `13-examples/ordinary-program.md` | `13-examples/ordinary-program.md` | Traducido |
| `13-examples/trust-flow.md` | `13-examples/trust-flow.md` | Traducido |
| `14-contributing/spec-nep-adr.md` | `14-contributing/spec-nep-adr.md` | Traducido |
| `15-history/milestones.md` | `15-history/milestones.md` | Traducido |

## Etiquetas de estado

Las etiquetas conservan su rango. Nunca se traduce un estado a algo más fuerte; la
tabla de correspondencia es fija:

| Original | Español | Significado |
| -------- | ------- | ----------- |
| `IMPLEMENTED` | Implementado | Existe y funciona hoy en el repositorio. |
| `SPECIFIED` | Especificado | Está definido por la especificación; no implica que exista en el código. |
| `PLANNED` | Planificado | Diseñado y situado en la hoja de ruta; no está construido. |
| `PROPOSED` | Propuesto | Propuesta de diseño abierta, todavía no aceptada. |

Los valores compuestos y cualificados del original se traducen conservando el mismo
rango en cada parte (por ejemplo `Specified / Planned` → `Especificado /
Planificado`, `Implemented foundation` → `Base implementada`, `Proposed syntax` →
`Sintaxis propuesta`). El valor `Open` de
`02-language-design/expressions.md` no pertenece al vocabulario de cuatro etiquetas
del repositorio y se traduce literalmente como «Abierto», sin interpretarlo.

## Lo que no está traducido

Fuera del alcance de esta edición, por decisión del contrato de traducción
([`../../README.md`](../../README.md)):

* `book/src` — el original inglés, que sigue siendo canónico.
* `book/adr`, `book/quality`, `book/scripts`, `book/theme`, `book/typst` — el sistema
  del libro y sus herramientas, que leen quienes lo mantienen.
* `spec/`, `neps/`, `tests/` y los documentos de la raíz del repositorio
  (`README.md`, `ROADMAP.md`, `AGENTS.md`, `DESIGN.md`, `ARCHITECTURE.md`, …) — el
  repositorio está en inglés y traducir los documentos normativos crearía cinco
  especificaciones distintas.
* Los diagramas `book/src/diagrams/*.svg` — se comparten, no se duplican: los copia
  en este árbol `book/scripts/build_site.py` antes de compilar y se incluyen con la
  misma ruta relativa. Su texto interior, por tanto, permanece en inglés.
* Las imágenes de marca y las hojas de estilo compartidas.

## Decisiones de traducción registradas

1. Las etiquetas `**Status:**` y `**Summary:**` se mantienen literales: son mobiliario
   estructural del libro y el sistema visual las usa como tales. Solo se traduce su
   contenido.
2. Los bloques de código, los fragmentos en línea, los identificadores, las rutas, los
   códigos `NDO` y los nombres de comando no se traducen, incluidos los diagramas ASCII
   de los bloques ```` ```text ````.
3. Las líneas `{{#include …}}` son idénticas byte a byte al original; solo se traduce
   la leyenda en cursiva situada debajo.
4. En `index.md`, el enlace a `ROADMAP.md` apunta a la raíz del repositorio desde este
   árbol, que tiene dos niveles más, por lo que la ruta relativa se ajusta a su
   profundidad real. El destino sigue siendo el mismo documento inglés.
