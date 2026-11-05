# NEP-0017: Boolean, comparison and conditional expressions

| Field | Value |
| ----- | ----- |
| Status | Draft |
| Authors | @smouj |
| Created | 2026-11-06 |
| NEP number | 0017 |
| Supersedes | — |
| Superseded by | — |
| Specification chapters | [`spec/expressions.md`](../spec/expressions.md), [`spec/types.md`](../spec/types.md), [`spec/errors.md`](../spec/errors.md) |
| Related NEPs | NEP-0007, NEP-0008, NEP-0013, NEP-0014 |

## Summary

Define the operand and result types for Boolean operators, equality and ordering,
and the result rules for `if`. Boolean operators short-circuit, comparisons do
not coerce operands, and conditional branches must agree without an expected
type. This proposal supplies the missing M3.2 rules needed before M4 can execute
the full conditional and comparison slice. It changes no specification or
compiler behavior until accepted.

## Motivation

The grammar already accepts these expressions, but the type checker does not yet
have complete rules for their operands or results. M4 plans to evaluate
comparisons and `if`, so implementing them from familiar-language convention
would silently define NUDO. For example, this program needs decided rules for
short-circuit evaluation, numeric comparison and conditional result types:

```nudo
fn classify(value: Int, ready: Bool) -> Int {
    if ready && value >= 0 {
        if value == 0 { 0 } else { value }
    } else {
        -1
    }
}
```

The checker must also say whether `1 == 1.0`, `"a" < "b"`, or an `if`
expression without `else` has a type. Leaving those answers to the backend
would let two backends accept different programs.

## Guide-level explanation

`!`, `&&` and `||` operate only on `Bool`; `&&` and `||` evaluate the right
operand only when its value can affect the result. Equality compares values of
the same primitive type. Ordering is available for `Int` and `Float` only.
There is no truthiness, implicit numeric conversion or user-defined operator
overloading.

An `if` condition is `Bool`. With `else`, both branches produce the same type
unless an enclosing context supplies an expected type. Without `else`, the
`then` branch must produce `Unit`, and the whole expression has type `Unit`.

## Reference-level explanation

### Boolean and comparison operators

* `!` accepts one `Bool` and returns `Bool`.
* `&&` and `||` accept two `Bool` operands and return `Bool`. Operands are
  evaluated left to right. `false && right` does not evaluate `right`, and
  `true || right` does not evaluate `right`.
* `==` and `!=` accept two values of the same type among `Int`, `Float`, `Bool`
  and `Text`, and return `Bool`. `Text` equality compares the UTF-8 contents
  exactly; it performs no Unicode normalization. Functions and named types are
  not comparable in this edition.
* `<`, `>`, `<=` and `>=` accept two values of the same type, either `Int` or
  `Float`, and return `Bool`. `Int` ordering is signed. `Text` has no ordering
  operation in this edition.
* `Float` comparison follows IEEE-754: `NaN == NaN` is false, `NaN != NaN` is
  true, and each ordering comparison involving NaN is false. Positive and
  negative zero compare equal.
* Mixed `Int`/`Float` operands and all other unsupported operand pairs are
  rejected. No conversion or operator overloading is inferred.
* Comparison operators do not chain. The grammar already rejects `a < b < c`;
  write `a < b && b < c` instead.

### Conditional expressions

* The condition is evaluated first and must have type `Bool`.
* With an `else` branch, only the selected branch is evaluated. When the
  surrounding context supplies an expected type `T`, each branch is checked
  against `T` using ordinary assignment rules. This preserves the accepted
  `Verified<T>`-to-`T` rule without adding a conditional-specific conversion.
* Without an expected type, the two branch types must be equal. The checker does
  not choose a type from branch order or infer a common supertype.
* Without an `else`, the `then` branch must have type `Unit`; the expression's
  type is `Unit`. A value-producing `then` branch without an `else` is rejected
  with `NDO2004`, expected `Unit`.
* An `else if` is an `if` expression in the `else` position and follows the same
  rules.
* Static effect analysis considers both branches and both operands of `&&` and
  `||`, even though runtime evaluation may skip one. This keeps declared effects
  valid for every input.

### Diagnostics

This proposal assigns two type-diagnostic codes:

| Code | Name | Meaning |
| ---- | ---- | ------- |
| `NDO2008` | `INVALID_OPERATOR_TYPES` | An operator receives operand types outside its defined domain. |
| `NDO2009` | `IF_BRANCH_TYPE_MISMATCH` | Branch types differ when no expected type is available. |

The primary span for `NDO2008` is the operator token. The diagnostic includes
both operand types when there are two. A condition that is not `Bool` uses the
ordinary `NDO2004 TYPE_MISMATCH` at the condition. A branch that fails an
expected type also uses `NDO2004` at that branch. Without an `else`, a non-`Unit`
`then` branch uses `NDO2004`, expected `Unit`.

## What this makes impossible

* Treating arbitrary values as true or false.
* Comparing unlike numeric types through an implicit conversion.
* Running the right side of `&&` or `||` when short-circuiting says it is not
  needed.
* Letting branch order choose an `if` expression's result type.
* Comparing functions or named values without a separate equality design.
* Ordering `Text` by a locale-dependent collation.

## Alternatives

| Alternative | Why it lost |
| ----------- | ----------- |
| Truthiness for conditions | It makes a condition's meaning depend on the value's type and hides mistakes such as using an integer as a flag. |
| Coerce `Int` to `Float` during comparison | It can lose integer precision and makes the same expression behave differently from assignment and arithmetic. |
| Infer the type from the first `if` branch | Reordering branches would change the program's type. |
| Give an `if` without `else` the type of its `then` branch | It creates a value on only one path and makes the absent branch require an implicit value. |
| Allow user-defined operator overloads now | It adds trait resolution before the type system and M4 need it. |

## Impact

| Area | Effect |
| ---- | ------ |
| Specification | Defines Boolean operators, primitive comparisons and conditional result types in `spec/expressions.md` and `spec/types.md`; registers `NDO2008` and `NDO2009` in `spec/errors.md` |
| Grammar | No grammar change; the forms already parse and comparison chaining is already rejected |
| Compiler | Completes operand/result checking in `nudo-typeck`; MIR and interpreter implement the specified evaluation order in M4 |
| Conformance | Adds accepted and rejected operand pairs, NaN behavior, short-circuit cases, expected-type branches, mismatched branches and no-`else` cases |
| Tooling | No formatter or CLI syntax change |
| Security | Effects in skipped runtime branches still appear in static effect summaries |
| Compatibility | Previously unchecked expressions may now be rejected; no accepted program changes meaning because these rules were not implemented |
| Performance | Constant-time operator checks; short-circuit evaluation avoids evaluating an unused right operand |

## Open questions

None affect the rules proposed here. User-defined operators and comparisons for
named types require separate proposals.

## Prior art

Rust and other statically typed languages use Boolean-only conditions and
short-circuit logical operators. This proposal keeps those familiar rules and
does not adopt their unrelated operator traits or coercion rules.

## Unresolved objections

None recorded. This is a Draft and has not entered review.
