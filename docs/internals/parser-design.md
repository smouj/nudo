# Parser design (M2)

**Status: decision document. Nothing here is implemented.**

This is the design M2 is built from. It exists before the parser so that the
decisions are reviewable while they are still cheap, and so that the shape of
`nudo-syntax`, `nudo-parser` and `nudo-ast` is argued rather than discovered.

Read [`../../grammar/nudo.ebnf`](../../grammar/nudo.ebnf) and
[`../../spec/grammar.md`](../../spec/grammar.md) first; this document assumes the
grammar's two rules (no left recursion, stated precedence).

## What M2 must produce

| Deliverable | Contract |
| ----------- | -------- |
| Lossless syntax tree | Every byte of the source is reachable from the tree, including whitespace and comments |
| Error nodes | Malformed input produces `Error` nodes, never a truncated tree |
| Recovery | Parsing continues after a bad `;`, `}`, `)` or item, and never cascades |
| Typed AST | `nudo-ast` wraps the tree for consumers that want structure, not text |
| Diagnostics | `NDO1xxx` with an expected/found pair and an exact span |
| Round trip | The formatter (M10) must be able to reprint the file byte-for-byte from the tree alone |

The last row is the reason the tree must be lossless. A parser that discards
whitespace cannot support a formatter, an editor, or a safe rewrite — and "safe
rewrite" is the whole reason an agent can be allowed near source code.

## Strategy

**Recursive descent with precedence climbing.** One function per grammar
production, one function per precedence level, and the level functions call the
next one down — exactly the shape of the declared chain.

Alternatives considered:

| Alternative | Why not |
| ----------- | ------- |
| Parser generator (LALRPOP, pest, chumsky) | Adds a dependency and moves the grammar out of `grammar/nudo.ebnf` into an attribute or a macro. The EBNF is a specification artefact; a second, machine-only grammar would be a second source of truth. |
| Pratt parser | Excellent for a tree of independent operators. NUDO's operators form a strict chain rather than a loose table, and precedence climbing expresses a chain with less machinery. |
| Recursive ascent / GLR | Needed for genuinely ambiguous grammars. NUDO's grammar is not ambiguous, and a parser that tolerates ambiguity hides grammar bugs instead of reporting them. |
| Hand-written with a separate lexer (chosen) | Keeps the existing `nudo-lexer` and its tested guarantees, keeps the grammar readable, and makes recovery a local decision rather than a recovery-mode inside a generated table. |

Precedence climbing is why the grammar's levels are *linear*: `parse_logical_or`
calls `parse_logical_and`, and so on down to `parse_primary`. A level function
parses one operand at the next level down, then loops while the lookahead is one
of its operators.

### Lookahead requirements

The parser needs one token of lookahead in exactly three places:

| Place | Why |
| ----- | --- |
| `path-type` vs `generic-type` | `Foo` and `Foo<…>` share a prefix |
| Item dispatch | `fn`, `struct`, `agent`, a reserved word, or an error |
| Statement dispatch | `let`, a block, or an expression |

Two-token lookahead is not required anywhere. That is a design constraint, not an
accident: a grammar that needs unbounded lookahead cannot be parsed by the
strategy above without backtracking, and backtracking destroys both the error
messages and the recovery story.

## Tree representation

The decision that matters most, and the one most likely to be revisited.

| Option | Cost | Gets us | Verdict |
| ------ | ---- | ------- | ------- |
| **rowan** (the rust-analyzer library) | One dependency, plus its own conventions (`SyntaxKind` as `u16`, green/red split) | Mature, cheap sharing, the exact shape an editor wants, incremental reparse as a later addition | **Not now.** It is the right answer *if* incremental reparse becomes a requirement. Adopting it in M2 means taking a dependency for a property — incrementality — that nothing in M2 needs, and it would be the project's first third-party dependency. |
| **Hand-rolled immutable tree** (`Rc<SyntaxNode>` or an arena) | More code, our own invariants to test | No dependency, full control of the API, easy to keep the "every byte reachable" property explicit | **Chosen for M2.** |
| **Concrete syntax tree with typed wrappers only** | Least code | Structure without losslessness | No. It cannot support the formatter, and the formatter is a stated M2 exit criterion. |

### Shape

```text
nudo-syntax
  SyntaxKind    one enum, every node and token kind
  SyntaxTree    the root, owning the source text and the node arena
  SyntaxNode    a handle: kind, span, parent, children
  SyntaxToken   a handle: kind, span, text
```

* Nodes are stored in one arena inside the tree; handles are indices, so the tree
  is cheap to move and needs no reference counting.
* Trivia (whitespace, comments) is attached to the *following* token, which is
  the convention that makes reprinting straightforward.
* Every node's span covers exactly its children's spans; the root's span is the
  whole file.
* An `Error` node holds whatever tokens the parser could not place, and is
  reachable so that the formatter can reprint it and the editor can highlight it.

### The invariant that makes it lossless

For any source file:

```text
concatenating the text of every token in the tree, in order, equals the file
```

That is a property test, not a comment. If it holds, no byte was dropped, and a
formatter built on the tree can reproduce the original.

## Recovery

Recovery is a feature with its own contract, because a parser that reports fifty
cascading errors after one missing brace is unusable — for a person and for an
agent trying to repair the file.

* **Progress guarantee.** Every recovery step consumes at least one token or
  stops. A recovery that loops is a hang, and a hang on user input is a bug.
* **Sync points.** `;`, `}`, `)`, `]`, and item-starting words. On an unexpected
  token, the parser reports once, then skips to the next sync point.
* **Never synthesise.** A missing token is reported as missing; the parser does
  not insert one silently, because a silent insertion makes the tree disagree
  with the file.
* **One diagnostic per mistake.** A bad expression reports at its start, not once
  per token it skipped.
* **Bounded output.** A file of garbage produces a bounded number of diagnostics,
  not one per token.

## Diagnostics

`NDO1001 UNEXPECTED_TOKEN` is already reserved in
[`../../spec/errors.md`](../../spec/errors.md). The parser emits it with:

* the span of the offending token;
* a `label` naming what was found;
* the expected set, as a note;
* no help text unless there is a specific, valid suggestion.

Rendering stays in `nudo-diagnostics`. The parser constructs
[`nudo-diagnostics`](../../compiler/nudo-diagnostics) values and prints nothing.

## Testing

| Level | What it covers |
| ----- | -------------- |
| Fixtures | One rule per file, expectations declared in the file (`fixtures/README.md`) |
| Conformance | Language-visible behaviour, implementation-neutral, in `tests/conformance/parser/` |
| Property: losslessness | Token texts re-concatenate to the original file, for every input |
| Property: robustness | Random and malformed input never panics, always terminates, and always produces a tree |
| Property: recovery | A file with N independent mistakes produces about N diagnostics, not N² |
| Round trip | `parse → print` is the identity, byte for byte (the M10 formatter's own test, prepared now) |

The property tests follow the pattern already used for the lexer
(`compiler/nudo-lexer/tests/robustness.rs`): a fixed-seed generator, printed on
failure, no third-party harness.

**Every parser bug becomes a permanent conformance case.** A bug that was found
once and not added to the corpus will be found again.

## Decisions required before implementation

These block a *correct* parser, not a compiling one. Each is a NEP:

| # | Decision | Why it blocks |
| - | -------- | ------------- |
| 1 | Keyword policy ([NEP-0005](../../neps/0005-keyword-policy.md)) | `true` and `false` currently lex as identifiers, so `literal` and `path-expression` overlap. Item dispatch needs to know which words are reserved. |
| 2 | Generic syntax | Whether `Foo<A>` exists decides whether the type parser needs the one-token lookahead above, or a different syntax entirely. |
| 3 | `verify` form ([NEP-0002](../../neps/0002-generated-verified.md)) | Whether `verify` is a keyword, a function or a protocol changes a primary production. |
| 4 | Mutability | Whether `let` is the only binding form, and therefore whether an `assignment` level exists. |

Decisions 1 and 2 are cheap and are recommended first.

## Non-goals for M2

* **Incremental reparse.** An editor wants it; M2 does not need it. The tree API
  is designed so a rowan-backed implementation could satisfy it later without
  changing consumers.
* **Formatting.** M10 owns the formatter. M2 only guarantees that the tree makes
  it possible.
* **Semantic analysis.** Names, types and effects are M3 and later. The parser
  does not resolve anything.
* **Error-tolerant AST for every consumer.** `nudo-ast` exposes what is
  unambiguously well-formed; the syntax tree exposes everything, including
  errors.

## First block of work

1. `nudo-syntax`: `SyntaxKind`, the arena, the losslessness property test.
2. `nudo-parser`: item dispatch, `fn`, `let`, blocks, statements.
3. The expression chain, one level per function, in the declared order.
4. Recovery, with the progress guarantee test.
5. `nudo-ast` wrappers for functions, bindings and expressions.
6. `nudo check` runs the parser and reports `NDO1xxx`; `--dump-tree` (provisional
   name) prints the tree in a stable format, as `--dump-tokens` does today.
7. Conformance cases and fixtures for all of the above.

Steps 1–3 are a vertical slice that can be reviewed on its own, and step 6 is the
point at which `nudo check` stops meaning "lexically correct".
