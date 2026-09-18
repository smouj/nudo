# Expressions and statements

**State: the expression grammar is implemented** (M2). Precedence,
associativity and the shape of a block are what the parser accepts and what
`tests/conformance/parser` pins. The agentic expressions at the end of this
chapter are still provisional: their productions are implemented as written, but
their form can change through the NEPs named there.

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
multiplicative      *  /                        left
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
* **No truthiness.** A condition is a `Bool`.

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
* `match` must be exhaustive.
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
