# NEP-0007: Integer semantics

| Field | Value |
| ----- | ----- |
| Status | Accepted |
| Created | 2026-09-18 |
| Accepted | 2026-09-18, as gate 2 of M3.0 |
| Supersedes | — |
| Superseded by | — |
| Specification chapters | [`spec/types.md`](../spec/types.md), [`spec/expressions.md`](../spec/expressions.md), [`spec/errors.md`](../spec/errors.md) |
| Related NEPs | NEP-0006, NEP-0008 |

## Summary

Fix `Int` completely enough to write a type checker and two backends that agree:
**64-bit two's complement**, arithmetic that **traps** on overflow and on
division by zero, truncating division, and a **compile error** for a literal that
does not fit.

## Motivation

`spec/types.md` left the width open, and said it would not be settled by
accident. It cannot be settled later, because "what does `a + b` do when it does
not fit" is observable in every program, and an interpreter and a WASM backend
that answer differently make the language's results depend on which one ran.

The three candidate answers all cost something: wrapping is silent and wrong;
saturating is silent and wrong in a different way; `Result` arithmetic is honest
and unusable as the default. The choice below pays the cost where it is visible.

## Reference-level explanation

* **`Int` is a signed 64-bit integer**, two's complement, no padding, no
  platform dependence.
* **Overflow traps.** `Int::MAX + 1` does not wrap and does not saturate: it
  stops the task with a runtime diagnostic (`NDO6xxx`). The compiler may fold
  constant arithmetic, but folding must not change the outcome — a trap it can
  see is still a trap, not a compile error.
* **A literal that does not fit is a compile error** (`NDO2xxx`), because the
  program can never be right. `9223372036854775808` is rejected at compile time,
  not truncated at run time.
* **Division truncates toward zero**, and `%` takes the sign of the dividend
  (`-7 / 2 == -3`, `-7 % 2 == -1`), matching every language a reader will have
  used.
* **Division by zero traps**, and so does `Int::MIN / -1` — the one division
  whose result is not representable.
* **No implicit conversion.** `Int` and `Float` do not mix: `1 + 1.0` is a type
  error, and the conversion is written down. This is what keeps "which arithmetic
  is this" from depending on the operands.
* **Trap is not an exception.** A trap ends the task with a diagnostic and
  cannot be caught; a *recoverable* failure is a `Result`
  ([NEP-0008](0008-error-model.md)). The distinction is the language's, not the
  runtime's.
* **Sized and unsigned integers are not in this edition.** `Int32`, `UInt` and
  friends are additive and need their own NEP, which must say what happens when
  a `Int` is narrowed.

## Consequences for the rest of the language

* The interpreter and every later backend must reproduce these rules, and the
  conformance corpus gains cases for them: overflow, division by zero,
  `Int::MIN / -1`, truncation with negative operands, and a literal out of range.
* A budget or a counter is an `Int`. When `Int` overflows, that is a trap rather
  than a silent wrap, which matters where the value is a security limit.
* `Float` remains IEEE-754 binary64 and is *not* affected: `Float` arithmetic
  follows IEEE, and mixing the two is a type error.

## Alternatives

| Alternative | Why it lost |
| ----------- | ----------- |
| Arbitrary precision | Honest and slow, and it makes `Int` mean something different from what every systems language means; also it removes the ability to state a width in a signature |
| Wrapping | Silent and wrong: the classic integer bug, in a language whose point is that the compiler refuses what it cannot justify |
| Saturating | Silent, and it makes a budget look spent in a way the arithmetic did not say |
| Every operation returns `Result` | Correct and unusable: `a + b` would need handling at every site, and the language would be about arithmetic rather than about trust |
| Platform-sized (`usize`-like) | Two machines, two results — exactly the backend divergence this NEP exists to prevent |

## Unresolved questions

* Sized and unsigned types, and the narrowing rules between them.
* Whether a *checked* arithmetic exists as an explicit operation (`checked_add`)
  for the rare case that wants a `Result` instead of a trap.
* Whether `Float` needs its own NEP for NaN and comparison semantics, which
  IEEE leaves to the program.

## Implementation status

**Not implemented.** `nudo-lexer` produces integer and float literals and does
not compute their values; range checking and evaluation arrive with the type
checker (M3) and the interpreter (M4).
