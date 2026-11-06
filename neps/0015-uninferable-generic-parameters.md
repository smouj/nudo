# NEP-0015: Uninferable generic parameters

| Field | Value |
| ----- | ----- |
| Status | Draft |
| Authors | @smouj |
| Created | 2026-10-25 |
| NEP number | 0015 |
| Supersedes | — |
| Superseded by | — |
| Specification chapters | [`generics.md`](../spec/generics.md), [`errors.md`](../spec/errors.md) |
| Related NEPs | [NEP-0013](0013-generic-inference.md) |

## Summary

Specify `NDO2008` (`TYPE_PARAMETER_NOT_INFERRED`) for an ordinary generic
function call whose signature parameter cannot be determined from its arguments
or from an already-known expected result type. Also reject a generic function
whose type parameter does not occur in its parameter or result types with
`NDO2009` (`TYPE_PARAMETER_NOT_IN_SIGNATURE`), because no caller can ever
supply that type through NUDO's accepted inference rules. Generic `tool` and
`task` inference remains out of scope while NEP-0013 leaves those call rules
open.

## Motivation

NEP-0013 says an uninferable parameter is a compile error, but does not assign a
stable code or define its location and recovery behavior. The current checker
returns an unknown type silently when neither call arguments nor context can
determine the parameter:

```nudo
fn empty<T>() -> [T] { … }

let values = empty();
```

Silence makes an invalid call look accepted to an editor or an automated repair
tool. The context can make the same call valid when it supplies the missing
type:

```nudo
let values: [Text] = empty();
```

There is a related declaration that no call-site annotation can repair:

```nudo
fn answer<T>() -> Int { 42 }
```

`T` affects neither the inputs nor the result. NUDO has no explicit type
arguments, so callers cannot choose `T`; reporting `NDO2008` at every call would
repeat an error whose cause is fixed at the declaration. The declaration should
therefore be rejected once, at `T`.

## Guide-level explanation

The compiler reports one `NDO2008` error for each generic parameter that is
present in the function signature but still unknown after argument and
expected-type inference. Its location is the whole call expression, its message
names the parameter, and its help asks for an annotation at the value's use. A
generic function parameter absent from every parameter and result type gets one
`NDO2009` at its declaration instead. NUDO never chooses a default and this
proposal does not add per-call type-argument syntax. These rules apply to
ordinary function calls; they do not decide how generic `tool` or `task` calls
work.

```text
error[NDO2008]: type parameter `T` cannot be inferred here
  --> main.nudo:3:14
  = help: add a type annotation to the value that uses this call
```

## Reference-level explanation

* The proposed stable code is `NDO2008`, named `TYPE_PARAMETER_NOT_INFERRED`,
  with severity `error`.
* The proposed stable code is `NDO2009`, named
  `TYPE_PARAMETER_NOT_IN_SIGNATURE`, with severity `error`.
* Inference first uses call arguments in source order, then any expected result
  type already known at the call site, following NEP-0013.
* For each generic function parameter that appears in neither a parameter type
  nor the result type, emit one `NDO2009` at that parameter's declaration. If
  several parameters meet this rule, emit diagnostics in declaration order.
  The body does not make a parameter inferable: inference is bounded to the
  written signature, as NEP-0013 requires. This rule does not decide generic
  `tool` or `task` declarations, and it does not reject phantom parameters on
  nominal `struct` or `enum` types, whose type arguments remain part of nominal
  type identity.
* For each signature parameter that remains unresolved at a call, emit one
  `NDO2008` at the full call expression. Emit them in declaration order and
  name the parameter in the message.
* If arguments produce conflicting inferences, keep the existing
  `NDO2004` type-mismatch diagnostic at the later argument; do not also report
  that parameter as uninferable.
* A wrong argument count remains `NDO2005`; do not add `NDO2008` diagnostics to
  that call.
* If an earlier error leaves an argument's type unknown, suppress
  `NDO2008` for parameters whose inference depended on that argument. Continue
  reporting independently uninferable parameters.
* The call's result is `Error` after reporting, preventing the same unknown
  type from causing follow-up mismatches.

## What this makes impossible

* A call with an unknown generic result silently passing type checking.
* A compiler choosing a default type or making inference depend on unrelated
  declarations.
* A function declaration hiding a type parameter that cannot be supplied by any
  caller under the language's bounded inference rules.
* Repair tools having to infer missing type information from message text alone.

## Alternatives

| Alternative | Why it lost |
| ----------- | ----------- |
| Reuse `NDO2004` (`TYPE_MISMATCH`) | An absent type argument is not a disagreement between two known types; reusing the code would make a stable code ambiguous. |
| Choose a default type | This is a guess, which NEP-0013 rules out. |
| Keep returning an unknown type without an error | Makes an invalid call look successful and hides a required annotation from tools. |
| Report a signature-absent parameter at every call | No call-site annotation can affect it; repeating the same declaration defect creates noise and still leaves the function unusable. |
| Allow the parameter as a phantom on a function | Unlike a nominal type argument, it is absent from the function type and cannot be written at a call site; removing it leaves every call's type unchanged. |

## Impact

| Area | Effect |
| ---- | ------ |
| Specification | Add both diagnostic behaviors to `spec/generics.md` and `spec/errors.md`. |
| Grammar | None. |
| Compiler | Register and emit `NDO2008` and `NDO2009` from `nudo-typeck`; keep a call result as `Error` after `NDO2008`. |
| Conformance | Add cases for an unresolved result, a context that resolves it, multiple unresolved parameters, recovery after an earlier error, and a function parameter absent from the signature. |
| Tooling | The diagnostic code and parameter name let editors offer a type annotation. |
| Security | Prevents unknown generic types and uncallable phantom parameters from being silently accepted or guessed. |
| Compatibility | Newly rejects calls that NEP-0013 already defines as compile errors, and functions with uninstantiable parameters. |
| Performance | Uses the existing call-local substitutions and one signature scan per function; no global solver or extra inference pass is needed. |

## Open questions

No open questions remain within this proposal's scope. A type parameter absent
from a function's parameter and result types is a declaration error
(`NDO2009`); a signature parameter that lacks enough call-site information is
an inference error (`NDO2008`). Nominal type declarations are outside the
declaration check because their type arguments are explicit parts of nominal
identity. Generic `tool` and `task` calls remain outside scope, as their
inference rules are still open in NEP-0013.

## Prior art

NEP-0013 is the governing NUDO design: inference is local and bounded, and an
uninferable parameter is an error rather than a default. This proposal assigns
the diagnostic contract needed to implement that accepted rule.

## Unresolved objections

None recorded; this draft has not entered discussion.
