# 02.1 — Grammar and syntax discipline

> **Status:** Implemented / Proposed  
> **Summary:** The grammar is a contract between source text and parser, implemented by the parser and enforced by a checker; provisional agent syntax remains visibly provisional until accepted.

## Small syntax, strong semantics

NUDO's design goal is not to invent a keyword for every AI concept. Syntax is
expensive because every new form affects parsing, tooling, formatting, learning and
compatibility.

## Parser pressure

A recursive-descent parser works best when expression grammar is explicit about
precedence and avoids left recursion. A robust expression hierarchy should be
structured conceptually as:

```text
logical-or
  → logical-and
  → equality
  → comparison
  → additive
  → multiplicative
  → unary
  → postfix
  → primary
```

Postfix parsing is where calls, field access and indexing naturally compose.

## Contextual versus reserved words

Only words actually accepted as reserved tokens should be treated as reserved by
the lexer. Future syntax such as `agent`, `task`, `ask` and `verify` needs a NEP-level
decision before tools rely on it.
