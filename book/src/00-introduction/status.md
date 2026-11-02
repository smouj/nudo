# 00.3 — Current project status

> **Status:** Implemented / Planned  
> **Summary:** The repository has a working compiler front end and partial type-checking support; execution and effect enforcement are still future milestones.

The status below reflects the repository roadmap. “Implemented” means code and
tests exist for the stated scope. It does not mean the whole language or a
user-facing workflow is complete.

| Milestone | State | What is available |
| --- | --- | --- |
| M0 — Repository foundation | Complete | Workspace, contribution rules, CI and documentation infrastructure |
| M1 — Source model and lexer | Complete | Source files, spans, tokens, lexical diagnostics and recovery |
| M2 — Parser and syntax tree | Complete | Lossless syntax, parser, typed AST and syntax diagnostics |
| M3.0 — Semantic gates | Complete | Accepted decisions recorded in the specification and NEPs |
| M3.1 — HIR and name resolution | Complete | HIR lowering, scopes, name resolution and NDO2001–NDO2003 |
| M3.2 — Type system | In progress | Partial type checker for accepted rules; not yet called by the CLI |
| M3.3 — Trust types | Partial | Type identity and the accepted Verified-to-inner assignment rule |
| M4 — Interpreter | Planned | First end-to-end program and <code>nudo run</code> |
| M5 — Effect system | Planned | Effect inference, checking and diagnostics; semantics remain proposed |

The CLI command <code>nudo check</code> currently loads source, lexes, parses
and resolves names. It can display tokens, syntax trees and name resolutions.
A clean run means those stages reported no errors; it does not establish that
the program is type-correct, executable or safe at runtime.

The type-checking crate covers primitive and literal types, annotations, calls,
bounded generic inference, field access, selected arithmetic, Bool conditions
and accepted trust-type assignment. Its current conformance corpus exercises
those rules independently of the CLI. Unsupported rules remain deliberately
incomplete until their semantic contracts are ready.

M4 and M5 have dependency gates. The first needs the remaining type-system and
trust work plus a real execution path. M5 needs those foundations and an
accepted effect-system design. The roadmap defines milestone completion by
working exit criteria and conformance evidence, not by a calendar date.

## Why status labels matter

A polished example can look like a shipped feature. Every chapter therefore
labels whether it describes implemented behaviour, accepted design, a proposal
or planned work. The
<a href="https://github.com/smouj/nudo/blob/main/ROADMAP.md">roadmap</a> is the
authoritative, changing status record.
