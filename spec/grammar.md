# Grammar

**State: provisional.** The pre-alpha lexer implements the token set in
[`lexical-structure.md`](lexical-structure.md). The parser does not exist
(milestone M2), so the grammar in [`../grammar/nudo.ebnf`](../grammar/nudo.ebnf)
describes the language NUDO is being designed towards, not a language that can
be compiled today.

That distinction is the point of this chapter: a grammar that pretends to be
implemented is worse than no grammar, because tools will be built against it.

## Where the grammar lives

* [`../grammar/nudo.ebnf`](../grammar/nudo.ebnf) — the normative EBNF.
* [`../grammar/syntax-reference.md`](../grammar/syntax-reference.md) — the same
  grammar in a form a person can read, with examples.

Both are specification artefacts. Changing either is a
[NEP](../neps/README.md)-level change when it alters what a program means, and a
normal pull request when it only clarifies.

## How to read the EBNF

```ebnf
(* comment *)
rule    = expression ;
```

| Notation | Meaning |
| -------- | ------- |
| `rule = … ;` | A production |
| `a b` | `a` followed by `b` |
| `a \| b` | `a` or `b` |
| `[ a ]` | `a` is optional |
| `{ a }` | zero or more repetitions of `a` |
| `"text"` | A literal token |
| `(* … *)` | A comment |

The grammar describes accepted syntax, not meaning. `[types.md](types.md)`
defines what the accepted forms mean, and a grammar that accepts something the
type system rejects is normal.

## Provisional markers

Productions whose surface syntax is **not** settled are marked in the EBNF with
`(* PROVISIONAL *)`. At the time of writing that includes:

| Production | Why it is not settled |
| ---------- | --------------------- |
| `agent-decl` | `role:`, `tools:` and `allow:` blocks may become ordinary fields |
| `task-decl` | Whether `verify:` is a clause or a value is open |
| `ask-expr` | Whether `ask` is a keyword, a function, or a protocol |
| `verify-expr` | Same, plus whether `with` is the right connective |
| `capability-set` | The syntax for granting several capabilities at once |
| `generic-params` | Whether NUDO has angle-bracket generics at all |

A production is removed from this table when a NEP decides it and conformance
cases exist.

## What the grammar may not do

* It may not accept a program the specification cannot describe. An unexplained
  production is an unfinished design, not a flexible one.
* It may not collide with the reserved-word list in
  [`lexical-structure.md`](lexical-structure.md). Adding a keyword is a breaking
  change and needs a NEP.
* It may not be the only place a rule is written down. If the grammar is the
  only description of a feature, the feature is underspecified.

## Relationship to the parser

When the parser exists, it will be a recursive-descent parser for this grammar,
and the two must agree:

* **Disagreement is a bug in the parser**, never in the grammar, unless a NEP
  changed the grammar in the same pull request.
* A production the parser accepts but the grammar does not is a silent language
  extension, which is exactly what [`../AGENTS.md`](../AGENTS.md) forbids.
* Every accepted production needs a conformance case in
  [`../tests/conformance/parser`](../tests/README.md).
