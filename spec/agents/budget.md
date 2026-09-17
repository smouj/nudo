# Budgets

**State: proposed.** No runtime exists (milestone M7).

## Why a budget is a language concept

An autonomous run without a limit is an unbounded liability. If the only thing
that stops it is a person watching a dashboard, then the language has delegated
its most important safety property to an operator's attention span.

A budget is therefore declared, checked and enforced by the runtime.

## Shape

```nudo
budget:
    Budget(tokens: 20_000, tool_calls: 30, spend: Money(2, "EUR"), time: 5m)
```

| Dimension | Measured in |
| --------- | ----------- |
| `tokens` | Model tokens, input and output |
| `tool_calls` | Number of tool invocations |
| `spend` | Money, in a named currency (never implicitly converted) |
| `time` | Wall-clock duration |
| `attempts` | Retries of a task |

A dimension that is not declared is not unbounded by default at the language
level; whether it is unbounded or zero is an **open question**, and the answer
must be the safe one.

## Rules

1. **Charged on request, not on success.** A call that is made consumes budget
   even if it fails or is cancelled. Otherwise failure becomes free and retries
   become unbounded in practice.
2. **Exhaustion is a normal outcome.** It fails the task with a diagnostic that
   names the exhausted dimension and what was spent. It is not a panic.
3. **Budgets do not widen by delegation.** A child task receives a budget that is
   part of the parent's, never an addition to it
   ([`delegation.md`](delegation.md)). Otherwise an agent can multiply its own
   limit by spawning work.
4. **Budgets are visible in the trace.** What was spent, on what, and by whom.
5. **No silent overrides.** A call site cannot raise a budget. Raising it is a
   change to a declaration, which is a change a reviewer sees.

## What a budget is not

* Not a rate limit for a provider account. That is infrastructure.
* Not a cost estimate. It is a hard limit, and the estimate is a separate
  question.
* Not a substitute for capabilities. A large budget does not grant anything.

## Open questions

* Whether budgets are structural types (any subset of dimensions) or a single
  record with defaults.
* How `spend` interacts with provider price changes mid-run.
* Whether the runtime may *reduce* a budget dynamically, and how a running task
  learns about it.
* How time budgets behave under cancellation and suspension.
