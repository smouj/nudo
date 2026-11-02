# 06.1 — Capabilities, effects and authority

> **Status:** Proposed / Planned
>
> **Summary:** NUDO proposes static effect tracking and explicit capability authority; neither effect checking nor runtime capability enforcement exists in the current toolchain.

{{#include ../diagrams/authority.svg}}

*This is the proposed authority model, not a security boundary enforced by
today’s runtime.*

## Three related ideas

- **Effect:** a static description of what a computation may do, such as using
  the network.
- **Capability grant:** authority made available to a program or execution
  context.
- **Policy:** rules that restrict when or how authority may be exercised.

The design links the effect required by a function to the authority available
at its call site. It aims for deny-by-default behaviour and prevents a delegated
tool or agent from widening the caller’s authority.

## Current implementation status

M5 is planned. The effect chapter is still proposed and
<code>neps/0004-effects.md</code> is Draft. There is no effect inference,
effect checker, <code>NDO3xxx</code> implementation or runtime capability
enforcement today. The <code>nudo-effects</code> crate is a roadmap boundary,
not a finished subsystem.

The example below uses the Draft proposal’s syntax; it is not accepted syntax
that a user can rely on:

~~~nudo
fn fetch(url: Text) -> Text with Network {
    // Proposed: calling fetch requires Network.
}
~~~

M5’s roadmap exit criterion is an effectful call rejected from an effect-free
function, with a diagnostic that names both the effect and the call path that
introduced it. The design must be accepted before that checker can be
implemented. Runtime policy enforcement is a separate later milestone.

\n