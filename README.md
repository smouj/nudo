<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/logo/nudo-symbol-dark.png">
    <img src="assets/logo/nudo-symbol-light.png" alt="The NUDO symbol" width="104">
  </picture>
</p>

<h1 align="center">NUDO</h1>

<p align="center"><strong>A programming language for humans and agents.</strong></p>

<p align="center">
  <a href="actions/workflows/ci.yml"><img src="https://github.com/smouj/nudo/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="LICENSE-MIT"><img src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg" alt="License: MIT OR Apache-2.0"></a>
  <img src="https://img.shields.io/badge/status-pre--alpha-orange.svg" alt="Status: pre-alpha">
  <img src="https://img.shields.io/badge/rust-1.85%2B-orange.svg" alt="Rust 1.85+">
</p>

---

## What is NUDO?

NUDO is a general-purpose programming language. It runs ordinary deterministic
software, and it treats the things that AI-era programs actually contain —
agents, tools, models, tasks, permissions, budgets, memory, traces — as
concepts the language itself understands.

It is not a wrapper around a model provider, not a prompt framework, not an
agent library and not a dialect of an existing language. NUDO connects people,
code, agents, tools and models under one type system, one permission model and
one verification model.

NUDO is **pre-alpha**. The language is not stable, the toolchain is incomplete,
and nothing here is fit for production use. That is the current state, not a
disclaimer: see [Current status](#current-status).

## Why NUDO?

Today, the parts of a program that involve a model are usually held together
from the outside: a prompt in a string, a tool schema in JSON, permissions in
configuration, a budget in a dashboard, a trace in a log aggregator, and trust
in whatever the last step happened to return. The compiler sees none of it, so
it can check none of it.

NUDO's question is what happens if those things move inside the language:

| Today | In NUDO |
| ----- | ------- |
| Capabilities granted by configuration | `capability` declarations checked before a call |
| "Is this model output safe to use?" | `Generated<T>` and `Verified<T>` as distinct types |
| Budgets enforced by a monitoring dashboard | A `budget` attached to a task, enforced by the runtime |
| Traces assembled from logs after the fact | Traces produced by execution |
| Provenance reconstructed by hand | Provenance recorded as data |
| Approvals handled in a chat window | `approval` as a first-class, checkable step |

None of that is implemented yet. It is what the specification in [`spec/`](spec)
defines and what [ROADMAP.md](ROADMAP.md) schedules.

## The core idea

```text
Human
  ↕
Code
  ↕
Agent
  ↕
Tools / Models / Other Agents
```

A thing produced by a model is not automatically a thing you can rely on. NUDO
makes that distinction a type distinction rather than a habit:

```text
Generated<Article>
    ↓  verify
Verified<Article>
```

Autonomy is modelled as a conjunction, not as a vibe: an objective, a set of
capabilities, a set of limits, a budget, acceptance criteria, policies and
approvals.

## Language preview

```nudo
fn add(a: Int, b: Int) -> Int {
    a + b
}

struct User {
    name: Text
    age: Int
}

agent Researcher {
    role:
        "Research reliable information."

    tools:
        web.search
        web.open

    allow:
        Network
}

task Research(topic: Text) -> Verified<Report> {
    agent:
        Researcher

    verify:
        SourcesRequired
}

let draft: Generated<Article> =
    ask Writer {
        "Create an article."
    }

let checked: Result<Verified<Article>, VerificationError> =
    verify draft with ArticleVerifier
```

**This syntax is PRE-ALPHA and has not been stabilised.** It is the direction
the specification is being written towards, not a contract. The grammar lives
in [`grammar/nudo.ebnf`](grammar/nudo.ebnf) and every design decision that is
still open is listed in [ROADMAP.md](ROADMAP.md).

## Trust model

Four things are separated on purpose:

* **Trust** — what a value is allowed to be used for. `Generated<T>` is not
  `Verified<T>`, and nothing converts between them implicitly.
* **Capability** — what a program is allowed to touch. Nothing is granted
  implicitly; the default is deny. A tool that needs the network says so, and
  a caller that cannot grant it does not get to call it.
* **Provenance** — where a value came from. Recorded, not inferred.
* **Policy** — what is allowed to happen automatically. Budgets, approvals and
  acceptance criteria are checked by the runtime, not by convention.

The security consequences are spelled out in
[`docs/security/threat-model.md`](docs/security/threat-model.md).

## Architecture

The compiler is a pipeline with sharp edges between stages, not a monolith:

```text
Source
  ↓
Lexer            compiler/nudo-lexer        ← implemented (M1)
  ↓
Tokens
  ↓
Parser           compiler/nudo-parser       ← implemented (M2)
  ↓
Syntax tree      compiler/nudo-syntax       ← implemented (M2)
AST              compiler/nudo-ast          ← implemented (M2)
  ↓
HIR              compiler/nudo-hir          ← implemented (M3)
Typeck           compiler/nudo-typeck       ← implemented (M3.2, first slice; not wired into `check` yet)
  ↓
Type checking    compiler/nudo-typeck       ← planned (M3)
  ↓
Effect checking  compiler/nudo-effects      ← planned (M5)
  ↓
MIR
  ↓
Backend
  ├── Interpreter   backends/interpreter    ← planned (M4)
  └── WebAssembly   backends/wasm           ← planned (M11)
```

[`ARCHITECTURE.md`](ARCHITECTURE.md) lists every crate, its responsibility and
the dependency rules between them.

## Current status

| Area | State |
| ---- | ----- |
| Language specification | Written, pre-alpha, expected to change |
| Lexer | Implemented and tested (milestone M1) |
| Parser, lossless syntax tree, AST | Implemented and tested (milestone M2) |
| HIR and name resolution | Implemented and tested (milestone M3.1) |
| `nudo check` | Implemented: reads `.nudo` files, reports lexical, syntax and name diagnostics |
| `nudo --version`, `nudo --help` | Implemented |
| Types, effects | Planned |
| Interpreter, WASM backend | Planned |
| Agents, tools, models, policies | Designed only |

```console
$ nudo check examples/00-hello-world/main.nudo
checked 1 file: 0 errors and 0 warnings
```

`nudo check` lexes, parses and resolves names. A clean run means "no lexical,
syntax or name diagnostics", not "this program is correct": nothing is
type-checked, and nothing runs.

## Roadmap

Milestones, not dates. See [ROADMAP.md](ROADMAP.md) for exit criteria.

| Milestone | Scope |
| --------- | ----- |
| M0 | Repository foundation |
| M1 | Source model and lexer |
| M2 | Parser and AST |
| M3 | Type system |
| M4 | Interpreter |
| M5 | Effect system |
| M6 | `Generated<T>` and `Verified<T>` |
| M7 | Agent runtime |
| M8 | Capabilities and policies |
| M9 | MCP and A2A interoperability |
| M10 | Tooling |
| M11 | WebAssembly backend |

## Repository structure

```text
compiler/     lexer, parser, AST, types, effects, diagnostics, MIR, codegen
runtime/      runtime, agents, models, tools, tasks, policies, capabilities
crates/       shared, dependency-free foundation crates
cli/          the `nudo` binary
tooling/      formatter, language server, doc generator, test runner, REPL
backends/     interpreter and WebAssembly backends
std/          the standard library (planned)
spec/         the language specification — the source of truth
grammar/      the EBNF grammar
neps/         NUDO Enhancement Proposals
docs/         user- and contributor-facing documentation
examples/     example programs, in increasing order of ambition
tests/        conformance corpus
fixtures/     unit-level inputs with declared expectations
fuzz/         fuzzing infrastructure
benchmarks/   reproducible measurements
scripts/      bootstrap, check, conformance, release
assets/       brand assets (see assets/brand/BRAND.md)
```

## Contributing

Contributions are welcome, and the process is deliberately concrete: read
[CONTRIBUTING.md](CONTRIBUTING.md), then
[`AGENTS.md`](AGENTS.md) if you are an agent or you work with one. Language
changes go through a [NEP](neps/README.md).

If you are an AI agent working in this repository: [`AGENTS.md`](AGENTS.md) is
not optional reading.

## Security

Do not open a public issue for a vulnerability. See [SECURITY.md](SECURITY.md).

## License

Dual-licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
* MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option. Unless you explicitly state otherwise, any contribution you
intentionally submit for inclusion in this project, as defined in the Apache-2.0
licence, shall be dual-licensed as above, without any additional terms.
