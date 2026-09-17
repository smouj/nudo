# 00.3 — Current project status

> **Status:** Implemented / Planned  
> **Summary:** The repository is intentionally honest about the gap between implemented compiler stages and designed future stages.

## Implemented foundation

The working path is currently centered on source loading, spans, lexing, tokens,
diagnostics and lexical `nudo check` behaviour. The project also has repository,
CI, security, governance, conformance and documentation infrastructure.

## Planned compiler path

```text
Source
  ↓
Lexer             implemented
  ↓
Lossless syntax   planned
  ↓
Parser / AST      planned
  ↓
HIR / typecheck   planned
  ↓
Effect checking   planned
  ↓
MIR
  ↓
Interpreter / WASM
```

## Why status labels matter

A book about a pre-alpha language can easily become misleading because polished
examples look real. Every chapter therefore carries a status, and examples of
future syntax are described as proposed where appropriate.
