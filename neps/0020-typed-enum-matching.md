# NEP-0020: Typed enum matching

| Field | Value |
| ----- | ----- |
| Status | Draft |
| Authors | — |
| Created | 2026-10-26 |
| NEP number | 0020 |
| Supersedes | — |
| Superseded by | — |
| Specification chapters | [`spec/declarations.md`](../spec/declarations.md), [`spec/expressions.md`](../spec/expressions.md), [`spec/grammar.md`](../spec/grammar.md), [`spec/errors.md`](../spec/errors.md) |
| Related NEPs | NEP-0006, NEP-0008, NEP-0010, NEP-0013, NEP-0014, NEP-0015, NEP-0019 |

## Summary

Define `match` over enums: a scrutinee must be an enum, each arm names one of
its variants or uses an explicit `_` wildcard, payload fields bind positionally,
every variant is covered, and the arm expressions agree on one result type.
Variant names never fall back to bindings, so a misspelling cannot silently turn
an exhaustive match into a catch-all. `Result<T, E>` follows the same rules as
every other enum. This proposal changes no specification or compiler behavior
until it is accepted.

## Motivation

The grammar accepts `match`, the HIR records each arm head and its payload
bindings, and the specification requires exhaustive matches. It does not yet say
how to distinguish a variant from a binding, how payload types reach bindings,
or how arm result types are chosen. Implementing those rules from convention
would silently define the language.

The ambiguity also affects safety. If an unknown head is treated as a binding,
a typo can swallow every variant and make an apparently exhaustive check skip a
case:

```nudo
match status {
    Runnng => handle_running()
    Failed(reason) => report(reason)
}
```

The checker should report that `Runnng` is not a variant. It should not infer a
catch-all from the fact that the name is absent.

## Guide-level explanation

An arm names a variant of the scrutinee's enum. Payload patterns bind fields in
declaration order; `_` discards a field or explicitly covers every remaining
variant:

```nudo
enum Status {
    Running
    Failed(reason: Text)
}

fn message(status: Status) -> Text {
    match status {
        Running => "running"
        Failed(reason) => reason
    }
}
```

`match` rejects a non-enum scrutinee, an unknown variant, a payload pattern with
the wrong number of fields, duplicate or unreachable arms, and a match that
leaves a variant uncovered. A wildcard is visible in the source:

```nudo
match status {
    Running => "running"
    _ => "not running"
}
```

Without an expected result type, every arm must have the same type. When an
annotation, parameter or function result already supplies an expected type, each
arm is checked against it using ordinary assignment rules. This lets a caller
choose `Int` for a match whose arms produce `Verified<Int>` and `Int`, while
keeping inference independent of arm order.

## Reference-level explanation

### Scrutinee and patterns

* The scrutinee must have a resolved enum type. User-declared enums and intrinsic
  `Result<T, E>` qualify; primitives, structs and unresolved generic parameters
  do not. An already erroneous scrutinee does not cause a second match-type
  diagnostic.
* A top-level pattern is one unqualified identifier or `_`. An identifier must
  name a variant of the scrutinee's enum. An unknown or qualified name is an
  invalid pattern, not a new binding. This deliberately leaves module-qualified
  patterns for the module design.
* `_` is a wildcard and introduces no binding. It matches any remaining
  variants, is allowed at most once, and makes every later arm unreachable.
* A variant with no payload may be written as `Ready` or `Ready()`; both mean the
  same zero-field pattern. A variant with payload must provide exactly one
  sub-pattern per field. Each sub-pattern is a binding identifier or `_`.
  Nested variant patterns and literal patterns are not part of this edition.
* Payload bindings are immutable locals in that arm's scope. They receive the
  corresponding field type in declaration order, after substituting the
  scrutinee's generic arguments. Repeated binding names in one arm remain
  `NDO2002`.
* An enum declaration has at least one variant. The grammar is narrowed to
  require one; uninhabited enums and a bottom result type need a separate design.

### Coverage and result types

* With no wildcard, every variant must appear exactly once. With a wildcard,
  variants may be omitted, but the wildcard must follow all explicit variants.
  A repeated variant, or any arm after a wildcard, is unreachable.
* Coverage is checked against the enum definition, not against the variant names
  that happen to appear in the source. For `Result<T, E>`, the two variants are
  `Ok(T)` and `Err(E)` as defined by NEP-0008. NEP-0014's no-must-use rule still
  applies; only the match itself must be exhaustive.
* If the surrounding context already requires type `R`, each arm body is
  checked against `R`. Its ordinary assignment rules apply, including the
  top-level `Verified<T>`-to-`T` relation. A valid match has type `R`.
* Without an expected type, all arm bodies must have the same resolved type,
  including invariant generic arguments. The checker does not choose a common
  supertype or infer one arm from another. Each arm whose type differs from the
  first arm's type gets a diagnostic. If the first arm has type `Error`, the
  match also has type `Error`; otherwise `Error` in a later arm participates in
  recovery without creating another mismatch.
* Calls inside arms may still infer their own generic arguments from their call
  arguments. When an expected match type exists, NEP-0013 permits that type to
  reach each arm's result expression.

### Proposed diagnostics

These codes are proposed for the registry; none is allocated or emitted before
acceptance:

| Code | Name | Use |
| ---- | ---- | --- |
| `NDO2011` | `MATCH_REQUIRES_ENUM` | The scrutinee has a known non-enum type |
| `NDO2012` | `INVALID_MATCH_PATTERN` | An unknown or qualified variant path, nested variant pattern, or payload on `_` |
| `NDO2013` | `WRONG_VARIANT_PATTERN_ARITY` | A variant pattern has the wrong number of payload sub-patterns |
| `NDO2014` | `NON_EXHAUSTIVE_MATCH` | One or more variants are uncovered; list them in declaration order |
| `NDO2015` | `UNREACHABLE_MATCH_ARM` | A duplicate variant arm or an arm after `_` |
| `NDO2016` | `MATCH_ARM_TYPE_MISMATCH` | Arm bodies disagree when no expected result type exists |

When an expected result type exists, a body that does not satisfy it uses the
ordinary `NDO2004` diagnostic. Pattern and coverage errors do not prevent the
checker from checking other arm bodies for independent errors.

## What this makes impossible

* A misspelled variant becoming a binding that silently covers the enum.
* A `match` whose meaning depends on a runtime fallback for an unhandled case.
* Arm-order-dependent result inference.
* A `Result`-specific exhaustiveness or must-use rule that diverges from other
  enums.
* Nested variant patterns or literal patterns in this edition; they need a
  recursive pattern representation and their own coverage rules.

## Alternatives

| Alternative | Why it lost |
| ----------- | ----------- |
| Treat an unknown top-level name as a binding | A typo can become a catch-all and suppress a useful exhaustiveness error; `_` makes that intent explicit |
| Choose the first arm's type and coerce later arms to it | The inferred result changes when arms are reordered and can silently choose a trust-related type |
| Infer a common supertype for all arms | Adds a type-joining algorithm beyond NEP-0013's bounded inference; an expected type already provides the explicit context |
| Permit a runtime fallback for missing variants | Defers a statically checkable error to a path that may be reached with untrusted data |
| Implement nested patterns now | Requires HIR to preserve recursive pattern structure; the current arm record intentionally stores only a head and bindings |

## Impact

| Area | Effect |
| ---- | ------ |
| Specification | Defines enum `match` in declarations, expressions, grammar and errors |
| Grammar | Requires at least one enum variant; narrows match patterns to one variant head with binding or wildcard payloads |
| Compiler | `nudo-hir` must preserve wildcard and payload positions; `nudo-typeck` resolves variants, substitutes payload types, checks coverage and arm types |
| Conformance | Adds valid, non-enum, unknown-variant, wrong-payload-count, non-exhaustive, unreachable, generic payload and arm-type cases |
| Tooling | Parser and HIR dumps must display the accepted pattern shape; formatter and LSP can follow later |
| Security | Explicit wildcard and static exhaustiveness make omitted `Result` and enum cases reviewable; no trust-specific bypass is introduced |
| Compatibility | Existing syntax is pre-alpha; nested patterns and empty enums would be rejected by the accepted grammar |
| Performance | Coverage can use a bit set indexed by declaration-order variant IDs, O(V + A) time and O(V) temporary space for V variants and A arms; benchmark large enums before stabilisation |

## Open questions

* Should a later edition add nested variant patterns, and if so, how should
  exhaustiveness work for nested payloads?
* Should literal patterns for `Bool`, `Int` and `Text` be a separate feature, or
  should `match` remain enum-only?
* Should module-qualified variant patterns be accepted when modules arrive, or
  should a pattern always name a variant relative to the scrutinee type?

These questions do not alter the rules proposed for current enum patterns. They
must be closed before extending those rules.

## Prior art

Rust and ML-family languages use constructor patterns and exhaustiveness checks.
This proposal keeps that useful combination while keeping payload destructuring
flat until HIR has a recursive pattern representation.

## Unresolved objections

None recorded. This is a Draft and has not entered review.
