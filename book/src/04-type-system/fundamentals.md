# 04.1 — Type-system fundamentals

> **Status:** Implemented / Specified / Planned
>
> **Summary:** NUDO has an implemented partial checker for accepted type rules; the full type system remains in progress and is not yet part of the CLI check command.

The type system is specified in the language chapters and accepted NEPs. The
Rust crate <code>nudo-typeck</code> implements a growing subset and consumes HIR,
not source text. Its tests and conformance cases exercise the implemented rules.
The CLI does not invoke the checker yet.

## Rules implemented by the current checker

| Rule | Current behaviour |
| --- | --- |
| Primitive and literal types | <code>Int</code>, <code>Float</code>, <code>Bool</code>, <code>Text</code> and <code>Unit</code> are represented |
| Annotations and function signatures | Values are compared with written types; function types and calls are checked |
| Calls | Arity uses <code>NDO2005</code>, incompatible arguments use <code>NDO2004</code>, and non-callable values use <code>NDO2006</code> |
| Generic calls | Type parameters are inferred from arguments and already-known expected result types, in source order |
| Named types | Type-argument arity uses <code>NDO2017</code>; nominal identity and supported field access are checked |
| Arithmetic | Matching numeric operands support <code>+</code>, <code>-</code>, <code>*</code>, <code>/</code>; <code>%</code> is Int-only and unary <code>-</code> preserves numeric type |
| Conditions | The condition of <code>if</code> must be <code>Bool</code>; branch expressions are still visited |
| Trust assignment | The built-in <code>Verified&lt;T&gt;</code> can be used where <code>T</code> is expected in supported positions |

Generic inference is deliberately bounded. It does not solve types across the
whole program, generalize a local binding or guess an unresolved parameter.
Generic arguments remain invariant, and numeric conversions are not implicit.

## Rules still incomplete

The checker does not yet define a result type for <code>if</code>, agreement
between branch values, all operators, indexing, nominal construction,
<code>match</code> or <code>Result&lt;T, E&gt;</code> matching. A stable diagnostic
for a generic parameter that remains unresolved is also pending. Integer literal
range checking, agents, tasks and tools have not been implemented in this
checker.

These omissions are deliberate. A rule is implemented only when its semantic
and diagnostic contract is accepted. The roadmap and the specification record
which work is ready and which decisions remain open.

## What a clean CLI check means

<code>nudo check</code> runs lexical, syntax and name-resolution stages. Since
the partial type checker is not wired into that command, a clean CLI result does
not establish that every type rule has passed. The checker is currently
exercised through its own tests and the type-checking conformance corpus.

\n