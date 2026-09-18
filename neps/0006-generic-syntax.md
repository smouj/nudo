# NEP-0006: Generic syntax

| Field | Value |
| ----- | ----- |
| Status | Accepted |
| Created | 2026-09-18 |
| Supersedes | — |
| Superseded by | — |
| Specification chapters | [`spec/generics.md`](../spec/generics.md), [`spec/types.md`](../spec/types.md), [`spec/grammar.md`](../spec/grammar.md) |
| Related NEPs | NEP-0002, NEP-0005 |

## Summary

Decide how a type is parameterised: `Generated<Article>`, `Result<Int, Text>`,
and nothing else. Angle brackets are accepted, type arguments are **invariant**,
and there is no bound syntax and no declaration-site parameter list in this
edition. The decision is deliberately narrow, because it is a parser decision
before it is a type-system one.

## Motivation

Three types already need parameters, and one of them is the reason the language
exists:

| Use | Needs |
| --- | ----- |
| `Generated<T>`, `Verified<T>` | A type parameter with no constraints |
| `[T]` | The same |
| `Result<T, E>` | Two parameters, and the ability to match on them |

`spec/generics.md` recorded the question and refused to answer it by accident.
The roadmap made the answer a gate for M2, because it decides one thing the
parser cannot guess: whether `Foo<…>` is a form the type grammar has to contain,
and therefore whether the parser needs a token of lookahead after a path.

## Guide-level explanation

```nudo
let draft: Generated<Article> = ask Writer { "create" };
let article: Verified<Article> = verify draft with ArticleVerifier;
let outcome: Result<Int, Text> = divide(10, 2);
let pages: [Text] = [];
```

A type argument is a type. There is no expression position that can start a type
argument, so no program contains a token sequence that could be read two ways:

* in **type position** — after `:`, after `->`, inside `[T]`, inside `Fn(…)` — a
  path followed by `<` starts a generic type;
* in **expression position**, `<` is only ever the comparison operator. There is
  no turbofish: `verify::<Article>(x)` is not a form of the language, and writing
  it is a syntax error rather than a guess.

That is the whole cost, and it is one token of lookahead in one production.

## Reference-level explanation

The production is already in [`grammar/nudo.ebnf`](../grammar/nudo.ebnf):

```ebnf
generic-type = path, "<", type, { ",", type }, ">" ;
```

Consequences that follow, and are normative:

* **Longest match at the lexer, not the parser.** `>>` is two `Gt` tokens, because
  NUDO has no shift operator. `Result<Result<Int, Text>, Text>` closes with `>`,
  `>` and needs no special case.
* **Arguments are always named.** A generic type is written with a path, never
  with a nested type in head position.
* **Variance is invariant.** `Generated<A>` and `Generated<B>` are unrelated even
  when `A` and `B` are related. This is a safety decision, not a convenience one:
  the trust types carry provenance, and a covariance mistake in `Verified<T>` is a
  soundness bug rather than an inconvenience. An invariant rule is the one that
  cannot be wrong by accident.
* **No bounds.** `T: SomeCapability` is not in the grammar. A bound that
  duplicates the effect system would be a second way to say the same thing, and
  the language has decided to have one.
* **No declaration-site parameters yet.** `enum Outcome<T, E>` is a **syntax
  error** today. The grammar has no place for a parameter list on a declaration,
  and adding one is a separate decision with separate consequences (it decides
  how inference must work). `examples/04-results` writes it, is marked as a
  preview, and says so.

## Security implications

* A type argument list is parsed where a type is expected, and nowhere else. That
  is what keeps `a < b` a comparison in every expression, without lookahead and
  without backtracking — and a parser that never backtracks cannot accept a
  program whose meaning depends on how hard it looked.
* Invariance keeps `Verified<T>` from being widened by a subtyping rule nobody
  wrote deliberately. In a language whose trust model is the reason for its
  existence, a variance mistake is a security bug.

## Alternatives

| Alternative | Why it loses |
| ----------- | ------------ |
| Square brackets: `Generated[Article]` | Collides with the sequence type `[T]`, so `[Result]` and `Result[Text]` would both need a rule to tell apart — and the collision is in the same grammar position |
| A prefix form: `Generated of Article` | Reads as prose, and needs a reserved word (`of`) that every program must then avoid |
| Turbofish in expressions | Adds a second place to look for a type argument, and with it an ambiguity with `<` that only a longer lookahead resolves |
| Defer the decision to M3 | It is a gate for M2: the parser cannot be written correctly without knowing whether the production exists. Deferring would mean writing the parser twice |

## Drawbacks

* Angle brackets are the most collision-prone choice at the *lexical* level, and
  the language pays for it by having no shift operator and no turbofish. That is a
  real cost, paid once, in exchange for a form every reader of the examples
  already understands.
* Invariance is the strictest rule, and the most annoying one when a program wants
  to pass `[Int]` where `[T]` is expected. It is the choice that cannot be
  silently wrong, and it can be relaxed later by a NEP that argues a specific case
  rather than by a variance rule that was never examined.
* Declaration-site parameters are still missing, so a user cannot write a generic
  type of their own. That is a visible gap, and it is named as one.

## Compatibility

Nothing is published, so nothing breaks. Reserving no new word is the point: the
decision adds *no* keyword, so no existing program loses a name.

## Unresolved questions

* Declaration-site parameters: `enum Outcome<T, E>` and `fn f<T>(x: T)`. This
  needs its own NEP, because it decides how much inference the language must
  have.
* Whether bounds exist at all once the effect system is implemented (M5).
* Whether `Verified<T>` needs a variance rule of its own once provenance is part
  of the type (M6).
* How a *generic* item is compiled, which is M4's problem and not a syntax one.

## Implementation status

**Implemented** in M2: `nudo-parser` parses `generic-type` as written, the tree
has a `GenericType` node, `nudo-ast` exposes it as `Type::Generic` with its
arguments, and `tests/conformance/parser/0011-types` pins it — including that
`>>` closes two levels.
