# 04.1 — Type-system fundamentals

> **Status:** Proposed  
> **Summary:** The type system favors explicit public signatures, nominal relationships and compiler-visible trust/effect information.

Current design constraints include primitive scalar types, structs, enums,
sequences, functions, generics and optional/result-like forms.

Public interfaces should be annotated. Local inference may reduce noise, but public
API types should not secretly depend on implementation details.

Open questions such as integer width, overflow semantics, variance and the exact
representation of runtime-supplied capabilities must be resolved deliberately.
