# 03.3 — Parser, lossless syntax and AST

> **Status:** Implemented  
> **Summary:** M2 turned the token stream into a lossless syntax structure, recovered from malformed input, and exposed a typed AST for later semantic stages.

## Why lossless syntax first

A lossless tree retains trivia and every source byte. That property matters for a
formatter, editor tooling and automated agents that need to modify code without
silently destroying comments or layout.

The parser keeps it as a checked property rather than an intention. Every fixture,
every conformance case and every property test asserts that concatenating the
text of the tree's tokens reproduces the file byte for byte. `nudo check
--dump-tree` prints the tree, and the dump is longer than the file it came from
precisely because whitespace and comments are in it.

That honest cost is the reason the tree is shaped the way it is. `nudo-ast`
wraps the tree in typed values — a `Function` with a name and a body, a
`Binding` with a value — and returns nothing when the tree does not hold the
part. A consumer that needs to be sure, such as a formatter deciding whether it
may rewrite, has to be able to tell "this is a function" from "this looked like
one until the eighth token".

## Error recovery

A useful parser does not stop at the first missing token. Recovery points include
item boundaries, semicolons and closing delimiters so one mistake does not turn
the rest of a file into noise.

Three rules make that concrete, and each one came out of a failure the tests
found:

* **Recovery always consumes a token or stops.** A recovery that loops is a hang,
  and a hang on user input is a bug.
* **Recovery reports before it skips.** An early version skipped what it could not
  place, silently, and `nudo check` then exited `0` on a file it had not
  understood. That is worse than rejecting the file, because the caller believes
  it was read.
* **One position, one diagnostic.** If the lexer already rejected a byte, the
  parser does not report the statement that byte made unreadable. One character
  produces one message, and an agent repairing the file fixes one byte.

A token the parser cannot place is kept in an `Error` node rather than dropped,
so the file can still be reprinted — including the parts that are wrong.

## Typed AST

The AST gives downstream passes semantic shapes (`Function`, `Call`, `Type`)
without requiring them to know raw syntax-node details. Conversions from syntax to
AST remain span-preserving, and a node's span begins exactly at its first token.

Nothing in `nudo-ast` resolves a name or checks a type: a path is a list of names,
not a reference. That is milestone M3, and the parser deliberately knows nothing
about it.

## What is deliberately missing

* Nothing, right now: declaration-site generic parameters (`enum Outcome<T, E>`)
  and `verify`'s named operand both parse since M3.1, and each has a conformance
  case that pins it. What is still missing from the language is `return`, which
  the EBNF never had.
* The grammar's paths use `::` and its lists are comma-separated, while several
  examples in the manual and in `examples/` write `web.search` one per line. The
  parser follows the grammar; the examples say they are previews, and which
  spelling the language should have is a NEP, not a parser bug.
