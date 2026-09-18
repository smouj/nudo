# NEP-0012: Early exit

| Field | Value |
| ----- | ----- |
| Status | Accepted |
| Created | 2026-09-18 |
| Accepted | 2026-09-18 |
| Supersedes | — |
| Superseded by | — |
| Specification chapters | [`spec/declarations.md`](../spec/declarations.md), [`spec/expressions.md`](../spec/expressions.md), [`spec/grammar.md`](../spec/grammar.md) |
| Related NEPs | NEP-0005, NEP-0008, NEP-0009 |

## Summary

Decide whether NUDO has a `return`. It does not, in this edition. A block's
value is its final expression, `if` and `match` are expressions, and this edition
has neither loops nor a `?` operator, so no body needs to leave a block early.
Two documents claimed a `return` the frozen EBNF never had; they are corrected to
match the grammar rather than the grammar extended to match them.

## Motivation

The specification and the grammar disagreed, and both could not be right. The
grammar is the frozen artefact, the parser implements it, and the claim lived
only in prose:

```nudo
// spec/declarations.md and docs/language/tour.md described this as valid.
// The EBNF has no `return-statement`, so nudo-parser rejects it.
fn divide(a: Int, b: Int) -> Outcome<Int, Text> {
    if b == 0 {
        return Outcome.Err("division by zero")
    }
    Outcome.Ok(a / b)
}
```

The version of this program that the language already accepts needs nothing
added:

```nudo
fn divide(a: Int, b: Int) -> Outcome<Int, Text> {
    if b == 0 {
        Outcome.Err("division by zero")
    } else {
        Outcome.Ok(a / b)
    }
}
```

That second program is not a workaround. It is the same function with the
conditional written as a conditional expression, and it is the shape
[NEP-0008](0008-error-model.md) already chose for exactly this example. `return`
would be a second way to say what the block value already says.

## Guide-level explanation

A function's result is the value of its body. The body is a block, and a block's
value is its final expression:

```nudo
fn add(a: Int, b: Int) -> Int {
    a + b
}
```

When a result is conditional, the condition is written where the value is:

```nudo
fn classify(n: Int) -> Text {
    if n < 0 {
        "negative"
    } else if n == 0 {
        "zero"
    } else {
        "positive"
    }
}
```

There is no `return`, no early exit, and no statement that leaves a block. The
word `return` is an ordinary identifier: it is neither in the reserved core nor
in the contextual set of [NEP-0005](0005-keyword-policy.md), so a program may
still bind it. Nothing about the language changes; a claim about it is withdrawn.

## Reference-level explanation

* **No new production.** `block = "{", { block-statement }, [ expression ], "}"`
  is unchanged, and `return` is not a `block-statement`. The EBNF has no
  `return-statement` and gains none.
* **`return` is not reserved, and not contextual.** Reserving a word is the
  expensive move in [NEP-0005](0005-keyword-policy.md): it is a compatibility
  promise, and it is reserved for a construct that cannot be written another way.
  `return` introduces no construct, so nothing is reserved and `return` stays a
  usable name.
* **A stray `return` is an ordinary syntax error.** The parser reads it as a path
  where a statement is expected, then reports the token it could not place:
  `NDO1001` with the expected/found pair. No special diagnostic is added, because
  a design that removes a feature should not add machinery to explain its
  absence. `tests/conformance/parser/0017-no-return` pins the rejection.
* **The rule this rests on is already implemented.** "A block is a statement list
  with an optional final expression, which is the block's value" is in
  [`spec/expressions.md`](../spec/expressions.md) and is what the parser builds;
  `if` and `match` are primary expressions, so a conditional value needs no
  statement form.
* **Typing is not affected.** Without `return` there is no diverging expression
  and no new typing rule for the M3 checker to carry. The type of a block remains
  the type of its final expression, or `Unit` when it has none.

## What this makes impossible

* **Guard clauses.** A body with several independent checks nests instead of
  flattening:

  ```nudo
  fn transfer(amount: Int, balance: Int) -> Result<Int, Text> {
      if amount < 0 {
          Err("negative amount")
      } else {
          if amount > balance {
              Err("insufficient funds")
          } else {
              Ok(balance - amount)
          }
      }
  }
  ```

  This is the real cost, and it is accepted for three reasons: this edition has
  no loops, so there is nothing to break out of; NEP-0008 already deferred the
  `?` operator on the same reasoning — a few extra lines per call site for an
  unambiguous control flow; and an exit form is cheaper to add, and harder to get
  wrong, once the loop and error-propagation forms it belongs with are decided.
* **Early exit from a loop.** Not possible. Loops are not decided either
  ([`spec/expressions.md`](../spec/expressions.md) leaves them provisional), and
  `break`, `continue` and `return` are one design, not three. Deciding one before
  the others is how a language ends up with three spellings of the same idea.
* **Reusing the word for a binding** is not impossible — it is explicitly
  allowed, and that is a property of this decision rather than a loss.

## Alternatives

| Alternative | Why it lost |
| ----------- | ----------- |
| Add `return` to the EBNF and implement it | It would need a reserved keyword, because `return x` and `return(x)` are otherwise indistinguishable from a path and a call on a name called `return`; and it would give the block-value rule a second spelling for the same idea. Additive later, so it does not have to be taken now — and should be taken with the loop design, not before it |
| Leave the sentence and the grammar as they are | That is the current state: a feature documented in two places, absent from the grammar and rejected by the parser. [`../AGENTS.md`](../AGENTS.md) forbids a production that exists only in the implementation; a feature that exists only in prose is the same dishonesty with the sign flipped |
| Keep `return` but make it contextual | Contextual words are recognised "only where the grammar expects that construct" (NEP-0005). A `return` at statement position is exactly where a path-expression is also expected, so the word could not be read without lookahead, and NEP-0005 rules that out |
| A `?`-style early exit instead | [NEP-0008](0008-error-model.md) already decided there is no `?` in this edition, for the same reason: the propagation path must be visible |
| Another spelling (`yield`, `give`, `exit`) | Same keyword cost, same second spelling, no new capability — `yield` in particular would collide with the generator meaning it has elsewhere |

## Impact

| Area | Effect |
| ---- | ------ |
| Specification | `spec/declarations.md` and `spec/expressions.md` state that there is no early exit; `spec/grammar.md` moves `return` from its open questions to its decided table |
| Grammar | None: the production never existed, so the EBNF is unchanged and `scripts/check-grammar.py` is unaffected |
| Compiler | None: the parser already rejects `return`, and no crate changes |
| Conformance | One case: `tests/conformance/parser/0017-no-return` pins the rejection |
| Tooling | None |
| Security | None. An exit form is not a capability and does not affect trust |
| Compatibility | Nothing breaks: `return` was never accepted, so no program that parsed before stops parsing |
| Performance | None |
| Documentation | The manual's tour and the book's parser chapter are corrected, in every language the book is published in |

## Open questions

* Whether a later edition adds an exit form at all, and whether it is spelled
  `return` or settled together with `break`/`continue` when loops are decided.
* Whether an exit form, if added, is a statement or an expression. This edition
  answers neither, because it has neither the loops nor the `?` that would make
  the answer obvious.

## Prior art

* **Elm** is the closest analogue and the deliberate precedent. It has no
  `return`, no loops, no `break`, no `continue` and no mutation; `if` and `case`
  are expressions; and failure is a `Result` handled by explicit combinators. It
  is a language people ship in, which is the evidence that "no early exit" is a
  workable position rather than an unfinished one.
* **Rust** has both the block value and `return`, and the reason is instructive:
  `return` earns its place there because of loops and the `?` operator, both of
  which NUDO has deferred. Rust without loops and without `?` would not need it
  either.
* **Scheme and other expression languages** treat early exit as an escape
  continuation rather than a statement — a design that confirms the general
  point: an exit is a control-flow feature, and control flow is designed as a
  whole.

## Unresolved objections

Accepted 2026-09-18 with none outstanding. One objection was considered and
answered on the record: that a language without early exit makes guard clauses
nest. It does, it is shown under "What this makes impossible", and it is accepted
because the nesting is bounded by a language with no loops, and because the exit
form belongs with the loop and propagation forms that are still undecided.

## Implementation status

**Implemented — and there was nothing to implement.** This NEP removes a claim,
so the "implementation" is the correction: the EBNF is untouched, the parser
already rejects `return` with `NDO1001`, and the specification, the manual, the
example and the conformance corpus now agree with both. Because no crate changes,
the specification change and its consequence are a single pull request, which
[`../CONTRIBUTING.md`](../CONTRIBUTING.md) allows when there is no implementation
to sequence after it.
