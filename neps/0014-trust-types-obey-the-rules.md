# NEP-0014: Trust types obey the ordinary rules

| Field | Value |
| ----- | ----- |
| Status | Accepted |
| Created | 2026-09-18 |
| Accepted | 2026-09-18, as the gate before M3.2 |
| Supersedes | — |
| Superseded by | — |
| Specification chapters | [`spec/trust/verified.md`](../spec/trust/verified.md), [`spec/expressions.md`](../spec/expressions.md), [`spec/errors.md`](../spec/errors.md) |
| Related NEPs | NEP-0002, NEP-0008, NEP-0013 |

## Summary

`Generated<T>`, `Verified<T>` and `Result<Verified<T>, E>` get **no rules of
their own**. `match` is exhaustive over a `Result` exactly as it is over any
other `enum`, an unhandled `Err` is an error because it is an unhandled variant,
and NUDO has **no must-use rule** in this edition. A rule that said "a `Result`
value may not be silently discarded" would be a language-wide decision about all
`Result`s, not a privilege of the trust types, and it is not taken here.

## Motivation

There is a tempting design where `verify` gets special treatment: the checker
notices an ignored verification, or refuses to let a `Result<Verified<T>, E>` be
dropped, or treats `Err` as more serious than another enum's variant. It is
tempting because the failure mode it prevents is real — an agent that quietly
throws away a rejection.

It is also a second set of rules for something that already has rules. The trust
model (NEP-0002) works because `Generated<T>` and `Verified<T>` are *types*: they
flow through every existing rule, and a reader who knows how types work already
knows how they work. Adding exhaustiveness exceptions, severity exceptions or
implicit lints for these two names would mean the most important part of the
language is also the part with the most special cases — which is exactly
backwards.

## Reference-level explanation

* **`match` over a `Result` is exhaustive, and that is the whole enforcement.**
  `Ok(article) => publish(article)` without an `Err` arm is a compile error
  (`NDO2xxx`) because a variant is unhandled — the same error a `match` over any
  other enum produces. The message names the variant, not the trust model.
* **No must-use rule.** A `Result` value that is ignored is not an error and not
  a warning in this edition. If NUDO ever wants one, it is a NEP about *all*
  fallible values, with its own alternatives, and it would land for `Result`
  before it lands for anything with a type parameter.
* **No severity exception.** A verification that fails is not "more wrong" than
  any other `Err`. Diagnostics from `Result` handling are the ordinary `2xxx`
  family.
* **No implicit conversion, and that is not a special rule either**: it follows
  from `Generated<T>` and `Verified<T>` being distinct types with no subtyping
  (NEP-0002).
* **`verify` is an ordinary expression** whose type is
  `Result<Verified<T>, VerificationError>` (NEP-0002). It composes: it can be
  passed, stored, returned, matched, and put in a sequence, because everything
  else can.
* **The one asymmetry that does exist is a type rule.** `Verified<T>` can be used
  where a `T` is expected; `Generated<T>` cannot be used where a `Verified<T>` is
  expected. That is subtyping, decided in NEP-0002, and it is uniform.

## What this makes impossible

* A language where the trust types are also the exception types.
* Two ways to express "this must be handled" — a `match` and a linter rule — with
  different messages and different escape hatches.
* A checker whose behaviour for `Result<Verified<T>, E>` diverges from
  `Result<Int, MathError>`, which would make the interesting case the one with
  the least predictable rules.

## Alternatives

| Alternative | Why it lost |
| ----------- | ----------- |
| `Err` of a verification is an error even when handled | It is handled. Saying otherwise makes the word mean nothing |
| A must-use rule for `Result` now | A real decision, but a language-wide one, and not one to smuggle in with the trust types |
| A `#[must_verify]`-style annotation | Annotations do not appear in signatures, which is the whole argument of NEP-0002 |
| Treating `Err` as a warning | Warnings are for things that are probably wrong; a handled failure is not wrong |
| A lint rather than a rule | Then it is advisory in exactly the place the language claims to be strict, and it needs a suppression story |

## Unresolved questions

* Whether NUDO wants a must-use rule for `Result` at all. If it does, it is its
  own NEP, and it should argue the escape hatch and the interaction with `match`
  before it argues the trust model.
* Whether an *unhandled* verification failure inside a `task` should be visible
  in a trace (M7). That is a runtime decision about traces, not a typing rule.
* Whether `provenance` being implicitly dropped ever deserves a warning (M6).

## Implementation status

**Not implemented, and not implementable yet**: this NEP says what M3.2/M3.3 must
*not* special-case. Its evidence is the absence of tests — the M3.3 conformance
cases cover conversion rejection and verification failure, and none of them may
assert an error that is about anything other than an unhandled variant or a type
mismatch.
