# A tour of NUDO

**Most of this page is a design description, not a working language.** Read the
milestone next to each feature before assuming it runs. Today the toolchain
lexes, parses and resolves names: the lexer (M1), the parser and lossless syntax
tree (M2) and name resolution in `nudo-hir` (M3.1) are implemented, and
`nudo check` runs all three. Type checking is not implemented yet, and neither is
anything after it. The toolchain's full state is in
[`../../ROADMAP.md`](../../ROADMAP.md).

## The shape of a program

```nudo
// Items: functions, structs, enums, agents, tasks, tools, models.
// Executable statements: bindings and expressions.

fn add(a: Int, b: Int) -> Int {
    a + b
}
```

A block's last expression is its value, so no `return` is needed for the common
case. There is no `return` at all: early exit is not part of the language, and a
conditional result is written as the final expression of the block
([NEP-0012](../../neps/0012-early-exit.md)).

## Types — M3

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

`match` must be exhaustive, there is no implicit numeric conversion, and there
is no truthiness: a condition is a `Bool`.

## Trust — M6

The idea the language is built around:

```nudo
let draft: Generated<Article> = ask Writer { "Create an article." };

let checked: Result<Verified<Article>, VerificationError> =
    verify draft with ArticleVerifier
```

* `Generated<T>` — a model produced it. Readable, storable, not trusted.
* `Verified<T>` — an explicit verification step passed, and left a record.
* `verify` yields a `Result`, because a verifier can reject and a rejection is a
normal outcome, not a panic. A `Verified<T>` cannot be obtained without handling
it.

There is no implicit conversion between them, or from either to `T`. That is the
point: the absence of a verification step has to be visible in the source,
because it is the single most important thing a reviewer needs to see.

`verify` is not a cast. It runs a verifier, can fail, and costs something. See
[`../../spec/trust/verified.md`](../../spec/trust/verified.md).

## Capabilities — M8

```nudo
fn fetch(url: Text) -> Text with Network {
    // this function needs Network, and the compiler knows it
}
```

The default is deny. A program holds exactly the capabilities it was granted,
and nothing is granted implicitly — not by the runtime, not by a tool, not by an
agent's prompt.

## Agents — M7

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

An agent declaration is a capability statement, not a prompt. It says what the
agent may call, what it may reach and what it may spend. `role` is for humans;
it is not a security boundary, and nothing enforces it.

## Tasks — M7

```nudo
task Research(topic: Text) -> Verified<Report> {
    agent:
        Researcher

    verify:
        SourcesRequired
}
```

A task binds an objective to an agent, an acceptance criterion and a budget. It
returns evidence, which is why its type is `Verified<…>` rather than `Report`.

## Interoperability — M9

NUDO consumes and exposes tools through MCP, and delegates to other agents
through A2A. In both directions the other side is untrusted by construction: a
remote agent's claims about itself are input, not evidence, and content that
arrives from outside is data, never instruction. See
[`../interoperability/README.md`](../interoperability/README.md).

## What is deliberately missing

* **Mutable bindings.** Immutability is the default, and a mutable binding needs
  a spelling and a reason. Neither exists yet.
* **Loops.** `for` and `while` are unsettled. A loop syntax invented early is a
  loop syntax the language is stuck with.
* **Classes and inheritance.** No structural typing, no subtyping between
  structs. If two types must be interchangeable, that is a named relationship.
* **Exceptions.** Failures are values. Effects are not exceptions.
* **A package ecosystem.** Nothing resolves dependencies yet.

The full list of open design questions is in
[`../../DESIGN.md`](../../DESIGN.md#open-questions), and each one needs a
[NEP](../../neps/README.md).
