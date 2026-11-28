# Examples

Ten examples, in increasing order of ambition. Each states whether the current
toolchain checks it cleanly or leaves it as a preview. A syntax preview may be
rejected by the parser; a semantic preview may parse but still be rejected by
`nudo check` while its rules remain unaccepted.

## What the toolchain reads today

| Example | Contents | `nudo check` |
| ------- | -------- | ------------ |
| [`00-hello-world/`](00-hello-world/main.nudo) | Bindings, functions, a text literal | **Clean** |
| [`01-variables/`](01-variables/main.nudo) | Literals, `_` separators, nesting, escapes | **Clean** |
| [`02-functions/`](02-functions/main.nudo) | Declarations, parameters, return types | **Clean** |
| [`03-types/`](03-types/main.nudo) | Structs, enums, variants with fields, `match` | **Clean** |
| [`04-results/`](04-results/main.nudo) | Errors as values: an `Outcome` and exhaustive `match` | **Clean** |

```console
$ nudo check examples/00-hello-world/main.nudo
checked 1 file: 0 errors and 0 warnings
```

"Clean" means no diagnostics from implemented stages: lexical and syntax
analysis, name resolution and the accepted type rules implemented so far. It
does not mean every planned language rule is checked, or that the file runs;
there is no interpreter yet (M4).

## Design previews

The rest illustrate proposals and are **not** accepted by `nudo check`. Each
header names the syntax or semantic gate that remains open. The parser rejects
syntax previews; semantic previews can parse but must still fail a later check.

| Example | Illustrates | Why it is not accepted yet |
| ------- | ----------- | -------------------------- |
| [`05-agent/`](05-agent/main.nudo) | An agent as a bounded executor | Agent, task and capability semantics remain gated by NEP-0001/0003; the sample keeps `Report` unresolved |
| [`06-tools/`](06-tools/main.nudo) | Tools, scoped capabilities, effects | Dotted tool names; a capability with an argument |
| [`07-generated-verified/`](07-generated-verified/main.nudo) | `Generated<T>` vs `Verified<T>` | Two `let`s with no `;` |
| [`08-policy/`](08-policy/main.nudo) | Policies, approvals, budgets | `approve` is not in the grammar |
| [`09-multi-agent/`](09-multi-agent/main.nudo) | Delegation that narrows authority | A named-field request block is outside the accepted grammar; delegation semantics remain proposed |

Specification chapters: [`spec/types.md`](../spec/types.md),
[`spec/expressions.md`](../spec/expressions.md),
[`spec/agents/agent.md`](../spec/agents/agent.md),
[`spec/agents/tool.md`](../spec/agents/tool.md),
[`spec/trust/verified.md`](../spec/trust/verified.md),
[`spec/trust/policies.md`](../spec/trust/policies.md),
[`spec/agents/delegation.md`](../spec/agents/delegation.md).

Every preview carries a `PREVIEW` marker and a `Why not:` line in its header.
Both are required: the first stops a reader from mistaking an intention for a
feature, and the second says exactly which part of the grammar is missing.

## Conventions

* One directory per example, one `main.nudo` per directory, named after the
  example rather than its number.
* Preview examples do not call standard library functions that do not exist.
  Where an example needs behaviour the language cannot express yet, it says so in
  a comment instead of inventing an API.
* Examples are documentation, not tests. The corpus checks that readable
  examples pass `nudo check`, syntax previews fail parsing, and each preview
  documents the gate that keeps it from acceptance.
