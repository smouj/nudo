# NEP-0018: Statically checkable verifiers

| Field | Value |
| ----- | ----- |
| Status | Draft |
| Authors | @smouj |
| Created | 2026-10-26 |
| NEP number | 0018 |
| Supersedes | — |
| Superseded by | — |
| Specification chapters | [`spec/declarations.md`](../spec/declarations.md), [`spec/expressions.md`](../spec/expressions.md), [`spec/trust/verified.md`](../spec/trust/verified.md) |
| Related NEPs | NEP-0002, NEP-0004, NEP-0014 |

## Summary

Decide how a compiler identifies a valid verifier, checks the type of the value
it verifies, and enforces the deterministic-verifier rule before `verify` can
produce `Result<Verified<T>, VerificationError>`. The result type and failure
behavior are already decided by NEP-0002; this proposal does not change them.

## Motivation

The parser accepts `verify value with Name`, and HIR resolves `Name` in the
ordinary value namespace. That is not enough to type-check the expression
safely: the resolved definition could be an integer binding or an unrelated
function. Returning the decided `Result` type for every resolved name would
make the syntax appear to verify a value without proving that a verifier ran.

```nudo
let checked: Result<Verified<Article>, VerificationError> =
    verify draft with unrelated_function;
```

The checker needs a declaration identity and a contract it can check. It also
needs to reject a verifier that asks a model to judge its own output, as
required by NEP-0002, without guessing which capabilities or effects a
deterministic verifier may use.

## Guide-level explanation

No source syntax is selected by this Draft. One candidate is a dedicated
declaration whose input type and deterministic body are visible to the compiler:

```nudo
verifier ArticleVerifier(article: Article) {
    // declared verification criteria
}

let checked = verify draft with ArticleVerifier;
```

Another candidate is an ordinary function with an explicit verifier contract.
The accepted behavior remains: `verify` can fail, yields
`Result<Verified<T>, VerificationError>`, and never yields a `Verified<T>` on
failure.

## Reference-level explanation

Any accepted design must let HIR preserve a verifier's declaration identity so
the type checker can reject an arbitrary value used after `with`. It must define
the relationship between the verifier's input and the verified `T`, and it must
make the deterministic restriction enforceable across calls. A naming
convention or a runtime promise alone is not sufficient evidence for the static
trust boundary.

The following choices need review before the proposal can be accepted:

* **Declaration form.** A dedicated `verifier` declaration gives HIR a distinct
  kind to resolve. A function contract reuses function declarations but needs a
  non-forgeable marker the checker can see.
* **Input contract.** The verifier could inspect `T` or `Generated<T>`. The
  language must state whether `verify` accepts only generated values, and how
  the verifier obtains the underlying value without dropping its provenance.
* **Outcome.** The accepted outer result remains
  `Result<Verified<T>, VerificationError>`. The proposal must decide whether a
  verifier reports a Boolean rejection, a domain error, or a structured
  verification outcome, and how each failure maps to `VerificationError`.
* **Determinism and effects.** NEP-0002 forbids model calls in a verifier. The
  language must decide whether other capabilities are forbidden, explicitly
  declared, or governed by the effect system before the type checker may trust
  the declaration.

Verification by a model remains deferred to M6 as already decided. It cannot be
introduced by this proposal as an alternative path to `Verified<T>`.

## What this makes impossible

* Treating any resolved value as a verifier merely because it follows `with`.
* Obtaining `Verified<T>` from a verifier that the compiler cannot distinguish
  from an ordinary function or value.
* Claiming a verifier is deterministic based only on its name or a runtime
  convention.

## Alternatives

| Alternative | Why it is insufficient |
| ----------- | ---------------------- |
| Accept any resolved value after `with` | HIR resolution proves only that a name exists, not that it has verifier semantics |
| Use a naming convention such as `*Verifier` | Names can be shadowed and do not express a signature or effect contract |
| Validate only at runtime | A caller can type-check and route `Verified<T>` before the runtime discovers the verifier is invalid |
| Allow a model to verify its own output | Rejected by NEP-0002; it closes the trust loop without independent evidence |

## Impact

| Area | Effect |
| ---- | ------ |
| Specification | Declaration, expression and verified-trust chapters gain a checkable verifier contract |
| Grammar | May add a dedicated verifier declaration; `verify` expression syntax already exists |
| Compiler | HIR must retain verifier identity and contract; type checking must reject invalid verifier uses |
| Conformance | Valid verifier, wrong type, ordinary value used as verifier, effect violation and failure cases |
| Tooling | Formatter and language server recognize any new declaration form |
| Security | Closes the path from an arbitrary name to a `Verified<T>` result |
| Compatibility | Depends on the selected declaration form and input contract |
| Performance | Static identity and signature checks; measure with the type-checker benchmark suite |

## Open questions

* Should verifiers have a dedicated declaration kind or a checked function
  contract?
* Does a verifier receive `T` or `Generated<T>`?
* Which non-model effects, if any, can a deterministic verifier use?
* Which verifier outcomes map to the already-decided `VerificationError`?

## Prior art

No comparison is included in this Draft. The choice must be made against NUDO's
own requirement that the compiler, not a convention or runtime promise, enforce
the trust boundary.

## Unresolved objections

None recorded yet. This Draft is not ready for implementation.
