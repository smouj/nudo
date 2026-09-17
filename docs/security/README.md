# Security

NUDO is a language for programs that call models, use tools, and act with
partial autonomy. That makes security a design constraint rather than a feature,
and it makes a written threat model part of the specification work rather than
an afterthought.

## Start here

* [`threat-model.md`](threat-model.md) — the enumerated threats, their
  mitigations, and **which ones are implemented today** (almost none, and the
  document says so per row).
* [`../../SECURITY.md`](../../SECURITY.md) — how to report a vulnerability.
* [`../../spec/trust/capabilities.md`](../../spec/trust/capabilities.md) — the
  deny-by-default model.
* [`../../spec/trust/policies.md`](../../spec/trust/policies.md) and
  [`../../spec/trust/approvals.md`](../../spec/trust/approvals.md) — restricting
  what may happen automatically.

## The posture, in four sentences

1. The default is deny: a program holds no capability it was not granted, and
   nothing is granted implicitly.
2. Content from outside the program — a model, a tool, a web page, a remote
   agent — is **data**, never instruction.
3. A value a model produced is `Generated<T>`, not `Verified<T>`, and the
   conversion is explicit, fallible and recorded.
4. Anything that can affect the world goes through one door, so the door can be
   locked and what passes through it can be recorded.

## Honesty about the current state

The toolchain today lexes source files. It cannot execute a program, call a
model, use a tool or reach the network, so it has almost no attack surface and
almost none of the mitigations described in the threat model.

That is why every mitigation in
[`threat-model.md`](threat-model.md) carries an explicit status. A threat model
that claims protections an implementation does not have is worse than no threat
model, because people rely on it.
