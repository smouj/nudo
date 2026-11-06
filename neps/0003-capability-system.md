# NEP-0003: Capability system

| Field | Value |
| ----- | ----- |
| Status | Draft |
| Created | 2026-09-17 |
| Supersedes | — |
| Superseded by | — |
| Specification chapters | [`spec/trust/capabilities.md`](../spec/trust/capabilities.md), [`spec/agents/agent.md`](../spec/agents/agent.md) |
| Related NEPs | NEP-0001, NEP-0004, NEP-0008 |

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

For the M4 slice, the proposed `print(Int) -> Unit with Console` writes the
integer's ASCII decimal representation followed by one LF byte (`\n`) to
standard output. The host grants that capability explicitly:

```text
nudo run --allow Console main.nudo
```

A program has `Network` or `Console` only when an authority boundary grants it.
If a call chain does not declare the required effect, the compiler reports an
error and the path that introduced it. At run time, the effect boundary still
checks the grant before performing the external action.

## Reference-level explanation

* This proposal's **closed set** is `Network`, `Filesystem`, `Shell`, `Git`,
  `Secrets`, `Model`, `Human`, and `Console`. `Console` grants writing to
  standard output only. Growing the accepted set is a NEP.
* The default is deny. There is no ambient authority anywhere, including in
  `std`.
* A capability grant comes from an authority boundary, such as an agent's
  `allow` declaration or a host grant for a root program. A function or tool
  effect clause declares what the code may attempt; it does not create or widen
  a grant ([NEP-0004](0004-effects.md)).
* Code may receive a grant, pass a **subset** onward, and drop it. It may not
  create one.
* Delegation narrows; never widens
  ([`spec/agents/delegation.md`](../spec/agents/delegation.md)).
* The compiler checks the declared effect path. At runtime, the effect boundary
  checks the grant in force before an external action. These checks answer
  different questions; neither replaces the other.
* A root program receives only grants named by its host invocation. In M4,
  `nudo run --allow Console` grants `Console`; the function's `with Console`
  clause declares the effect but does not grant it.
* Denials are diagnosed before the effect, with the capability and the call site
  named.
* If an effect is declared but the host did not grant its capability, execution
  fails before the effect with `NDO5001 CAPABILITY_NOT_GRANTED`.
* `print` checks the grant before writing any bytes. If the host then fails to
  write stdout, the call traps with the proposed `NDO6003 CONSOLE_WRITE_FAILED`
  at the call site. Any bytes already accepted by the host remain written; the
  runtime cannot roll them back. The diagnostic does not expose host-specific
  error text. This narrow boundary failure is not a `Result`: M4 has one fixed
  stdout sink, and the language has no retry or alternate-sink handler. A later
  API for choosing or retrying a writer must expose that failure as a `Result`.
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
| Return `Result` from the fixed M4 `print` sink | M4 has no retry or alternate sink; this would add result plumbing without a recovery action. A future configurable writer API must return `Result`. |

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
* How grants are represented in the language. NEP-0004 proposes that callable
  signatures declare effects while grants come from a program or agent boundary;
  whether a grant is represented as a type or value remains open.
* How hosts other than the M4 CLI supply a root program's runtime grant. This
  proposal defines the M4 CLI boundary only; other hosts need a later decision.
* Revocation during a run, and what a task does when a grant disappears.
* How a capability is presented to a person before approval, briefly enough that
  they read it.
* Whether unenforceable-but-declared capabilities should fail the build instead
  of being forbidden by the specification.
