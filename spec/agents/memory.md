# Memory

**State: proposed.** No memory store exists (milestone M7).

## What memory is for

An agent that cannot remember anything cannot do multi-step work. An agent that
remembers everything without limits leaks data, drifts, and eventually spends
its budget re-reading its own history.

So memory is a **declared resource with a scope and a lifetime**, not an
implicit conversation log.

## Scope and lifetime

Provisional shape:

| Scope | Lifetime | Visible to |
| ----- | -------- | ---------- |
| Invocation | One task attempt | The agent performing it |
| Agent | The agent's lifetime | That agent |
| Task | The whole task, across attempts | Every attempt of that task |
| Program | The whole run | Explicitly declared consumers |

There is no global memory. Anything shared between agents is shared because a
declaration says so.

## Rules

1. **Memory is capability-gated.** Reading or writing a persistent store is an
   effect, and an agent without the capability cannot do it.
2. **Memory is not a side channel for capabilities.** An agent cannot store a
   credential, a handle or an authority and retrieve it later to use it. Memory
   holds values, not powers.
3. **Eviction is specified.** A memory with no eviction rule is a memory that
   grows until it fails. Every scope declares what happens when it is full.
4. **Written memory carries provenance.** What is remembered records where it
   came from, so that a remembered `Generated<T>` cannot become a trusted value
   by the passage of time.
5. **Memory is inspectable.** What an agent remembered is part of its trace, and
   a run that cannot be explained from its trace is a bug.
6. **Content from outside is untrusted.** A remembered web page is data, and
   remembering it does not make it instruction
   ([`../../docs/security/threat-model.md`](../../docs/security/threat-model.md),
   context poisoning).

## Open questions

* Whether memory is typed (a store of `T`) or untyped with conversion at the
  boundary.
* Whether the language provides a vector-similarity store or leaves retrieval to
  libraries.
* How ownership of persistent memory works across runs and users.
* Whether memory can be redacted after the fact, and what that does to traces
  that already reference it.
