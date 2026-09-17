# Expressions and statements

**State: proposed.** Nothing here is implemented beyond the tokens involved.

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

Loops are provisional. Whether NUDO has `for`, `while`, iterators, or both is
not decided, and a loop syntax invented early is a loop syntax the language is
stuck with.

## The agentic expressions

These are the expressions this language exists for. All are **provisional**, and
each needs a NEP that settles whether it is a keyword, a function or a protocol.

### `ask` — produce a `Generated<T>`

```nudo
let draft: Generated<Article> = ask Writer { "Create an article." }
```

`ask` performs a model call. It requires the model capability, consumes budget,
and **always** yields `Generated<T>`. There is no form of `ask` that yields a
plain `T`, because that would be a name for "trust this without checking".

### `verify` — produce a `Verified<T>`

```nudo
let article: Verified<Article> = verify draft with ArticleVerifier
```

`verify` runs a verifier against a value and either produces `Verified<T>` with
provenance attached, or fails. It is not a cast: a verifier can reject, and a
rejected verification is a normal, checkable outcome.

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
