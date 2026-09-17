# 06.1 — Capabilities, effects and authority

> **Status:** Proposed  
> **Summary:** Effects describe what execution may do; capabilities describe which authority the current context can actually grant.


{{#include ../diagrams/authority.svg}}

*Authority narrows along a delegation chain, and cannot flow back.*

These concepts must be related without being confused.

- **Effect**: a static description that a computation can perform an operation such
  as network access.
- **Capability grant**: authority made available to a context.
- **Policy**: rules restricting when or how that authority may be exercised.

The default is intended to be deny. Code should not acquire network, filesystem,
shell or secret access simply because the host process happens to have it.

A critical invariant is non-amplification: a tool or delegated agent must not be
able to widen its own authority beyond what the caller granted.
