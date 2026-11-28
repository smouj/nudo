# 02.2 — Expressions and precedence

> **Status:** Implemented / Specified / Planned
> **Summary:** The parser implements the grammar; core operator types and `if` branch rules are also checked. `match` and indexing still need semantic decisions.

Precedence is not formatting trivia. `a + b * c` must have one meaning in every
conforming implementation, and the parser produces exactly that shape: the tree
for `1 + 2 * 3` has the multiplication as the inner node and the addition above
it.

These are no longer open questions. They are stated once, as a chain of grammar
levels in
[`grammar/nudo.ebnf`](https://github.com/smouj/nudo/blob/main/grammar/nudo.ebnf),
copied identically into the two documents that describe them, and checked
mechanically by
[`scripts/check-grammar.py`](https://github.com/smouj/nudo/blob/main/scripts/check-grammar.py):

- postfix operations bind tighter than unary operators, and they chain, so
  `a.b(c)[d]` is a single expression;
- multiplication and division bind tighter than addition and subtraction, both
  left associative;
- comparison does **not** chain: `a < b < c` is a syntax error rather than
  `(a < b) < c`, and comparisons combine with `&&`;
- Boolean operators have fixed precedence, with `||` the loosest;
- there is no assignment level, because whether NUDO has mutable bindings at all
  is undecided. Writing the level now would be inventing a feature to fill a gap
  in a diagram;
- `ask`, `verify` and `delegate` are primary expressions rather than operators,
  which is what keeps `verify draft with V` parseable: `with` is not an operator,
  and no postfix form begins with it.

The rule that makes this checkable rather than aspirational is **no left
recursion**. A recursive-descent parser cannot handle a production that derives a
form starting with itself, and the first revision of this grammar had four such
productions. That is now a build failure, not a discovery waiting to happen.

The formatter should print enough syntax to preserve meaning without depending on
human intuition.

## Core operator and `if` rules

[NEP-0017](https://github.com/smouj/nudo/blob/main/neps/0017-boolean-comparison-conditionals.md)
defines the operator domains and conditional result types:

- `!`, `&&` and `||` use `Bool`; `&&` and `||` short-circuit from left to
  right. Invalid operator operands receive `NDO2018` at the operator.
- Equality compares matching primitive views of `Int`, `Float`, `Bool` or
  `Text`. Ordering is limited to matching `Int` or `Float`. There are no
  numeric conversions or user-defined overloads. The accepted
  `Verified<T>`-to-`T` view applies to operator inputs.
- Float comparisons follow IEEE-754: NaN is unequal to itself and unordered;
  positive and negative zero compare equal.
- An `if` condition is `Bool`. With `else`, an expected type checks both
  branches; without one, both branches must have exactly the same type. A
  mismatch is `NDO2019` at the `else` branch.
- An `if` without `else` has type `Unit` and its `then` branch must also be
  `Unit`.

At runtime, skipped short-circuit operands and unselected branches do not run.
Static effect checking still considers all of them, so a declaration remains
valid for every input.
