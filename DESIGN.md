# NUDO — design

This document explains what NUDO is trying to be, what it refuses to become,
and where each kind of thing belongs. It is the document to read before
proposing a language change; [`spec/`](spec) defines the rules in detail, and
[`ARCHITECTURE.md`](ARCHITECTURE.md) says which crate implements them.

**Status: pre-alpha.** Nothing here is stable. Everything in this document is
subject to change through a [NEP](neps/README.md).

## Vision

A program should be able to say, in its own source, that it is about to ask a
model to do something, that the answer is not yet trustworthy, that a
capability is required to continue, that a budget limits how many attempts are
allowed, and that a human has to approve the result. Those statements should be
checked by a compiler and enforced by a runtime, not written in a comment and
remembered by a person.

NUDO is a general-purpose language that runs deterministic software and treats
agentic concepts as language concepts:

* agents, tools, models, tasks;
* capabilities, policies, budgets, approvals;
* memory, traces, provenance;
* generated results and verified results;
* interoperability with other agents and tools.

The one-sentence version:

> NUDO connects people, code, agents, tools and models under one system of
> types, permissions and verification.

## Goals

1. **Deterministic software first.** Ordinary programs must be the easy case.
   A language where the normal path is awkward has failed before it starts.
2. **Trust that the compiler can see.** `Generated<T>` and `Verified<T>` are
   different types. The boundary between them is explicit, checkable, and
   carries a record of how it was crossed.
3. **Safe by default.** A program has no capabilities it was not granted. No
   ambient network, no ambient filesystem, no ambient shell, no ambient
   credentials — including for agents.
4. **Model-agnostic.** No provider, protocol or vendor is privileged in the
   language.
5. **Local-first.** Local models, local tools, local storage and local
   execution must be first-class, not a degraded mode.
6. **Observable by default.** An autonomous run that matters leaves a trace.
7. **Determinism around probability.** Probabilistic work happens inside
   deterministic, validating code — never the other way around.
8. **Readable by humans, manipulable by agents.** Syntax, AST and diagnostics
   are designed so that a person can read them and a tool can rewrite them
   without guessing.
9. **Specification first.** The specification defines the language; the
   implementation follows it.
10. **Minimal syntax, strong semantics.** New keywords are the expensive way to
    solve a problem.
11. **Self-hosting as a proof, not a prerequisite.** Rust is the bootstrap
    implementation. Once NUDO is expressive and stable enough to implement a
    compiler without special treatment, the canonical compiler and toolchain
    should progressively move to NUDO. Current milestones are not delayed or
    weakened to force that transition early.

## Non-goals

* **Not a wrapper or SDK.** A thin client for one provider's API is a library;
  it does not need a language.
* **Not a prompt framework.** Prompt templating and chain plumbing belong in
  libraries.
* **Not a Python replacement.** NUDO is not competing for Python's role, and
  claims about replacing existing languages are not part of this project.
* **Not a DSL for AI.** A language that can only do AI is not a language.
* **Not a new syntax over an existing runtime.** NUDO has its own compiler and
  its own semantics; it is not a front end that produces another language.
* **Not an AGI project, and not a research lab.** It is a language, a compiler
  and a toolchain.
* **Not production-ready.** Pre-alpha means what it says.

## Philosophy

**Specification first.** The hierarchy of authority is:

1. language specification (`spec/`)
2. accepted NEPs (`neps/`)
3. conformance tests (`tests/conformance/`)
4. compiler implementation (`compiler/`)
5. documentation (`docs/`)
6. examples (`examples/`)

If the compiler and the specification disagree, **the compiler contains the
bug**.

**Small syntax, explicit semantics.** Most of what other systems express as new
keywords should be expressible with existing types, and most of what should be
a compile-time error should not be a runtime convention.

**Nothing implicit about trust or authority.** Any implicit grant of trust or
capability is a bug in the design, not a convenience.

**Boring where it does not matter.** The build system, the package layout and
the CLI are not places for novelty.

## What belongs where

The most common design mistake in a language like this is putting a runtime
concern into the type system, or a library concern into the compiler. The
boundary is:

| Concern | Belongs in | Example |
| ------- | ---------- | ------- |
| Meaning of a value, type, or expression | **Language** (`spec/`, `compiler/`) | `Verified<T>` is not assignable to `Generated<T>` |
| Whether a grant exists | **Language** (capabilities) | A function that needs `Network` cannot be called without it |
| How a specific model is called | **Runtime** (`runtime/nudo-model`) | Provider-specific request shapes |
| How a tool is invoked, and its isolation | **Runtime** (`runtime/nudo-tool`, `runtime/nudo-sandbox`) | Process spawn, HTTP call |
| A convenient helper around the above | **Library** (`std/`) | Retry policy for a flaky tool |
| Which policies a project enables | **Manifest + policy**, not the language | Budget defaults per package |

Rules that follow from the table:

* If two implementations could reasonably disagree about a value's meaning, it
  belongs in the language and must be specified.
* If it can be changed without changing what a program means, it belongs in a
  library.
* If it can be observed by another process, it belongs in the runtime, and it
  is capability-gated.

## Core concepts

### `Generated<T>` and `Verified<T>`

A value produced by a model is `Generated<T>`. A value that has been checked
against explicit criteria is `Verified<T>`. They are different types, and
conversion is one-directional and explicit:

```nudo
let draft: Generated<Article> = ask Writer { "Create an article." }

let article: Verified<Article> = verify draft with ArticleVerifier
```

`verify` is not a cast and not a formality. It runs a verifier, produces a
provenance record, and can fail. There is no implicit conversion from
`Generated<T>` to `T`: the absence of a verification step must be visible in the
source, because it is the single most important thing a reviewer needs to see.

### Capabilities

A capability is a named, grantable right to affect the world outside the
program: `Network`, `Filesystem`, `Shell`, `Git`, `Secrets`, and others defined
by the specification. The rules:

* the default is deny;
* a capability exists for a program only if it was granted;
* a grant is part of a value's type, so it is checked statically;
* capabilities cannot be invented at runtime, and cannot be widened by the code
  that receives them.

### Provenance

Provenance is the record of where a value came from: which model, which
version, which tool, which inputs, which verification step, which approvals.
Provenance is data, attached to values, and available to the program. It is
what makes a trace useful after the fact and what makes an audit possible at
all.

### Agents

An agent is a declared, bounded executor: a role, a set of tools, the
capabilities it is allowed, the budget it may consume, and the criteria its
output must meet. An agent is not an unconstrained process with a system
prompt. Its declaration is what a reviewer reads to know what it can do — and
what the compiler reads to prevent anything else.

### Tasks

A task is a unit of work with an objective, acceptance criteria, a budget, and
a trace. Where a function call is expected to return a value, a task is
expected to return evidence. Tasks are how autonomy stays legible.

## Open questions

Recorded here so that they are answered deliberately rather than by accident.
Each one is expected to become a NEP:

* Unicode identifiers, and whether the language should have them at all.
* The exact concurrency model, and whether it is structured.
* Error handling beyond `Result`: effects, union types, or neither.
* Whether `verify` is a keyword, a function, or a protocol.
* How editions and compatibility work before 1.0.
* How far the capability model can be enforced statically when tools are
  supplied at runtime.
