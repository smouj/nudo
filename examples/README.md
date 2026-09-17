# Examples

Ten examples, in increasing order of ambition. Each one states at the top of the
file whether the current toolchain can read it, and that statement is true — a
test in `compiler/nudo-lexer/tests/examples.rs` checks it.

## What the toolchain reads today

| Example | Contents | `nudo check` |
| ------- | -------- | ------------ |
| [`00-hello-world/`](00-hello-world/main.nudo) | Bindings, functions, a text literal | **Clean** |
| [`01-variables/`](01-variables/main.nudo) | Literals, `_` separators, nesting, escapes | **Clean** |
| [`02-functions/`](02-functions/main.nudo) | Declarations, parameters, return types | **Clean** |

```console
$ nudo check examples/00-hello-world/main.nudo
checked 1 file: 0 errors, 0 warnings
```

"Clean" means no **lexical** diagnostics. It does not mean the file is correct or
that it runs: there is no parser (M2), no type checker (M3) and no interpreter
(M4).

## Design previews

The rest illustrate proposed syntax and are **not** accepted by the current
toolchain. `nudo check` reports unknown characters for them, which is the honest
result of asking a lexer about a language it does not implement yet.

| Example | Illustrates | Specification |
| ------- | ----------- | ------------- |
| [`03-types/`](03-types/main.nudo) | Structs, enums, exhaustive `match` | [`spec/types.md`](../spec/types.md) |
| [`04-results/`](04-results/main.nudo) | Errors as values, no exceptions | [`spec/expressions.md`](../spec/expressions.md) |
| [`05-agent/`](05-agent/main.nudo) | An agent as a bounded executor | [`spec/agents/agent.md`](../spec/agents/agent.md) |
| [`06-tools/`](06-tools/main.nudo) | Tools, scoped capabilities, effects | [`spec/agents/tool.md`](../spec/agents/tool.md) |
| [`07-generated-verified/`](07-generated-verified/main.nudo) | `Generated<T>` vs `Verified<T>` | [`spec/trust/verified.md`](../spec/trust/verified.md) |
| [`08-policy/`](08-policy/main.nudo) | Policies, approvals, budgets | [`spec/trust/policies.md`](../spec/trust/policies.md) |
| [`09-multi-agent/`](09-multi-agent/main.nudo) | Delegation that narrows authority | [`spec/agents/delegation.md`](../spec/agents/delegation.md) |

Each preview file carries a `PREVIEW` marker in its header. That marker is
required: it is what stops a reader from mistaking an intention for a feature.

## Conventions

* One directory per example, one `main.nudo` per directory, named after the
  example rather than its number.
* Preview examples do not call standard library functions that do not exist.
  Where an example needs behaviour the language cannot express yet, it says so in
  a comment instead of inventing an API.
* Examples are documentation, not tests. What is checked is whether the
  toolchain can read them, and that a preview says it is a preview.
