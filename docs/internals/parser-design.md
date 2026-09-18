# Parser design (M2)

**Status: implemented.** `nudo-syntax`, `nudo-parser` and `nudo-ast` exist and
are covered by `tests/conformance/parser`, `fixtures/` and the property tests
next to each crate. This document is kept as the record of *why* the parser has
the shape it has, and of what was deliberately left out; the code is the
implementation of it, not a description in a separate place.

Read [`../../grammar/nudo.ebnf`](../../grammar/nudo.ebnf) and
[`../../spec/grammar.md`](../../spec/grammar.md) first; this document assumes the
grammar's two rules (no left recursion, stated precedence).

## What M2 produced

| Deliverable | Contract | Where |
| ----------- | -------- | ----- |
| Lossless syntax tree | Every byte of the source is reachable from the tree, including whitespace and comments | `compiler/nudo-syntax` |
| Error nodes | Malformed input produces `Error` nodes, never a truncated tree | `compiler/nudo-syntax`, `nudo-parser` |
| Recovery | Parsing continues after a bad `;`, `}`, `)` or item, never cascades, never loops | `compiler/nudo-parser` |
| Typed AST | `nudo-ast` wraps the tree for consumers that want structure, not text | `compiler/nudo-ast` |
| Diagnostics | `NDO1001` with an expected/found pair and an exact span | `compiler/nudo-diagnostics` |
| Round trip | `parse → print` is the identity, byte for byte | `SyntaxTree::reprint`, checked by every conformance case |

`nudo check` now lexes **and parses**; `--dump-tree` prints the tree in a stable
format, as `--dump-tokens` prints tokens.

## Strategy

**Recursive descent with precedence climbing**, one function per grammar
production and one per precedence level, each calling the next one down. Built as
designed.

Alternatives considered:

| Alternative | Why not |
| ----------- | ------- |
| Parser generator (LALRPOP, pest, chumsky) | Adds a dependency and moves the grammar out of `grammar/nudo.ebnf` into an attribute or a macro. The EBNF is a specification artefact; a second, machine-only grammar would be a second source of truth. |
| Pratt parser | Excellent for a tree of independent operators. NUDO's operators form a strict chain rather than a loose table, and precedence climbing expresses a chain with less machinery. |
| Recursive ascent / GLR | Needed for genuinely ambiguous grammars. NUDO's grammar is not ambiguous, and a parser that tolerates ambiguity hides grammar bugs instead of reporting them. |
| Hand-written with a separate lexer (chosen) | Keeps the existing `nudo-lexer` and its tested guarantees, keeps the grammar readable, and makes recovery a local decision rather than a recovery-mode inside a generated table. |

### Lookahead requirements

The parser needs one token of lookahead, and uses it in exactly the three places
this document predicted:

| Place | Why |
| ----- | --- |
| `path-type` versus `generic-type` | `Foo` and `Foo<…>` share a prefix |
| Item dispatch | `fn`, `let`, `const`, `struct`, `enum`, a contextual word, or an error |
| Statement dispatch | `let`, or an expression |

No two-token lookahead, and no backtracking anywhere.

## Tree representation

| Option | Cost | Gets us | Verdict |
| ------ | ---- | ------- | ------- |
| **rowan** | One dependency, plus its own conventions | Mature, cheap sharing, the shape an editor wants, incremental reparse later | **Not now.** It is the right answer *if* incremental reparse becomes a requirement; adopting it in M2 would take the project's first third-party dependency for a property nothing needs yet. |
| **Hand-rolled immutable tree** (arena) | More code, our own invariants to test | No dependency, full control, the losslessness property made explicit | **Chosen, and built.** |
| **Concrete syntax tree with typed wrappers only** | Least code | Structure without losslessness | No. It cannot support the formatter, which is a stated M2 exit criterion. |

### Shape

```text
nudo-syntax
  SyntaxKind    one enum, every node and token kind
  SyntaxTree    the root, owning the source text, the tokens and the node arena
  SyntaxNode    a handle: kind, span, children
  SyntaxToken   a handle: kind, span, text
  TreeBuilder   how the parser builds a tree
  dump_tree     the stable text format the corpus compares
```

Two decisions were added during implementation, because the design as written
could not express them:

* **Checkpoints.** A left operand is already a sibling of the operator that
  should own it by the time the operator is seen. `TreeBuilder::checkpoint`
  marks a position in the node being built, and `start_node_at` wraps everything
  added since then in a new node. Without it, `a + b * c` and `a.b(c)[d]` cannot
  be built with the correct shape.
* **Trivia belongs to the enclosing node.** Trivia is still attached to the
  following token, but a node opened at that token first lets the trivia land in
  its parent. So `PathType` for ` Int` is `Int`, not ` Int`, and a node's span
  begins exactly at its first token. Comments written above an item therefore
  belong to the file, which is where a later formatter can find them.

### The invariant that makes it lossless

```text
concatenating the text of every token in the tree, in order, equals the file
```

This is checked as a property (`SyntaxTree::is_lossless`), by every conformance
case, by every fixture, and by `SyntaxTree::validate`, which also checks that
every token appears exactly once, in order, and that no node is unreachable.

Making it true for *every* file, including broken ones, forced one change in the
lexer: a character that starts no token used to be reported and dropped. It is now
reported **and kept** as an `Unknown` token. A byte that belongs to no token is a
byte the tree cannot account for, and that is precisely the case — a file a
formatter or a repair tool has to round-trip — where dropping it is worst.

## Recovery

Built as designed, with three rules that are tested rather than intended:

* **Progress guarantee.** Every recovery step consumes at least one token or
  stops. `parsing_never_hangs_and_always_produces_a_tree` runs a list of
  pathological inputs through the parser.
* **Never synthesise.** A missing token is reported as missing; the parser does
  not insert one. The tree therefore always agrees with the file.
* **Bounded output.** At most [`MAX_ERRORS`] syntax diagnostics per file
  (24), and at most one diagnostic per position.

Two rules came out of testing, and both fix a failure mode that is worse than a
crash:

* **Report before you skip.** An early version skipped unplaceable tokens
  silently, which made `nudo check` exit **0** on a file it had not understood —
  see `examples/05-agent` before M2. Every recovery that follows a token a loop
  could not start now reports once, then skips.
* **One position, one diagnostic.** If the lexer already rejected a byte, the
  parser does not also report the construct that byte made unreadable. Without
  this, `let x = @;` produced two errors for one character, and an agent
  repairing the file would fix the same byte twice.

## Diagnostics

`NDO1001 UNEXPECTED_TOKEN` is emitted with the span of the offending token, the
token in the message and the expected set as a note:

```text
error[NDO1001]: unexpected `}`
  --> main.nudo:3:1
  |
3 | }
  | ^
  = note: expected `;`
```

A contextual word with the wrong shape after it says what it needs rather than
leaving an unexplained identifier: `` = note: `role` needs a text literal, as in
`role: "…"` ``. Rendering stays in `nudo-diagnostics`; the parser prints nothing.

There is also `MAX_DEPTH` (96 nested blocks or expressions). The parser is
recursive, so without a limit a file of open parentheses would exhaust the stack
instead of producing a diagnostic.

## Known limitations

Recorded here rather than discovered by whoever writes the formatter.

* **`verify (expr) with V` is rejected.** `verify` is a contextual word, so the
  parser decides from one token whether `verify` is being used as a name or is
  starting a verification. It reads a verification when the next token starts an
  operand and is not `(`, `.` or `[`. `verify draft with V` works,
  `verify(draft)` is a call, and `verify (draft) with V` is a syntax error. The
  limitation disappears when NEP-0002 decides whether `verify` is a keyword, a
  function or a protocol.
* **Declaration-site generic parameters are not in the grammar**, so
  `enum Outcome<T, E>` is rejected. That is NEP-0006's deliberately narrow
  scope, not an oversight; see its unresolved questions.
* **The grammar's `path` uses `::`, but the examples write `web.search`,** and a
  list inside a declaration is comma-separated while the examples write one per
  line. The parser follows the grammar; the examples are marked as previews and
  say which syntax stops them. Whether the spelling should change is a NEP.
* **Error nodes are not typed.** `nudo-ast` returns `None` for anything it cannot
  cast, which is deliberate: a consumer that needs to be sure has to be able to
  tell "this is a function" from "this looked like one until the eighth token".

## Testing

| Level | What it covers | Where |
| ----- | -------------- | ----- |
| Fixtures | One rule per file, expectations declared in the file | `compiler/nudo-parser/tests/fixtures.rs` |
| Conformance | Language-visible behaviour, implementation-neutral | `tests/conformance/parser/` (13 cases) |
| Property: losslessness | Token texts re-concatenate to the original file | every fixture and every conformance case |
| Property: recovery | Malformed input never panics, always terminates, always produces a tree | `compiler/nudo-parser/src/lib.rs` unit tests |
| Property: bounded diagnostics | A file of nonsense produces a bounded number | same |
| Examples | A preview is rejected; a readable example is accepted | `compiler/nudo-parser/tests/examples.rs` |

**Every parser bug becomes a permanent conformance case.** The bugs found while
building M2 are in the corpus: `0008-comparison-does-not-chain`,
`0009-missing-semicolon`, `0010-recovery`, `0012-contextual-words`,
`0013-error-node`.

## Decisions required before implementation

The four decisions this document listed as gates, and where they stand:

| # | Decision | State |
| - | -------- | ----- |
| 1 | Keyword policy ([NEP-0005](../../neps/0005-keyword-policy.md)) | **Accepted and implemented.** A ten-word reserved core, everything else contextual. |
| 2 | Generic syntax | **Accepted and implemented** ([NEP-0006](../../neps/0006-generic-syntax.md)): angle brackets, invariant arguments, no bounds, no declaration-site parameters. |
| 3 | `verify` form ([NEP-0002](../../neps/0002-generated-verified.md)) | **Still open.** The parser implements the grammar as frozen and records the one-token limitation above. If NEP-0002 makes `verify` a keyword, the limitation disappears. |
| 4 | Mutability | **Still open.** There is no assignment level in the grammar, so there is nothing for the parser to do; when mutability is decided it adds a level above `logical-or`. |

Decisions 3 and 4 do not block a correct parser for the frozen grammar: the
parser implements what the grammar says today, and a NEP that changes the grammar
changes the parser in the same pull request.

## Non-goals for M2

* **Incremental reparse.** An editor wants it; M2 does not need it, and the tree
  API is designed so a rowan-backed implementation could satisfy it later without
  changing consumers.
* **Formatting.** M10 owns the formatter. M2 guarantees the tree makes it
  possible — and proves it, by reprinting every fixture and conformance case.
* **Semantic analysis.** Names, types and effects are M3 and later. The parser
  resolves nothing: a `Path` is a list of names, not a reference.
* **Error-tolerant AST for every consumer.** `nudo-ast` exposes what is
  unambiguously well-formed; the syntax tree exposes everything, including errors.

## First block of work

Done, in this order:

1. `nudo-syntax`: `SyntaxKind`, the arena, the losslessness property test.
2. `nudo-parser`: item dispatch, `fn`, `let`, blocks, statements.
3. The expression chain, one level per function, in the declared order.
4. Recovery, with the progress guarantee test.
5. `nudo-ast` wrappers for functions, bindings and expressions.
6. `nudo check` runs the parser and reports `NDO1xxx`; `--dump-tree` prints the
   tree in a stable format, as `--dump-tokens` does today.
7. Conformance cases and fixtures for all of the above.
