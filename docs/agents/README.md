# Agents, tasks and trust

A reader's guide to the part of NUDO that is not a general-purpose programming
language. The normative text is in
[`../../spec/agents`](../../spec/agents/agent.md) and
[`../../spec/trust`](../../spec/trust/generated.md); this page is the
orientation.

**None of it is implemented.** Agents, tasks, tools, models, budgets, policies
and approvals are specified and are milestones M6–M8 of
[`../../ROADMAP.md`](../../ROADMAP.md).

## The four ideas

### 1. A model's output is not a fact

```nudo
let draft: Generated<Article> = ask Writer { "Create an article." }
let article: Verified<Article> = verify draft with ArticleVerifier
```

`Generated<T>` and `Verified<T>` are different types, and nothing converts
between them implicitly. Verification is an operation that can fail and that
leaves a record, not a cast.

The honest limitation: a `Verified<T>` is exactly as trustworthy as its verifier.
The type system cannot make a bad verifier good. It can make the choice of
verifier visible, reviewable and recorded — and that is the difference that
matters when something goes wrong at 3 a.m.

### 2. Nothing is granted implicitly

```nudo
fn fetch(url: Text) -> Text with Network { … }
```

A program holds the capabilities it was granted and no others. Not from the
runtime, not from a tool it calls, not from an agent's prompt.

### 3. An agent is a bounded executor

```nudo
agent Researcher {
    role: "Research reliable information."
    tools: web.search, web.open
    allow: Network
    budget: Budget(tokens: 20_000, tool_calls: 30)
}
```

Read it as bounds, not as a description. The tool list is exhaustive; the
capability list is exhaustive; the budget is enforced. An agent that did not
declare `Network` cannot reach the network even if a tool it holds can.

### 4. Autonomy is a conjunction

An autonomous action is permitted when all of these hold: an objective, the
capabilities, the limits, the budget, the acceptance criteria, the policies and
the approvals. Dropping any one of them does not make the action faster; it makes
it unbounded.

## Why the prompt is not a boundary

Every one of these designs exists because the alternative is text:

| If this is text | Then |
| --------------- | ---- |
| "You may only read the docs directory" | A file path elsewhere is one clever sentence away |
| "Do not send customer data anywhere" | Nothing checks, and nothing records that it checked |
| "Ask before publishing" | The check is a person's memory |
| "You are a careful researcher" | No reviewer can tell what the agent can reach |

Prompt text is a good way to explain intent to a model. It is a bad way to
enforce a rule, because enforcement requires something that can refuse. In NUDO
that something is the compiler and the runtime, and the prompt is left to do the
job it is actually good at.

## Where to read more

* [`../../spec/agents/agent.md`](../../spec/agents/agent.md) — agents
* [`../../spec/agents/task.md`](../../spec/agents/task.md) — tasks and acceptance
* [`../../spec/agents/tool.md`](../../spec/agents/tool.md) — tools and isolation
* [`../../spec/agents/budget.md`](../../spec/agents/budget.md) — budgets
* [`../../spec/agents/delegation.md`](../../spec/agents/delegation.md) — delegation
* [`../../spec/trust/capabilities.md`](../../spec/trust/capabilities.md) — capabilities
* [`../../spec/trust/policies.md`](../../spec/trust/policies.md) — policies
* [`../../spec/trust/approvals.md`](../../spec/trust/approvals.md) — approvals
* [`../security/threat-model.md`](../security/threat-model.md) — what can go wrong
