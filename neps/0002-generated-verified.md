# NEP-0002: `Generated<T>` and `Verified<T>`

| Field | Value |
| ----- | ----- |
| Status | Discussion |
| Created | 2026-09-17 |
| Supersedes | — |
| Superseded by | — |
| Specification chapters | [`spec/trust/generated.md`](../spec/trust/generated.md), [`spec/trust/verified.md`](../spec/trust/verified.md), [`spec/types.md`](../spec/types.md) |
| Related NEPs | NEP-0001, NEP-0003 |

## Summary

Distinguish, in the type system, a value a model produced from a value that
passed an explicit verification step, and forbid implicit conversion between
them.

## Motivation

The characteristic failure of an AI-era program is treating a model's output as
if it were a fact. The treatment is not a bug in one place; it is the default
shape of the code, because there is nothing in the type system that says
otherwise.

```text
summary = model.complete(prompt)          # a string
publish(summary)                          # and now it is a fact
```

Nothing between those two lines marks the transition. A reviewer cannot see it,
a signature cannot state it, and a refactor can remove it without a diff that
looks important.

## Guide-level explanation

```nudo
let draft: Generated<Article> = ask Writer { "Create an article." }

let article: Verified<Article> = verify draft with ArticleVerifier
```

`Generated<T>` may be displayed, stored and passed around. It cannot be used
where a `Verified<T>` or a `T` is required.

`verify` is an operation, not a cast: it runs a verifier, can fail, and leaves a
provenance record. `publish(article: Verified<Article>)` now states its
precondition in its signature, and the compiler enforces it.

## Reference-level explanation

* `Generated<T>` and `Verified<T>` are distinct, non-interchangeable types. No
  subtyping in either direction; no implicit conversion to `T`.
* `ask` yields `Generated<T>` and has no form that yields plain `T`.
* `verify value with Verifier` yields `Verified<T>` or a failure value. Failure
  is handled by the caller, not by a panic.
* `Verified<T>` carries provenance: which verifier, which criteria, which inputs.
* `T` is assignable from `Verified<T>` — a verified value can always be used
  where an ordinary value is expected.
* A verifier that is itself a model call produces `Generated<Bool>`. The rules
  for that case are **not settled by this NEP** and are listed as its principal
  open question.

## What this makes impossible

* Passing model output to a function that requires checked input, without an
  explicit and reviewable verification step.
* Using a single type to mean both "the model said this" and "this is true".
  Programs relying on that ambiguity must now write down which one they mean.
* Silent degradation: a `Generated<T>` can no longer become trusted by being
  copied through three layers of code.

## Alternatives

| Alternative | Why it lost |
| ----------- | ----------- |
| A `confidence: Float` field | A score is a claim by the thing being judged; treating it as trust makes the model the authority on its own correctness |
| Runtime validation only | Too late and too invisible: the wrong path is a runtime failure instead of a compile error |
| An annotation, such as `@verified`, on a variable | Annotations are not types: they do not appear in signatures, so a caller cannot see the requirement |
| A single `Trusted<T>` with a level parameter | Two meanings in one type, and the interesting distinction is exactly the one the level would blur |

## Impact

| Area | Effect |
| ---- | ------ |
| Specification | `spec/trust/generated.md`, `spec/trust/verified.md`, `spec/types.md`, `spec/expressions.md` |
| Grammar | `ask-expression`, `verify-expression`, and `generics` syntax |
| Compiler | `nudo-typeck`, `nudo-provenance`, `nudo-ast` |
| Conformance | Conversion rejection cases; verification failure cases |
| Security | The central control against treating untrusted output as fact |
| Compatibility | Adds two types and two expressions; reserves `ask` and `verify` |
| Performance | Provenance has a memory cost, to be measured before stabilisation |

## Open questions

* **Verification by model.** If a verifier is a model call, is the result
  `Generated<Verified<T>>`? `Verified<T>` with a caveat? Something else? This is
  the most consequential question in the trust model, and it is unresolved.
* Whether provenance can be dropped explicitly, and with what ceremony.
* Growth and retention limits on provenance.
* Whether a person editing a `Generated<T>` produces a third kind of value.
* Whether `verify` is a keyword, a function or a protocol — the syntax here is
  provisional.
