# 03.1 — Compiler pipeline

> **Status:** Specified  
> **Summary:** NUDO is structured as explicit compiler stages so each transformation has a testable responsibility.


{{#include ../diagrams/pipeline.svg}}

*The pipeline, with the accent marking what exists today.*

```text
SourceFile
   ↓
Lexer → Tokens + lexical diagnostics
   ↓
Lossless Syntax Tree
   ↓
Typed AST
   ↓
HIR + name resolution
   ↓
Type checking
   ↓
Effect checking
   ↓
MIR
   ↓
Interpreter / code generation
```

Sharp stage boundaries improve debugging and allow the formatter, language server
and documentation tools to share stable representations rather than reparsing text
with independent assumptions.

The architecture should remain flexible during pre-alpha. Placeholder crates are
not a reason to freeze an API before implementation teaches the project what the
right boundary is.
