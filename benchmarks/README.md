# Benchmarks

Reproducible measurements of the compiler front end, with no third-party harness
and no nightly toolchain:

```sh
cargo run --release --package nudo-bench
cargo run --release --package nudo-bench -- --iterations 1000 --warmup 100
```

Every run records the commit, whether the tree was dirty, the Rust version, the
operating system, the architecture, the CPU model when the host exposes it (or
`unknown`), and the iteration counts. A number without that context is not a
measurement, it is an anecdote.

## What it measures

The suite separates stages that can be isolated and says plainly when it cannot:

| Stage | Timed region | Isolated? |
| ----- | ------------ | --------- |
| `lex` | `tokenize` on a pre-registered source | yes |
| `parse` | `tokenize` + `parse` on a pre-registered source | no: a parser needs a token stream |
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
generated deterministically before timing, and a run reports each stage's
observed output count as `units`. The cases are a fixed synthetic workload,
**not** a conformance corpus or a representative sample of all supported
language features. In particular, they do not exercise the bounded generic
inference, trust-type assignment or malformed-call paths introduced by PR #31.

## Metrics

| Metric | Meaning |
| ------ | ------- |
| `min` | Fastest timed iteration; the least noisy signal for a quiet machine |
| `median` | Middle sample; used for throughput and for comparisons |
| `p95` | 95th percentile; shows the tail |
| `mean` | Arithmetic mean; reported for completeness, not used for verdicts |
| `throughput` | MiB/s of source text, from the median |
| `units` | Tokens (lex), syntax nodes (parse), definitions (hir), or expression-table slots (typeck) reported |

The text report shows the count and its unit in separate columns. JSON stores
the same values as `units` and `unit`. For `typeck`, the implementation reports
`expression_types.len()`: the number of allocated expression-table slots,
**including untyped entries**. This is not a count of rules executed, errors
reported or expressions successfully type-checked. Equal counts do not imply
equal semantic work.

Percentiles are nearest-rank. Memory and allocation counts are **not** reported:
measuring them without a profiler would mean injecting an allocator into the
timed region, and a number that changes what it measures is worse than no number.

## Results as data

```sh
cargo run --release --package nudo-bench -- --format json --output results.json
```

The schema is `nudo-bench/v1`. The file contains the metadata above and one
object per case with all the metrics. It can be attached to a pull request or
stored by a CI job so that measurements are traceable without screenshots.
**No project-wide reference baseline is currently committed.** The
`baseline.json` name below denotes a file generated from a real run, not a
shipped reference or an accepted performance target.

## Comparing two runs

On the **baseline revision**, record a clean, quiet run outside the working
tree (the paths below are illustrative and must be adapted to the host):

```sh
cargo run --release --package nudo-bench -- --format json --output /tmp/nudo-baseline.json
```

On the **candidate revision**, using the baseline file from that first run:

```sh
cargo run --release --package nudo-bench -- --compare /tmp/nudo-baseline.json
```

The comparison accepts the `nudo-bench/v1` schema and uses the median. It
compares matching stage/case pairs only when the source-byte count, output-unit
name and reported unit count agree. Cases with different reported work are
**excluded and listed separately**; cases absent from the baseline are listed
as missing. An increase over the reporting threshold (10% by default,
configurable with `--threshold`) is an **investigation signal**, not a proven
regression. Matching counts are necessary checks, **not proof** of identical
source contents, semantics, or executed operations.

### Comparability after PR #31

The `typeck` stage still measures `check()` on pre-built HIR, but PR #31 changed
what that checker does: numeric operator checking, bounded generic inference,
recursive checking of deferred expression children, named type-argument arity,
trust-type assignment and malformed-call recovery. It can therefore do more
work on the **same source** while reporting an unchanged expression-table length.
A `typeck` delta from commit `1251cf7` to a post-PR #31 revision measures the
cost of **different checker implementations and semantic coverage**. It must
**not** be described as an equal-work performance regression or improvement.

The `functions` case exercises arithmetic, while the other three cases mainly
exercise bindings, literal expressions and comments. There is no claim that
these four workloads cover new generic or trust-type paths. The `lex`, `parse`
and `hir` rows retain their stage boundaries and input shapes, but interpretation
still requires confirming that the work performed is comparable on both sides.

Always check the full metadata (commit, dirty tree, Rust version, OS, CPU,
architecture, iterations and warm-up). Rebuild both revisions with the same
release profile and test them on the same quiet machine. An identical CPU label
is not sufficient to prove identical load, frequency or environment.

The comparison does **not** fail CI. Shared runners have enough variance that a
single timing swing proves nothing. Use the report as evidence, and re-measure
both revisions before acting. The 10% threshold is not a statistical confidence
interval and the suite has **no accepted performance target**.

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
