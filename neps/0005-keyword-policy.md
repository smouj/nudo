# NEP-0005: Keyword policy

| Field | Value |
| ----- | ----- |
| Status | Accepted |
| Created | 2026-09-17 |
| Accepted | 2026-09-18; implemented in M2 |
| Supersedes | — |
| Superseded by | — |
| Specification chapters | [`spec/lexical-structure.md`](../spec/lexical-structure.md), [`spec/grammar.md`](../spec/grammar.md) |
| Related NEPs | NEP-0001, NEP-0002, NEP-0004 |

## Summary

Decide how a word becomes reserved: reserved outright, recognised only in
context, or a hybrid — and settle the status of the words the grammar already
uses (`struct`, `agent`, `task`, `tool`, `model`, `with`, `verify`, `ask`,
`delegate`, `if`, `match`, `true`, `false`, and the rest).

## Motivation

The lexer reserves exactly two words, `fn` and `let`. Every other word in
[`../grammar/nudo.ebnf`](../grammar/nudo.ebnf) is currently an ordinary
identifier, which was the right call while the language was being designed and is
the wrong call for a parser.

Two concrete problems, both already visible:

**A parse conflict.** The grammar has `literal = … | "true" | "false"` and
`path-expression = path`, where a path is an identifier. Because `true` and
`false` were not reserved, an input beginning with `true` could start either a
literal or a path, and the parser could not choose with one token of lookahead.
This was not hypothetical: it is visible in the grammar, and
`scripts/check-grammar.py` reported it before any parser existed. Reserving the two
words in M2 settled it, and the checker's motivating example became a case that
passes.

**A compatibility debt.** Every word reserved later breaks every program that
used it as a name. The longer the provisional period, the more example code,
documentation and test fixtures there are to change, and the more painful the
transition becomes. Deciding the *policy* now — not the final word list —
makes each later decision cheap.

## Guide-level explanation

The policy NUDO should adopt:

1. **A small reserved core.** Words that introduce a construct that cannot be
   written another way: `fn`, `let`, `struct`, `enum`, `if`, `else`, `match`,
   `const`, `true`, `false`. These are reserved, everywhere, always.
2. **Contextual everywhere else.** Declarations that appear at item position and
   in a fixed shape — `agent`, `task`, `tool`, `model`, `role`, `tools`,
   `allow`, `budget`, `verify`, `ask`, `delegate`, `with` — are recognised in
   their own position and remain usable as identifiers elsewhere.
3. **One rule, stated once.** A contextual word is a keyword only where the
   grammar expects that construct, and never changes the meaning of code outside
   it.

The user-visible effect: `agent` is reserved in item position, and a program may
still have a variable named `agent` — while `fn` can never be a variable name.

## Reference-level explanation

* The reserved set is closed and lives in `compiler/nudo-lexer`
  (`KEYWORDS`), because reserving a word is a lexical decision. Adding a word
  there is a NEP-level change.
* The contextual set lives in the parser: `nudo-parser` recognises
  `agent`/`task`/… when it is dispatching at item position, and treats the same
  word as a path segment elsewhere.
* Contextual recognition must not require unbounded lookahead. Each contextual
  construct is introduced by its word at a position where nothing else can
  appear, so one token suffices.
* Error messages must remain honest: when a contextual word is used where the
  construct is expected but the shape is wrong, the diagnostic says so, rather
  than reporting an unexplained identifier.
* `true` and `false` are reserved, which resolves the conflict above the moment
  the lexer is updated.

### Status of the words the grammar uses today

| Word | Proposed status | Rationale |
| ---- | --------------- | --------- |
| `fn`, `let` | Reserved (today) | Already reserved; no change |
| `true`, `false` | Reserved | Resolves the literal/path conflict |
| `struct`, `enum`, `if`, `else`, `match`, `const` | Reserved | Introduce constructs; the core vocabulary of the language |
| `agent`, `task`, `tool`, `model` | Contextual | Item-position declarations; plausible variable names elsewhere |
| `role`, `tools`, `allow`, `budget` | Contextual | Clause heads inside a declaration only |
| `with`, `verify`, `ask`, `delegate` | Contextual | Appear in fixed shapes; `with` must stay usable, and it is not an operator |
| Everything else | Identifier | No justification to reserve |

## Security implications

* An agent that rewrites source must be able to tell a keyword from an
  identifier without guessing. A contextual rule that depends on lookahead it
  cannot compute would make automated rewriting unsafe.
* A reserved word is a compatibility promise. Reserving a word already used in
  the wild breaks code silently at the *lexical* level, where a diff will not
  show it — so reserving requires a NEP and a migration note.
* Reserved words are checked mechanically. A list that can be edited casually is
  a list that will disagree with the grammar.

## Alternatives

| Alternative | Why it loses |
| ----------- | ------------ |
| Reserve everything the grammar mentions | Every listed word becomes unusable as a name, and the language is permanently stuck with names chosen before it had a design |
| Reserve nothing; make everything contextual | `if` and `match` in expression position would need lookahead to distinguish a construct from a path, and `true`/`false` cannot be contextual at all without ambiguity |
| Reserve nothing now; decide at 1.0 | Moves the worst of the pain to the moment the most code exists |
| A sigil for keywords (`@agent`) | Solves it, and costs readability for every reader forever to save a one-time migration |

## Drawbacks

* Two mechanisms for one concept is a real cost. Mitigated by keeping the
  contextual set small, listing it here, and requiring that a contextual word is
  only ever contextual in a position where nothing else can appear.
* Contextual keywords make the parser slightly harder to write and the error
  messages slightly harder to phrase well.
* A reader must know which mechanism applies to which word. The table above is
  the answer, and it is normative.

## Compatibility

* Reserving `true` and `false` breaks any program using those names. Nothing was
  implemented beyond the lexer when this NEP was accepted, so the cost was zero
then and grows every day it is deferred. That is the argument for taking it
  early, and it is why M2 adopted it as one of its two gates.
* Reserving the rest of the core breaks example code in `examples/`, which uses
  them only as keywords.
* After 1.0, adding a reserved word would require an edition.

## Unresolved questions

Settled when this NEP was accepted:

* **`with`, `verify` and `delegate` stay contextual.** Whether NEP-0002 and
  NEP-0004 make them keywords, functions or protocols is still open, and until
  then the parser recognises them in position. If `verify` becomes a keyword, the
  one-token rule that reads it today (see [`spec/expressions.md`](../spec/expressions.md))
  collapses into a reserved word and stops being a limitation.
* **The reserved set is a flat list**, ordered as the table above is. Grouping it
  by purpose would be documentation, not a lexical distinction, and a list that
  cannot disagree with its own structure cannot drift.
* **A contextual word in the wrong position is told what it needs.** `role` with
  no `:` produces `unexpected \`}\`` with the note “`role` needs a text literal,
  as in `role: "…"`” — an explanation, not an unexplained identifier.

Still open:

* Whether a future edition mechanism can *unreserve* a word. It probably cannot,
  which raises the cost of reserving too much today.
* How a diagnostic should describe a contextual keyword used in the wrong
  position once there is more than one such position.

## Implementation status

**Implemented** in M2. `compiler/nudo-lexer` reserves exactly the ten words this
NEP lists, and `spec/lexical-structure.md` states them as normative.
`compiler/nudo-parser` recognises the contextual set in position: `agent`, `task`,
`tool`, `model` at item position; `role`, `tools`, `allow`, `budget`, `agent`,
`verify` as clause heads; `ask`, `verify`, `delegate` as primaries; `with` where
an effect, verification or budget clause is expected.

`tests/conformance/parser/0012-contextual-words` pins the part that is easy to get
wrong: `agent`, `verify` and `delegate` remain usable as names inside a function
body.
