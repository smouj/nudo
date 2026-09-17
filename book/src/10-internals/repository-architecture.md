# 10.1 — Repository architecture

> **Status:** Implemented foundation  
> **Summary:** The monorepo keeps specification, compiler, runtime and tooling evolution in one auditable history while preserving conceptual crate boundaries.

The workspace separates compiler stages, runtime services, shared foundation
crates, CLI, tooling and backends. This makes ownership visible, but pre-alpha crate
boundaries should remain revisable.

A placeholder crate is a roadmap marker, not evidence that its public API is ready.
Implementation should be allowed to merge or split internal units when experience
shows a better boundary.
