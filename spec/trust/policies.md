# Policies

**State: proposed.** No policy engine exists (milestone M8).

## What a policy is

A policy is a rule about what may happen automatically. It is the answer to
"under what conditions can this run without a person deciding?".

Policies are needed because the interesting question about an autonomous system
is not whether it can do something, but **when it must stop and ask**.

## What a policy can decide

Provisional, and deliberately a small set:

| Kind | Example |
| ---- | ------- |
| Budget limits | Refuse work that exceeds a declared budget |
| Capability conditions | Allow `Network` only for an allow-listed destination |
| Approval requirements | Require a person before an irreversible action |
| Data handling | Forbid sending declared-sensitive values to a remote model |
| Model conditions | Allow only a local model for a given class of input |
| Rate and volume | Limit how much work may happen in a window |

## Rules

1. **Policies are declared, versioned and reviewable.** A policy that lives only
   in a dashboard is not a policy.
2. **A policy can only restrict.** A policy may deny something that was granted;
   it may never grant something that was not. Otherwise every capability
   declaration can be bypassed by configuration.
3. **Evaluation is deterministic and explainable.** For a given state, the
   decision is reproducible, and the result names the policy that decided.
4. **Deny wins.** When two policies disagree, the restrictive one applies.
5. **A policy failure is a refusal, not a crash.** If a policy cannot be
   evaluated, the action does not happen.
6. **The decision is recorded.** Which policy applied, to what, and what the
   outcome was, is part of the trace.

## Policies versus capabilities

A capability says *may*; a policy says *under which conditions*. Both can refuse;
neither can widen the other. In practice: capabilities are what a program
declares it needs, and policies are what an operator allows to actually run.

## Open questions

* The policy language. A general-purpose one is a language inside a language,
  and the cost of that must be justified before it is paid. A closed set of
  declarative rules may be enough, and is much easier to reason about.
* Where policies live: the manifest, a separate file, or supplied by the host at
  run time.
* How policy decisions are surfaced to a person in a way that informs rather
  than trains them to click through warnings.
* Whether policies can be composed from other policies, and how conflicts
  resolve beyond "deny wins".
