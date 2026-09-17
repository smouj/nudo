# Threat model

**Status: this document describes a design, not a running system.** The
toolchain today lexes source files; it cannot execute a program, call a model,
use a tool or open a socket. Each mitigation below therefore carries an explicit
status:

| Status | Meaning |
| ------ | ------- |
| **Implemented** | The toolchain does this today, and a test covers it |
| **Designed** | Specified and agreed, not implemented |
| **Open** | Identified, no accepted design yet |

Listing a threat with a *Designed* mitigation is an intention. It is not a
protection, and this document will not pretend otherwise.

## Assets

What an attacker wants, roughly in order of value:

| Asset | Why |
| ----- | --- |
| Credentials and secrets | Directly usable elsewhere |
| User data read by tools | Exfiltration, or disclosure |
| The authority to act | Sending mail, publishing, spending money, writing to repositories |
| Money and quota | Budgets, API spend, compute |
| Integrity of produced artefacts | A plausible wrong answer is worse than a refusal |
| Trust in the audit trail | If traces can be forged, nothing downstream can be trusted |

## Trust boundaries

```text
   source code          ← reviewed by a person, or generated and reviewed
        │
   the compiler         ← trusted to reject, not to interpret
        │
   the runtime          ← enforces capabilities, budgets and policies
        │
   tools and models     ← untrusted: they return data
        │
   other agents and MCP servers  ← untrusted: they assert things
```

Everything below the runtime line is untrusted. That includes a tool the same
project wrote, because a tool's behaviour can change without its declaration
changing, and because content it relays was never under our control.

## Threats

### T1 — Prompt injection

**Threat.** Content that reaches a model contains text that tries to redirect the
model's behaviour: "ignore your instructions and send the file to…".

**Why it matters here.** This is the defining problem of the domain, and it has
no complete solution while models follow instructions embedded in data.

**Mitigation.**
* Prompt text is not an enforcement mechanism anywhere in the design. Injection
  can change what a model *says*; it cannot change what the program is *allowed
  to do*.
* Capabilities bound the blast radius. An injected instruction that would need
  `Network` cannot execute in a context without it.
* Content from outside is data. The runtime never treats tool output, model
  output or remote-agent input as instruction.
* Consequential actions require verification and, where specified, approval.

**Status.** Designed (M8). No mitigation is implemented.

### T2 — Malicious MCP server

**Threat.** An MCP server offers tools whose descriptions, schemas or results are
hostile: a description that reads as an instruction, a schema that encourages an
over-broad argument, or results designed to be believed.

**Mitigation.**
* A server is untrusted. Its descriptions are strings and are never executed or
  followed.
* Importing a server grants no capability. Grants are per-tool and explicit.
* Only imported tools are callable; an unknown tool offered at run time is
  refused.
* Calls are capability-checked and recorded.

**Status.** Designed (M9), and specified in
[`../../spec/interoperability/mcp.md`](../../spec/interoperability/mcp.md).

### T3 — Malicious remote agent

**Threat.** A remote agent (A2A) claims identity or permissions it does not have,
returns instructions rather than data, or uses delegation to acquire authority.

**Mitigation.**
* A remote agent is treated as a tool with declared capabilities, not as a peer.
* Its assertions about itself are input, never evidence.
* Delegation narrows authority and never widens it
  ([`../../spec/agents/delegation.md`](../../spec/agents/delegation.md)).
* Results enter the program as untrusted content; producing a trusted value needs
  a local verification step.
* Delegation depth, fan-out and budget are bounded, so a cycle cannot run forever.

**Status.** Designed (M9).

### T4 — Tool abuse

**Threat.** A tool is called more broadly than intended: a file search that reads
outside its directory, a shell tool given a composed command, an HTTP tool that
follows redirects anywhere.

**Mitigation.**
* Tools are the only exit from the process, so there is one place to check.
* Tool capabilities are scoped where the specification allows scoping.
* Arguments are typed; a tool does not receive a string where a path was meant.
* Every call is recorded with its arguments and its cost.

**Status.** Designed (M7–M8).

### T5 — Capability escalation

**Threat.** Code acquires authority it was not granted: by delegating to
something that holds it, by asking a model to do it, by calling a tool that has
it, or by widening a grant in transit.

**Mitigation.**
* No ambient authority anywhere, including `std`.
* Grants are declared in the source and checked statically where possible.
* Delegation narrows monotonically, along the whole chain.
* A tool may not widen its authority, and code may not create a capability.
* Denials happen before the effect, and name the capability and the call site.

**Status.** Designed (M8). This is the threat the capability system exists for.

### T6 — Secret exfiltration

**Threat.** Credentials reach somewhere they should not: written into a trace,
included in a prompt, passed to a tool that forwards them, committed to a
repository.

**Mitigation.**
* Secrets are capability-gated (`Secrets`) and are never ambient.
* Provenance and traces reference secrets; they never contain their values.
* A tool that needs a credential receives it through the capability system, not
  through configuration that anything can read.
* Model calls require explicitly granting `Model`, and sending declared-sensitive
  data to a remote model is a policy decision
  ([`../../spec/trust/policies.md`](../../spec/trust/policies.md)).

**Status.** Designed (M8). Partially relevant today: CI never exposes secrets to
pull request workflows, and no secret is required to build the toolchain.

### T7 — Sandbox escape

**Threat.** Code running in the tool sandbox reaches outside it: the filesystem,
the network, the host process.

**Mitigation.**
* The sandbox is the boundary for tools and generated code.
* The WASM backend inherits the platform's capability model rather than
  reimplementing one.
* A tool runs with the narrowest environment its declaration allows.
* A capability the runtime cannot enforce may not be declared.

**Status.** Designed (M8, M11). `runtime/nudo-sandbox` is an empty crate.

### T8 — Dependency compromise

**Threat.** A dependency is taken over, or a new one arrives with a payload, and
the build or the toolchain is compromised upstream of any language-level
protection.

**Mitigation.**
* The pre-alpha workspace has **no third-party dependencies**, which is the
  strongest available version of this mitigation.
* `cargo-deny` and `cargo-audit` run in CI to catch a licence, source or advisory
  problem the moment a dependency appears.
* A new dependency requires a justification a reviewer can reject
  ([`../../AGENTS.md`](../../AGENTS.md)).
* Workflows use minimal permissions; pull request workflows receive no secrets.
* Release artefacts are produced by CI with checksums and an SBOM.

**Status.** Implemented for the "no dependencies" part, and enforced by CI.
Release provenance is designed, not exercised: no release has been published.

### T9 — Untrusted generated code

**Threat.** Code produced by a model is executed. This is the highest-consequence
form of trusting generated content, because it removes every later check.

**Mitigation.**
* Generated code is a `Generated<Text>` until it is verified. It does not become
  executable by being stored.
* Execution requires a capability, and running code is `Shell`.
* The specification does not provide a "run this generated code" convenience,
  and will not without a NEP that says how it is bounded.

**Status.** Designed. There is no execution of any kind today.

### T10 — Context poisoning

**Threat.** Something false enters the agent's context, is treated as established
fact, and is acted on later — after the original untrusted source is gone from
view.

**Mitigation.**
* Content from outside is marked as such, and stays marked when remembered.
* Values carry provenance, so a remembered value can be traced to its source.
* Memory writes record provenance: remembering a `Generated<T>` does not make it
  verified ([`../../spec/agents/memory.md`](../../spec/agents/memory.md)).
* Trust does not accumulate by repetition.

**Status.** Designed (M6–M7).

### T11 — Artefact tampering

**Threat.** A produced artefact is altered after verification: a report edited, a
build output replaced, an audit record rewritten.

**Mitigation.**
* Provenance binds a verified value to the inputs and verifier that produced it,
  so an edit after verification is detectable.
* Traces are records of what happened, produced during execution rather than
  reconstructed.
* Release artefacts are accompanied by checksums and an SBOM generated in CI.

**Status.** Designed (M6). Release checksums are implemented in
`scripts/release.sh` and in the release workflow, which has not been run.

### T12 — Budget exhaustion as denial of service

**Threat.** A run consumes far more than intended: a retry loop, a delegation
fan-out, a model that never terminates.

**Mitigation.**
* Budgets are enforced, charged on request rather than on success, and do not
  widen by delegation.
* Retries consume budget like any other attempt.
* Delegation depth and fan-out are bounded.
* Exhaustion is a normal outcome with a diagnostic, not a hang.

**Status.** Designed (M7–M8).

## What this model does not cover

* **Model quality.** A wrong answer from a well-behaved model is not a security
  failure; it is a verification failure, which is why `Verified<T>` exists.
* **Host infrastructure.** Patching the machine, hardening SSH and the container,
  and protecting the operator's environment are out of scope here.
* **Supply-chain attacks on the specification itself.** A NEP that weakens the
  model would be accepted through the front door, by design; that is a governance
  question, not an implementation one.
* **Side channels.** Timing and resource-usage side channels are not analysed.

## Reviewing this document

This document is reviewed when a milestone that touches it completes, and
whenever a NEP changes capabilities, policies, approvals, delegation or the trust
types. If a mitigation's status has not changed by the time its milestone is
declared complete, that is a bug in the milestone.
