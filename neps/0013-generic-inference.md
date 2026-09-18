# NEP-0013: Generic inference

| Field | Value |
| ----- | ----- |
| Status | Accepted |
| Created | 2026-09-18 |
| Accepted | 2026-09-18, as the gate before M3.2 |
| Supersedes | — |
| Superseded by | — |
| Specification chapters | [`spec/generics.md`](../spec/generics.md), [`spec/expressions.md`](../spec/expressions.md) |
| Related NEPs | NEP-0006, NEP-0010, NEP-0014 |

## Summary

Inference is **local, left to right, and never a guess**: type arguments come
from the arguments at the call site and from the type the context already
expects, a `let` is monomorphic, and a parameter that cannot be inferred is a
compile error that asks for a written type rather than a silently chosen one.

## Motivation

NEP-0010 added declaration-site parameters and left the inferable question open
on purpose. It has to be closed before `nudo-typeck` exists, because inference is
the one type-system rule that can be *unbounded*: the difference between "infer
from the arguments" and "solve the program" is the difference between a checker
that answers in one pass and one that answers differently depending on what it
read first.

The language's premise decides the question. NUDO's requirements are supposed to
be visible in signatures; a compiler that quietly picks a type parameter for you
is doing exactly the thing `Generated<T>` and `Verified<T>` exist to prevent, one
layer down. So inference is kept where it removes noise and nowhere else.

## Reference-level explanation

* **Call-site inference, from the arguments.** Each argument's type is matched
  against its parameter's type; a type parameter that appears in that match is
  solved by it. Left to right, one pass, no backtracking.
* **Expected-type inference.** Where the context already knows the type — an
  annotation, a parameter, a return type, an arm of a `match` — that expected
  type solves what the arguments did not.
* **A `let` is monomorphic.** `let x = identity(1);` gives `x: Int`. There is no
  generalisation, no let-polymorphism, and no type variable escaping into a
  binding.
* **Unconstrained is an error, not a default.** A parameter that appears only in
  the return type cannot be inferred:

  ```nudo
  fn empty<T>() -> [T] { … }

  let xs = empty();              // NDO2xxx: T is not inferable here
  let ys: [Text] = empty();      // fine: the annotation says it
  ```

  The diagnostic names the parameter and the fix: write the type where the value
  is used.
* **No turbofish.** NEP-0006 rejected it, and this NEP does not bring it back:
  the way to say "this call produces `Text`" is to say the type of the value,
  which is a sentence a reader can see. Widening to per-call type arguments would
  be its own NEP.
* **No whole-program inference.** A function's signature is what it says, in both
  directions: nothing infers a function's parameters from its body, and nothing
  infers one item's types from another item's use.
* **Deterministic by construction.** Because inference only ever consumes what is
  written to the left, two compilations of the same file agree, and a diagnostic
  cannot depend on evaluation order.
* **Arity is separate and checked** (NEP-0010): `Pair<Int>` is an error at the
  type argument, whatever inference does.

## What this makes impossible

* A program whose meaning depends on how much the compiler guessed.
* A type error that disappears when an unrelated function is added: with no
  whole-program solving, there is nothing to be solved twice.
* A binding that becomes generic by accident, and therefore behaves differently
  at two call sites.

## Alternatives

| Alternative | Why it lost |
| ----------- | ----------- |
| Full inference with unification and let-generalisation | Unbounded cost in the checker and in the reader: the type of a value becomes something to derive rather than to read |
| Bidirectional inference everywhere (checking against expected types only) | Loses the noise removal that makes `identity(1)` pleasant, and makes every call need a context |
| A turbofish for explicit arguments | NEP-0006 rejected the syntax; an annotation says the same thing in a place a reader already looks |
| Defaulting an unconstrained parameter to its first bound | There are no bounds, and a default is a guess that becomes a rule nobody wrote down |
| Infer `T` by scanning the body for a value that fits | Reads backwards, and makes a function's signature depend on its implementation |

## Unresolved questions

* Whether closures (if they arrive) infer parameter types from an expected
  function type — likely yes, and it is the same rule, not a new one.
* Whether a *sequence literal* may infer its element type from later elements.
  Today: no, and the fix is to annotate.
* Whether `tool` and `task` calls, which cross a runtime boundary, want stricter
  rules than ordinary calls.

## Implementation status

**Not implemented.** This NEP is the gate M3.2 opens with; the inference rules
above are what `nudo-typeck` will implement, in one pass, and what the M3.2
conformance cases will pin.
