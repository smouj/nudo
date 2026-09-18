# Tests

Two corpora, two purposes:

| Corpus | Purpose | Driven by |
| ------ | ------- | --------- |
| [`../fixtures`](../fixtures) | Pin one rule of one stage. | `cargo test` (unit/integration tests of the owning crate) |
| [`conformance/`](conformance) | Pin language-visible behaviour that **any** implementation must reproduce. | `scripts/conformance.sh` |

Crate-local tests live next to the code they test, in `src/` (`#[cfg(test)]`
modules) and in each crate's `tests/` directory.

## Planned corpus areas

The conformance corpus is organised by pipeline stage. Only the first stage
exists today, and the rest are listed so that contributors do not invent a
competing layout later:

| Area | Status | Milestone |
| ---- | ------ | --------- |
| `conformance/lexer/`       | **present** | M1 |
| `conformance/parser/`      | **present** | M2 |
| `conformance/resolve/`     | **present** | M3 |
| `conformance/typecheck/`   | planned     | M3 |
| `conformance/effects/`     | planned     | M5 |
| `conformance/runtime/`     | planned     | M4 |
| `conformance/agents/`      | planned     | M7 |
| `conformance/policies/`    | planned     | M8 |
| `conformance/diagnostics/` | planned     | grows with each family of `NDO` codes |

Directories are created when they hold their first real case, never before: an
empty corpus directory is a promise the repository cannot keep, and a suite
that silently tests nothing is worse than no suite.

## What a stage may not do

A stage's tests may only cover behaviour that the specification describes. If
a test needs behaviour the specification does not describe, the specification
is incomplete: write the specification change first. See
[`../AGENTS.md`](../AGENTS.md).
