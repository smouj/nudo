# `Verified<T>`

**State: proposed.** The type exists in the design, not in the compiler.

## What it means

`Verified<T>` is a value that passed an **explicit verification step** against
**stated criteria**. It means:

* something checked it, and that something is named;
* the criteria were declared, not inferred;
* the check can fail, and did not;
* the whole thing is recorded as provenance.

## Producing one

```nudo
let draft: Generated<Article> = ask Writer { "Create an article." };
let checked: Result<Verified<Article>, VerificationError> =
    verify draft with ArticleVerifier
```

`verify` is not a cast. It is an operation that can fail, that costs something,
and that leaves a record. If it could not fail, it would be a lie about what
verification is. And because it can fail, it yields a
**`Result<Verified<T>, VerificationError>`** ([NEP-0002](../../neps/0002-generated-verified.md)):
there is no way to obtain a `Verified<T>` without handling the rejection.

## Rules

1. **The verifier is named.** A `Verified<T>` always knows what verified it. An
   anonymous verification is not verification.
2. **Criteria are declared.** A verifier implements criteria a reviewer can
   read, either built in (`SourcesRequired`) or user-defined.
3. **Failure is a value.** A failed verification returns an error the caller
   handles; it does not produce a `Verified<T>` and it does not panic. The type
   system enforces this: `verify` yields `Result<Verified<T>, VerificationError>`,
   so the only way to a `Verified<T>` is through a handled failure path.
4. **Verification is not transitive.** `Verified<T>` means the criteria of *that
   verifier* passed. It does not mean the content is true, and it does not mean
   a downstream consumer agrees with the criteria.
5. **A verifier is deterministic.** A verifier that calls a model is producing
   `Generated` judgements, not `Verified` ones; the language must not let that
   loop close silently. The case is deferred to M6, which has to say whether the
   result is `Generated<Verified<T>>` and how the loop closes. This was the
   principal open question of the trust model, and it is answered by
   *not* answering it in this edition.
6. **No downgrade without a reason.** `Verified<T>` can be treated as a `T`; it
   cannot be silently turned back into `Generated<T>` as a way to launder
   provenance.

## The honest limitation

A `Verified<T>` is exactly as trustworthy as its verifier. The type system
cannot make a bad verifier good; it can only make the choice of verifier
visible, reviewable and recorded.

That is a real limitation, and it is stated here rather than papered over with a
confidence score.

## Open questions

* **Verification by model.** If a verifier is itself a model call, is its output
  `Generated<Bool>`? Almost certainly yes, and the rules for what that means for
  the resulting `Verified<T>` need a NEP. This is the most important open
  question in the trust model.
* Whether verification results are cached, and how caching interacts with
  provenance.
* Whether a verifier can require capabilities (fetching a source, running a
  test), and how its budget is accounted.
* Whether `Verified<T>` can be demoted back to `Generated<T>` explicitly, for
  example when its provenance is no longer valid.
