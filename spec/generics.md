# Generics

**State: proposed and unsettled.** This chapter is short on purpose: the design
is not decided, and writing more would be inventing it.

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

* **Syntax.** `Generated<Article>` (angle brackets) is what the examples use,
  and it collides with comparison operators in a parser unless the parser is
  careful. An alternative form may be chosen.
* **Bounds.** Whether `T: SomeCapability` exists, and how a bound interacts with
  the effect system rather than duplicating it.
* **Inference.** How much inference is allowed. The rest of the language leans
  towards explicitness at declaration sites, and generics are where that
  principle is most expensive to hold.
* **Variance.** Whether `Generated<A>` is compatible with `Generated<B>` when
  `A` and `B` are related. Trust types make this a safety question, not merely a
  convenience question: a variance mistake in `Verified<T>` is a soundness bug.
* **Specialisation.** Whether it exists at all. The answer is probably no before
  1.0, and if it arrives it needs a NEP that says what it costs.

## What is decided

* Trust types are generic: `Generated<T>` and `Verified<T>` work for any `T`.
* Generic parameters are declared explicitly at declaration sites. There is no
  implicit generic anything.
* A generic type parameter may not be used to smuggle a capability: `fn f<T>(x:
  T)` cannot perform effects by virtue of `T`.

## Process

The next step is a NEP that picks a syntax and a variance rule, with worked
examples — including the parser cost of whatever syntax it chooses. Until then,
this chapter exists so that contributors know the question is open and do not
each answer it differently in their own pull request.
