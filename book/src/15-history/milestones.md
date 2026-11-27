# 15.1 — Milestones and evidence

> **Status:** Historical / Living  
> **Summary:** NUDO uses evidence-based milestones; M3 remains in progress, while the interpreter and effect system needed to reach M5 are planned.

The roadmap measures progress by exit criteria and conformance evidence rather
than calendar dates. This page summarizes the path through M5; the repository
roadmap remains authoritative.

{{#include ../diagrams/status.svg}}

| Milestone | State | Exit evidence |
| --- | --- | --- |
| M0 — Repository foundation | Complete | A fresh checkout can be built, checked and reviewed |
| M1 — Source model and lexer | Complete | Deterministic tokens, diagnostics and recovery |
| M2 — Parser and syntax tree | Complete | Lossless parsing, recovery and conformance cases |
| M3.0 — Semantic gates | Complete | Core type-system decisions recorded before implementation |
| M3.1 — HIR and name resolution | Complete | Resolved HIR and stable name diagnostics |
| M3.2 — Basic type system | In progress | Accepted rules are added as vertical slices with conformance evidence |
| M3.3 — Trust types | Partial | Type identity and Verified assignment exist; verify and provenance remain |
| M4 — Interpreter | Planned | The example program computes and prints 42 through <code>nudo run</code> |
| M5 — Effect system | Planned | An effectful call from an effect-free function is rejected with its effect path |

M4 and M5 are not consecutive labels for interchangeable work. The M4 exit
criterion requires an actual source-to-output execution path. M5 follows the
type and trust foundations and requires accepted effect semantics. A Draft
proposal cannot be used as an implementation contract.

## What the milestones protect

The order prevents a polished demo from standing in for compiler guarantees.
Before M5 can be marked complete, the earlier type and trust criteria must pass,
M4 must run the documented example, the effect-system semantics must be
accepted, and M5 diagnostics and conformance cases must demonstrate the exit
criterion.

There is no promise in the roadmap to reach M5 on a calendar date. Progress is
reported from what builds and passes its specified tests.

\n