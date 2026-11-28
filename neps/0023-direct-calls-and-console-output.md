# NEP-0023: M4 execution, direct calls, and console output

| Field | Value |
| ----- | ----- |
| Status | Discussion |
| Authors | Codex, for @smouj |
| Created | 2026-11-28 |
| NEP number | 0023 |
| Supersedes | — |
| Superseded by | — |
| Specification chapters | [`expressions.md`](../spec/expressions.md), [`errors.md`](../spec/errors.md) |
| Related NEPs | [NEP-0007](0007-integer-semantics.md), [NEP-0014](0014-trust-types-obey-the-rules.md), [NEP-0017](0017-boolean-comparison-conditionals.md), [NEP-0004](0004-effects.md) |

## Summary

This proposal defines the M4 execution boundary for `nudo run`, the observable
order of direct function-call arguments, and the `print` intrinsic: one
primitive argument, a `Unit` result, and one deterministic line written to
standard output. It also records the stable runtime diagnostics for this bounded
slice. Formatting writes directly to the output sink without a temporary output
string; M5 effect checking remains a separate decision under NEP-0004.

## Motivation

M4 needs one small program that can run from source to output. The current
interpreter already evaluates call arguments in source order, selects `main`,
implements `print`, and reports runtime failures, but those observable rules do
not yet have a normative contract. That makes programs depend on the first Rust
implementation, contrary to the specification-first rule.

The intended slice is:

~~~nudo
fn add(a: Int, b: Int) -> Int {
    a + b
}

fn main() {
    print(add(20, 22));
}
~~~

The proposal closes the contracts needed for the M4 single-file execution
slice. It does not decide effect signatures, handlers, runtime grants,
first-class function values, package loading, or general formatting for
user-defined types.

## Guide-level explanation

Arguments to a direct function call run once, from left to right, before the
function body starts. `print` accepts one `Int`, `Float`, `Bool` or `Text`,
writes its display form followed by a line feed to standard output, and returns
`Unit`:

~~~nudo
fn main() {
    print(42);       // writes `42` and a line feed
    print("ready"); // writes `ready` and a line feed
}
~~~

It is a built-in operation, not a first-class function. A declaration or local
binding named `print` shadows the built-in under ordinary name resolution.

`nudo run` accepts one source file and invokes its top-level `main()` only after
the full supported source has passed checking and lowering. A missing or
incorrectly typed entry point, an unsupported M4 construct, or a runtime trap is
a diagnostic; it never produces guessed execution behavior.

## Reference-level explanation

### `nudo run` and the M4 runtime boundary

* `nudo run FILE.nudo` accepts exactly one source file. It parses, resolves,
  type-checks and lowers the whole file before executing user code. Any error
  from those stages prevents execution; unsupported syntax in the M4 executable
  subset is reported during lowering, before `main` can produce output.
* After parsing, resolution and type checking succeed, the entry point must be
  a top-level function named `main`, with no parameters and result type `Unit`.
  An omitted result annotation means `Unit` under NEP-0016. No matching
  function produces `NDO6001` at the source start; a function with parameters
  or a non-`Unit` result produces `NDO6002` at its declaration. Earlier errors
  stop the pipeline before either entry-point diagnostic is considered.
* `NDO6003` reports signed `Int` overflow at the arithmetic operator, including
  the unrepresentable `Int::MIN / -1` and `Int::MIN % -1` cases. `NDO6004`
  reports `Int` division or remainder by zero at that operator. These implement
  NEP-0007's trap rules; `Float` division remains IEEE-754.
* The reference M4 interpreter limits execution to 32 active function calls.
  It reports `NDO6005` at a call that would exceed that resource limit, before
  entering it. This is an interpreter resource bound, not a restriction on
  recursion in the language or a required limit for future backends.
* A source form accepted by the checker but not executable in the bounded M4
  subset produces `NDO6006` at the source form during lowering. It must not
  silently become a no-op or partially execute the program. The supported subset
  is the one listed in [`ROADMAP.md`](../ROADMAP.md) under M4.
* `NDO6007` reports a host write failure at the `print` call, or a final flush
  failure at the `main` entry. Host I/O may have written a prefix before it
  fails; the interpreter cannot promise line-atomic writes.

### Direct function calls

* A direct call evaluates each argument exactly once, in lexical source order.
  It begins the callee only after all arguments have produced values.
* If evaluation of an argument fails, later arguments are not evaluated and the
  callee is not entered.
* This rule applies to ordinary direct calls and built-in direct calls. It does
  not define calls through function values, which are not part of the M4 slice.

### The `print` intrinsic

* Value-name lookup checks lexical declarations first. Only an otherwise
  unresolved, unqualified `print` resolves to the built-in. A shadowing
  declaration is an ordinary function or value, not the intrinsic.
* The intrinsic is callable only directly as `print(value)` and requires exactly
  one argument. A wrong count uses `NDO2005` and the checker still checks each
  supplied argument without an expected type.
* The argument must be assignable to `Int`, `Float`, `Bool` or `Text` under the
  ordinary assignment rules, including NEP-0014's accepted `Verified<T>` view.
  Other types use `NDO2020` at the argument. `Generated<T>` is not implicitly
  unwrapped.
* A well-typed call has type `Unit`.
* `Int` uses signed base-10 digits, without separators or a leading plus sign.
  `Bool` uses lowercase `true` and `false`. `Text` writes its UTF-8 bytes
  unchanged; it is not quoted, escaped or normalized.
* A finite `Float` uses a shortest round-tripping decimal significand, written
  in normalized scientific notation with lowercase `e`, no leading `+` in the
  exponent or leading zeroes, and an explicit decimal point and at least one
  fractional digit. The closest shortest significand is chosen; an exact tie
  chooses an even final digit. Positive and negative zero are `0.0e0` and
  `-0.0e0`. Positive infinity, negative infinity and NaN are `inf`, `-inf` and
  `NaN`; NaN payload bits are not represented in text.
* Each successful call appends one ASCII line feed (`U+000A`) after the value.
  It does not promise that a failed or interrupted host write is atomic. An
  output write or final flush failure stops execution and reports `NDO6007` at
  the relevant `print` call or `main` entry, respectively.
* The intrinsic writes to the host's standard-output stream. It performs no
  effect or capability check in M4. This proposal does not assign an effect
  identity; NEP-0004 may do so for M5 without changing the output contract.
* The formatter writes to the output sink without constructing a temporary
  formatted `String`. Text-value ownership and the interpreter's other
  allocations are outside this narrow cost guarantee.

## What this makes impossible

* An implementation cannot reorder call arguments or skip one because it
  believes the result is unused. Programs may observe argument evaluation.
* An implementation cannot start executing a file before the complete M4
  source has parsed, type-checked and lowered successfully.
* `print` cannot silently become a dynamic `Display` protocol that allocates,
  dispatches by runtime type, or accepts arbitrary user-defined values.
* A platform cannot change `print`'s line ending or normalize its text payload.

This is an intentionally small output contract. Rich formatting, interpolation,
user-defined display, and fallible output as a returned `Result` need separate
designs if they become necessary.

## Alternatives

| Alternative | Why it lost |
| ----------- | ----------- |
| Leave direct-call order unspecified | Observable nested calls could differ between implementations and optimization levels. |
| Make `print` a generic library function | Function values, generic constraints and effects are not accepted yet; it would enlarge the M4 slice. |
| Print arbitrary values using runtime reflection | It adds dynamic dispatch, unclear output for nominal values, and a hidden formatting allocation. |
| Restrict output to `Int` only | It makes the first output operation unnecessarily narrow when the language already has primitive `Text`, `Bool` and `Float`. |
| Return an output `Result` from `print` | It changes the M4 vertical slice and error path; host output failures remain diagnostics for this initial CLI slice. |

## Impact

| Area | Effect |
| ---- | ------ |
| Specification | Adds the bounded M4 execution, direct-call order and output contract to `expressions.md`; records runtime diagnostics in `errors.md`. |
| Grammar | No change. |
| Compiler | HIR keeps built-in resolution; type checking validates the fixed signature; MIR and the interpreter lower and execute direct calls. |
| Conformance | Adds `nudo run` entry/error cases, accepted and rejected `print` type cases, runtime formatting and call-order cases. |
| Tooling | `nudo run` exposes the output operation for the single-file M4 slice. |
| Security | M4 has no effect checker or runtime capability grants; running source can write to standard output. |
| Compatibility | Programs using `nudo run`, `print` and direct calls gain defined entry, output and evaluation-order contracts. |
| Performance | Formatting is streamed to the output sink with no intermediate formatted string. The benchmark must report startup and dispatch separately from compilation and process startup. |

## Open questions

None within this proposal's scope. The effect identity and any runtime grant for
console access are expressly deferred to their own accepted design.

## Prior art

Rust, Swift and Kotlin evaluate call arguments in source order. Many languages
also expose a built-in output operation; NUDO's proposal keeps that operation
small and statically typed instead of defining a general reflection-based
formatting protocol. The interpreter's 32-frame guard is a reference-runtime
resource limit, similar to the execution limits imposed by other hosted
interpreters.

## Unresolved objections

None recorded.
