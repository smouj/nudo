# Examples

Ten examples, in increasing order of ambition. Each one states at the top of the
file whether the current toolchain can read it, and that statement is true — a
test in `compiler/nudo-parser/tests/examples.rs` checks it.

## What the toolchain reads today

| Example | Contents | `nudo check` |
| ------- | -------- | ------------ |
| [`00-hello-world/`](00-hello-world/main.nudo) | Bindings, functions, a text literal | **Clean** |
| [`01-variables/`](01-variables/main.nudo) | Literals, `_` separators, nesting, escapes | **Clean** |
| [`02-functions/`](02-functions/main.nudo) | Declarations, parameters, return types | **Clean** |
| [`03-types/`](03-types/main.nudo) | Structs, enums, variants with fields, `match` | **Clean** |

```console
$ nudo check examples/00-hello-world/main.nudo
checked 1 file: 0 errors and 0 warnings
```

"Clean" means no lexical **and no syntax** diagnostics: the file lexes and it
parses. It does not mean the file is correct or that it runs — there is no type
checker (M3) and no interpreter (M4).

## Design previews

The rest illustrate proposed syntax and are **not** accepted by the current
toolchain. Each one says in its header which syntax stops it, and `nudo check`
reports that as an `NDO1001` syntax error rather than pretending the file was
read.

| Example | Illustrates | Why it is not accepted yet |
| ------- | ----------- | -------------------------- |
| [`04-results/`](04-results/main.nudo) | Errors as values, no exceptions | Generic parameters on a declaration; `return` |
| [`05-agent/`](05-agent/main.nudo) | An agent as a bounded executor | Newline-separated, dotted tool names |
| [`06-tools/`](06-tools/main.nudo) | Tools, scoped capabilities, effects | Dotted tool names; a capability with an argument |
| [`07-generated-verified/`](07-generated-verified/main.nudo) | `Generated<T>` vs `Verified<T>` | Two `let`s with no `;` |
| [`08-policy/`](08-policy/main.nudo) | Policies, approvals, budgets | `approve` is not in the grammar |
| [`09-multi-agent/`](09-multi-agent/main.nudo) | Delegation that narrows authority | Dotted, newline-separated tools; a block with named fields |

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
* Examples are documentation, not tests. What is checked is whether the
  toolchain can read them, why it cannot when it cannot, and that a preview says
  it is a preview.
