# 08.2 — Benchmarking the front end

> **Status:** Implemented / Planned
>
> **Summary:** NUDO has a small reproducible front-end benchmark suite; its results describe the recorded workload and host, not a universal speed rating.

The benchmark program measures compiler stages that exist today. It does not
measure execution of NUDO programs, since the interpreter and <code>nudo run</code>
do not exist yet.

## Workloads and stage boundaries

The suite generates four deterministic synthetic inputs: function declarations,
let bindings, text literals and comment-heavy source. Each is measured at four
stages, for sixteen rows:

| Stage | Timed work |
| --- | --- |
| Lex | Tokenize a registered source file |
| Parse | Tokenize and parse through the public parser |
| HIR | Lower a prebuilt syntax tree |
| Type checker | Check prebuilt HIR |

The parser measurement includes tokenization because parsing requires a token
stream. HIR lowering and type checking prepare their input before timing, so the
reported values isolate those stages. The workloads are deliberately small and
synthetic; they do not represent every language feature or a full application.

The type-checker unit is the length of the expression-type table, including
slots with no type. It is not a count of rules executed or expressions
successfully checked. Equal unit counts do not prove equal semantic work.

## What a result records

A run records the source revision and dirty state, Rust compiler, operating
system, architecture, CPU description when available, iteration counts, input
bytes, output units and timing statistics. The report includes minimum, median,
95th percentile and mean; lexer and parser rows also report throughput.
There is no project-wide reference baseline and no automatic performance gate
in CI.

## Comparing results

Use release builds. Compare the same workloads, stage boundaries, iteration
settings and compiler toolchain. Run both revisions on the same machine under
similar quiet conditions, repeat them, and keep the JSON reports with the
change being reviewed. A single comparison is an investigation signal; repeat
it before attributing a timing difference to code.

The checker changed semantic coverage in PR #31. A type-checking measurement
before and after that change can describe two implementations doing different
work, even when the source and expression-table count match. It cannot by itself
establish a regression or improvement at equivalent work.

## Measurements across machines

A result from one computer is a measured point for that computer and setup. It
can help with rough planning, but it does not support a reliable conversion to
another CPU from clock speed, core count or model name alone. Microarchitecture,
memory and cache, sustained power limits, operating-system scheduling, compiler
version and background load can affect stages by different amounts.

For a target-machine estimate, run the same commit and workloads on that
machine, record the metadata, and repeat the measurements. If that is not
possible, label any estimate as a rough extrapolation and include the assumptions
and uncertainty. Do not present it as a benchmark result.

{{#include ../diagrams/benchmark-portability.svg}}

*The diagram shows why measurements travel with their setup and why a second
hardware target needs its own run.*

See the
<a href="https://github.com/smouj/nudo/blob/main/benchmarks/README.md">benchmark
README</a> for the current command-line options and JSON report format.
