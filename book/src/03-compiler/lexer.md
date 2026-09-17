# 03.2 — Source model and lexer

> **Status:** Implemented  
> **Summary:** The lexer is the first real language implementation layer and is responsible for deterministic tokenization plus recoverable lexical diagnostics.

A robust lexer must preserve byte positions, line/column mapping and source IDs so
later diagnostics can refer precisely to original source.

Important properties include:

- UTF-8 source loading;
- deterministic token streams;
- nested/block comment rules according to the lexical specification;
- malformed input recovery rather than panic-on-first-error;
- stable diagnostic codes;
- conformance cases that another implementation can reproduce.

The current `nudo check` success path is lexical only. A clean result does not yet
mean that a program is syntactically or semantically valid.
