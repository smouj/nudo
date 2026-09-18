# NEP-0009: Mutability and bindings

| Field | Value |
| ----- | ----- |
| Status | Accepted |
| Created | 2026-09-18 |
| Accepted | 2026-09-18, as gate 4 of M3.0 |
| Supersedes | — |
| Superseded by | — |
| Specification chapters | [`spec/declarations.md`](../spec/declarations.md), [`spec/expressions.md`](../spec/expressions.md) |
| Related NEPs | NEP-0005 |

## Summary

**Bindings are immutable. There is no `mut`, no assignment, and no ownership or
borrow checker.** Mutation is not "not yet decided": it is deliberately absent
from this edition, and adding it later needs a NEP that says what a mutable value
means for a task, an agent and a trace.

## Motivation

The grammar has no assignment level, and `spec/expressions.md` says why: writing
the level with nothing to assign to would be inventing a feature to fill a gap in
a diagram. That was the right call while the expression grammar was being frozen.
It has to become a *decision* before the type checker, because "is this binding
reassignable" is the first thing a checker needs to know about a `let`.

## Reference-level explanation

* **`let name = value;` introduces an immutable binding.** Rebinding a name is a
  new declaration; assigning to one is not in the language.
* **`const NAME: Type = value;` is a compile-time constant.** The difference from
  `let` is *when* it is evaluated, not whether it can be assigned.
* **There is no `mut`, no `let mut`, and no assignment operator.**
* **Shadowing** is allowed inside a nested scope: a new `let` may reuse a name,
  and it is a new binding rather than a mutation. Shadowing in the same scope is
  a compile error (`NDO2xxx`) rather than a silent replacement, because in a
  language read by agents a duplicate name is almost always a mistake.
* **No ownership or borrow checker is introduced, by accident or otherwise.**
  Values are immutable, so aliasing them is safe, and the memory model does not
  need to be part of the type system yet. A future NEP that wants mutation must
  also say what it costs, and whether it needs ownership to be sound.
* **State is expressed by structure**: a new value computed from an old one, a
  task's input and output, or a store the runtime owns (M7). Loops, when they
  arrive, are a separate decision.

## What this makes impossible

* Reassignment, and with it the class of bug where a value read earlier in a
  function is not the value it was.
* Any need for a borrow checker in this edition: nothing aliases a value that can
  change, because nothing changes.
* An `assignment` level in the precedence chain — there is nothing for it to
  assign to.

## Alternatives

| Alternative | Why it lost |
| ----------- | ----------- |
| `let mut` now | It brings mutation into the language before anything has needed it, and it would be chosen before the effect system says what a mutation costs |
| Mutable by default | Contradicts the language's premise, and makes every function's behaviour depend on what it was handed |
| `var` | A second binding keyword for a feature nothing uses yet |
| Decide later, at 1.0 | The checker must be written against an answer, so "later" means "written twice" |

## Unresolved questions

* Whether mutation arrives at all, and if it does, whether it is restricted to a
  task's own working set — a mutable value shared between agents is a
  concurrency question as much as a typing one.
* Whether shadowing in the same scope should stay an error or become a warning.
* Whether loops need mutation, or can be expressed with recursion and sequence
  operations.

## Implementation status

**Not implemented, and nothing to implement.** `nudo-parser` already parses `let`
and `const` as the grammar specifies, with no assignment level. The decision
constrains M3's checker: a binding is never a place.
