# NEP-0021: M4 runtime trap semantics and diagnostics

| Field | Value |
| ----- | ----- |
| Status | Draft |
| Authors | @smouj |
| Created | 2026-11-06 |
| NEP number | 0021 |
| Supersedes | — |
| Superseded by | — |
| Specification chapters | [`spec/errors.md`](../spec/errors.md), [`spec/expressions.md`](../spec/expressions.md) |
| Related NEPs | NEP-0003, NEP-0004, NEP-0007, NEP-0008, NEP-0017, NEP-0019 |

## Summary

Settle the M4 remainder edge case and assign stable, source-mapped runtime
diagnostics for the traps required by M4: `NDO6001 INTEGER_OVERFLOW`,
`NDO6002 INTEGER_ZERO_DIVISOR`, and `NDO6003 CONSOLE_WRITE_FAILED`. `NDO6003`
is a narrow addition to NEP-0008's trap set for the fixed M4 stdout boundary.
These remain runtime failures, not compile-time diagnostics, and do not expose
backend panic text or host-specific I/O errors.

## Motivation

NEP-0007 decides that integer overflow and division by zero trap, and NEP-0008
decides that a trap ends the program and cannot be caught. The `NDO6xxx` family
is reserved, but neither the codes nor the source locations are assigned yet.
Without that contract, an interpreter could report a Rust panic, a WebAssembly
backend could trap without a source location, and a tool could give each failure
a different code. M4 needs one result across implementations.

The M4 `Console` effect also has a host boundary: a grant can be checked before
writing, but stdout can fail after execution begins. The Console proposal treats
that fixed-sink failure as a non-catchable runtime trap; this NEP gives it the
same stable diagnostic contract.

## Guide-level explanation

```nudo
fn main() -> Unit with Console {
    let value = 9223372036854775807 + 1;
    print(value);
}
```

The program is well-typed. If execution reaches `+`, the signed 64-bit result
overflows and the run ends with `NDO6001` at that operator. The compiler may
recognise the overflow while compiling, but it must preserve it as a runtime
trap rather than reject the source as a compile-time error.

The runtime diagnostic uses the source operation as its primary span:

```text
error[NDO6001]: integer overflow
  --> main.nudo:2:37
```

The same rule applies to division or remainder by zero (`NDO6002`). If a
`Console` grant is present but the host fails while writing `print` output, the
run ends with `NDO6003`. A missing `Console` grant is a capability denial
(`NDO5001` under the NEP-0003 proposal), not a stdout write failure.

## Reference-level explanation

### Registered codes proposed for M4

| Code | Name | Severity | Meaning |
| ---- | ---- | -------- | ------- |
| `NDO6001` | `INTEGER_OVERFLOW` | error (runtime trap) | An evaluated `Int` operation has no representable signed 64-bit result. |
| `NDO6002` | `INTEGER_ZERO_DIVISOR` | error (runtime trap) | An evaluated `Int` `/` or `%` operation has a zero right operand. |
| `NDO6003` | `CONSOLE_WRITE_FAILED` | error (runtime trap) | The host failed while writing the fixed M4 `Console` output. |

The runtime diagnostic messages are respectively `integer overflow`,
`integer divisor is zero`, and `writing to standard output failed`. Messages
must not contain host panic text or operating-system error strings. The stable
code carries the machine-readable cause.

### Integer operation rules

* `NDO6001` covers overflow from `Int` addition, subtraction, multiplication,
  unary negation of a runtime `Int::MIN` value, and `Int::MIN / -1`. The special
  negative literal form in NEP-0019 denotes `Int::MIN` directly and does not
  itself trap; a later negation of that value does.
* `NDO6002` covers both integer division and remainder by zero. `Float` division
  follows IEEE-754 binary64 and does not use this diagnostic.
* `Int::MIN % -1` evaluates to `0`. The mathematical remainder is representable;
  quotient overflow is specific to `/`. A backend must not trap merely because
  its machine division instruction also computes a quotient for remainder.
* A compile-time-known overflow or zero divisor remains a runtime trap if and
  when execution reaches that expression. The compiler may lower it to an
  explicit trap instruction. It must not turn it into a compile diagnostic or
  trap while evaluating a skipped short-circuit branch.
* Out-of-range integer literals are rejected by the type checker under the
  separate `NDO2xxx` contract proposed in NEP-0019; they are not `NDO6001`.

### Source mapping and process result

* `NDO6001` and `NDO6002` underline the operator token that trapped.
* `NDO6003` underlines the `print` name at the call site.
* The diagnostic is written to stderr, and `nudo run` exits with status `1`.
  A failure to start or operate the tool itself retains status `2`.
* A runtime trap ends the current root-program execution. It is not catchable
  and is not converted to `Result`, consistent with NEP-0008.
* For `NDO6003`, the runtime attempts to write the ASCII decimal spelling and
  one LF byte. If the host accepts a prefix and then fails, that prefix remains
  visible; the runtime cannot roll back bytes already written.
* The same trap at the same source operation produces the same code and source
  span on every backend. A backend may not surface its own panic or native trap
  as the user-facing diagnostic.

## What this makes impossible

* A Rust panic message or backend-specific trap becoming NUDO's error interface.
* Reporting an arithmetic trap as a compile error merely because constant
  folding discovered it early.
* Treating a missing capability grant as if the host's stdout had failed.
* Making `Int::MIN % -1` depend on whether a backend uses a signed division
  instruction internally.

## Alternatives

| Alternative | Why it lost |
| ----------- | ----------- |
| One generic `NDO6000` for every trap | It hides whether the program overflowed, divided by zero, or failed to write output. |
| Surface backend panic text | It is unstable, can expose host details, and differs across backends. |
| Make integer traps compile errors when statically known | It changes NEP-0007's runtime semantics and traps code that might not execute. |
| Return `Result` for the M4 fixed stdout sink | M4 exposes one fixed sink and no retry or alternate-sink handler; a configurable writer API must use `Result`. |
| Trap for `Int::MIN % -1` because a machine divide instruction overflows | It makes the language depend on the backend instruction rather than the representable remainder. |

## Impact

| Area | Effect |
| ---- | ------ |
| Specification | Registers the M4 runtime trap codes and source-span rules in `spec/errors.md` after acceptance. |
| Grammar | None. |
| Compiler | Constant folding and MIR lowering preserve trap kind and operator span. |
| Runtime | Converts arithmetic failures and the M4 stdout boundary failure into stable NUDO diagnostics. |
| Conformance | Adds runtime cases for integer overflow, zero divisors, `Int::MIN / -1`, `Int::MIN % -1`, and stdout failure. |
| Tooling | `nudo run` prints traps to stderr and returns status `1`; tool failures remain status `2`. |
| Compatibility | Assigns permanent codes to runtime failures that previously had no implementation. |
| Performance | No added work on successful arithmetic beyond the checked operations already required by NEP-0007. |

## Open questions

None affect the M4 contract. Trap supervision across concurrent tasks remains
outside this milestone and is already open in NEP-0008.

## Prior art

The proposal keeps NUDO's runtime diagnostics separate from host panics and
assigns one stable code per observable failure category. It does not copy a
backend's native trap numbering into the language interface.

## Unresolved objections

None recorded. This is a Draft and has not entered review.
