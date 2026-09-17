# 02.2 — Expressions and precedence

> **Status:** Open  
> **Summary:** Expression precedence and associativity must be unambiguous before M2 can be considered stable enough for downstream tools.

Precedence is not formatting trivia. `a + b * c` must have one meaning in every
conforming implementation.

A parser design should make these properties explicit:

- postfix operations bind tighter than unary operators;
- multiplication/division bind tighter than addition/subtraction;
- comparison and equality have defined chaining behaviour;
- Boolean operators have fixed precedence;
- assignment, if introduced, must define associativity;
- future `ask`, `verify` or `delegate` forms must state whether they are primary
  expressions, prefix expressions or declarations.

The formatter should print enough syntax to preserve meaning without depending on
human intuition.
