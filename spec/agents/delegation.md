# Delegation

**State: proposed.** No runtime exists (milestone M7).

## The rule

> Delegation may narrow authority. It may never widen it.

Everything else in this chapter follows from that sentence. An agent that hands
work to another agent, to a task, or to a remote system can grant less than it
holds, never more, and never something it does not hold itself.

## Why this is the rule

Without it, every capability boundary in the language can be walked around by
one line: an agent that cannot reach the network asks an agent that can. The
capability system would then be a description of intent rather than an
enforcement mechanism, which is exactly the situation NUDO exists to fix.

## Mechanics

Provisional shape:

```nudo
let report = delegate Researcher {
    topic: "…"
} with budget: Budget(tokens: 4_000)
```

* the delegated work receives a **subset** of the delegator's capabilities;
* the delegated work receives a **part** of the delegator's remaining budget;
* the delegator remains responsible for the result;
* the delegation appears in the trace with what it was granted.

## Transitivity

Authority narrows monotonically along a delegation chain. A chain of five
delegations can never hold more than the first agent started with, and a
capability that is dropped at any step cannot reappear later.

## Remote delegation

Delegating to another process, machine or implementation
([`../interoperability/a2a.md`](../interoperability/a2a.md)) is where this rule
becomes hardest to keep, because the remote side may not share the model. The
intended posture:

* the remote agent is treated as a **tool** with the capabilities it was
  declared with, not as an equal peer;
* what comes back is untrusted data, and producing a trusted value from it
  requires a verification step
  ([`../trust/verified.md`](../trust/verified.md));
* a remote agent's claims about its own identity or permissions are not
  evidence.

## Cycles and runaway delegation

* Delegation must be bounded: depth, fan-out and budget. A chain that can call
  itself is a chain that can run forever.
* A delegation cycle is a runtime error with a diagnostic, not a hang.

## Open questions

* How a subset of capabilities is expressed in the type system.
* Whether delegation is synchronous (the delegator waits) or returns a handle.
* How identity is represented for remote agents, and what verification of that
  identity would even mean.
* Whether a delegated task may itself delegate, and how depth is declared.
