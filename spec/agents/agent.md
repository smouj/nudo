# Agents

**State: proposed.** No agent runtime exists (milestone M7).

## What an agent is

An agent is a **bounded executor**: a named role, a set of tools it may use, the
capabilities it holds, a budget it may spend, and the criteria its output must
meet. It is a declaration the compiler can read and a reviewer can audit.

An agent is **not** a process with a system prompt. If the only description of
what an agent can do is the text of its prompt, then nothing enforces it, and
this language has no reason to exist.

## Declaration

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
        Budget(tokens: 20_000, tool_calls: 30, spend: Money(2, "EUR"))
}
```

Every clause is a constraint, not a hint:

| Clause | Meaning |
| ------ | ------- |
| `role` | What the agent is for. Documentation for humans and for the agent's prompt |
| `tools` | Exactly which tools it may call. Anything else is a compile error |
| `allow` | Capabilities granted to it ([`../trust/capabilities.md`](../trust/capabilities.md)) |
| `budget` | Limits on what it may spend ([`budget.md`](budget.md)) |

## Rules

1. **Nothing is granted implicitly.** An agent holds only what its `allow` clause
   names. It does not inherit capabilities from its caller, from its tools, or
   from the module it lives in.
2. **A tool call must be permitted twice:** the agent must list the tool, and the
   agent must hold every capability that tool requires. Listing a tool whose
   capabilities it does not have is a compile error, not a runtime surprise.
3. **`role` is not a capability.** Prose in `role` grants nothing and constrains
   nothing. It exists because a person has to understand what the agent is for.
4. **An agent cannot widen its own grants.** Not by calling a tool, not by
   delegating, not by asking a model to.
5. **Every run is traceable.** An agent run produces a trace, including what it
   spent and what it called ([`../trust/provenance.md`](../trust/provenance.md)).

## Lifecycle

Provisional, and the subject of a NEP:

* an agent is instantiated for a task, or for a longer-lived scope;
* instantiation checks the declaration against the capabilities in force;
* work is performed through tasks ([`task.md`](task.md));
* the agent stops when its task completes, its budget is exhausted, it is
  cancelled, or its scope ends — and all of those are normal outcomes, not
  exceptional ones.

## Open questions

* Whether agents may hold state between tasks, and where that state lives.
* Whether an agent declaration can be parameterised (a template agent with
  different tools per instantiation).
* How a model is bound to an agent: at declaration, at instantiation, or at the
  call site.
* Whether multiple agents run concurrently within one task, and how budgets are
  shared between them.
