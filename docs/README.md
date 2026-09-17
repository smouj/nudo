# Documentation

Two layers, for two audiences:

| Layer | For | Where |
| ----- | --- | ----- |
| **Specification** | What NUDO means. Normative, reviewable, and the thing the compiler must obey | [`../spec`](../spec/README.md) |
| **Documentation** | How to use it, and how it works. Explanatory, and allowed to be incomplete | this directory |

If the two disagree, the specification wins — see the authority order in
[`../spec/README.md`](../spec/README.md).

Documentation also exists for contributors:

* [`../ARCHITECTURE.md`](../ARCHITECTURE.md) — the shape of the code
* [`../DESIGN.md`](../DESIGN.md) — why the language is the way it is
* [`../CONTRIBUTING.md`](../CONTRIBUTING.md) and [`../AGENTS.md`](../AGENTS.md) —
  how to change it
* [`../ROADMAP.md`](../ROADMAP.md) — what exists and what does not

## Sections

| Section | Contents |
| ------- | -------- |
| [`getting-started/`](getting-started/installing.md) | Installing the toolchain, and the first program |
| [`language/`](language/tour.md) | A tour of the syntax, with the provisional parts clearly marked |
| [`agents/`](agents/README.md) | Agents, tasks and trust, explained for a reader |
| [`security/`](security/README.md) | The threat model, and the security posture |
| [`compiler/`](compiler/pipeline.md) | How the compiler is put together |
| [`tooling/`](tooling/cli.md) | The `nudo` command line |
| [`interoperability/`](interoperability/README.md) | MCP, A2A and WASM |
| [`internals/`](internals/README.md) | Notes for people working on the compiler |

## A warning about completeness

NUDO is pre-alpha. Most of the language documented here does not run yet, and
each page says so where it matters. A page that describes unimplemented
behaviour as though it worked is a bug — report it as one.
