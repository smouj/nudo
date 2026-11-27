# 03.1 — Compiler pipeline

> **Status:** Implemented / Planned
>
> **Summary:** NUDO’s front end is divided into explicit stages, with HIR and name resolution implemented and later checking and execution stages still in progress.

{{#include ../diagrams/pipeline.svg}}

*The accent path marks implemented front-end work. Type checking is partial and
tested separately; effect checking, MIR and execution are planned.*

~~~text
SourceFile → Lexer → Lossless syntax tree → AST → HIR / name resolution
                                                        ↓
                                     partial type checker (not in nudo check)
                                                        ↓
                                    effects → MIR → interpreter / backend
~~~

The CLI command <code>nudo check</code> runs source loading, lexing, parsing and
HIR lowering/name resolution. It can dump tokens, the syntax tree and name
resolutions. It does not call <code>nudo-typeck</code>, and it does not execute
the source.

The <code>nudo-hir</code> crate lowers the typed AST into identifiers and
resolved definitions. The <code>nudo-typeck</code> crate consumes that HIR and
implements a partial set of accepted type rules. Unit tests and the typeck
conformance corpus exercise it directly. Its remaining rules must be specified
before they are implemented.

The planned effect checker, MIR, interpreter and WebAssembly backend are not
available capabilities. Their boxes in the diagram show the intended
architecture and roadmap order; they are not a promise that a program can pass
through those stages today.

Explicit stage boundaries make each transformation testable and let future
tools share representations rather than re-parse source with independent
assumptions. During pre-alpha, crate boundaries may still change as implementation
evidence accumulates.
