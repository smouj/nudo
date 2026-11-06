# NEP-0019: Integer literal boundary

| Field | Value |
| ----- | ----- |
| Status | Draft |
| Authors | @smouj |
| Created | 2026-11-06 |
| NEP number | 0019 |
| Supersedes | — |
| Superseded by | — |
| Specification chapters | [`spec/lexical-structure.md`](../spec/lexical-structure.md), [`spec/types.md`](../spec/types.md), [`spec/errors.md`](../spec/errors.md) |
| Related NEPs | NEP-0007, NEP-0008 |

## Summary

Define how decimal integer literals map to signed 64-bit `Int`, especially the
`2^63` magnitude needed to spell `Int::MIN` with unary `-`. Values through
`Int::MAX` are valid; exactly `2^63` is valid only as the direct, optionally
parenthesized operand of unary minus; larger magnitudes are compile errors. The
proposal assigns `NDO2010` to every invalid magnitude and changes no behavior
until accepted.

## Motivation

NEP-0007 fixes `Int` as signed 64-bit two's-complement and requires a literal
that does not fit to fail at compile time. Its minimum value is `-2^63`, but the
grammar treats `-` as a unary operator and the digits as a separate positive
literal. Without a boundary rule, the compiler must either reject the minimum
value or accidentally wrap an unrepresentable positive literal:

```nudo
let minimum: Int = -9223372036854775808;
```

The checker needs to accept this exact spelling without accepting an
out-of-range literal in any other context.

## Guide-level explanation

The full `Int` range is available in source:

```nudo
let minimum: Int = -9223372036854775808;
let maximum: Int =  9223372036854775807;
```

The positive magnitude `9223372036854775808` has no `Int` value on its own. It
is accepted only when unary `-` directly applies to it, because that operation
denotes `Int::MIN`. A larger magnitude is always rejected, even when preceded
by `-`.

## Reference-level explanation

* Integer literal digits are decimal, as defined by the existing grammar. `_`
  separators do not affect the value.
* After separators are removed, a magnitude in `[0, 2^63 - 1]` denotes that
  non-negative `Int` value.
* A magnitude exactly equal to `2^63` denotes `Int::MIN` only when it is the
  direct operand of unary `-`. Parentheses that contain only that literal may
  appear between the minus and the literal: `-((9223372036854775808))` is also
  valid. The literal itself is not assigned the positive value `2^63`.
* A magnitude greater than `2^63` is invalid in every context, including under
  unary minus. For example, `-9223372036854775809` is a compile error.
* A magnitude equal to `2^63` outside the unary-minus form is invalid. This
  includes `9223372036854775808`, `0 - 9223372036854775808`, and an integer
  literal whose expected type is `Float`. NUDO does not infer an implicit
  numeric conversion to make an invalid `Int` literal fit another type.
* Unary minus remains an operator. Once the special operand form has produced
  `Int::MIN`, another negation is ordinary `Int` arithmetic and traps for
  overflow as required by NEP-0007. Constant folding must preserve that trap.
* Every invalid magnitude emits `NDO2010 INTEGER_LITERAL_OUT_OF_RANGE`, with the
  primary span on the integer literal. For a magnitude equal to `2^63` used
  outside the permitted form, help may explain that it must be the direct,
  optionally parenthesized operand of unary `-` to denote `Int::MIN`.
* A literal that already produced `NDO2010` has recovery type `Error`, so unary
  operators and enclosing expressions do not emit cascaded type diagnostics.

No grammar change is needed: the leading minus is already a unary expression,
and integer literals are unsigned digit sequences in the token stream.

## What this makes impossible

* Silently wrapping a positive `2^63` literal into a negative `Int`.
* Accepting an arbitrary out-of-range magnitude because it appears after `-`.
* Making the same digit sequence fit or fail depending on an implicit `Float`
  conversion.
* Giving integer literal range a platform-dependent result.

## Alternatives

| Alternative | Why it lost |
| ----------- | ----------- |
| Reject `2^63` in every context | It makes `Int::MIN` impossible to write even though NEP-0007 defines it as a valid `Int`. |
| Parse a leading sign as part of the integer token | It changes the grammar and makes unary negation precedence and expressions such as `a - 1` harder to describe consistently. |
| Wrap every out-of-range magnitude | It contradicts NEP-0007 and silently changes the written value. |
| Allow `2^63` anywhere and defer its meaning to the expected type | It creates an unrepresentable positive `Int` and makes literal meaning depend on context. |

## Impact

| Area | Effect |
| ---- | ------ |
| Specification | Defines integer literal range in `spec/types.md` and registers `NDO2010` in `spec/errors.md` |
| Grammar | No grammar change; unary `-` remains separate from the integer token |
| Compiler | `nudo-typeck` parses magnitudes without machine overflow and recognizes the special unary-minus operand form |
| Conformance | Adds `Int::MIN`, `Int::MAX`, out-of-range positive and negative magnitudes, separators, parentheses, expected `Float`, and nested negation cases |
| Tooling | No formatter or CLI syntax change |
| Security | Integer bounds cannot wrap into a different budget or counter value |
| Compatibility | Previously unchecked boundary literals receive a stable compile diagnostic; ordinary in-range literals keep their meaning |
| Performance | One bounded decimal scan per integer literal; the checker need not allocate an arbitrary-precision integer |

## Open questions

None affect the proposed rule.

## Prior art

The proposal keeps the grammar's separate unary-minus operator while making the
one exceptional magnitude needed for the signed minimum explicit. It does not
use wrapping or platform-sized arithmetic.

## Unresolved objections

None recorded. This is a Draft and has not entered review.
