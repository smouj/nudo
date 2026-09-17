# Agent-to-agent delegation (A2A)

**State: proposed.** No adapter exists (milestone M9). `protocols/` is empty.

## Intent

A NUDO agent should be able to hand work to an agent implemented elsewhere, and
receive work from one, without either side having to trust the other's claims
about itself.

## The posture

A remote agent is treated as a **tool**, not as a peer:

* it is declared, with the capabilities it requires;
* calling it is an effect that is checked and recorded;
* what comes back is untrusted data;
* producing a trusted value from it requires verification
  ([`../trust/verified.md`](../trust/verified.md)).

## The rule that makes this safe

> Delegation may narrow authority. It may never widen it.

Applied to a remote agent, this means:

1. A remote agent receives a subset of the delegator's capabilities — and in
   practice, since the remote side may not share NUDO's model, it receives
   capabilities expressed as credentials scoped to the specific action.
2. A remote agent's assertion about its own identity or permissions is **not
   evidence**. It is input.
3. A result from a remote agent enters the program as untrusted content unless a
   local verification step makes it trusted. Trust does not cross a network
   boundary because the other side said the right words.

## Threats specific to A2A

| Threat | Mitigation |
| ------ | ---------- |
| A remote agent claims authority it does not have | Treat claims as data; grant explicitly |
| A remote agent returns content that tries to act as instructions | Untrusted content is data, never instruction |
| Delegation loops and runaway fan-out | Bounded depth, fan-out and budget |
| Exfiltration through a delegation | Capability scoping; a delegated task sees only what it was granted |
| Impersonation of a known agent | Identity verification is required for high-consequence actions |
| A remote result that is plausible and wrong | Verification, and provenance that names the remote source |

## Open questions

* Identity: what an agent identity is, how it is verified, and by whom. A
  protocol-level answer is not enough; the language has to say what it does with
  one.
* Whether delegation is synchronous or returns a handle.
* How a remote agent's cost is accounted against a local budget.
* Whether NUDO defines its own wire format or adopts an existing one. Defining
  one is a large commitment and needs a NEP with a strong argument.
