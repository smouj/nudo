# Generics

**State: the syntax is decided and implemented; the rest is unsettled.**
[NEP-0006](../neps/0006-generic-syntax.md) settles how a type is parameterised —
angle brackets, invariant arguments — and `compiler/nudo-parser` accepts it. How
much generics can *express* is still open, and this chapter says so rather than
inventing it.

## What is needed

Three uses already require generic types:

| Use | Needs |
| --- | ----- |
| `Generated<T>`, `Verified<T>` | A type parameter with no constraints |
| `[T]` | The same |
| `Result<T, E>` | Two parameters, and the ability to match on them |

So a form of type parameterisation exists in the language's future. The
question is only what it looks like and how much it can express.

## What is not decided

* **Bounds.** Whether `T: SomeCapability` exists, and how a bound interacts with
  the effect system rather than duplicating it. The grammar has no bound syntax,
  on purpose.
* **Inference.** How much inference is allowed. The rest of the language leans
  towards explicitness at declaration sites, and generics are where that
  principle is most expensive to hold.
* **Specialisation.** Whether it exists at all. The answer is probably no before
  1.0, and if it arrives it needs a NEP that says what it costs.
* **How a generic item is compiled**, which is M4's problem rather than a syntax
  one.

## What is decided

* **Syntax: angle brackets.** `Generated<Article>`, `Result<Int, Text>`. A type
  argument list is parsed only where a type is expected, so `<` in an expression
  is always a comparison and the parser needs one token of lookahead, not
  backtracking. There is no turbofish. [NEP-0006](../neps/0006-generic-syntax.md)
  argues the alternatives that lost.
* **Declarations declare their parameters**
  ([NEP-0010](../neps/0010-declaration-site-generics.md)): `fn identity<T>(value:
  T) -> T`, `enum Outcome<T, E> { … }`, `struct Pair<A, B> { … }`. A program can
  write a parameterised type of its own, and there is one instantiation story
  rather than two tiers of type. Arity is checked, and a mismatch is `NDO2xxx`.
* **`Result<T, E>` is intrinsic**: two variants, `Ok` and `Err`, part of the
  language rather than of `std/`, and it cannot be shadowed. It is the type of
  every fallible operation ([NEP-0008](../neps/0008-error-model.md)).
* **Variance: invariant.** `Generated<A>` and `Generated<B>` are unrelated even
  when `A` and `B` are related. A variance mistake in `Verified<T>` would be a
  soundness bug, and invariance is the rule that cannot be wrong by accident.
* Trust types are generic: `Generated<T>` and `Verified<T>` work for any `T`.
* No bounds, and no implicit generic anything.
* A generic type parameter may not be used to smuggle a capability: `fn f<T>(x:
  T)` cannot perform effects by virtue of `T`.

## Process

Done, in two steps. [NEP-0006](../neps/0006-generic-syntax.md) decided how a type
is parameterised, and [NEP-0010](../neps/0010-declaration-site-generics.md)
decided where parameters are declared and that `Result` is intrinsic. What
remains open — bounds, inference, specialisation — needs further NEPs, and until
then this chapter is the place that says so.
