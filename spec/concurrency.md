# Concurrency

**State: proposed.** Nothing is implemented. This chapter is deliberately
minimal: concurrency designs that are written before execution exists are
usually replaced by the first real implementation.

## Principles

1. **Structured concurrency.** A concurrent operation is bounded by the scope
   that started it. Nothing outlives its parent silently.
2. **Cooperative cancellation.** Cancelling a task is observable, and every
   long-running operation has a defined response to it.
3. **Bounded everything.** Concurrency, memory, retries and spend have limits
   that are declared, not assumed.
4. **No ambient state.** Concurrent work does not share mutable state implicitly.
   Sharing is explicit, and the mechanism is specified before it is used.

## The unit of concurrency

The primary unit is the **task** ([`agents/task.md`](agents/task.md)), not the
thread. Tasks are how both parallel computation and agent work are expressed,
which keeps one scheduler and one cancellation story for the whole language.

This is a deliberate simplification: a second concurrency model exists only if a
concrete program cannot be expressed with the first, and that argument has not
been made yet.

## Determinism

Where concurrency is used for parallel work, the observable result must not
depend on scheduling order. Where it is used for agent work, order is irrelevant
in a different way: the trace records what actually happened, so a non-
deterministic run is still explainable.

## Budgets and cancellation interact

A cancelled task still reports what it spent. A budget is consumed when work is
requested, not when it succeeds — otherwise cancellation becomes a way to spend
without accounting. See [`agents/budget.md`](agents/budget.md).

## What is undecided

* Whether `async`/`await` exists as syntax, and what a "future" is called.
* Whether the runtime is work-stealing, single-threaded, or both with a
  selectable backend.
* How the interpreter and the WASM backend differ in scheduling, and whether
  that difference is observable to a program. It should not be, and if it is,
  that is a specification bug.
* Whether there is a `spawn` primitive at all, as opposed to expressing
  everything through tasks.

## What will not happen

Concurrency will not be added to the language as an afterthought to make a
benchmark look better. A scheduler that cannot be described cannot be relied on
for the budgets and traces this language promises.
