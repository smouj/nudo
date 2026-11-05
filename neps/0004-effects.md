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

### Effect sets and function values

`with` takes a comma-separated set of capability names. A missing clause means
the empty set. The compiler infers effects from direct effectful operations and
calls, then requires the inferred set to be contained in the function's clause.
There is no implicit signature propagation: each caller declares the effects it
requires, and a missing declaration is an error.

Effects are part of function types as well as function declarations:

```nudo
fn apply(f: Fn(Int) -> Int with Network, value: Int) -> Int with Network {
    f(value)
}
```

A function value may be used where a function type declares the same or a larger
effect set; it may not be used where the type promises fewer effects. Calling an
effectful function value requires the caller's effect set to cover its declared
effects. This keeps higher-order calls visible without adding a second
capability type system.

The first M5 diagnostics are proposed as:

| Code | Name | Meaning |
| ---- | ---- | ------- |
| `NDO3001` | `EFFECT_NOT_DECLARED` | A function body or call path uses an effect absent from its declaration. |
| `NDO3002` | `UNKNOWN_EFFECT` | An effect clause names a capability outside the closed set. |
| `NDO3003` | `DUPLICATE_EFFECT` | An effect clause lists the same capability more than once. |

### Static effects and runtime grants

An effect clause such as `with Network` declares what a function may attempt
and what callers must account for. It is not itself a capability grant. The
compiler checks the function body and its transitive calls against the declared
effect set, and reports the path that introduced an undeclared effect.

The authority to perform that effect comes separately from the grant in force
for the program or agent. Before an external action, the runtime checks that
grant at the effect boundary. Static checking makes the code's effect path
visible; runtime enforcement checks the authority available in this particular
run. Runtime checks do not replace static checking, and a function signature
does not grant ambient authority.

M5 covers the static checker. Every external effect also needs a runtime grant
check before it can execute. M4's `print` uses the explicit `Console` host grant
proposed in NEP-0003; M8 generalizes the capability and policy machinery to
agents and tools.

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

* **Generic bounds.** How effect sets interact with generic callable bounds.
* Whether effect handlers exist, and if so whether they belong in this NEP or a
  superseding one.
* Whether effects can be *subtracted* (a function that requires the network to be
  unavailable), which is occasionally useful and hard to specify.
* How a runtime-supplied tool set, whose effects are not known at compile time,
  is expressed. It must not be silently treated as effect-free.
* Whether effect sets on generic function values need syntax beyond the function
  type form proposed above.
