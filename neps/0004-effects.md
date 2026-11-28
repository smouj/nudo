# NEP-0004: Effect system

| Field | Value |
| ----- | ----- |
| Status | Discussion |
| Authors | Codex, for @smouj |
| Created | 2026-09-17 |
| NEP number | 0004 |
| Supersedes | — |
| Superseded by | — |
| Specification chapters | [`spec/effects.md`](../spec/effects.md), [`spec/expressions.md`](../spec/expressions.md), [`spec/generics.md`](../spec/generics.md), [`spec/errors.md`](../spec/errors.md), [`spec/trust/capabilities.md`](../spec/trust/capabilities.md) |
| Related NEPs | [NEP-0003](0003-capability-system.md), [NEP-0013](0013-generic-inference.md), [NEP-0018](0018-statically-checkable-verifiers.md) |

## Summary

This proposal defines the M5 effect checker as a compile-time pass over
statically resolved calls. Function declarations name the exact set of effects
their bodies require; the checker infers that set from built-in operations and
called functions, then verifies the declaration. Function types carry the
same effect set. Effect identities are resolved statically, so this pass adds
no runtime dispatch or allocation. Runtime capability grants remain M8 work.

## Motivation

The grammar already accepts `with Network`, but no accepted rule says whether
it is a permission, a requirement, an upper bound, or documentation. Without a
contract, two implementations could accept different calls, lose effects
through function values, or disagree about recursive functions.

The M5 exit criterion requires an effectful call from an effect-free function
to fail with a diagnostic that explains where the effect came from. NUDO also
needs an effect model that remains bounded at compile time and has no hidden
runtime cost for statically known calls.

## Guide-level explanation

`with` declares the effects a function actually uses, including effects that
arrive through calls:

~~~nudo
fn show_answer() with Console {
    print(20 + 22);
}

fn main() with Console {
    show_answer();
}
~~~

`print` requires the `Console` effect. The checker infers `Console` for both
functions. Omitting it from either signature is an error. Listing an effect a
body does not use is also an error, so a signature cannot silently request
extra authority.

This is a Discussion proposal. It changes no specification or compiler
behavior until the maintainer accepts it.

## Reference-level explanation

### Effect identities

* An effect is a statically resolved capability identity, not a string looked
  up at runtime. A local value with the same spelling does not shadow an effect
  name.
* The proposed built-in effect set is `Console`, `Network`, `Filesystem`,
  `Shell`, `Git`, `Secrets`, `Model`, and `Human`. `Console` covers the M4
  `print` operation. Other built-in operations acquire their effect mapping in
  the NEP that defines them.
* The set is closed for an edition. Adding an effect identity requires a NEP.
  NEP-0003's runtime grant model, scopes and revocation rules remain outside
  this proposal and M5. Adopting these names does not accept NEP-0003's grant
  semantics.

For M5, `print` contributes `Console` and `ask` contributes `Model` when its
accepted expression rules apply. `verify` itself is local and deterministic;
its verifier body's allowed operations remain governed by NEP-0018. Agent,
task, tool and `delegate` effects remain outside this proposal until their
declaration and runtime contracts are accepted.

### Declarations and inference

* A function's `with` clause declares the exact set of effects its body
  requires. An omitted clause means the empty set.
* The checker infers the body's set from intrinsic operations, direct calls,
  and calls through function values. A call contributes the effect set in the
  callee's resolved signature. A built-in such as `print` contributes its
  registered set (`Console`). The function's declared and inferred sets must
  be equal.
* Inference checks every branch and operand, including both sides of `&&` and
  `||`, both `if` branches, and every `match` arm. This is static analysis;
  runtime short-circuiting does not make an effect disappear from a signature,
  as NEP-0017 already requires for these expressions.
* Function signatures are collected before bodies are checked. Recursive and
  mutually recursive calls use those declared sets; they do not require
  whole-program type inference or a fixed-point over effect variables. A
  function that calls another function requiring `Network` must declare
  `Network` itself, whether or not the call is recursive.
* A generic function body is checked once against its declared effect set;
  effect requirements do not vary by type argument or call-site instantiation.
* An effect error invalidates that function's effect summary. Callers suppress
  only diagnostics that depend on that invalid summary; independent errors
  remain visible.

### Function values

The function-type production gains the existing effect-clause form:

~~~ebnf
function-type = "Fn", "(", [ type, { ",", type } ], ")", "->", type,
                [ effect-clause ] ;
~~~

For example, `Fn(Text) -> Text with Network` is a function value whose call
requires `Network`.

Function parameter and result types must otherwise match exactly. A value with
required effects `A` is assignable to an expected function type requiring `B`
when `A` is a subset of `B`. A caller that promises it can handle `B` can safely
call a function requiring fewer effects. Local type inference carries the full
function type, including its effect set; it does not infer a new set from call
sites. No effect variables or generic effect bounds are added.

### Calls and host boundaries

* A direct call is valid only when its enclosing function's declared set
  contains every effect in the callee's declared set.
* A function-value call uses the effect set on that value's function type and
  follows the same rule.
* The top-level `main` function has no implicit effect exemption. If it calls
  `print`, its signature declares `Console` like any other function. `nudo run`
  is the host entry point, not a language-level effect handler.
* M5 checks effect declarations statically. It does not implement runtime grant
  validation. The host's grant checks and any scoped authority remain NEP-0003
  and M8 work.
* This NEP adds no language-level handler syntax. Algebraic handlers and
  subtracting effects remain future design work; they are not prerequisites for
  checking the M5 call graph.

### Diagnostics and recovery

| Code | Name | Proposed rule |
| ---- | ---- | -------------- |
| `NDO3001` | `UNDECLARED_EFFECT` | An operation or call requires an effect absent from the enclosing function's clause. |
| `NDO3002` | `UNUSED_EFFECT` | A declared effect is absent from the body's inferred set. |
| `NDO3003` | `UNKNOWN_EFFECT` | An effect name is not in the edition's built-in effect set. |
| `NDO3004` | `DUPLICATE_EFFECT` | A resolved effect identity occurs more than once in one clause. |

* Emit one `NDO3001` per missing effect in a function, at the earliest direct
  source use that requires it. Its notes name the function that requires the
  effect and show the shortest call path to the operation that introduced it;
  when a function-value parameter introduces the requirement, the path ends at
  that parameter's function type. Ties use source order. This keeps a single
  root cause from producing a diagnostic at every caller.
* Emit one `NDO3002` for each unique, known effect absent from the inferred
  body set, at its first occurrence in the clause. Unknown names do not also
  produce `NDO3002`.
* `NDO3003` points at the first occurrence of each distinct unknown effect
  name. Repeating the same unknown spelling does not repeat the diagnostic.
  Unknown names are not local bindings or runtime capabilities and are excluded
  from the declared set.
* `NDO3004` points at each repeated occurrence of a known effect after its
  first occurrence. The set retains only the first occurrence, so duplicates
  do not multiply `NDO3001` or `NDO3002` diagnostics.
* A function-value effect-set incompatibility uses `NDO2004`, with the
  expected and found function types, including their effect sets.
* If type resolution or a child expression already failed, suppress only
  effect diagnostics that depend on that result. Independent effect errors
  remain reportable. Effect errors recover the affected function summary as
  `Error`, preventing cascades into callers.

Diagnostic order is deterministic: function source order, effect declaration
order, then source position. Messages, spans and call-path notes are part of
the conformance contract.

### Complexity and performance

The effect registry is finite within an edition. Implementations can represent
its identities as a bit set; the proposed initial set fits in one `u64`.
Checking bodies is linear in call and intrinsic-operation sites, plus fixed-size
set unions. For diagnostics, build caller adjacency once, then run one reverse
multi-source breadth-first search per effect that needs a path. Intrinsic
operation sites and function-value call sites seed the search; source order
orders seed sites and caller edges to break ties; one predecessor per function
reconstructs the shortest path. With `F` functions, `C` call sites, `I`
intrinsic sites, fixed registry size `K = 8`, and `P` bytes of rendered path
notes, path reporting is `O(K(F + C + I) + P)` time and `O(F + C + I)` working
memory, excluding diagnostic output. Because `K` is fixed for an edition, the
graph work is linear in program size. No runtime effect-set lookup, allocation,
or dispatch is added.

These are compiler bounds, not a claim that executing an effect costs nothing.
Printing, network access and every other real operation retain their actual
costs. Runtime authorization is a separate M8 concern.

### Conformance

M5 conformance should pin pure functions, direct and transitive effects,
missing, unused, unknown and duplicate annotations, effectful function-value
calls, the function-type subset rule, recursion, branch and short-circuit
analysis, exact spans, deterministic shortest call paths, and error recovery
without cascades. The accepted `Console` mapping should include the M4
`print(42)` vertical slice. A compiler benchmark should vary `F`, `C` and `I`
to check the stated scaling; runtime tests should confirm that static checking
adds no effect table or per-call check. Runtime denial and grants belong to M8.

## What this makes impossible

* An effectful call hidden inside a function whose signature says it is pure.
* A function value that loses the effects its calls require.
* A typo in an effect clause silently becoming a value or new capability.
* A function declaring extra effects to make callers request unnecessary
  authority.
* Effect checking that adds a runtime registry lookup to every static call.

## Alternatives

| Alternative | Why it was not selected |
| ----------- | ----------------------- |
| Infer public effect signatures silently | Hides requirements from readers and makes an implementation change alter a function's interface without a source edit |
| Treat `with` as an upper bound and permit unused effects | Makes callers require authority the body does not use; exact equality keeps declarations least-privilege and reviewable |
| Drop effects from `Fn` types | A function value could then smuggle an effect through an apparently pure parameter |
| Require exact equality for function-value effect sets | Rejects using a function with fewer requirements where the caller already promises to handle a larger set |
| Add algebraic handlers in M5 | Adds control-flow and runtime semantics beyond the M5 goal; the existing host boundary is enough to check effect propagation |
| Enforce runtime grants in the type checker | A compile-time type checker cannot know which scoped grants the host supplies; runtime enforcement belongs to M8 |

## Impact

| Area | Effect |
| ---- | ------ |
| Specification | Define effect identities, exact body inference, function-type effects, diagnostics and the M5 boundary |
| Grammar | Add an optional effect clause to `function-type`; declarations already accept one |
| Compiler | Implement the `nudo-effects` stage over typed, statically resolved calls |
| Conformance | Add M5 effect cases; keep runtime grants and denial in M8 |
| Runtime | No M5 runtime checks; host enforcement remains a separate M8 gate under NEP-0003 |
| Security | Signatures cannot omit or overstate a statically visible effect; runtime grant policy remains outside this proposal |
| Performance | Fixed-size compile-time sets and no runtime effect dispatch for static calls |

## Open questions

1. Should a function's declared effect set exactly equal its inferred set, as
   proposed, or be allowed to conservatively contain unused effects?
2. Should function-value effect sets use the proposed subset compatibility
   rule, or require exact set equality?
3. Should M5 adopt the seven capability names proposed by NEP-0003 and add
   `Console` for `print`, while leaving grants and scopes to M8, as proposed?
4. Should M5 defer language-level handlers and runtime authorization, leaving
   `nudo run` as the host entry point, as proposed?
5. Are `NDO3001`–`NDO3004`, their spans and deterministic path reporting the
   right diagnostic contract?

An accepted NEP must close all five questions. No effect behavior is
implemented by this Discussion proposal.

## Prior art

NEP-0003 proposes the capability vocabulary and deny-by-default grant model.
NEP-0013 keeps generic inference local and bounded; this proposal adds no
effect-variable inference across generic bounds. NEP-0018 proposes a
call-free verifier body until effect checking is accepted. That restriction
remains until NEP-0018 itself is accepted and updated; this NEP does not change
it implicitly.

## Unresolved objections

None recorded.
