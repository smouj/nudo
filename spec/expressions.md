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

## Operators

Provisional precedence, highest first:

| Level | Operators |
| ----- | --------- |
| 1 | postfix call `f(x)`, field `a.b`, index `a[i]` |
| 2 | unary `-`, `!` |
| 3 | `*`, `/` |
| 4 | `+`, `-` |
| 5 | comparison `<`, `>`, `<=`, `>=` |
| 6 | equality `==`, `!=` |
| 7 | logical `&&` |
| 8 | logical `||` |

Operator precedence is a common source of quiet bugs, and an `Int`/`Float`
mismatch must be an error rather than a coercion. The exact table is provisional
and will be frozen with conformance cases when the parser lands.

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
