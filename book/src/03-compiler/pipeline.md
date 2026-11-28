# 03.1 — Compiler pipeline

> **Status:** Implemented / Planned
>
> **Summary:** NUDO’s front end is divided into explicit stages, with HIR and name resolution implemented and later checking and execution stages still in progress.

{{#include ../diagrams/pipeline.svg}}

*The accent path marks front-end work used by the CLI. Type checking is partial;
effect checking, MIR and execution are planned.*

~~~text
SourceFile → Lexer → Lossless syntax tree → AST → HIR / name resolution
                                                        ↓
                                  partial accepted-rule type checker
                                                        ↓
                                    effects → MIR → interpreter / backend
~~~

The CLI command <code>nudo check</code> runs source loading, lexing, parsing,
HIR lowering/name resolution and the implemented subset of accepted type rules.
It can dump tokens, the syntax tree and name resolutions. It does not execute
the source.

The <code>nudo-hir</code> crate lowers the typed AST into identifiers and
resolved definitions. The <code>nudo-typeck</code> crate consumes that HIR and
implements a partial set of accepted type rules. The CLI invokes it after
successful name resolution; unit tests and the typeck conformance corpus also
exercise it directly. Draft-gated rules remain unimplemented.

The planned effect checker, MIR, interpreter and WebAssembly backend are not
available capabilities. Their boxes in the diagram show the intended
architecture and roadmap order; they are not a promise that a program can pass
through those stages today.

Explicit stage boundaries make each transformation testable and let future
tools share representations rather than re-parse source with independent
assumptions. During pre-alpha, crate boundaries may still change as implementation
evidence accumulates.
