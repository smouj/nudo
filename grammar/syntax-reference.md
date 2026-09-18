# Syntax reference

A readable companion to [`nudo.ebnf`](nudo.ebnf).

**The lexical and syntactic structure below is what the toolchain accepts.**
The lexer (milestone M1) and the parser (milestone M2) implement it, and
`tests/conformance/lexer` and `tests/conformance/parser` pin it. What is still
PRE-ALPHA is the *meaning*: several items are marked provisional because a NEP
can still change their shape. See [`../spec/grammar.md`](../spec/grammar.md).

## Lexical structure — implemented

```nudo
// A line comment
/* A block comment, /* which nests */ like this */

fn add(a: Int, b: Int) -> Int {
    a + b
}

let answer = 42;
let grouped = 1_000_000;
let ratio = 3.5;
let message = "escapes: \n \r \t \\ \" \0";
```

| Kind | Form |
| ---- | ---- |
| Identifier | `[A-Za-z_][A-Za-z0-9_]*` — ASCII |
| Integer | `42`, `1_000_000` |
| Float | `3.5`, `1_0.2_5` — a `.` followed by a digit |
| Text | `"…"`, closed on the same line |
| Reserved words | `fn`, `let`, `struct`, `enum`, `if`, `else`, `match`, `const`, `true`, `false` |
| Contextual words | `agent`, `task`, `tool`, `model`, `role`, `tools`, `allow`, `budget`, `with`, `verify`, `ask`, `delegate` — keywords in position, ordinary names elsewhere |
| Punctuation | `( ) { } [ ] : :: , . ? -> => = == != < > <= >= && \|\| ! + - * / ;` |

Details, including what is rejected and why, are in
[`../spec/lexical-structure.md`](../spec/lexical-structure.md).

## Items — syntax implemented

### Functions

```nudo
fn add(a: Int, b: Int) -> Int {
    a + b
}

fn nothing() {}

fn fetch(url: Text) -> Text with Network {
    // effect clause: calling this requires the Network capability
}
```

### Structs and enums

```nudo
struct User {
    name: Text
    age: Int
}

enum Status {
    Pending
    Running
    Complete
    Failed(reason: Text)
}
```

### Bindings and constants

```nudo
let answer = 42;
let draft: Generated<Article> = ask Writer { "Create an article." };

const LIMIT: Int = 1_000;
```

### Agents

```nudo
agent Researcher {
    role:
        "Research reliable information."

    tools:
        web::search, web::open

    allow:
        Network

    budget:
        Budget(tokens: 20_000, tool_calls: 30)
}
```

A list inside a declaration is comma-separated, and a tool's name is a `path`:
its segments are separated by `::`, so the tool is `web::search`. Writing one
name per line, or separating segments with `.`, is a syntax error — see the
recorded disagreements in [`../spec/grammar.md`](../spec/grammar.md).

An agent declaration is a capability statement. Read it as: this agent may call
these tools, may hold these capabilities, and may spend this much. See
[`../spec/agents/agent.md`](../spec/agents/agent.md).

### Tasks

```nudo
task Research(topic: Text) -> Verified<Report> {
    agent:
        Researcher

    verify:
        SourcesRequired

    budget:
        Budget(tokens: 20_000, tool_calls: 30)
}
```

A task returns evidence, not a claim, which is why its result is `Verified<…>`.
See [`../spec/agents/task.md`](../spec/agents/task.md).

### Tools and models

```nudo
tool web::search(query: Text) -> [Result] with Network, Budget {
    // implementation
}

model LocalLlama {
    provider: "local"
}
```

## Expressions — provisional

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

<!-- PRECEDENCE: expression > logical-or > logical-and > equality > comparison > additive > multiplicative > unary-expression > postfix-expression > primary-expression -->

The levels below are the whole definition of precedence. Each level is written in
terms of the next one, so the grammar and this table cannot disagree — and
`scripts/check-grammar.py` fails the build if they do.

| Level | Operators | Associativity |
| ----- | --------- | ------------- |
| `logical-or` | `\|\|` | left |
| `logical-and` | `&&` | left |
| `equality` | `==`, `!=` | left |
| `comparison` | `<`, `>`, `<=`, `>=` | **none** — `a < b < c` is an error |
| `additive` | `+`, `-` | left |
| `multiplicative` | `*`, `/` | left |
| `unary-expression` | `-x`, `!x` | prefix, right |
| `postfix-expression` | `f(x)`, `a.b`, `a[i]` | left, chains |
| `primary-expression` | literals, paths, `( … )`, blocks, `if`, `match`, `ask`, `verify`, `delegate` | — |

Postfix forms chain, so `a.b(c)[d]` is one expression. Comparison does not chain:
write `a < b && b < c` instead, because `a < b < c` reads as arithmetic on a
boolean more often than it reads as a mistake.

There is no `?` operator in expressions. `?` is a type operator, and
expression-level error propagation is an open question that needs a NEP before it
has syntax. There is also no assignment expression: mutability itself is
undecided, so assignment has nothing to assign to yet.

No implicit numeric conversion, no truthiness, and `match` must be exhaustive.

## The agentic expressions — provisional

```nudo
// Produce a Generated<T>. There is no form of `ask` that produces a plain T.
let draft: Generated<Article> = ask Writer { "Create an article." }

// Produce a Verified<T>. This can fail; it is not a cast.
let article: Verified<Article> = verify draft with ArticleVerifier

// Delegate with a narrowed budget. Authority narrows, never widens.
let report = delegate Researcher {
    topic: "nudo language"
} with budget: Budget(tokens: 4_000)
```

## Types — provisional

| Form | Meaning |
| ---- | ------- |
| `Int`, `Float`, `Bool`, `Text`, `Unit` | Primitives |
| `[T]` | Sequence |
| `Fn(A, B) -> C` | Function value |
| `T?` | Optional |
| `Generated<T>` | Produced by a model, unchecked |
| `Verified<T>` | Passed an explicit verification step |

`Generated<T>` and `Verified<T>` are different types, and there is no implicit
conversion between them or to `T`. See
[`../spec/trust/generated.md`](../spec/trust/generated.md).

## Reading the examples

Examples in [`../examples`](../examples) follow this syntax. Those from
`05-agent/` onwards illustrate proposed syntax and are **not** accepted by the
current toolchain; each says so in its own header, and
[`../examples/README.md`](../examples/README.md) marks which ones the toolchain
can read today.
