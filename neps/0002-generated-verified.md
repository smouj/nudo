# NEP-0002: `Generated<T>` and `Verified<T>`

| Field | Value |
| ----- | ----- |
| Status | Accepted |
| Created | 2026-09-17 |
| Accepted | 2026-09-18, as gate 1 of M3.0 |
| Supersedes | — |
| Superseded by | — |
| Specification chapters | [`spec/trust/generated.md`](../spec/trust/generated.md), [`spec/trust/verified.md`](../spec/trust/verified.md), [`spec/types.md`](../spec/types.md), [`spec/expressions.md`](../spec/expressions.md) |
| Related NEPs | NEP-0001, NEP-0003, NEP-0008 |

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
let draft: Generated<Article> = ask Writer { "Create an article." };

let checked: Result<Verified<Article>, VerificationError> =
    verify draft with ArticleVerifier;
```

`Generated<T>` may be displayed, stored and passed around. It cannot be used
where a `Verified<T>` or a `T` is required.

`verify` is an operation, not a cast: it runs a verifier, can fail, and leaves a
provenance record. It yields a `Result`, so a rejected verification is an
ordinary value the caller handles — not a panic, and not a silent downgrade.
`publish(article: Verified<Article>)` states its precondition in its signature,
and the compiler enforces it.

## Reference-level explanation

* `Generated<T>` and `Verified<T>` are distinct, non-interchangeable types. No
  subtyping in either direction; no implicit conversion to `T`.
* `ask` yields `Generated<T>` and has no form that yields plain `T`.
* **`verify value with Verifier` yields
  `Result<Verified<T>, VerificationError>`.** A verification that fails is a
  value, not a trap and not a panic; a caller cannot obtain a `Verified<T>`
  without handling the failure. This is rule 3 of
  [`spec/trust/verified.md`](../spec/trust/verified.md) made into a type.
* **The operand is a named value**: a path, optionally called (`verify draft`,
  `verify parse(text)`). A parenthesised or compound operand
  (`verify (draft) with V`) is not in the grammar, which is what lets the parser
  read `verify` as a verification with one token of lookahead and no
  backtracking. Widening the operand is additive and needs a NEP.
* `Verified<T>` carries provenance: which verifier, which criteria, which inputs.
* `T` is assignable from `Verified<T>` — a verified value can always be used
  where an ordinary value is expected.
* **A verifier may not be a model call in this edition.** A verifier is a
dedicated declaration whose body is deterministic; verification by a model is
deferred to M6, which must say whether the result is `Generated<Verified<T>>`
and how the loop closes. Until then the compiler must not accept a verifier
that calls a model, because a model's judgement about its own output cannot be
the step that promotes it.

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

Settled when this NEP was accepted:

* **The shape of a verification.** It yields `Result<Verified<T>,
  VerificationError>`, and the failure is a value. The alternative — yielding
  `Verified<T>` and trapping on failure — was rejected because a verifier
  rejecting its input is normal, not exceptional, and a language that turns a
  normal outcome into a trap cannot express retry.
* **Verification by model.** Not in this edition. A verifier is deterministic;
  the model-verifier is M6's decision, with the loop-closing question stated
  there.
* **`verify` stays a contextual word** ([NEP-0005](0005-keyword-policy.md)), and
  the earlier one-token limitation is removed by restricting the operand rather
  than by reserving the word. `verify(draft)` therefore remains a call on a name
  called `verify`, and `verify draft with V` is a verification: the parser still
  decides with one token, and no valid program is reinterpreted.
* **Provenance cannot be dropped implicitly.** An explicit step for dropping it
  is wanted (audit trails, logs) and is deferred to M6, with the ceremony it
  needs.

Still open:

* Growth and retention limits on provenance.
* Whether a person editing a `Generated<T>` produces a third kind of value.

## Implementation status

**Not implemented.** This NEP is a gate for M3, taken before the type checker so
that `nudo-typeck` is written against a decided shape rather than a guess.
`nudo-parser` already parses `ask`, `verify` and `delegate`; the operand
restriction and the two trust types themselves land in M3.1–M3.3.
