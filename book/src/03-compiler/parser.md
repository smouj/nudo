# 03.3 — Parser, lossless syntax and AST

> **Status:** Planned  
> **Summary:** M2 should turn the token stream into a lossless syntax structure, recover from malformed input and expose a typed AST for later semantic stages.

## Why lossless syntax first

A lossless tree retains trivia and every source byte. That property matters for a
formatter, editor tooling and automated agents that need to modify code without
silently destroying comments or layout.

## Error recovery

A useful parser does not stop at the first missing token. Recovery points should
include item boundaries, semicolons and closing delimiters so one mistake does not
turn the rest of a file into noise.

## Typed AST

The AST should give downstream passes semantic shapes (`Function`, `Call`, `Type`)
without requiring them to know raw syntax-node details. Conversions from syntax to
AST must remain span-preserving.
