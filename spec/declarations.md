# Declarations

**State: the syntax is implemented; the meaning is proposed.** The parser accepts
every declaration form below (`compiler/nudo-parser`, pinned by
`tests/conformance/parser`). Nothing resolves a name or checks a type yet: that
is M3. The reserved words are the core [lexical-structure.md](lexical-structure.md)
fixes; the words that introduce the declarations — `agent`, `task`, `tool`,
`model` — are contextual, so they remain usable as names elsewhere.

## Functions

```nudo
fn add(a: Int, b: Int) -> Int {
    a + b
}

fn identity<T>(value: T) -> T {
    value
}
```

* Parameters are annotated. The return type is annotated, or omitted for `Unit`.
* A function body is a block whose last expression is its value.
* A function may declare type parameters ([NEP-0010](../neps/0010-declaration-site-generics.md)).
* A function that performs an effect declares it
  ([`effects.md`](effects.md)).
* Functions are values: `add` has type `Fn(Int, Int) -> Int`.
* **`return` is not in the grammar yet.** This chapter has always described it as
  existing for early exit; the frozen EBNF has no `return-statement`, so
  `examples/04-results` is rejected for that reason as well as for its generics.
  Adding it is a language change and needs a NEP.

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
        web::search, web::open

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
const LIMIT: Int = 1_000;     // compile-time constant
```

**Bindings are immutable, and that is a decision, not a postpone
([NEP-0009](../neps/0009-mutability.md)):** there is no `mut`, no assignment
operator, and no ownership or borrow checker. Shadowing in a nested scope is
allowed; a duplicate name in the same scope is `NDO2xxx`. State is expressed by
structure — a new value computed from an old one — or by a store the runtime owns
(M7).

## Visibility and modules

Items are visible inside their module. `pub` and explicit re-exports are
provisional and described in [`modules.md`](modules.md).

What is implemented today is narrower and decided
([NEP-0009](../neps/0009-mutability.md) and the M3.1 resolver):

* **Two namespaces.** A *type* (a struct, an enum, a built-in type, a type
  parameter) and a *value* (a function, a binding, a constant, a parameter) may
  share a name, and using one where the other belongs is `NDO2003` rather than
  "not found".
* **A duplicate name in one scope is `NDO2002`; shadowing in a nested scope is
  allowed.** The scope is what makes the difference.
* **An unresolved name is `NDO2001`**, reported at the name.
* **A path with `::` reaches into a module, which is planned (M10).** Until then
  a multi-segment path is reported as unresolved, with a note saying why.

## What a declaration must never do

* Annotate a type the specification does not define.
* Grant a capability implicitly — there is no "inherits its caller's
  capabilities".
* Hide an effect. If calling something can reach the network, the declaration
  says so.
