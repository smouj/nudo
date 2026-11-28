# NEP-0021: Sequence indexing and bounds

| Field | Value |
| ----- | ----- |
| Status | Discussion |
| Authors | Codex, for @smouj |
| Created | 2026-11-28 |
| NEP number | 0021 |
| Supersedes | — |
| Superseded by | — |
| Specification chapters | [types.md](../spec/types.md), [expressions.md](../spec/expressions.md), [errors.md](../spec/errors.md) |
| Related NEPs | [NEP-0007](0007-integer-semantics.md), [NEP-0008](0008-error-model.md), [NEP-0013](0013-generic-inference.md), [NEP-0014](0014-trust-types-obey-the-rules.md) |

## Summary

This proposal defines immutable finite sequences as zero-based random-access
collections and makes indexing return Result<T, IndexError>, so an invalid
index is an ordinary, exhaustively handled value rather than a trap or an
unchecked access. It recommends Int indices, a unit OutOfBounds error, and O(1)
indexing with no per-access heap allocation; sequence literals and construction
remain outside this proposal.

## Motivation

The grammar already parses [T] and a[i], and HIR preserves index expressions,
but the type checker currently returns Error for every index. A reader cannot
tell whether the index must be Int, whether Text is indexable, or whether a
negative or too-large index traps, wraps, or returns a value.

The existing error model already distinguishes compile errors, fallible values,
and unrecoverable traps. Bounds errors are expected input-dependent outcomes a
caller can handle, so treating them as Result is consistent with NEP-0008 and
avoids panics or silent memory access.

## Guide-level explanation

The proposed rule gives an indexed sequence a fallible result:

~~~nudo
fn lookup(values: [Int], index: Int) -> Result<Int, IndexError> {
    values[index]
}
~~~

Indexing starts at zero. The expression evaluates its base and index once, in
that order. If the index is valid, it yields Ok(element); a negative index or
one at or beyond the sequence length yields Err(OutOfBounds). There is no
negative-from-the-end convention, clamping, wraparound, panic, or unchecked
access.

This is a proposal, not accepted syntax or executable behavior. It adds no
sequence literal or constructor; nominal construction remains a separate M3.2
gate.

## Reference-level explanation

### Proposed type and evaluation rules

* [T] is a finite, ordered, immutable sequence whose elements have type T.
  Nested sequences such as [[Int]] are valid. Generic arguments remain
  invariant under NEP-0010.
* In a[i], the base must be assignable to a sequence type [T] under the
  accepted trust assignment rules. The index must be assignable to Int. There
  is no implicit numeric conversion; Verified<Int> follows NEP-0014's ordinary
  assignment rule, while Generated<Int> does not.
* The expression type is Result<T, IndexError>. IndexError is an intrinsic
  enum with exactly one unit variant, OutOfBounds. This error has no payload:
  callers that need the original index already have the expression they
  supplied, and the successful path does not allocate an error object.
* Indexing is zero-based. It succeeds exactly when 0 <= i < length. Negative
  indices and indices equal to or greater than the length return
  Err(OutOfBounds). They never trap, wrap, clamp, or read outside the sequence.
* The base is evaluated first, then the index, each exactly once. The bounds
  check occurs after both values are available. This order remains fixed when
  calls later gain effects.
* Text is not indexable in this edition. It is UTF-8 text, and byte, scalar,
  and grapheme indexing are different contracts. A separate NEP may decide
  which text operation, if any, is appropriate.
* Indexing a non-sequence produces proposed NDO2022, NOT_INDEXABLE, at the
  base. A known non-Int index uses NDO2004 with expected Int and found type at
  the index expression. An already erroneous child suppresses only dependent
  diagnostics, and an invalid index expression recovers as Error.
* The result follows NEP-0008 and NEP-0014: callers can handle it with an
  exhaustive match, and this proposal adds no must-use rule.

The syntax is already in the grammar, so this proposal requires no grammar
change. M3.2 would implement the static receiver/index/result rules; execution
of the bounds check and the Result value belongs to M4, when a runtime exists.
This separates the proposed type contract from code that cannot yet run.

### Complexity and performance

An index operation performs one bounds check and one O(1) element-location
lookup, independent of sequence length. It uses O(1) temporary space for the
lookup and does not allocate to locate an element or construct the error
variant. Returning a composite immutable T may share its backing storage; the
whole sequence uses O(n) element slots plus storage for its values. This
proposal does not freeze the physical layout, but every implementation must
preserve O(1) location lookup. A benchmark should measure the interpreter path
when M4 lands; no performance claim is made before an executable implementation
exists.

The Result representation is an intrinsic value, not a boxed object.
Implementations may use a niche representation when the element type permits
it, provided observable behavior and the type remain unchanged.

### Conformance

M3.2 conformance should pin: [Int] index type, a nested [T] element type,
Verified<Int> assignability, rejection of non-sequence bases and non-Int
indices, the Result<T, IndexError> result type, recovery after an erroneous
base or index, and the fact that Text is not indexable. M4 runtime cases should
pin index zero, the final valid index, negative values, exactly length, and
values beyond length, including left-to-right single evaluation.

## What this makes impossible

* An index whose type depends on an implicit conversion from Float or Bool.
* Negative indexing whose meaning varies between implementations.
* A bounds failure that panics, wraps, clamps, or reads outside the sequence.
* A Result-specific must-use rule that contradicts NEP-0014.
* Indexing UTF-8 text without first deciding whether the unit is a byte, scalar,
  or grapheme.

## Alternatives

| Alternative | Why it was not selected |
| ----------- | ----------------------- |
| Return T and trap on an invalid index | Makes an expected bounds failure uncatchable, contrary to the explicit error model |
| Return T? | Loses the distinction between an absent element and the reason indexing failed; NEP-0008 already chooses Result for fallible operations |
| Permit negative indices | Adds a second index interpretation and adjustment path without a corresponding requirement |
| Clamp or wrap the index | Silently returns an element other than the one requested |
| Leave bounds behavior to each backend | Makes the same NUDO program differ across interpreter and WASM |
| Index Text by UTF-8 byte | Exposes encoding details as character semantics and can split a scalar value |

## Impact

| Area | Effect |
| ---- | ------ |
| Specification | Define [T] and the type, ordering, and bounds behavior of indexing |
| Grammar | None; [T] and a[i] are already accepted syntax |
| Compiler | Type Index in nudo-typeck and resolve the intrinsic IndexError type |
| Conformance | Add M3.2 static cases and M4 runtime boundary cases |
| Runtime | Perform a checked O(1) access and return Result<T, IndexError> |
| Security | Prevents unchecked reads and backend-dependent bounds behavior |
| Compatibility | Programs that use a[i] will now have a Result type and must match or return it |
| Performance | One bounds check and O(1) location lookup without an allocation for the lookup; benchmark once M4 executes it |

## Open questions

1. Is a unit OutOfBounds error the right low-cost contract, or should it carry
   the requested index and sequence length?
2. Should the O(1) access bound be a language guarantee, as proposed, or should
   the specification leave complexity to each implementation?
3. Should Text remain non-indexable until a separate UTF-8 text proposal, as
   proposed?
4. Should a value of type Verified<[T]> be indexable directly under the
   top-level Verified<T>-to-T assignment rule, or require an explicit boundary?
5. Should runtime boundary tests be M4 evidence while M3.2 accepts only the
   static typing contract, as proposed?

An accepted NEP must close all five questions. This Discussion draft changes no
language behavior.

## Prior art

NEP-0008 makes Result the value for fallible operations and distinguishes it
from a trap. NEP-0007 fixes Int as signed 64-bit, and NEP-0014 keeps trust
types under ordinary assignment and Result rules. This proposal applies those
existing NUDO decisions to the already-parsed index expression.

## Unresolved objections

None recorded.
