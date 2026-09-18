# `std/` — the NUDO standard library

**PLANNED. Empty on purpose.**

The standard library cannot be written before the language can run a program.
Today the toolchain lexes and parses and nothing else (milestones M1–M2), so
there is no code to call and no way to test what a standard library does.

This directory holds a README instead of thirteen empty directories, because an
empty directory is a promise the repository cannot keep.

## Planned modules

The layout is fixed by the specification work ahead, not by taste:

| Module        | Purpose                                              | Milestone |
| ------------- | ---------------------------------------------------- | --------- |
| `core/`       | always-available primitives: text, math, results      | M4 |
| `collections/`| lists, maps, sets                                     | M4 |
| `io/`         | streams, readers and writers                          | M4 |
| `fs/`         | filesystem access, behind a capability                | M4 |
| `net/`        | network access, behind a capability                   | M9 |
| `time/`       | clocks, durations, dates                              | M4 |
| `concurrent/` | tasks and structured concurrency                      | M4 |
| `agent/`      | agents, delegation and supervision                    | M7 |
| `tool/`       | tool registration and invocation                       | M7 |
| `model/`      | model interface, provider-agnostic                    | M7 |
| `policy/`     | policies, budgets and approvals                       | M8 |
| `trace/`      | traces, provenance and audit                          | M6 |
| `testing/`    | test helpers and assertions                           | M10 |

Each module is created when it holds its first working, tested code.

## Rules

* Standard library code is NUDO code, and is subject to the same specification
  as user code. It gets no private language features.
* Anything that touches the outside world is capability-gated. There is no
  ambient authority in `std`, including in `core`.
* Nothing in `std` may be the only place a language rule is defined. If the
  specification does not describe it, it does not belong here yet.
