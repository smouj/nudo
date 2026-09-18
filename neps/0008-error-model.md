# NEP-0008: The error model

| Field | Value |
| ----- | ----- |
| Status | Accepted |
| Created | 2026-09-18 |
| Accepted | 2026-09-18, as gate 3 of M3.0 |
| Supersedes | — |
| Superseded by | — |
| Specification chapters | [`spec/types.md`](../spec/types.md), [`spec/expressions.md`](../spec/expressions.md), [`spec/errors.md`](../spec/errors.md) |
| Related NEPs | NEP-0002, NEP-0007 |

## Summary

Decide the whole error story once: **`Result<T, E>` is intrinsic**, failure is a
**value**, a **trap** is not, there is **no `?`** in this edition, and the three
things a reader can meet — a compile diagnostic, a `Result`, a trap — are
distinguished by their code family.

## Motivation

"Errors are values, and there are no exceptions" was already a principle, and it
was not yet a decision: the shape of `Result`, the fate of `?`, and what happens
to a division by zero were all open. Each one changes what the type checker
does, so they belong before it.

## Guide-level explanation

```nudo
fn divide(a: Int, b: Int) -> Result<Int, MathError> {
    if b == 0 {
        Result::Err(MathError::DivisionByZero)
    } else {
        Result::Ok(a / b)
    }
}

match divide(10, 2) {
    Ok(value) => report(value)
    Err(reason) => report(reason)
}
```

Three kinds of failure, and a reader can tell them apart from the code:

| Kind | Looks like | Who handles it |
| ---- | ---------- | -------------- |
| The program is wrong | a compile diagnostic (`NDO1xxx`, `NDO2xxx`, …) | the author, before it runs |
| The operation can fail | `Result<T, E>` | the caller, in the source |
| The program cannot continue | a trap (`NDO6xxx`) | nobody: the task ends |

## Reference-level explanation

* **`Result<T, E>` is intrinsic.** It has exactly two variants, `Ok(T)` and
  `Err(E)`, it is not declared in `std/`, and it is the type of every fallible
  operation. The compiler must not depend on a standard library that does not
  exist, and a language whose central fallible type is a user declaration would
  let a program shadow it.
* **`match` on a `Result` is exhaustive**, like any other enum: an unhandled
  `Err` is a compile error, not a runtime surprise.
* **There is no `?` operator.** Propagation is written with `match`, which is
  where the reader can see it. A `?`-like shortcut is additive and would need a
  NEP; leaving it out now costs a few lines per call site and buys an unambiguous
  control flow. `?` remains a *type* operator only (`T?`).
* **A trap is not a value and cannot be caught.** It ends the task with a
  diagnostic. Traps are: integer overflow, division by zero
  ([NEP-0007](0007-integer-semantics.md)), a failed `Verified<T>` downgrade
  attempt is a *compile* error rather than a trap, and budget exhaustion is a
  trap in this edition (M8 may revisit it, because a budget is a normal
  outcome).
* **A verification failure is a `Result`, not a trap**
  ([NEP-0002](0002-generated-verified.md)).
* **Compile diagnostics and runtime traps never share a code.** A diagnostic
  means "this program was rejected"; a trap means "this program was accepted and
  could not continue". `spec/errors.md` allocates the families, and the
  distinction is visible in the code alone.
* **Traps are deterministic.** The same program with the same inputs traps at
  the same point on every backend, which is why arithmetic is fixed-width
  ([NEP-0007](0007-integer-semantics.md)).

## What this makes impossible

* A silently swallowed failure: an unhandled `Err` cannot reach the end of a
  `match`, and there is no `?` to forget to annotate.
* A catchable panic, and with it the "exceptional control flow" style that makes
  error paths untestable.
* A program whose meaning depends on whether an error was a value or a trap: the
  two are different types of thing, decided at compile time.

## Alternatives

| Alternative | Why it lost |
| ----------- | ----------- |
| Exceptions | Invisible in signatures, and the language's premise is that requirements are visible |
| `?` from the start | Convenient, and it hides the propagation path the reader needs; it is additive later, so it is not taken now |
| Every fallible operation returns `Result`, including `/` | Makes arithmetic unusable, and division by zero is not a value the caller can do anything with |
| `Result` in `std/` | The compiler would depend on a library that does not exist, and a program could shadow the type its own failure uses |
| A single `Error` type | Loses the reason, which is the only part a caller can act on |

## Unresolved questions

* Whether budget exhaustion stays a trap once policies and approvals exist (M8).
* Whether a `?`-like operator is wanted at all, and if so whether it requires a
  declared error type on the enclosing function.
* Whether traps need a task-level handler for supervision (M7).

## Implementation status

**Not implemented.** The parser accepts `Result<…>` as a type and `match` as a
control-flow form; the type itself, exhaustiveness checking and the trap family
arrive with M3 and M4.
