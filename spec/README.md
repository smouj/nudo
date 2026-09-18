# The NUDO specification

This directory defines the NUDO language. It is not documentation *about* the
language; it **is** the language, in the same sense that a standards document is
the standard.

**Status: pre-alpha.** Chapters say what is decided, what is provisional, and
what is still open. A chapter that claims more certainty than the project has is
a bug in the chapter.

## Authority

The order of authority, highest first:

1. **This specification** — what NUDO means.
2. **Accepted NEPs** ([`../neps`](../neps/README.md)) — the decisions that
   produced it, and the reasoning.
3. **Conformance tests** ([`../tests/conformance`](../tests/conformance/README.md))
   — the executable form of the parts that are decided.
4. **The compiler** ([`../compiler`](../compiler)) — one implementation.
5. **Documentation** ([`../docs`](../docs)) — explanations for people.
6. **Examples** ([`../examples`](../examples)) — illustrations, not contracts.

If the compiler and this specification disagree, **the compiler contains the
bug**. That is not a slogan; it is the rule that keeps a language from being
defined by whatever its first implementation happened to do.

Changing this specification is a [NEP](../neps/README.md). Editing it to make a
test pass is forbidden — see [`../AGENTS.md`](../AGENTS.md).

## Chapters

| Chapter | Contents | State |
| ------- | -------- | ----- |
| [`lexical-structure.md`](lexical-structure.md) | Encoding, whitespace, comments, tokens, literals | **Implemented** (M1) |
| [`grammar.md`](grammar.md) | Where the grammar lives and how to read it | **Implemented** (M2) |
| [`declarations.md`](declarations.md) | Items: functions, structs, enums, agents, tasks, tools | Syntax implemented (M2); meaning proposed |
| [`expressions.md`](expressions.md) | Expressions, statements, control flow | Syntax implemented (M2); agentic forms provisional |
| [`types.md`](types.md) | The type system, including trust types | Proposed |
| [`generics.md`](generics.md) | Generic parameters, bounds and inference | Syntax implemented (M2); the rest open |
| [`modules.md`](modules.md) | Modules, packages, manifests, visibility | Proposed |
| [`errors.md`](errors.md) | Diagnostic model and the stable `NDO` code registry | **Registry in use** (M1–M2) |
| [`effects.md`](effects.md) | Effects and how they are checked | Proposed |
| [`concurrency.md`](concurrency.md) | Tasks, structured concurrency, cancellation | Proposed |

### Agents

| Chapter | Contents | State |
| ------- | -------- | ----- |
| [`agents/agent.md`](agents/agent.md) | Agents: role, tools, grants, budgets | Proposed |
| [`agents/task.md`](agents/task.md) | Tasks, acceptance criteria, budgets | Proposed |
| [`agents/tool.md`](agents/tool.md) | Tool declarations and invocation | Proposed |
| [`agents/model.md`](agents/model.md) | Provider-agnostic model interface | Proposed |
| [`agents/memory.md`](agents/memory.md) | Memory: scope, lifetime, eviction | Proposed |
| [`agents/budget.md`](agents/budget.md) | Budgets and exhaustion | Proposed |
| [`agents/delegation.md`](agents/delegation.md) | Delegating work, and what may not be delegated | Proposed |

### Trust

| Chapter | Contents | State |
| ------- | -------- | ----- |
| [`trust/generated.md`](trust/generated.md) | `Generated<T>`: what a model produced | Proposed |
| [`trust/verified.md`](trust/verified.md) | `Verified<T>`: what passed verification | Proposed |
| [`trust/provenance.md`](trust/provenance.md) | Where a value came from | Proposed |
| [`trust/capabilities.md`](trust/capabilities.md) | Capabilities; deny by default | Proposed |
| [`trust/policies.md`](trust/policies.md) | Policies and how they are evaluated | Proposed |
| [`trust/approvals.md`](trust/approvals.md) | Human approvals | Proposed |

### Interoperability

| Chapter | Contents | State |
| ------- | -------- | ----- |
| [`interoperability/mcp.md`](interoperability/mcp.md) | Model Context Protocol | Proposed |
| [`interoperability/a2a.md`](interoperability/a2a.md) | Agent-to-agent delegation | Proposed |
| [`interoperability/wasm.md`](interoperability/wasm.md) | WebAssembly/WASI | Proposed |

## Conventions

* **MUST**, **MUST NOT**, **SHOULD** and **MAY** carry their usual meanings.
* `text` in a code span is literal source.
* A chapter marked *Proposed* has no implementation. It describes an intention
  that reviewers should attack. A chapter whose syntax is implemented but whose
  meaning is not says so in its own state line.
* A chapter marked *Implemented* has cases in
  [`../tests/conformance`](../tests/conformance/README.md). If the behaviour and
  the chapter disagree, that is a bug in the implementation.
