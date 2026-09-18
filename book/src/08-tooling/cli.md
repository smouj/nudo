# 08.1 — Toolchain and CLI

> **Status:** Implemented / Planned  
> **Summary:** The `nudo` command provides one coherent entry point from lexing and parsing today to build, run, format, test, audit and trace workflows later.

Current implemented commands include `check` — which lexes and parses, with
`--dump-tokens` and `--dump-tree` — and basic CLI metadata. Future roadmap
commands include build, run, REPL, formatter, documentation, testing, evaluation,
doctor/audit and tracing.

The command surface should remain predictable: stable exit codes, machine-readable
output where needed, diagnostic codes and no silent network activity.
