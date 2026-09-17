# Summary

[The NUDO Technical Book](index.md)

# 00 — Introduction
- [What is NUDO?](00-introduction/what-is-nudo.md)
- [Why NUDO exists](00-introduction/why-exists.md)
- [Current project status](00-introduction/status.md)

# 01 — Philosophy
- [Deterministic software first](01-philosophy/deterministic-first.md)
- [Explicit trust](01-philosophy/explicit-trust.md)
- [Local-first and provider-agnostic](01-philosophy/local-first.md)

# 02 — Language design
- [Grammar and syntax discipline](02-language-design/grammar.md)
- [Expressions and precedence](02-language-design/expressions.md)
- [Modules and program boundaries](02-language-design/modules.md)

# 03 — Compiler
- [Compiler pipeline](03-compiler/pipeline.md)
- [Source model and lexer](03-compiler/lexer.md)
- [Parser, lossless syntax and AST](03-compiler/parser.md)
- [Diagnostics as product design](03-compiler/diagnostics.md)

# 04 — Type system
- [Type-system fundamentals](04-type-system/fundamentals.md)
- [`Generated<T>` and `Verified<T>`](04-type-system/generated-verified.md)

# 05 — Agents and AI
- [Agents as bounded executors](05-agents-and-ai/agents.md)
- [Tasks, tools and models](05-agents-and-ai/tasks-tools-models.md)

# 06 — Security
- [Capabilities, effects and authority](06-security/capabilities-effects.md)
- [Threat model](06-security/threat-model.md)

# 07 — Runtime
- [Traces and provenance](07-runtime/trace-provenance.md)
- [Budgets and approvals](07-runtime/budgets-approvals.md)

# 08 — Tooling
- [Toolchain and CLI](08-tooling/cli.md)

# 09 — Interoperability
- [MCP, A2A and WebAssembly](09-interoperability/mcp-a2a-wasm.md)

# 10 — Internals
- [Repository architecture](10-internals/repository-architecture.md)

# 11 — Design rationale
- [Why Rust and why a compiler of its own?](11-rationale/why-rust-own-compiler.md)
- [Why trust belongs in the type story](11-rationale/why-trust-types.md)

# 12 — Alternatives rejected
- [Alternatives deliberately rejected](12-alternatives/rejected.md)

# 13 — Examples
- [Ordinary deterministic program](13-examples/ordinary-program.md)
- [Model output and verification flow](13-examples/trust-flow.md)

# 14 — Contributing
- [Specification, NEPs and ADRs](14-contributing/spec-nep-adr.md)

# 15 — History
- [Milestones and evidence](15-history/milestones.md)
