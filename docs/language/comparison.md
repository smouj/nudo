# NUDO compared with Python, Rust, Go and TypeScript

This page answers one question honestly: **what does NUDO have that those four
languages do not, and what do they have that NUDO does not yet have?**

It is deliberately not a marketing table. NUDO is pre-alpha. Nothing here says
NUDO is faster, safer or more expressive in general, because no comparable
measurement exists and most of the language is not implemented. Claims about
performance and security guarantees are separated on purpose: a measurement and
a semantic promise are different kinds of evidence.

## How to read the status column

Feature rows use one of six status labels:

| Label | Meaning |
| ----- | ------- |
| **Implemented** | Exists in the compiler and is covered by tests or conformance cases |
| **Partial** | Exists, but the coverage is visibly incomplete |
| **Specified** | Decided in an accepted NEP, but not yet implemented |
| **Proposed** | Described in a chapter marked Proposed or a NEP still in Draft |
| **Planned** | Listed on the roadmap, with design details still open |
| **Undecided** | No accepted decision, active proposal or roadmap commitment exists |

For the other languages the question is different: their features exist, and the
honest comparison is *how* they provide them — in the language, in the standard
library, or through an ecosystem convention. Production readiness and ecosystem
size are current facts, so those rows state them directly.

## Language-level comparison

| Concern | NUDO | Python | Rust | Go | TypeScript |
| ------- | ---- | ------ | ---- | -- | ---------- |
| Static typing | **Partial** — primitives, calls, fields and name resolution work; generic instantiation and enum-match typing are incomplete | Dynamic, optional annotations | Full, with inference and traits | Static, structural interfaces | Static, erased at runtime |
| Memory model | **Undecided** — no garbage-collection or runtime allocation model is specified; NEP-0009 rules out a borrow checker | Reference counting + cycle collector | Ownership and borrowing | Garbage collected | Garbage collected (engine) |
| Nullability | **Proposed** — absence is written as `T?`; the types chapter is still proposed | `None` everywhere | `Option<T>` | Nil pointers for some types | `strictNullChecks` |
| Error handling | **Specified** — `Result<T, E>` intrinsic, no `?`, traps distinct from failures (NEP-0008) | Exceptions | `Result` + `?` | `error` returns | Exceptions, or `Result` conventions |
| Generics | **Partial** — syntax is accepted and parsed; type-checker instantiation and inference remain incomplete (NEP-0010, NEP-0013) | Duck typing / `typing` | Generics + traits | Generics since 1.18 | Generics, erased |
| Pattern matching | **Partial** — `match` syntax and HIR are implemented; enum-variant typing remains undecided and is not implemented | `match` since 3.10 | `match`, exhaustive | Limited `switch`, no destructuring | Discriminated unions by convention |
| Immutability | **Specified** — bindings immutable by rule, no `mut` (NEP-0009) | No | Opt-in `let` / `mut` | No | `const` only |
| Effects | **Proposed** — the effects chapter and NEP-0004 are not accepted; checking is planned for M5 | No | No (traits approximate) | No | No |
| Capabilities / permissions | **Proposed** — the capability chapter is proposed and NEP-0003 is Draft; no call checks exist | Library or OS | Library or OS | Library or OS | Library or OS |
| Trust types | **Specified** — `Generated<T>` and `Verified<T>` are distinct (NEP-0002, NEP-0014); implementation is pending | Convention | Convention | Convention | Convention |
| Provenance | **Proposed** — the provenance chapter is proposed; runtime records are planned for M6 | Convention | Convention | Convention | Convention |
| First-class agents, tools, models | **Proposed** — syntax is parsed, but declaration meaning and runtime chapters remain proposed | Libraries (LangChain, etc.) | Libraries | Libraries | Libraries |
| Verification of generated values | **Specified** — accepted NEP-0002 requires a named deterministic verifier and a `Result`; model calls cannot act as verifiers | Custom code | Custom code | Custom code | Custom code |
| Tooling | **Partial** — `nudo check` only; no formatter, LSP or REPL yet | Mature | Mature | Mature | Mature |
| Package management | **Proposed** — the modules chapter is proposed; no package manager exists | pip / uv | Cargo | modules | npm |
| Execution | **Planned** — interpreter is M4 | Interpreter | Native | Native | Engine |
| Compilation targets | **Planned** — WASM backend is a planned milestone | CPython, PyPy | Native, WASM | Native, WASM | JavaScript |
| Self-hosting | **Planned** — Rust is the bootstrap language by decision | No | Yes | Yes | No |
| Production readiness | **No** — pre-alpha, explicitly not fit for production | Yes | Yes | Yes | Yes |
| Ecosystem size | Tiny: this repository | Very large | Large | Large | Very large |

## What NUDO intends to put in the language

These ideas explain why the language exists, but their design status differs.
Some rules have been accepted; other chapters and NEPs remain proposals. None
of these features is fully implemented.

1. **Trust as a type — Specified.** `Generated<T>` and `Verified<T>` are distinct,
   with no conversion between them, and `verify` is the only operation that
   produces the second. In Python, Rust, Go and TypeScript this is a naming
   convention plus a reviewer. NUDO's design specifies a compiler-checkable rule;
   the checker does not implement it yet.
2. **Capabilities checked before a call — Proposed.** A function that needs a
   capability declares it, and the checker refuses a call without it. The other
   four can do this with a library, an OS sandbox or a framework; the difference
   would be whether the build can prove it.
3. **Budgets, memory and traces as language concepts — Proposed.** A task can
   carry a token budget and a trace identifier that the type system and the
   runtime know about. In the other languages these live in configuration and
   dashboards.

None of these is unique as an *idea*. The accepted decisions and open proposals
show which parts of the language-level design are settled and which still need
review before a build can demonstrate them.

## What the others already do better, today

| Area | Who leads | Why it matters |
| ---- | --------- | -------------- |
| Execution | All four | NUDO cannot run a program yet |
| Type-system completeness | Rust, TypeScript | NUDO types a small slice and stays silent elsewhere |
| Tooling | All four | No formatter, LSP, debugger or package manager in NUDO |
| Libraries | Python, TypeScript, Go, Rust | NUDO has none, by design at this stage |
| Compile-time guarantees | Rust | Ownership and lifetimes are real; NUDO's model is not built |
| Documentation and examples | All four | NUDO has a specification and a book, not a user community |
| Performance evidence | Rust, Go | NUDO has front-end micro-benchmarks and nothing else |

## On performance

The repository contains a benchmark suite for the compiler front end (`lex`,
`parse`, `hir`, `typeck`). It measures **NUDO's own compiler**, not the speed of
programs written in NUDO, because no program can run.

There is therefore no defensible statement today of the form "NUDO is faster
than X". When `nudo run` exists (M4), a fair comparison needs the same program,
the same algorithm, the same input, the same machine and a published harness.
Until then, the only performance claim this project makes is about its front
end, with the methodology in [`benchmarks/README.md`](../../benchmarks/README.md).

## On safety

"Safe" is used in two different senses and they should not be mixed:

* **Memory safety** — Rust can demonstrate it; Go and TypeScript rely on a
  runtime; Python relies on a runtime; NUDO has no execution model implemented,
  so it demonstrates nothing yet.
* **Semantic safety** — refusing to pass unverified model output to a function
  that requires verification. NUDO *specifies* this. It does not implement it,
  and no conformance case pins it yet (that is M3.3).

A language that has not run a program cannot be called safe. It can only be
called explicit about what it intends to guarantee.

## Where NUDO stands

NUDO is not a better Python, Rust, Go or TypeScript today. It is a pre-alpha
language project with a written specification, a working lexer, parser, HIR and
a first type-checking slice. Its intended direction is to treat agents, tools,
models, permissions, budgets and provenance as language concepts, then make the
compiler able to prove the guarantees that become accepted rules.

The accepted trust rules become testable at M3.3, and a running program is
planned for M4. Until then, this page keeps claims smaller than the evidence.
