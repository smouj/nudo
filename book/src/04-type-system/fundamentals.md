# 04.1 — Type-system fundamentals

> **Status:** Implemented / Specified / Planned
>
> **Summary:** NUDO has an implemented partial checker for accepted type rules; the full type system remains in progress, while the implemented subset is part of the CLI check command.

The type system is specified in the language chapters and accepted NEPs. The
Rust crate <code>nudo-typeck</code> implements a growing subset and consumes HIR,
not source text. Its tests and conformance cases exercise the implemented rules.
The CLI invokes it after successful name resolution.

## Rules implemented by the current checker

| Rule | Current behaviour |
| --- | --- |
| Primitive and literal types | <code>Int</code>, <code>Float</code>, <code>Bool</code>, <code>Text</code> and <code>Unit</code> are represented |
| Integer literal boundaries | <code>Int::MAX</code> is accepted; magnitude <code>2^63</code> is accepted only directly under unary <code>-</code> to spell <code>Int::MIN</code>; larger or otherwise invalid magnitudes use <code>NDO2010</code> |
| Annotations and function signatures | Values are compared with written types; function types and calls are checked |
| Function results | An omitted result annotation means <code>Unit</code>; value-returning functions declare their result type; body mismatches use <code>NDO2004</code> |
| Calls | Arity uses <code>NDO2005</code>, incompatible arguments use <code>NDO2004</code>, and non-callable values use <code>NDO2006</code> |
| Generic calls | Type parameters are inferred from arguments and already-known expected result types, in source order; unresolved call parameters use <code>NDO2008</code> and unused function parameters use <code>NDO2009</code> |
| Named types | Type-argument arity uses <code>NDO2017</code>; nominal identity and supported field access are checked |
| Operators | Arithmetic uses matching numeric views, <code>%</code> is Int-only, Boolean operators use <code>Bool</code>, equality accepts matching primitive views, and ordering accepts matching <code>Int</code> or <code>Float</code>; invalid domains use <code>NDO2018</code> |
| Conditions | <code>if</code> conditions are <code>Bool</code>; an expected type checks both branches, otherwise they must agree exactly; without <code>else</code>, the result and then-branch are <code>Unit</code>; an uncontextualised mismatch uses <code>NDO2019</code> |
| Enum matching | User enums and <code>Result&lt;T, E&gt;</code> use explicit variant patterns, flat typed payload bindings, wildcard coverage, exhaustiveness and exact arm-result rules; diagnostics use <code>NDO2011</code>–<code>NDO2016</code> |
| Trust assignment | The built-in <code>Verified&lt;T&gt;</code> can be used where <code>T</code> is expected in supported positions |

Generic inference is deliberately bounded. It does not solve types across the
whole program, generalize a local binding or guess an unresolved parameter.
Generic arguments remain invariant, and numeric conversions are not implicit.
[NEP-0015](https://github.com/smouj/nudo/blob/main/neps/0015-uninferable-generic-parameters.md)
makes this boundary visible: `NDO2008` points to an unresolved generic call,
while `NDO2009` points to a function parameter absent from its signature.
Type parameters from an enclosing function stay fixed when a function-valued
parameter is called. Recovery suppresses only diagnostics that depend on an
earlier error.

## Rules still incomplete

Sequence indexing and its runtime bounds behavior, and nominal construction,
remain pending. Verifier execution and provenance, agents, tasks and tools have
not been implemented in this checker.

These omissions are deliberate. A rule is implemented only when its semantic
and diagnostic contract is accepted. The roadmap and the specification record
which work is ready and which decisions remain open.

## What a clean CLI check means

<code>nudo check</code> runs lexical, syntax, name-resolution and partial
type-checking stages. A clean CLI result means the implemented accepted rules
reported no errors; unsupported and Draft-gated type rules remain unchecked.
The type checker is also exercised through its own tests and the type-checking
conformance corpus.

\n
