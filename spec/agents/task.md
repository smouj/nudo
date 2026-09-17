# Tasks

**State: proposed.** No runtime exists (milestone M7).

## What a task is

A task is a unit of work with an objective, an executor, acceptance criteria and
a budget. Where a function is expected to return a value, a task is expected to
return **evidence**.

```nudo
task Research(topic: Text) -> Verified<Report> {
    agent:
        Researcher

    verify:
        SourcesRequired

    budget:
        Budget(tokens: 20_000, tool_calls: 30)
}
```

## Why a task is not a function call

A function call is deterministic: same inputs, same output. A task is not. It
involves a model, tools, and possibly more than one attempt. The things a
reviewer needs to know about it are therefore different:

| Question | Function | Task |
| -------- | -------- | ---- |
| What can it do? | Its effects | Its effects, its tools, its agent |
| What does it cost? | Usually nothing | Tokens, tool calls, money, time |
| Can it fail? | By returning an error | Also by exhausting a budget or failing verification |
| What does it produce? | A value | A value **with provenance** |
| Can it be retried? | Usually irrelevant | Yes, and the retry policy matters |

Modelling a task as a function call loses all five answers.

## Rules

* **Return type.** A task returns `Verified<T>`, not `T` and not
  `Generated<T>`. Producing a report nobody checked is the failure mode this
  language is designed against.
* **Acceptance criteria are data.** `verify: SourcesRequired` names a criterion
  the runtime evaluates. Criteria are declared, so a failed criterion is a
  checkable outcome with a diagnostic, not a judgement call.
* **Budgets are enforced, not reported.** Exceeding a budget stops the task. It
  does not warn and continue.
* **Failure is normal.** A task that fails verification, exhausts a budget or is
  cancelled returns a failure that the caller must handle. None of these are
  panics.
* **A task leaves a trace.** What it asked, which tools it called, what it
  spent, and how verification ended.

## Retries

Provisional. What is decided: a retry consumes budget like any other attempt,
and a task's retry policy is declared, not inferred. A silent retry is a silent
cost.

## Open questions

* Whether tasks can be composed (a task calling a task) and how budgets compose
  with them.
* Whether a task can be resumed after a failure, and what "resume" means when
  the executor is a model.
* How acceptance criteria are expressed in general — a closed set of built-in
  verifiers, user-defined verifiers, or both.
* Whether tasks are schedulable across processes, or confined to one runtime.
