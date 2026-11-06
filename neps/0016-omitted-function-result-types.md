# NEP-0016: Omitted function result types

| Field | Value |
| ----- | ----- |
| Status | Draft |
| Authors | @smouj |
| Created | 2026-10-25 |
| NEP number | 0016 |
| Supersedes | NEP-0012, for function-result typing only (proposed) |
| Superseded by | — |
| Specification chapters | [`declarations.md`](../spec/declarations.md), [`types.md`](../spec/types.md) |
| Related NEPs | [NEP-0012](0012-early-exit.md), [NEP-0013](0013-generic-inference.md) |

## Summary

Clarify that a function with no written result type has result type `Unit`, and
that its body must therefore have type `Unit`. A function that returns a value
must declare that result type. This preserves the explicit-signature rule in
`spec/types.md` and the existing declaration wording, and supersedes NEP-0012's
conflicting rule that a function's result is its body value. NEP-0012's decision
against `return` and early exit is unchanged.

## Motivation

The specification, an accepted NEP and the type checker currently disagree:

```nudo
fn answer() {
    42
}

let value: Int = answer();
```

`spec/declarations.md` says an omitted result means `Unit`, and
`spec/types.md` says signatures must annotate their types. NEP-0012 says a
function's result is its body's value, while `nudo-typeck` currently infers
`Int` here. The type of a function must not depend on which of these passages a
compiler follows. Since function signatures are interfaces, changing a body
must not silently change the type seen by callers.

## Guide-level explanation

An omitted result annotation means `Unit`. The body may end without a value:

```nudo
fn prepare() {
    let ready = true;
}
```

A function that returns a value writes its type in the signature:

```nudo
fn answer() -> Int {
    42
}
```

The compiler checks the body against that result type. A non-`Unit` final
expression in a function with no result annotation is a type mismatch; the
compiler does not infer a public result type from it.

## Reference-level explanation

* If a function writes `-> T`, its body is checked against `T` as today.
* If a function omits `-> T`, its function type has result `Unit`. The body is
  checked against `Unit`; a non-`Unit` body is a type mismatch (`NDO2004`) at the
  function body.
* Calls use the result type from the function signature. They do not inspect
  the body to infer a result.
* This is local to ordinary functions. It does not decide signatures for
  `agent`, `task` or `tool` declarations.
* No syntax is added. The existing optional result annotation retains its
  spelling.
* This supersedes NEP-0012 only where its phrase “a function's result is the
  value of its body” implies result-type inference. Its no-`return` decision,
  block-value rule and statement that a block with no final expression is
  `Unit` remain in force.

## What this makes impossible

* A function's callable type changing when its implementation's final
  expression changes, without a signature edit.
* A non-`Unit` result being exposed to callers without being visible in the
  function declaration.

## Alternatives

| Alternative | Why it lost |
| ----------- | ----------- |
| Infer an omitted result from the body | It makes an implementation detail part of the interface and conflicts with the explicit-signature rule in `spec/types.md`. |
| Require every function to write a result type, including `Unit` | It is explicit but adds repeated syntax for the common no-value case; the grammar already makes the annotation optional. |

## Impact

| Area | Effect |
| ---- | ------ |
| Specification | Reconcile `spec/declarations.md` and `spec/types.md` with this decision and clarify the function-result wording from NEP-0012. |
| Grammar | None. |
| Compiler | Change `nudo-typeck` to use `Unit` when a function omits its result annotation; reuse `NDO2004`. |
| Conformance | Pin a valid unannotated `Unit` function and reject a non-`Unit` body without a result annotation. |
| Tooling | No syntax or diagnostic code changes. |
| Security | Keeps function result types visible in signatures for human and automated review. |
| Compatibility | Rejects functions that currently rely on inferred non-`Unit` results; adding `-> T` makes the intended result explicit. |
| Performance | No extra pass; the result is known from the signature before checking the body. |

## Open questions

None within this proposal's scope.

## Prior art

The relevant prior decisions are NUDO's explicit-signature rule in
`spec/types.md`, the declaration rule that an omitted result is `Unit`, and
NEP-0013's rejection of whole-program inference. This proposal makes an omitted
result independent of how callers use the function.

## Unresolved objections

None recorded; this draft has not entered discussion.
