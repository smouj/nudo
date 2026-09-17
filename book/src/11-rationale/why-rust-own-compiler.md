# 11.1 — Why Rust and why a compiler of its own?

> **Status:** Specified rationale  
> **Summary:** Rust provides a strong implementation substrate, while a dedicated compiler is necessary if NUDO semantics are meant to exist independently of another host language.

## Why Rust for the implementation

Rust offers explicit ownership, strong static checking, good performance,
well-developed parser/compiler tooling and practical cross-platform distribution.
That does not make Rust part of NUDO's language semantics; it is the implementation
language.

## Why not compile to Python as the definition?

If NUDO semantics were simply Python semantics plus syntax, Python would become the
real specification. Features such as trust types, capability checks and stable
compiler diagnostics would inherit host-language constraints.

A dedicated compiler lets the NUDO specification remain the authority. Backends can
change later without redefining what a program means.
