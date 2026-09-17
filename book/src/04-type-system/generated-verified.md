# 04.2 — `Generated<T>` and `Verified<T>`

> **Status:** Proposed  
> **Summary:** The trust types are intended to prevent probabilistic output from becoming trusted application data through an implicit conversion.


{{#include ../diagrams/trust-flow.svg}}

*A model output becomes a verified value only through an explicit step.*

## Why not `T`?

If a model returns an `Article` directly, provenance about how that value was
produced disappears from the type boundary.

## Why not `Result<T, E>`?

`Result` can represent success/failure of an operation. It does not represent the
trust status of a successful value. A perfectly parsed model response can still be
unverified.

## Why not a Boolean flag?

```text
ModelOutput<T> { value: T, verified: Bool }
```

This pushes the rule into runtime convention. A caller can forget to inspect the
flag. Distinct types allow the checker to make the missing transition impossible to
ignore.

## Required design questions

Before stabilization, NUDO must specify verifier failure types, composition,
provenance retention, serialization boundaries and interactions with generics and
pattern matching.
