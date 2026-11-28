# Expressions and statements

**State: expression syntax is implemented; core operator and conditional rules
are accepted and implemented** (NEP-0017). `match`, indexing and the agentic
expressions at the end of this chapter still need their semantic gates.

## Shape

A block is a sequence of statements followed by an optional final expression,
which is the block's value:

```nudo
fn add(a: Int, b: Int) -> Int {
    a + b
}
```

This is deliberately Rust-shaped: it keeps expressions and statements from
needing two spellings of the same idea.

<!-- PRECEDENCE: expression > logical-or > logical-and > equality > comparison > additive > multiplicative > unary-expression > postfix-expression > primary-expression -->

## Operators

Precedence is decided, not provisional. It is expressed as a chain of grammar
levels, each defined in terms of the next, which is what makes a
recursive-descent parser possible and what stops the grammar and the prose from
drifting apart:

```text
expression
  │
logical-or          ||                          left
  │
logical-and         &&                          left
  │
equality            ==  !=                      left
  │
comparison          <  >  <=  >=                none: a < b < c is an error
  │
additive            +  -                        left
  │
multiplicative      *  /  %                     left
  │
unary-expression    -x  !x                      prefix, right
  │
postfix-expression  f(x)  a.b  a[i]             left, chains
  │
primary-expression  literals, paths, ( … ), blocks, if, match, ask, verify, delegate
```

Rules that follow, and are normative:

* **No left recursion.** `nudo-parser` is a recursive-descent parser, so no
  production may derive a form that starts with itself. The grammar is checked
  mechanically for this.
* **Postfix forms chain.** `a.b(c)[d]` is a single expression.
* **Comparison does not chain.** `a < b < c` is a syntax error, not
  `(a < b) < c`. Comparisons combine with `&&`.
* **No implicit numeric conversion.** An `Int`/`Float` mismatch is a type error,
  not a coercion.
* **Remainder is integer-only.** `%` takes and returns `Int`; its sign and
  division behavior are fixed by [NEP-0007](../neps/0007-integer-semantics.md).
* **No truthiness.** A condition is a `Bool`.

### Operator types and evaluation

These rules are normative ([NEP-0017](../neps/0017-boolean-comparison-conditionals.md)):

* `!` accepts an operand assignable to `Bool` and returns `Bool`. `&&` and
  `||` accept two operands assignable to `Bool` and return `Bool`; they evaluate
  left to right and short-circuit.
* `==` and `!=` accept operands with the same primitive view among `Int`,
  `Float`, `Bool` and `Text`. Text equality compares UTF-8 contents exactly,
  without Unicode normalization. Functions and named types are not comparable.
* `<`, `>`, `<=` and `>=` accept two operands with the same primitive view,
  either `Int` or `Float`. Integer ordering is signed. Text has no ordering
  operation.
* Float equality and ordering follow IEEE-754: NaN is unequal to itself and is
  unordered; positive and negative zero compare equal.
* The built-in `Verified<T>` is usable as `T` in operator inputs under the
  ordinary assignment rule. This view does not apply to `Generated<T>` or add
  any other conversion.
* No operator converts operands or uses user-defined overloads. A type-domain
  error is reported at the operator as `NDO2018`.
* Static effect analysis considers both sides of a short-circuit operator and
  both branches of an `if`, whether or not runtime evaluation reaches them.

**There is no assignment expression.** An assignment needs a mutable binding, and
whether NUDO has mutable bindings at all is an open question
([`declarations.md`](declarations.md)). When that is answered, assignment takes
the position above `logical-or`, and a NEP records it. Writing the level now,
with nothing to assign to, would be inventing a language feature to fill a gap in
a diagram.

**`?` is a type operator only.** `T?` is an optional type
([`types.md`](types.md)). Expression-level error propagation is an open question
with no syntax: the `?`-suffix form that other languages use is a proposal, not a
decision, and it would need a NEP because it changes control flow.

The precedence chain above is the single source of truth. The same marker appears
in [`../grammar/nudo.ebnf`](../grammar/nudo.ebnf) and
[`../grammar/syntax-reference.md`](../grammar/syntax-reference.md), and
`scripts/check-grammar.py` fails if the three disagree or if the productions stop
implementing it.

## `let`

```nudo
let draft: Generated<Article> = ask Writer { "Create an article." }
let article: Verified<Article> = verify draft with ArticleVerifier
```

* The type annotation may be omitted when the initialiser determines it.
* A binding is immutable. Reassignment is not part of the language yet.
* Shadowing rules are open.

## Control flow

```nudo
if ready {
    run()
} else {
    wait()
}

match status {
    Pending => wait()
    Running => poll()
    Complete => finish()
    Failed(reason) => report(reason)
}
```

* `if` is an expression, not a statement.
* The condition is evaluated first and must be `Bool`. With an `else`, only the
  selected branch runs. An expected type checks both branches using ordinary
  assignment rules; without one, their types must be identical. The conditional
  does not infer a common supertype or choose based on branch order. A mismatch
  without an expected type is `NDO2019`, reported at the `else` branch.
* Without an `else`, the `then` branch must have type `Unit`, and the whole `if`
  has type `Unit`.
* An `else if` follows these same rules.
* `match` must be exhaustive, and uniformly so: an unhandled variant is an
error whatever the enum is, including `Result<Verified<T>, E>`. NUDO has no
must-use rule in this edition, so an ignored `Result` is not an error
([NEP-0014](../neps/0014-trust-types-obey-the-rules.md)).
* There is no implicit truthiness: the condition is `Bool`.
* **There is no `return`, and no early exit.** A block's value is its final
  expression, and both `if` and `match` are expressions, so a conditional result
  is written where the value is. Nothing leaves a block early, and the word
  `return` stays an ordinary identifier
  ([NEP-0012](../neps/0012-early-exit.md)). The rule this rests on is the one in
  [Shape](#shape): the optional final expression is the block's value.

Loops are provisional. Whether NUDO has `for`, `while`, iterators, or both is
not decided, and a loop syntax invented early is a loop syntax the language is
stuck with. An exit form for a loop — `break`, `continue`, an early `return` —
is decided with the loop, not before it.

## The agentic expressions

These are the expressions this language exists for. All are **provisional**, and
each needs a NEP that settles whether it is a keyword, a function or a protocol.
Their productions are implemented — the parser accepts the forms below — but a
NEP that changes the shape changes the parser with it.

### `ask` — produce a `Generated<T>`

```nudo
let draft: Generated<Article> = ask Writer { "Create an article." }
```

`ask` performs a model call. It requires the model capability, consumes budget,
and **always** yields `Generated<T>`. There is no form of `ask` that yields a
plain `T`, because that would be a name for "trust this without checking".

### `verify` — produce a `Verified<T>`

```nudo
let checked: Result<Verified<Article>, VerificationError> =
    verify draft with ArticleVerifier
```

`verify` runs a verifier against a value and yields
**`Result<Verified<T>, VerificationError>`**: a rejected verification is a normal
value the caller handles, not a panic and not a cast
([NEP-0002](../neps/0002-generated-verified.md)). A `Verified<T>` cannot be
obtained without handling the failure, which is what makes the verification step
visible in the source.

**The operand is a named value**: a path, optionally called (`verify draft`,
`verify parse(text)`). A parenthesised or compound operand is not in the grammar,
and that is what lets the parser read `verify` as a verification with one token of
lookahead and no backtracking:

```nudo
let a = verify draft with Verifier;   // a verification
let b = verify(draft);                // a call on something named `verify`
```

`verify` remains a contextual word ([NEP-0005](../neps/0005-keyword-policy.md)),
and the earlier one-token limitation is gone: it was removed by restricting what
can be verified, not by reserving the word.

A verifier is a declaration whose body is **deterministic**. Verification by a
model is deferred to M6, which must say whether the result is
`Generated<Verified<T>>` and how the loop closes — a model's judgement about its
own output cannot be the step that promotes it.

### Tool calls

```nudo
let results = web.search("nudo language")
```

A tool call is capability-checked at the call site. `web.search` requires
`Network`; a context without it cannot make this call, and the error names the
capability and the call site.

### `delegate` — hand work to another agent

Provisional syntax, and the subject of
[`agents/delegation.md`](agents/delegation.md). The rule that will not change:
delegation may narrow capabilities and may never widen them.

## What expressions may not do

* Convert `Generated<T>` to `Verified<T>` implicitly.
* Perform an effect that the enclosing declaration does not declare.
* Hide a budget cost. If an expression spends, it is visible at the call site
  and in the trace.
* Introduce a new scope form to save a keyword.
