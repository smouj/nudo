# NEP-0019: Integer literal boundary

| Field | Value |
| ----- | ----- |
| Status | Accepted |
| Authors | @smouj |
| Created | 2026-11-06 |
| Accepted | 2026-11-28, after the maintainer delegated the semantic decision |
| NEP number | 0019 |
| Supersedes | The integer-literal boundary clause in [NEP-0007](0007-integer-semantics.md) |
| Superseded by | — |
| Specification chapters | [`spec/lexical-structure.md`](../spec/lexical-structure.md), [`spec/types.md`](../spec/types.md), [`spec/errors.md`](../spec/errors.md) |
| Related NEPs | NEP-0007, NEP-0008 |

## Summary

Define how decimal integer literals map to signed 64-bit `Int`, especially the
`2^63` magnitude needed to spell `Int::MIN` with unary `-`. Values through
`Int::MAX` are valid; exactly `2^63` is valid only as the direct, optionally
parenthesized operand of unary minus; larger magnitudes produce `NDO2010`.

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
is accepted only when unary `-` directly applies to it, because that expression
denotes `Int::MIN`. A larger magnitude is always rejected, even when preceded by
`-`.

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
* Unary minus remains an operator. Once the special operand form denotes
  `Int::MIN`, another negation is ordinary `Int` arithmetic and traps for
  overflow as required by NEP-0007. Constant folding must preserve that trap.
* A syntactically malformed number remains the lexer's `NDO1005` error and does
  not also receive `NDO2010`.
* Every invalid magnitude emits `NDO2010 INTEGER_LITERAL_OUT_OF_RANGE`, with the
  primary span on the integer literal and recovery type `Error`. The diagnostic
  explains the direct unary-minus form for exactly `2^63`; enclosing expressions
  do not emit cascaded type diagnostics.
* HIR parses the decimal digits once into a bounded magnitude: values through
  `2^63`, or an out-of-range marker. It ignores separators, allocates no
  arbitrary-precision integer and does not decide whether the magnitude is a
  valid signed value. Type checking supplies that context.

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
| Use arbitrary-precision integers during checking | It allocates and computes values the language cannot represent; a bounded scan decides this rule with fixed storage. |

## Impact

| Area | Effect |
| ---- | ------ |
| Specification | Defines integer literal range in `spec/types.md` and registers `NDO2010` in `spec/errors.md` |
| Grammar | No grammar change; unary `-` remains separate from the integer token |
| Compiler | HIR parses one bounded magnitude; `nudo-typeck` applies the context rule and emits `NDO2010` |
| Conformance | Pins `Int::MIN`, `Int::MAX`, out-of-range magnitudes, separators, parentheses, expected `Float`, binary subtraction and nested negation |
| Tooling | No formatter or CLI syntax change |
| Security | Integer bounds cannot wrap into a different budget or counter value |
| Compatibility | Previously unchecked boundary literals receive a stable compile diagnostic; ordinary in-range literals keep their meaning |
| Performance | One linear scan of each integer token, fixed-size storage, and no arbitrary-precision or heap allocation |

## Open questions

None affect this rule. Runtime overflow and constant-folding behavior remain
governed by NEP-0007.

## Prior art

The rule keeps the grammar's separate unary-minus operator while making the one
exceptional magnitude needed for the signed minimum explicit. It avoids wrapping,
platform-sized arithmetic and arbitrary-precision parsing in the compiler.

## Unresolved objections

None recorded.
