# Capabilities

**State: proposed.** No capability system exists (milestone M8). This is the
chapter the rest of the security design depends on.

## The rule

> The default is deny. A capability exists for a program only if it was granted.

Everything else is mechanism.

## What a capability is

A named, grantable right to affect the world outside the process:

| Capability | Grants |
| ---------- | ------ |
| `Network` | Making outbound network requests |
| `Filesystem` | Reading and writing files, within a declared scope |
| `Shell` | Running external commands |
| `Git` | Reading and modifying repositories |
| `Secrets` | Reading credential material |
| `Model` | Calling a model |
| `Human` | Asking a person for input or approval |

The set is specified, closed, and grows only through a NEP. A capability named
by a string at runtime is not a capability system.

## Rules

1. **Deny by default.** A program has no capability it was not granted. There is
   no ambient authority, including in the standard library and including for
   agents.
2. **A grant is declared and visible.** It appears in an agent declaration, a
   tool declaration, or a function's effect clause. A reader can find every
   grant in the source.
3. **A grant cannot be created by code.** Code can receive a capability, pass a
   subset onward, and drop it. It cannot invent one.
4. **Delegation narrows.** A grant passed to another agent or task is a subset of
   the holder's ([`../agents/delegation.md`](../agents/delegation.md)).
5. **Scoped, not binary, where possible.** `Filesystem` grants access to a
   declared scope, not to everything. A capability that can only be all-or-
   nothing is a design smell.
6. **Checked before the effect.** A denial happens before the request is made,
   not when the answer arrives.
7. **Denials are diagnosable.** The error names the capability, the call site,
   and the path that would have granted it.
8. **Unenforceable capabilities are not declared.** A capability the runtime
   cannot actually enforce is a false promise, and this specification may not
   make it.

## Capabilities versus trust types

| | Capability | Trust type |
| --- | --- | --- |
| Question | What may this code touch? | What may this value be used for? |
| Enforced | Statically (effects) and at runtime | Statically |
| Direction | Authority | Confidence |

They are independent. A program may hold `Network` and still be unable to pass a
`Generated<T>` where a `Verified<T>` is required — and that is the intended
combination.

## Threat model

The threats this chapter exists to address — capability escalation, tool abuse,
prompt injection that tries to acquire authority, malicious MCP servers,
exfiltration — are enumerated with their mitigations in
[`../../docs/security/threat-model.md`](../../docs/security/threat-model.md).

## Open questions

* Whether capability sets are types, effects, or both, given that
  [`../effects.md`](../effects.md) already tracks them.
* How a capability is represented when its scope is only known at runtime (a
  directory chosen by the user, for example).
* How capabilities are displayed to a person before approval, in a way that is
  actually readable.
* Whether a capability can be revoked mid-run, and what a running task does when
  it is.
