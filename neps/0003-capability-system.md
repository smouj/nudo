# NEP-0003: Capability system

| Field | Value |
| ----- | ----- |
| Status | Draft |
| Created | 2026-09-17 |
| Supersedes | — |
| Superseded by | — |
| Specification chapters | [`spec/trust/capabilities.md`](../spec/trust/capabilities.md), [`spec/agents/agent.md`](../spec/agents/agent.md) |
| Related NEPs | NEP-0001, NEP-0004 |

## Summary

Give a NUDO program no ambient authority. A capability exists only if it was
granted, and a grant is visible in the source.

## Motivation

An agent that can reach the network, the filesystem and the shell by default is
a program whose blast radius is decided by its worst prompt, not by its author.
The failure is not that permissions exist and are wrong; it is that they are
implicit, and therefore invisible in the code a reviewer reads.

```text
# Which of these can exfiltrate your files?
tools = [search, read_file, send_webhook]
```

The list cannot answer it. The tools' implementations can, and nobody reads the
implementations of a dependency.

## Guide-level explanation

```nudo
tool web.search(query: Text) -> [Result] with Network {
    // may reach the network, and nothing else
}

agent Researcher {
    role: "Research reliable information."
    tools: web.search
    allow: Network
}
```

A program holds `Network` because something said so. Every attempt to reach the
network from code that does not hold it is a compile error, and the error names
the capability and the call site.

## Reference-level explanation

* Capabilities are a **closed set**, named in the specification: `Network`,
  `Filesystem`, `Shell`, `Git`, `Secrets`, `Model`, `Human`. Growing the set is a
  NEP.
* The default is deny. There is no ambient authority anywhere, including in
  `std`.
* A capability is granted by a declaration: an agent's `allow`, a tool's effect
  clause, or a function's effect clause ([NEP-0004](0004-effects.md)).
* Code may receive a grant, pass a **subset** onward, and drop it. It may not
  create one.
* Delegation narrows; never widens
  ([`spec/agents/delegation.md`](../spec/agents/delegation.md)).
* Denials are diagnosed before the effect, with the capability and the call site
  named.
* A capability the runtime cannot enforce may not be declared. A promise the
  implementation cannot keep is worse than a missing feature.

## What this makes impossible

* One-line prototype scripts that read a file or call an API. Writing a
  capability declaration becomes mandatory for the simplest useful program.
* Portable code that "just works" on any host, regardless of what the host
  allows. That is intended: the alternative is code whose behaviour depends on
  ambient configuration.
* Denying by convention, such as a lint that advises adding permissions.
  Conventions do not survive contact with a deadline.

## Alternatives

| Alternative | Why it lost |
| ----------- | ----------- |
| Deny by default at runtime only, with a configuration allow-list | Invisible in the source; a reviewer must diff configuration to know what a program can do |
| Ambient authority with optional audit logging | Audit after the fact does not prevent exfiltration; it documents it |
| Per-tool permissions without a named capability vocabulary | Tool-specific permission names do not compose, and every new tool invents its own |
| OS-level sandboxing only | Necessary and insufficient: it cannot express "this function, at this call site" |

## Impact

| Area | Effect |
| ---- | ------ |
| Specification | `spec/trust/capabilities.md`, `spec/trust/policies.md`, `spec/agents/*` |
| Grammar | `effect-clause`, `allow-clause`, capability sets |
| Compiler | `nudo-effects`, `nudo-typeck`, `nudo-capability` |
| Runtime | `nudo-sandbox`, `nudo-capability`; every effect goes through one door |
| Conformance | Ungranted-effect cases; delegation-narrowing cases |
| Security | Removes ambient authority, the largest class of risk in the design |
| Compatibility | Breaking for every program written before it; reserves several words |
| Performance | Checking is static where possible; runtime checks where not |

## Open questions

* Scoped capabilities: how `Filesystem` expresses "this directory" when the
  directory is only known at run time.
* Whether capabilities are types, effects, or both, given NEP-0004 tracks effects
  separately. Two mechanisms for one question is the risk.
* Revocation during a run, and what a task does when a grant disappears.
* How a capability is presented to a person before approval, briefly enough that
  they read it.
* Whether unenforceable-but-declared capabilities should fail the build instead
  of being forbidden by the specification.
