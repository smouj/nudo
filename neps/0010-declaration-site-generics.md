# NEP-0010: Declaration-site generic parameters

| Field | Value |
| ----- | ----- |
| Status | Accepted |
| Created | 2026-09-18 |
| Accepted | 2026-09-18, as gate 5 of M3.0 |
| Supersedes | — |
| Superseded by | — |
| Specification chapters | [`spec/generics.md`](../spec/generics.md), [`spec/declarations.md`](../spec/declarations.md), [`spec/grammar.md`](../spec/grammar.md) |
| Related NEPs | NEP-0006, NEP-0002, NEP-0008 |

## Summary

Resolve the contradiction NEP-0006 left behind: types are parameterised
(`Generated<T>`, `Result<Int, Text>`) but a **declaration could not declare its
own parameters**, so `enum Outcome<T, E>` was a syntax error and a program could
not write a generic type of its own. This NEP adds the declaration-site list, and
states that `Result<T, E>` is intrinsic rather than declared.

## Motivation

The contradiction was visible in the repository before it was visible in a
program:

```nudo
enum Outcome<T, E> {        // examples/04-results writes this
    Ok(value: T)
    Err(error: E)
}
```

against NEP-0006's scope: *no declaration-site parameters*. Two answers are
possible, and only one of them leaves a language rather than a vocabulary:

| Answer | Consequence |
| ------ | ----------- |
| Only the built-in types are generic | Users can *use* `Result<T, E>` and `Verified<T>` and can never declare their own parameterised type, so the language has two tiers of type and the second one is missing |
| Declarations declare parameters | One rule for built-in and user types, and the checker has one instantiation story |

The second is chosen. It is also the smaller surprise: every reader of `Result<T,
E>` already assumes they could have written it.

## Reference-level explanation

```ebnf
generic-parameter-list = "<", identifier, { ",", identifier }, ">" ;
```

* **The list follows the name**, on `fn`, `struct`, `enum`, `tool` and `task`
  declarations:

```nudo
fn identity<T>(value: T) -> T {
    value
}

enum Outcome<T, E> {
    Ok(value: T)
    Err(error: E)
}

struct Pair<A, B> {
    first: A
    second: B
}
```

* **One token of lookahead, as NEP-0006 established.** After a declaration's
  name, `<` can start nothing else, and in a type position a path followed by `<`
  starts a generic type. There is still no turbofish in expressions, so `<` in
  expression position is always the comparison operator.
* **Removing a parameter list is unthinkable in this edition**: a declaration
  states its parameters, and none are implicit.
* **Variance stays invariant** (NEP-0006). This NEP changes where parameters are
  *declared*, not how they relate.
* **No bounds.** `T: SomeCapability` is still not in the grammar: a bound that
  duplicates the effect system would be a second way to say one thing.
* **Arity is checked.** `Result<Int>` and `Pair<Int>` are type errors
  (`NDO2xxx`), reported at the type argument, not at the declaration.
* **`Result<T, E>` is intrinsic** ([NEP-0008](0008-error-model.md)): two variants,
  `Ok` and `Err`, part of the language rather than of `std/`. It has the same
  shape as a user enum, and the differences are that it cannot be shadowed and
  that fallible operations produce it.

## What this makes impossible

* A user type that stands in for the language's own fallible type: `Result` is
  intrinsic, so a program cannot redeclare it and confuse every caller.
* A generic declaration whose parameters are guessed from its body: parameters
  are written down, and there is no implicit generic anything.
* Constraint syntax duplicated from the effect system.

## Alternatives

| Alternative | Why it lost |
| ----------- | ----------- |
| Only built-in generics | Two tiers of type; a program could not write the abstraction it had just used |
| Bounds now (`T: Trait`) | There are no traits, and capability bounds would duplicate the effect system (M5) |
| Prefix form (`<T> fn f`) | Reads as a tag rather than as part of the name, and makes the declaration's name harder to find |
| Defer the whole question | The checker cannot be written without it: instantiation is a type-checking rule, not a parse detail |

## Unresolved questions

* How much inference is allowed at call sites and at `let` (the language leans
  explicit; generic arguments may need to be written).
* Whether a parameter may have a default.
* Whether specialisation exists. Probably not before 1.0.
* Whether `tool` and `task` declarations genuinely need parameters, or whether
  their inputs are always concrete.

## Implementation status

**Not implemented.** The grammar in [`grammar/nudo.ebnf`](../grammar/nudo.ebnf)
gains the production with this NEP; `nudo-parser` accepts it in M3.1, and the
checker's instantiation rules are M3.2. Until then, `examples/04-results` remains
a preview and says so.
