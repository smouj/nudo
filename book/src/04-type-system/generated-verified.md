# 04.2 — Generated and Verified values

> **Status:** Implemented / Specified / Planned
>
> **Summary:** The checker enforces the distinction between generated and verified types, while executing verification and recording provenance remain future runtime work.

{{#include ../diagrams/trust-flow.svg}}

*The static type rules are implemented in part; the verify operation and
provenance path are a design preview and are not executable today.*

## The trust distinction

<code>Generated&lt;T&gt;</code> and <code>Verified&lt;T&gt;</code> are distinct
types. The checker does not implicitly convert between them, and their generic
arguments stay invariant. A generated result therefore cannot satisfy a
parameter that explicitly requires the verified type.

The accepted rules also allow the built-in <code>Verified&lt;T&gt;</code> to be
used where <code>T</code> is expected in supported positions. This is based on
the resolved built-in type identity; a user-defined type with the same spelling
does not acquire that rule. The behavior is covered by type-checker conformance
cases, but <code>nudo check</code> does not run type checking yet.

## The planned verification operation

The syntax <code>verify value with Verifier</code> is accepted by the parser and
represented in HIR. The verifier contract, execution, failure handling and
provenance are not implemented. The verifier design remains a proposal, so the
following shape illustrates the intended flow rather than a runnable program:

~~~nudo
let checked: Result<Verified<Article>, VerificationError> =
    verify draft with ArticleVerifier
~~~

The design makes two separate facts visible: verification can fail, and a
successful value carries a different trust type. It does not make probabilistic
generation deterministic; it makes the boundary reviewable.

## Why not represent trust with a Boolean?

~~~text
ModelOutput<T> { value: T, verified: Bool }
~~~

A caller can forget to inspect a flag. Distinct types let the checker reject a
value at a supported type-checking boundary instead of relying on a runtime
convention.

## What remains

The runtime operation, statically checkable verifier declarations, a stable
failure type, provenance records and traces remain planned or proposed work.
<code>Result&lt;T, E&gt;</code> can be named with its required type arguments,
but match typing and exhaustive handling are not implemented.

\n