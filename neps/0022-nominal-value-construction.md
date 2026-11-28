# NEP-0022: Nominal value construction

| Field | Value |
| ----- | ----- |
| Status | Discussion |
| Authors | Codex, for @smouj |
| Created | 2026-11-28 |
| NEP number | 0022 |
| Supersedes | — |
| Superseded by | — |
| Specification chapters | [declarations.md](../spec/declarations.md), [expressions.md](../spec/expressions.md), [grammar.md](../spec/grammar.md), [types.md](../spec/types.md), [errors.md](../spec/errors.md) |
| Related NEPs | [NEP-0002](0002-generated-verified.md), [NEP-0008](0008-error-model.md), [NEP-0010](0010-declaration-site-generics.md), [NEP-0013](0013-generic-inference.md), [NEP-0014](0014-trust-types-obey-the-rules.md), [NEP-0020](0020-typed-enum-matching.md) |

## Summary

This proposal adds direct construction for user structs and enums with
statically resolved, complete field and payload values. Struct fields use
explicit names; enum payloads use the declaration order already used by
NEP-0020 matching. Generic arguments use NEP-0013's bounded, local inference
from constructor values and an already-known expected type. Trust wrappers
remain constructible only by their language operations.

## Motivation

NUDO can declare structs and enums, preserve their fields and variants in HIR,
type-check field access, and match enum values. It cannot create either value
from source. A function can receive a nominal value or return one, but there is
no expression that makes the value. That leaves M3.2 incomplete and prevents
ordinary programs from using the types the language already checks.

Construction needs an explicit contract for field completeness, enum payload
order, generic inference, evaluation order, and trust wrappers. Those rules
cannot be guessed in the parser.

## Guide-level explanation

The proposed syntax names every struct field and qualifies an enum variant:

~~~nudo
struct Point {
    x: Int
    y: Int
}

struct Empty {}

enum Status {
    Ready
    Failed(reason: Text)
}

let origin = Point { x: 0, y: 0 };
let empty = Empty {};
let failure = Status::Failed("source unavailable");
let ready = Status::Ready;
~~~

Every struct field must appear exactly once. Enum payload values are positional
and follow the variant declaration. A unit variant is its qualified path; an
empty call, such as Status::Ready(), is accepted as the same value for
consistency with NEP-0020's zero-payload pattern spelling.

No constructor-specific type-argument syntax is added. A generic type's
arguments come from its field or payload values, then an expected type that is
already known. An unresolved parameter is a compile error; the checker never
chooses a default.

This is a Discussion proposal, not accepted syntax or runnable behavior.

## Reference-level explanation

### Proposed construction forms

#### Structs

* A struct expression has the form TypeName { field: expression, ... }. The
  head resolves in the type namespace to one user-declared struct. A zero-field
  struct is constructed with `TypeName {}`.
* Every declared field appears exactly once. Field initializers may appear in
  any order and use explicit field: expression syntax. This edition has no
  shorthand, positional construction, defaults, partial values, or update
  spread syntax.
* A field name not declared by the struct is NDO2007. A repeated initializer
  name produces proposed NDO2026, DUPLICATE_FIELD_INITIALIZER, at each repeated
  field name. A missing field produces proposed NDO2023,
  MISSING_FIELD_INITIALIZER, at the struct expression, with missing names
  listed in declaration order.
* Each value is checked against the field type after substituting the
  constructed type's generic arguments. A mismatch uses NDO2004 at the field
  value. All supplied values are checked, including values after an earlier
  field error.
* The nominal type's generic parameters are inferred from supplied fields in
  source order, then from an expected type that was already known, following
  NEP-0013. Conflicting field inferences use NDO2004 at the later value. An
  unresolved generic parameter uses NDO2008 at the whole constructor
  expression. This NEP adds no explicit constructor type arguments, defaults or
  whole-program inference.
* Field initializers are evaluated exactly once from left to right in source
  order. The resulting fields occupy their declaration-defined positions.

The proposed grammar addition is:

~~~ebnf
primary-expression  = ... | struct-construction ;
struct-construction = path, "{",
                      [ field-initializer, { ",", field-initializer } ],
                      "}" ;
field-initializer   = identifier, ":", expression ;
~~~

Initializer lists are comma-separated. This proposal adopts NEP-0011's policy
for this list: a trailing comma is allowed in a multi-line initializer and
rejected in a single-line initializer.

The parser recognizes an initializer-shaped brace after a path as struct
construction, except at the boundary between an `if` condition and its required
body. There, the body block keeps the existing `if condition { ... }` meaning.
An empty construction at the end of an `if` condition is grouped to make the
choice explicit, for example `if (Empty {}) { ... }`. A constructor followed
by another expression operator or postfix suffix is unambiguous and needs no
extra grouping. This is a syntactic rule; it does not require type resolution
or parser backtracking.

#### Enums

* A unit variant is constructed as EnumName::Variant. EnumName resolves in the
  type namespace and Variant resolves only among that enum's declared
  variants. An unknown variant produces proposed NDO2024,
  INVALID_VARIANT_CONSTRUCTOR, at the variant name; it is never a new binding.
* A payload variant is constructed as EnumName::Variant(value, ...). The
  number of values must equal the declared payload count; otherwise proposed
  NDO2027, WRONG_VARIANT_CONSTRUCTOR_ARITY, applies at the variant constructor,
  with expected and found counts. Values are positional, in declaration order,
  matching NEP-0020's flat payload pattern.
* A zero-payload variant may also be written with an empty call. A payload
  variant path without its required call is not a value or first-class
  constructor function in this edition; it reports NDO2027 with a found count
  of zero.
* Generic parameters are inferred from payload types in source order and then
  from an already-known expected type, following NEP-0013 and the same bounded
  process as struct construction. A parameter that remains unknown uses
  NDO2008.
* Enum payload values are evaluated exactly once from left to right in source
  order. The constructor writes the selected variant tag and payload directly;
  no runtime name lookup is required.
* Intrinsic Result uses the same constructor forms: Result::Ok(value) and
  Result::Err(error). Their T and E parameters use payload and expected-type
  inference; unresolved parameters use NDO2008.

Enum construction already fits the path and call syntax in the grammar, but
name resolution and HIR must preserve the enum type, variant identity and
payload positions rather than treating the path as an ordinary function.
Intrinsic `Result` behavior is selected by the resolved built-in definition,
not by the spelling `Result`; a user enum with that name uses its own variants.

### Trust and special types

* Generated<T> is produced only by ask and Verified<T> only by verify. Neither
  type has a source-level struct or enum constructor. This preserves NEP-0002's
  trust boundary.
* Result<T, E> is intrinsic and constructible through its Ok and Err variants,
  as required by NEP-0008. The compiler does not expose a constructor for the
  Result type itself.
* T? construction remains open. This proposal does not decide how an optional
  value is absent or present.
* Agent, task, tool and model construction remains outside this NEP; those
  declarations still require their own accepted contracts.

### Recovery and diagnostics

* A head that resolves to the wrong namespace uses NDO2003. A type with no
  constructor form in this proposal (including `Generated<T>` and
  `Verified<T>`) uses proposed NDO2025, NON_CONSTRUCTIBLE_TYPE, at the type
  head.
* Unknown, duplicate and missing struct fields use NDO2007, proposed NDO2026,
  and proposed NDO2023 respectively. NDO2026 points at each repeated field
  name; NDO2023 points at the constructor and lists absent fields in declaration
  order.
* An unknown enum variant uses proposed NDO2024. A payload count mismatch uses
  proposed NDO2027 at the variant constructor, with expected and found counts.
  Payload type disagreement uses NDO2004.
* NDO2008 applies to unresolved generic parameters in struct and enum
  constructor expressions, including `Result` variants. Emit one diagnostic per
  unresolved parameter in declaration order at the whole constructor. This
  extends NEP-0015's bounded diagnostic to nominal construction. An erroneous
  supplied expression, a missing field, or a payload-count error suppresses
  NDO2008 only for parameters whose inference depends on that input; independent
  unresolved parameters remain reportable, following NEP-0015's recovery rule.
* If an input already has Error type, suppress dependent constructor
  diagnostics. Check all independent field and payload expressions in source
  order, then recover the whole constructor as Error after any constructor
  error.

These diagnostic assignments remain proposals until this NEP is accepted.

### Complexity and performance

At compile time, field names resolve to declaration indices; generic
substitution and payload checking are linear in the supplied types and values.
No whole-program solver is introduced.

At runtime, evaluate and place each value once. Struct fields use known
declaration offsets; enum construction writes one tag and its payload. There is
no runtime reflection, field-name map, default-value pass, or per-construction
name lookup. The storage needed by the actual field values remains a real cost;
the language does not promise zero-cost storage for arbitrary values.

### Conformance

M3.2 cases should cover complete and empty structs, field order, unknown,
duplicate and missing fields, field type mismatches, unit and payload enum
variants, wrong payload count, invalid variants, generic inference from fields,
payloads and expected types, NDO2008 recovery, Result construction, and rejection
of source construction for Generated<T> and Verified<T>. Parser cases should
pin the initializer-list separators and constructor/block disambiguation. M4
should execute struct and enum values and confirm source-order evaluation.

## What this makes impossible

* A partially initialized struct whose omitted fields get accidental defaults.
* A positional struct initializer whose meaning silently changes when fields
  are reordered.
* A misspelled enum variant becoming a local binding or dynamic constructor.
* A generic nominal type whose arguments depend on whole-program guessing.
* A source-level cast or constructor that forges Generated<T> or Verified<T>.

## Alternatives

| Alternative | Why it was not selected |
| ----------- | ----------------------- |
| Positional struct construction | Field order is less visible, and a declaration edit can silently change each argument's meaning |
| User-written new functions only | Leaves nominal field checking to arbitrary function code and does not directly construct intrinsic Result variants |
| Default or partial struct fields | Hides which values exist and adds default evaluation and evolution rules before they are needed |
| Unqualified enum constructors | Makes the variant namespace depend on imports and local bindings rather than on its enum |
| First-class generic variant constructors | Adds a new generic function-value and inference rule beyond the direct construction slice |

## Impact

| Area | Effect |
| ---- | ------ |
| Specification | Define struct and enum construction, completeness, order and trust restrictions |
| Grammar | Add named struct initializers; enum paths reuse existing path/call syntax |
| Compiler | Preserve field initializers and enum variant identity in HIR; type and resolve constructors in nudo-typeck |
| Conformance | Pin valid and invalid constructors, generic inference and error recovery |
| Tooling | Formatter and editor can use named fields and stable constructor diagnostics |
| Security | User code cannot manufacture Generated<T> or Verified<T> by nominal construction |
| Compatibility | Previously accepted path-plus-brace syntax gains struct-construction meaning; no published programs exist |
| Performance | Static field offsets, direct enum tags and no runtime reflection or constructor lookup |

## Open questions

1. Is named, complete struct initialization preferable to requiring a named
   constructor function or allowing defaults?
2. Should enum payloads remain positional to match NEP-0020, should empty calls
   be accepted for unit variants, and should arity errors use proposed NDO2027
   rather than the function-call diagnostic NDO2005?
3. Should generic nominal construction reuse NDO2008 and NEP-0013 inference, as
   proposed, or require explicit type arguments in a future edition?
4. Are proposed NDO2023 for missing fields and NDO2026 for repeated field
   initializers the right diagnostics, with unknown fields continuing to use
   NDO2007?
5. Should `Result::Ok` and `Result::Err` use the same inference rules as user
   enum payload constructors, and should NDO2025 reject every otherwise
   non-constructible type head, as proposed?
6. Is the `if`-body priority and parenthesized empty-construction rule above a
   clear resolution of the constructor/block ambiguity?

An accepted NEP must close all six questions. This Discussion draft changes no
language behavior.

## Prior art

NEP-0010 makes nominal type parameters declaration-site and invariant.
NEP-0013 limits inference to written values and already-known expected types.
NEP-0020 fixes positional enum payload patterns and forbids misspelled variants
from becoming bindings. This proposal carries those existing NUDO decisions
into construction.

## Unresolved objections

None recorded.
