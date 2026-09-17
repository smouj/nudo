# NEP-0004: Effect system

| Field | Value |
| ----- | ----- |
| Status | Draft |
| Created | 2026-09-17 |
| Supersedes | — |
| Superseded by | — |
| Specification chapters | [`spec/effects.md`](../spec/effects.md), [`spec/trust/capabilities.md`](../spec/trust/capabilities.md) |
| Related NEPs | NEP-0003 |

## Summary

Track, statically, which capabilities each function may exercise, so that a call
requiring a capability is an error unless the caller holds it.

## Motivation

NEP-0003 says what a program may hold. It does not say where the capability is
used, and without that, the interesting review question — "which line reaches
the network?" — is unanswerable.

```nudo
fn render(report: Report) -> Text {
    // does this reach the network? Nothing in the signature says.
}
```

Effects put that answer in the signature, so it propagates the way types do.

## Guide-level explanation

```nudo
fn fetch(url: Text) -> Text with Network {
    // …
}

fn summary(url: Text) -> Text with Network {
    let body = fetch(url);
    body
}

fn render(text: Text) -> Text {
    text          // no effect clause: this cannot reach the network
}
```

`with Network` means: to call this, you need `Network`. A function without the
clause performs no external effect, and the compiler proves it.

## Reference-level explanation

* An effect clause lists capabilities. A call to an effectful function is an
  error unless the caller's clause covers it.
* Effects propagate transitively. There is no implicit propagation: a caller
  that acquires an effect must say so, because an invisible effect is an
  invisible risk.
* The diagnostic names the effect, the call site, and the path that introduced
  it — not only the line, because the useful information is how the effect
  arrived.
* Effects are orthogonal to `Result`. A failed tool call is a value, not an
  effect; effects describe what code touches, not what went wrong.
* Effects, not exceptions: no unwinding semantics, no hidden control flow.

## What this makes impossible

* Calling an effectful function from an effect-free one, "just this once". This
  is the property the NEP exists for: the effect clause becomes a boundary that
  cannot be crossed quietly.
* Passing capabilities implicitly through higher-order functions without
  declaring them. Generic code that must call effectful functions has to say so,
  and generic bounds for this are **unresolved** (see below).
* Silent retries that multiply spend, because a retry is an effect and inherits
  the caller's declared budget and capability.

## Alternatives

| Alternative | Why it lost |
| ----------- | ----------- |
| Runtime checks only, from NEP-0003 | Correct but late: the failure happens in production, and the review question stays unanswered |
| Annotate functions as `unsafe`/`safe` | A binary label cannot express `Network` versus `Filesystem`, and it carries no budget implication |
| Infer effects and report them in documentation only | Documentation is not a check; the error must be a compile error to be a boundary |
| Full algebraic effects with handlers from the start | A large design; the minimum viable version is a capability set, and handlers can come later through a superseding NEP |

## Impact

| Area | Effect |
| ---- | ------ |
| Specification | `spec/effects.md`, `spec/expressions.md`, `spec/generics.md` |
| Grammar | `effect-clause` on items and function types |
| Compiler | `nudo-effects`, `nudo-hir`, `nudo-typeck`; new `NDO3xxx` codes |
| Conformance | Effect propagation, denial, and generic-bound cases |
| Security | Makes NEP-0003 checkable rather than aspirational |
| Compatibility | Breaking for every function that gains an effect clause |
| Performance | Inference cost at compile time; none at run time for statically known calls |

## Open questions

* **Generics.** How a bound expresses "this function value may perform `Network`"
  without duplicating the capability system.
* Whether effect handlers exist, and if so whether they belong in this NEP or a
  superseding one.
* Whether effects can be *subtracted* (a function that requires the network to be
  unavailable), which is occasionally useful and hard to specify.
* How a runtime-supplied tool set, whose effects are not known at compile time,
  is expressed.
* Whether `with` is the right connective, or whether effects belong in the return
  type as `Fn(…) -> T effect Network`.
