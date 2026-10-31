# Benchmarks

Reproducible measurements of the compiler front end, with no third-party harness
and no nightly toolchain:

```sh
cargo run --release --package nudo-bench
cargo run --release --package nudo-bench -- --iterations 1000 --warmup 100
```

Every run records the commit, whether the tree was dirty, the Rust version, the
operating system, the architecture, the CPU model and the iteration counts. A
number without that context is not a measurement, it is an anecdote.

## What it measures

The suite separates stages that can be isolated and says plainly when it cannot:

| Stage | Timed region | Isolated? |
| ----- | ------------ | --------- |
| `lex` | `tokenize` on a registered source | yes |
| `parse` | `tokenize` + `parse` | no: a parser needs a token stream |
| `hir` | `lower` on a pre-built syntax tree | yes |
| `typeck` | `check` on a pre-built HIR | yes |

The syntax tree and the HIR are built before the timed region starts for the
`hir` and `typeck` stages, so those numbers measure the stage and not the work
that produced its input. `parse` is cumulative on purpose; splitting a lexer
from its parser would mean timing a private entry point, which measures the
harness rather than the parser.

### Cases

| Case | Input shape |
| ---- | ----------- |
| `functions` | 2 000 function declarations with parameters and arithmetic |
| `bindings` | 5 000 `let` bindings with numbers and `_` separators |
| `text-literals` | 2 000 text literals with escapes |
| `comments` | 2 000 comment-heavy blocks, line and block |

Each case runs at every stage above, so the table has sixteen rows. Inputs are
generated deterministically before timing, and a run reports the number of
tokens or definitions it observed as `units`.

## Metrics

| Metric | Meaning |
| ------ | ------- |
| `min` | Fastest timed iteration; the least noisy signal for a quiet machine |
| `median` | Middle sample; used for throughput and for comparisons |
| `p95` | 95th percentile; shows the tail |
| `mean` | Arithmetic mean; reported for completeness, not used for verdicts |
| `throughput` | MiB/s of source text, from the median |
| `units` | Tokens (lex, parse) or definitions (hir, typeck) observed |

Percentiles are nearest-rank. Memory and allocation counts are **not** reported:
measuring them without a profiler would mean injecting an allocator into the
timed region, and a number that changes what it measures is worse than no number.

## Results as data

```sh
cargo run --release --package nudo-bench -- --format json --output results.json
```

The schema is `nudo-bench/v1`. The file contains the metadata above and one
object per case with all the metrics. It is meant to be committed, attached to a
pull request or stored by a CI job, so that two commits can be compared months
later without trusting a screenshot.

## Comparing two runs

```sh
cargo run --release --package nudo-bench -- --compare baseline.json
```

That prints a Markdown table with the delta of every case and marks a case as a
regression when its median is slower by more than the threshold (10% by default,
`--threshold` to change it). Cases with no baseline are listed separately.

The comparison does **not** fail CI. Shared runners have enough variance that a
single 12% swing proves nothing, and a pipeline that goes red on noise teaches
people to ignore it. Use the report as evidence, then re-measure both sides on
the same machine before acting. When a stable performance target exists, this is
the point where a statistically justified threshold can become a gate.

## How to use it

Compare a change against the previous commit, on the same machine, with the same
kind of load. A 5% difference between two quiet runs is a measurement; a 5%
difference between a loaded laptop and a build server is noise.

Use it to answer "did this make things worse?" — not "is this fast?".

## What is not measured, and why

| Case | Status |
| ---- | ------ |
| Interpreter startup | Cannot exist: there is no `nudo run` (M4) |
| End-to-end build | Cannot exist: nothing executes a program yet |
| Agents, tools, models | Designed only; no runtime to measure |
| Peak memory | Deliberately absent; see above |

They are listed so the gap is visible. A suite that reports only what exists
invites the conclusion that what exists is all there is.

## Why not criterion

Criterion is the standard choice and it is a dependency. The pre-alpha workspace
has none, `cargo bench` needs nightly for the built-in harness, and the numbers
this project needs today are reproducible comparisons between adjacent commits —
which `std::time::Instant`, warm-up iterations, percentiles and a JSON file do
adequately.

When a real performance target exists, the measurement should move to a harness
that can defend it. Until then, adding the dependency would buy precision nobody
is using.
