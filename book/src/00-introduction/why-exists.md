# 00.2 — Why NUDO exists

> **Status:** Specified  
> **Summary:** The project starts from a compiler-visibility problem: critical AI-era program state is commonly hidden in strings, dashboards and conventions.

## The visibility gap

Consider a typical agent application. A prompt is a string. A tool schema is JSON.
Permissions live in configuration. Token or money budgets live in a provider
console. Human approval happens in chat. Traces are reconstructed after execution.

Each piece can work, but the programming language usually has no unified way to
reason about them.

## NUDO's thesis

If trust, authority, effects, budgets and provenance are represented explicitly,
then some failures can move from conventions to compiler/runtime checks.

This does **not** make probabilistic output deterministic. It makes the deterministic
code around probabilistic output more explicit and reviewable.

## Design pressure

The language therefore optimizes for:

- deterministic ordinary programs first;
- no ambient authority by default;
- explicit trust transitions;
- provider independence;
- local-first execution;
- traces and provenance as normal execution products.
