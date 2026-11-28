# NEP-0018: Statically checkable verifiers

| Field | Value |
| ----- | ----- |
| Status | Discussion |
| Authors | Codex, for @smouj |
| Created | 2026-11-28 |
| NEP number | 0018 |
| Supersedes | — |
| Superseded by | — |
| Specification chapters | [expressions.md](../spec/expressions.md), [types.md](../spec/types.md), [trust/verified.md](../spec/trust/verified.md), [errors.md](../spec/errors.md) |
| Related NEPs | [NEP-0002](0002-generated-verified.md), [NEP-0004](0004-effects.md), [NEP-0014](0014-trust-types-obey-the-rules.md) |

## Summary

This proposal gives the existing verify expression a statically resolved,
deterministic verifier contract with a fixed typed failure, bounded checker
work, and direct calls without a runtime registry or dynamic dispatch. In line
with accepted NEP-0002, it proposes a dedicated top-level `verifier`
declaration with a restricted body. The exact syntax, accepted input types, and
provenance boundary still need review before this becomes an implementation
contract.

## Motivation

NEP-0002 already fixes the trust transition: verify returns
Result<Verified<T>, VerificationError>, the verifier is named and deterministic,
and success records which verifier was used. The parser and HIR represent
verify, but the type checker deliberately treats it as unsupported. The M3.3
roadmap names this NEP as the missing semantic gate; this proposal aligns its
declaration form with NEP-0002's dedicated-verifier requirement.

Without a declaration contract, implementations could disagree about whether
the verifier is a function, a runtime lookup, a model call, or a conversion.
That is precisely the boundary where NUDO needs a narrow, reviewable rule.

## Guide-level explanation

The proposed form declares a named, statically checked verifier:

~~~nudo
verifier ArticleVerifier(article: Article) -> Bool {
    article.title != "" && article.body != ""
}

fn publish(article: Verified<Article>) { }

fn publish_draft(draft: Generated<Article>) {
    let checked: Result<Verified<Article>, VerificationError> =
        verify draft with ArticleVerifier;

    match checked {
        Ok(article) => publish(article)
        Err(_) => { }
    }
}
~~~

The name after `with` resolves at compile time to one dedicated verifier
declaration. It checks a value and returns Bool. A true result produces Ok with
the same value as Verified<T>; false produces Err with a fixed
VerificationError::Rejected value. The verifier runs once. The expression does
not search a registry, use reflection, or dispatch through a trait object.

`verifier` is proposed as a contextual declaration marker. A verifier occupies
the existing value namespace and follows its duplicate-name rules. Its ordinary
function type is `Fn(T) -> Bool`, so code may call it or pass it as a function
value. Such a call returns only `Bool`; only `verify ... with` can produce
`Verified<T>`. An ordinary function name used after `with` is rejected as
NDO2028.

This is a proposal, not accepted syntax or runnable behavior. The sample uses
Article and Generated<Article> to explain the boundary; other M3.2 gates still
determine which complete programs can construct those values.

## Reference-level explanation

### Proposed verifier contract

The first implementation is deliberately limited and keeps the dedicated
declaration required by NEP-0002:

* A verifier declaration has the form `verifier Name(input: T) -> Bool { ... }`.
  It is top-level, has exactly one non-generic parameter, and has an explicit
  `Bool` result. Its function value has type `Fn(T) -> Bool`; using it as a
  function never constructs `Verified<T>`. The declaration has no effect
  clause; its body must remain deterministic and effect-free.
* The verifier target is a statically resolved top-level verifier name, not a
  function-valued expression or runtime-provided name. It resolves in the
  existing value namespace and must have `DefKind::Verifier`; this adds no
  third namespace.
* The verifier body is checked once per declaration. Each verify use checks
  only the resolved target and its signature; it does not re-traverse the body.
  The body may use deterministic value expressions,
  immutable local bindings, field access, conditionals, match, and the accepted
  scalar operators. Value paths may resolve only to the verifier's input
  parameter or immutable locals declared in its body, nested blocks, or match
  arms. References to top-level bindings, constants, and functions are
  forbidden; their initialization and effects are outside the verifier's
  deterministic contract. Type names and enum variants used in type and
  pattern positions remain allowed. The body may not call functions or use
  ask, verify, delegate, indexing, tools, tasks, or other operations that can
  introduce effects or depend on runtime state. There are no loops in the
  accepted subset.
* A verifier body must have type Bool. Every path is checked; a type error
  recovers as Error and must not trigger a second verifier-result mismatch.
* verify first evaluates its named operand, then calls the statically resolved
  verifier once. The operand may be T, Generated<T>, or Verified<T>; the
  Generated wrapper is unwrapped only by this explicit operation, while the
  existing Verified<T>-to-T assignment rule remains the ordinary rule from
  NEP-0014. The verifier parameter type determines T; no global inference is
  introduced.
* The expression type is exactly Result<Verified<T>, VerificationError>.
  VerificationError is proposed as an intrinsic enum with the single unit
  variant Rejected. This keeps the failure typed and allocation-free; arbitrary
  user messages and user-defined error payloads are outside this slice.
* The typed HIR records the verifier DefId on the verify expression. That pins
  verifier identity for conformance. At runtime, Verified<T> must retain that
  identity with its value; the runtime representation and complete
  input/provenance graph remain M6 work.
* Result keeps NEP-0014's ordinary rules: match must cover Ok and Err, and no
  must-use rule is added.

The restricted body is a temporary first-edition boundary that makes
determinism statically provable before the M5 effect checker exists. A later
NEP may permit calls to functions whose effect-free contract is proven by the
accepted effect system. It must not silently widen this subset.

The declaration's grammar is proposed as:

~~~ebnf
verifier-declaration = "verifier", identifier, "(", parameter, ")",
                       "->", type, block ;
~~~

The parser accepts the result-type position using the ordinary type grammar;
the verifier checker requires it to resolve to `Bool`. The contextual word
`verifier` starts this form only at item position and remains available as a
name elsewhere.

### Diagnostics and recovery

* An unresolved verifier name uses NDO2001.
* A resolved name that is not a supported verifier function receives proposed
  NDO2028, INVALID_VERIFIER_TARGET, at the verifier name.
* An operation or value path forbidden in a verifier body receives proposed
  NDO2029, UNSUPPORTED_VERIFIER_BODY, at that operation or path.
* Parameter and result mismatches use NDO2004 with structured expected and
  found types. A prior error suppresses only diagnostics that depend on it.
* On any verifier error, the verify expression recovers as Error so the same
  unknown result does not create follow-up mismatches.

These codes and exact primary spans remain proposals until the NEP is accepted.

### Complexity and performance

The restricted body check is linear in its expression nodes and runs once per
declaration. Each use performs static name resolution and a structural check of
the one-parameter signature; it does not re-check the body. The runtime uses a
direct statically resolved call and does not need a verifier registry,
reflection, dynamic dispatch, or an allocation for verifier lookup. The
predicate's own work still costs what its body requires. Result layout and
provenance storage are not promised to be free; they must be measured with the
M6 runtime and bounded by the accepted provenance contract.

### Conformance

The M3.3 cases should pin verifier resolution and signature shape, exact generic
payload selection, type mismatch and forbidden-body diagnostics (including
rejected references to top-level values), error recovery without cascades, the
resolved verifier DefId in typed HIR, and exhaustive Result handling under
NEP-0020. M3.3 has no execution engine, so it pins verifier identity in typed
HIR rather than runtime provenance. Runtime tests for successful verification,
false-to-Err conversion, and provenance belong to M6, when trust values are
executable.

## What this makes impossible

* Choosing a verifier through a runtime registry, reflection, or a dynamic
  function value.
* Treating a Bool-returning arbitrary function as a verifier when its body
  performs effects or depends on runtime state.
* Promoting a value through a verifier whose input or output type is unknown.
* Returning an untyped string or panicking on ordinary verification failure.
* Allowing a verifier body to call another function before the effect system can
  prove that call effect-free.

The last restriction makes helpers less convenient. It is proposed as a small,
auditable first slice; relaxing it needs an accepted effect contract.

## Alternatives

| Alternative | Why it was not selected as the initial recommendation |
| ----------- | ------------------------------------------------------ |
| Runtime verifier registry or plugin name | Adds lookup, registration state, and a runtime failure where the language already has a statically resolved path |
| Arbitrary function calls in verifier bodies | Determinism cannot be proved transitively until effect semantics are accepted and checked |
| Verifier returns a user-defined error type | Requires nominal construction and adds type inference and runtime payload choices before the related M3.2 gate is settled |
| A model acts as its own verifier | Confuses generated judgement with deterministic verification; NEP-0002 defers model verification |
| Reusing an ordinary function as a verifier | Blurs an ordinary callable with the dedicated verifier declaration required by accepted NEP-0002 |

## Impact

| Area | Effect |
| ---- | ------ |
| Specification | Refine verify's verifier, input, failure, and typed-HIR contracts |
| Grammar | Add a contextual top-level `verifier` declaration; `verify` expression syntax stays unchanged |
| Compiler | Add `DefKind::Verifier`, preserve its resolved identity in typed HIR, and type verify expressions in nudo-typeck |
| Conformance | Add M3.3 valid, invalid, recovery, identity, and Result cases |
| Tooling | Expose verifier-specific diagnostics and spans to editors |
| Security | Prevents model or effectful calls from silently promoting a value to Verified<T> |
| Compatibility | Rejects unsupported verifier targets and bodies; syntax accepted by the parser may still fail type checking |
| Performance | No runtime verifier lookup or dispatch allocation; verifier body work and provenance storage remain real costs |

## Open questions

1. Is `verifier Name(input: T) -> Bool { ... }` a clear dedicated form, with
   `verifier` contextual and stored in the existing value namespace?
2. Should a verifier accept T, Generated<T>, and Verified<T> as proposed, or
   should the trust transition accept only Generated<T>?
3. Is a single VerificationError::Rejected variant sufficient for M3.3, with
   richer deterministic reasons left for a separate future NEP?
4. Is a call-free body restricted to its input and local immutable bindings an
   acceptable deterministic first slice, with effect-checked helper calls
   deferred until after NEP-0004 is accepted?
5. Should M3.3 conformance pin verifier identity in typed HIR, with M6 testing
   successful verification, failure and the complete provenance graph, and the
   roadmap updated to place that runtime work in M6?

An accepted NEP must close all five questions. No verifier behavior is
implemented by this Discussion draft.

## Prior art

NEP-0002 is the governing NUDO contract: verification is explicit, named,
deterministic, fallible, and associated with provenance. NEP-0014 requires its
Result to follow ordinary exhaustive-match and no-must-use rules. NEP-0004
is in Discussion for M5 effect checking, but does not authorize implementation
until accepted. This proposal does not assume that effect checking already
exists.

## Unresolved objections

None recorded.

