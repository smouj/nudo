# Syntax reference

A readable companion to [`nudo.ebnf`](nudo.ebnf).

**Everything in this file except the "Lexical structure" section is
PRE-ALPHA and provisional.** The lexer (milestone M1) accepts the lexical
structure; nothing accepts the rest yet. See
[`../spec/grammar.md`](../spec/grammar.md).

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
| Reserved words | `fn`, `let` — and nothing else, yet |
| Punctuation | `( ) { } : , -> = + - * / ;` |

Details, including what is rejected and why, are in
[`../spec/lexical-structure.md`](../spec/lexical-structure.md).

## Items — provisional

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
        web.search
        web.open

    allow:
        Network

    budget:
        Budget(tokens: 20_000, tool_calls: 30)
}
```

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
tool web.search(query: Text) -> [Result] with Network, Budget {
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

| Operator | Precedence |
| -------- | ---------- |
| `f(x)`, `a.b`, `a[i]` | 1 |
| `-`, `!` | 2 |
| `*`, `/` | 3 |
| `+`, `-` | 4 |
| `<`, `>`, `<=`, `>=` | 5 |
| `==`, `!=` | 6 |
| `&&` | 7 |
| `\|\|` | 8 |

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
