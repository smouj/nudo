# 02.2 — Expressions and precedence

> **Status:** Specified  
> **Summary:** Precedence, associativity and the ban on left recursion are decided and enforced. The parser that implements them is M2.

Precedence is not formatting trivia. `a + b * c` must have one meaning in every
conforming implementation.

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
