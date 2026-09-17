# NEP-0001: Agent types

| Field | Value |
| ----- | ----- |
| Status | Draft |
| Created | 2026-09-17 |
| Supersedes | — |
| Superseded by | — |
| Specification chapters | [`spec/agents/agent.md`](../spec/agents/agent.md), [`spec/declarations.md`](../spec/declarations.md) |
| Related NEPs | NEP-0002, NEP-0003 |

## Summary

Make an agent a declared type whose tools, capabilities, budget and role are
part of the declaration, and make the compiler enforce them.

## Motivation

Today an "agent" is a prompt, a tool list and a permission set held together by
application code. Three things are then unknowable at review time:

* what the agent can actually reach;
* what it is allowed to spend;
* whether its declared purpose has any relationship to what it does.

A reviewer reading the prompt cannot answer any of those questions, because the
answers live in configuration and in the runtime.

The program that hurts today:

```text
agent = Agent(
    prompt="You are a researcher.",
    tools=[search, open_url],
    max_tokens=20000,
)
```

Nothing above is checked. `open_url` may reach the network even if the caller
never intended to grant network access, and `max_tokens` is a number in a
dictionary.

## Guide-level explanation

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

Read it as a statement of bounds: these tools, these capabilities, this much
spend. A call to a tool that is not listed is a compile error. A tool that
requires a capability the agent does not hold is a compile error at the
declaration, not a failure at run time.

## Reference-level explanation

* `agent` declares a named type. Two agents are different types even if their
  declarations are identical, because their bounds are part of their meaning.
* `tools` is exhaustive. A tool is callable by an agent if and only if the agent
  lists it.
* `allow` grants capabilities ([NEP-0003](0003-capability-system.md)). A tool
  list entry whose required capabilities are not held is `NDO4xxx` — a
  declaration-time error.
* `budget` is enforced ([`spec/agents/budget.md`](../spec/agents/budget.md)).
  Exhaustion fails the task; it is not a warning.
* An agent does not inherit capabilities from its caller, its module, or its
  tools.
* `role` is documentation. It is not a security boundary and must never be
  treated as one; a proposal that relies on prompt text for enforcement is
  rejected by this NEP's rules.

## What this makes impossible

* An agent whose tools arrive at runtime, unlisted. This is the point: it is
  also the capability-escalation path this design exists to close.
* Concise dynamic agent construction, where the tool set is assembled from data.
  A program that needs that must declare a bounded set and select within it.
* Prompt-only enforcement of behaviour. If you wanted "the prompt says not to do
  this" to be enough, this design says it is not.

## Alternatives

| Alternative | Why it lost |
| ----------- | ----------- |
| Keep agents as library objects, add a checking pass | The checks would be invisible in the source, and a reviewer would still read configuration to know what an agent may do |
| Make `agent` a struct with conventional field names | Convention is not enforcement: nothing would stop a tool call that skips the convention |
| Encode bounds in the type of each tool rather than the agent | Tool types describe the tool, not the agent's authority; both are needed, and the agent is the unit a person reasons about |

## Impact

| Area | Effect |
| ---- | ------ |
| Specification | `spec/agents/agent.md`, `spec/declarations.md`, `spec/agents/tool.md` |
| Grammar | `agent-decl` becomes non-provisional; `agent-clause` fixed |
| Compiler | `nudo-parser`, `nudo-hir`, `nudo-typeck`, `nudo-agent` |
| Conformance | Agent declaration cases: valid, missing capability, unlisted tool |
| Security | Removes an implicit-capability path; adds declaration-time checks |
| Compatibility | New syntax; reserves the word `agent` |

## Open questions

* Whether two identical agent declarations are the same type.
* Whether an agent may be parameterised (a template with different tools per
  instantiation).
* How an agent's model binding is declared, and whether it belongs in NEP-0001.
* Whether agents may hold state between tasks, and where that state lives.
