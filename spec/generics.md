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

* **Declaration-site parameters.** `enum Outcome<T, E>` and `fn f<T>(x: T)` are
  **syntax errors** today: the grammar has no place for a parameter list on a
  declaration, and adding one decides how much inference the language must have.
  It needs its own NEP, and
  [`examples/04-results`](../examples/04-results/main.nudo) is a preview that
  writes it anyway.
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
* **Variance: invariant.** `Generated<A>` and `Generated<B>` are unrelated even
  when `A` and `B` are related. A variance mistake in `Verified<T>` would be a
  soundness bug, and invariance is the rule that cannot be wrong by accident.
* Trust types are generic: `Generated<T>` and `Verified<T>` work for any `T`.
* Generic parameters are declared explicitly at declaration sites — which is a
  statement about the shape of the language, not about syntax that exists yet.
* A generic type parameter may not be used to smuggle a capability: `fn f<T>(x:
  T)` cannot perform effects by virtue of `T`.

## Process

Done. The NEP this chapter asked for is
[NEP-0006](../neps/0006-generic-syntax.md): it picks a syntax and a variance rule,
with the parser cost of the syntax stated as part of the decision. What remains
open — declaration-site parameters, bounds, inference, specialisation — needs
further NEPs, and until then this chapter is the place that says so.
