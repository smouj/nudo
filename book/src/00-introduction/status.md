# 00.3 — Current project status

> **Status:** Implemented / Planned  
> **Summary:** The repository is intentionally honest about the gap between implemented compiler stages and designed future stages.

## Implemented foundation

The working path is currently centred on source loading, spans, lexing, tokens,
diagnostics, the lossless syntax tree, the parser, the typed AST, and `nudo check`
over all of it. The project also has repository, CI, security, governance,
conformance and documentation infrastructure.

## Planned compiler path

```text
Source
  ↓
Lexer             implemented
  ↓
Lossless syntax   implemented
  ↓
Parser / AST      implemented
  ↓
HIR / typecheck   planned
  ↓
Effect checking   planned
  ↓
MIR
  ↓
Interpreter / WASM
```

"Implemented" means a stage has behaviour and tests today. A clean `nudo check`
run means the file lexes and parses; it does not mean the program is correct, and
it does not run.

## Why status labels matter

A book about a pre-alpha language can easily become misleading because polished
examples look real. Every chapter therefore carries a status, and examples of
future syntax are described as proposed where appropriate.
