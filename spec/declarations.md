# Declarations

**State: proposed.** Nothing below is implemented. `fn` and `let` are the two
reserved words today ([`lexical-structure.md`](lexical-structure.md)), and even
they are only lexed. Every other keyword here is provisional and must pass a
[NEP](../neps/README.md) before it is reserved.

## Functions

```nudo
fn add(a: Int, b: Int) -> Int {
    a + b
}
```

* Parameters are annotated. The return type is annotated, or omitted for `Unit`.
* A function body is a block whose last expression is its value; `return` exists
  for early exit.
* A function that performs an effect declares it
  ([`effects.md`](effects.md)).
* Functions are values: `add` has type `Fn(Int, Int) -> Int`.

## Structs

```nudo
struct User {
    name: Text
    age: Int
}
```

Fields are named and typed. Whether a field may be mutable, and how
construction works, are open questions; the syntax above deliberately has no
separators, because a separator that carries no meaning is noise.

## Enums

```nudo
enum Status {
    Pending
    Running
    Complete
    Failed(reason: Text)
}
```

Variants may carry data. Matching over an enum must be exhaustive, and a
non-exhaustive match is a compile error, not a runtime fallback.

## Agents

```nudo
agent Researcher {
    role:
        "Research reliable information."

    tools:
        web.search
        web.open

    allow:
        Network
}
```

An agent declaration is a **capability statement**, not a prompt. It says:

* what the agent is for (`role`);
* which tools it may use (`tools`);
* which capabilities it holds (`allow`);
* what it may spend (`budget`, see [`agents/budget.md`](agents/budget.md)).

Nothing outside the declaration grants the agent anything. An agent that did not
declare `Network` cannot reach the network, even if a tool it was given can. The
full rules are in [`agents/agent.md`](agents/agent.md).

## Tasks

```nudo
task Research(topic: Text) -> Verified<Report> {
    agent:
        Researcher

    verify:
        SourcesRequired
}
```

A task binds an objective to an agent, an acceptance criterion and a budget. Its
return type is `Verified<...>`, because a task is expected to produce evidence,
not a claim. See [`agents/task.md`](agents/task.md).

## Tools

```nudo
tool web.search(query: Text) -> [Result] with Network, Budget {
    // …
}
```

A tool declaration says what may happen and what it costs. A tool is the only
way a program affects the world outside the process, and every tool is
capability-gated. See [`agents/tool.md`](agents/tool.md).

## Models

```nudo
model LocalLlama {
    provider: "local"
    // …
}
```

A model declaration names a provider and a configuration. The language does not
privilege any provider, and the interface is provider-agnostic by construction
([`agents/model.md`](agents/model.md)).

## Constants and bindings

```nudo
let answer = 42;              // immutable binding
const LIMIT: Int = 1_000;     // compile-time constant (provisional)
```

Whether NUDO has mutable bindings at all, and what they are called, is an **open
question**. The language is designed against immutability by default; a mutable
binding needs a reason and a spelling, and neither exists yet.

## Visibility and modules

Items are visible inside their module. `pub` and explicit re-exports are
provisional and described in [`modules.md`](modules.md).

## What a declaration must never do

* Annotate a type the specification does not define.
* Grant a capability implicitly — there is no "inherits its caller's
  capabilities".
* Hide an effect. If calling something can reach the network, the declaration
  says so.
