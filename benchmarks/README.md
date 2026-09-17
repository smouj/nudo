# Benchmarks

Reproducible measurements of the compiler front end, with no third-party harness
and no nightly toolchain:

```sh
cargo run --release --package nudo-bench
cargo run --release --package nudo-bench -- --iterations 1000
```

## What it measures

| Case | Input shape |
| ---- | ----------- |
| `functions` | 2 000 function declarations with parameters and arithmetic |
| `bindings` | 5 000 `let` bindings with numbers and `_` separators |
| `text-literals` | 2 000 text literals with escapes |
| `comments` | 2 000 comment-heavy blocks, line and block |

Each case registers the source (which builds the line index) and lexes it. Best
and mean of the timed iterations are reported, with throughput in MiB/s. Timings
are indicative, not a promise: the numbers depend on the machine, and the project
has **no accepted performance target**.

## How to use it

Compare a change against the previous commit, on the same machine, in the same
kind of machine load. A 5% difference between two quiet runs is a measurement; a
5% difference between a loaded laptop and a build server is noise.

Use it to answer "did this make things worse?" — not "is this fast?".

## What is not measured, and why

| Case | Status |
| ---- | ------ |
| Parsing | Not implemented (M2) |
| Type checking | Not implemented (M3) |
| Interpreter startup | Not implemented (M4) |
| End-to-end build | No build to measure |

They are listed here so that the gap is visible. A benchmark suite that reports
only what exists invites the conclusion that what exists is all there is.

## Why not criterion

Criterion is the standard choice and it is a dependency. The pre-alpha workspace
has none, `cargo bench` needs nightly for the built-in harness, and the numbers
this project needs today are comparisons between adjacent commits — which
`std::time::Instant`, warm-up iterations and best-of-N do adequately.

When a real performance target exists, the measurement should move to a harness
that can defend it. Until then, adding the dependency would buy precision nobody
is using.
